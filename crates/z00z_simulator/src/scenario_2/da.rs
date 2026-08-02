use std::{
    mem,
    path::{Path, PathBuf},
};

use celestia_types::{
    consts::appconsts::SHARE_SIZE,
    hash::Hash as CelestiaHash,
    nmt::{Namespace, NS_SIZE},
    Blob, DataAvailabilityHeader, ExtendedDataSquare, InfoByte, ValidateBasic,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use z00z_aggregators::{OrderedBatch, WorkPayload};
use z00z_storage::settlement::SettlementStateRoot;
use z00z_utils::{
    codec::{Codec, JsonCodec},
    io::{
        atomic_write_file_streaming, load_json_bounded, read_file_bounded, save_json, IoError,
        Write,
    },
};

use super::{config::DaCfg, runner::Scenario2Err};

const APP_PAYLOAD_MAGIC: [u8; 8] = *b"Z00ZAPV2";
const APP_PAYLOAD_VERSION: u16 = 2;
const MANIFEST_VERSION: u16 = 1;
const CELESTIA_TYPES_VERSION: &str = "1.0.0";
const MANIFEST_MAX_BYTES: u64 = 2 * 1024 * 1024;
const EDS_FILE: &str = "eds.bin";
const PAYLOAD_LABEL: &[u8] = b"z00z.simulator.scenario-2.da-payload.v2";
const RAW_ROOT_LABEL: &[u8] = b"z00z.simulator.scenario-2.raw-tx-root.v2";
const PROOF_ROOT_LABEL: &[u8] = b"z00z.simulator.scenario-2.tx-proof-root.v2";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CelestiaBlockManifest {
    schema_version: u16,
    celestia_types_version: String,
    cycle: u32,
    height: u64,
    batch_id: [u8; 32],
    pre_root: [u8; 32],
    post_root: [u8; 32],
    namespace: [u8; NS_SIZE],
    share_version: u8,
    blob_count: u32,
    blob_commitments: Vec<[u8; 32]>,
    package_count: u32,
    payload_bytes: u64,
    payload_digest: [u8; 32],
    raw_tx_root: [u8; 32],
    exact_proof_root: [u8; 32],
    proof_bytes: u64,
    original_shares: u64,
    padded_ods_shares: u64,
    eds_shares: u64,
    ods_width: u16,
    eds_width: u16,
    share_size: u32,
    eds_bytes: u64,
    codec: String,
    eds_file: String,
    data_hash: [u8; 32],
    dah: DataAvailabilityHeader,
    pfb_included: bool,
    network_published: bool,
}

pub(super) struct PreparedDa {
    eds: ExtendedDataSquare,
    manifest: CelestiaBlockManifest,
}

impl PreparedDa {
    pub fn package_count(&self) -> u32 {
        self.manifest.package_count
    }

    pub fn payload_bytes(&self) -> u64 {
        self.manifest.payload_bytes
    }

    pub fn artifact_bytes(&self) -> u64 {
        self.manifest.eds_bytes
    }
}

pub(super) struct DaCommit {
    pub path: PathBuf,
    pub payload_commitment: [u8; 32],
    pub raw_tx_root: [u8; 32],
    pub exact_proof_root: [u8; 32],
    pub payload_bytes: u64,
    pub proof_bytes: u64,
    pub package_count: u32,
    pub artifact_bytes: u64,
    eds_path: PathBuf,
}

struct CelestiaSquare {
    eds: ExtendedDataSquare,
    dah: DataAvailabilityHeader,
    commitments: Vec<[u8; 32]>,
    original_shares: usize,
    ods_width: usize,
}

struct CelestiaBlockBuilder {
    namespace: Namespace,
    max_blob_bytes: usize,
    max_ods_width: usize,
    chunk: Vec<u8>,
    blob_shares: Vec<Vec<Vec<u8>>>,
    commitments: Vec<[u8; 32]>,
}

impl CelestiaBlockBuilder {
    fn new(namespace: Namespace, max_blob_bytes: usize, max_ods_width: usize) -> Self {
        Self {
            namespace,
            max_blob_bytes,
            max_ods_width,
            chunk: Vec::new(),
            blob_shares: Vec::new(),
            commitments: Vec::new(),
        }
    }

    fn push_parts(&mut self, parts: &[&[u8]]) -> Result<(), Scenario2Err> {
        let incoming = parts.iter().try_fold(0_usize, |total, part| {
            total
                .checked_add(part.len())
                .ok_or_else(|| Scenario2Err::Da("Celestia blob length overflow".to_string()))
        })?;
        if incoming > self.max_blob_bytes {
            return Err(Scenario2Err::Da(
                "one framed transaction exceeds the Celestia blob bound".to_string(),
            ));
        }
        let next_len = self
            .chunk
            .len()
            .checked_add(incoming)
            .ok_or_else(|| Scenario2Err::Da("Celestia blob length overflow".to_string()))?;
        if !self.chunk.is_empty() && next_len > self.max_blob_bytes {
            self.flush_blob()?;
        }
        for part in parts {
            self.chunk.extend_from_slice(part);
        }
        Ok(())
    }

    fn flush_blob(&mut self) -> Result<(), Scenario2Err> {
        if self.chunk.is_empty() {
            return Ok(());
        }
        let blob = Blob::new(self.namespace, mem::take(&mut self.chunk), None)
            .map_err(|error| Scenario2Err::Da(format!("Celestia blob creation: {error}")))?;
        blob.validate()
            .map_err(|error| Scenario2Err::Da(format!("Celestia blob validation: {error}")))?;
        let shares = blob
            .to_shares()
            .map_err(|error| Scenario2Err::Da(format!("Celestia share encoding: {error}")))?;
        let reconstructed = Blob::reconstruct(&shares)
            .map_err(|error| Scenario2Err::Da(format!("Celestia blob reconstruction: {error}")))?;
        if reconstructed != blob {
            return Err(Scenario2Err::Invariant(
                "Celestia blob share roundtrip changed the blob".to_string(),
            ));
        }
        let max_shares = self
            .max_ods_width
            .checked_mul(self.max_ods_width)
            .ok_or_else(|| Scenario2Err::Da("Celestia ODS share bound overflow".to_string()))?;
        let current_shares = self.blob_shares.iter().try_fold(0_usize, |total, blob| {
            total
                .checked_add(blob.len())
                .ok_or_else(|| Scenario2Err::Da("Celestia ODS share count overflow".to_string()))
        })?;
        let next_shares = current_shares
            .checked_add(shares.len())
            .ok_or_else(|| Scenario2Err::Da("Celestia ODS share count overflow".to_string()))?;
        if next_shares > max_shares {
            return Err(Scenario2Err::Da(
                "block payload exceeds the configured Celestia ODS width".to_string(),
            ));
        }
        self.commitments.push(*blob.commitment.hash());
        self.blob_shares
            .push(shares.into_iter().map(|share| share.to_vec()).collect());
        Ok(())
    }

    fn finish(mut self) -> Result<CelestiaSquare, Scenario2Err> {
        self.flush_blob()?;
        if self.blob_shares.is_empty() || self.commitments.is_empty() {
            return Err(Scenario2Err::Da(
                "Celestia block contains no blob shares".to_string(),
            ));
        }
        let original_shares = self.blob_shares.iter().try_fold(0_usize, |total, blob| {
            total
                .checked_add(blob.len())
                .ok_or_else(|| Scenario2Err::Da("Celestia ODS share count overflow".to_string()))
        })?;
        let ods_width = layout_width(&self.blob_shares, self.max_ods_width)?;
        if ods_width > self.max_ods_width {
            return Err(Scenario2Err::Da(
                "Celestia ODS width exceeds the configured bound".to_string(),
            ));
        }
        let padded_shares = ods_width
            .checked_mul(ods_width)
            .ok_or_else(|| Scenario2Err::Da("Celestia ODS square overflow".to_string()))?;
        let blob_count = self.blob_shares.len();
        let mut ods_shares = Vec::with_capacity(padded_shares);
        for (index, shares) in self.blob_shares.into_iter().enumerate() {
            ods_shares.extend(shares);
            let row_remainder = ods_shares.len() % ods_width;
            if row_remainder != 0 {
                let padding_count = ods_width - row_remainder;
                let padding_namespace = if index + 1 == blob_count {
                    Namespace::TAIL_PADDING
                } else {
                    self.namespace
                };
                let padding = namespace_padding_share(padding_namespace)?;
                ods_shares.extend((0..padding_count).map(|_| padding.clone()));
            }
        }
        let tail_padding = namespace_padding_share(Namespace::TAIL_PADDING)?;
        ods_shares.resize(padded_shares, tail_padding);
        let eds = ExtendedDataSquare::from_ods(ods_shares)
            .map_err(|error| Scenario2Err::Da(format!("Celestia EDS encoding: {error}")))?;
        let dah = DataAvailabilityHeader::from_eds(&eds);
        dah.validate_basic()
            .map_err(|error| Scenario2Err::Da(format!("Celestia DAH validation: {error}")))?;
        let expected_eds_width = ods_width
            .checked_mul(2)
            .ok_or_else(|| Scenario2Err::Da("Celestia EDS width overflow".to_string()))?;
        if usize::from(eds.square_width()) != expected_eds_width {
            return Err(Scenario2Err::Invariant(
                "Celestia EDS width changed during encoding".to_string(),
            ));
        }
        reconstruct_blobs(&eds, self.namespace, &self.commitments)?;
        Ok(CelestiaSquare {
            eds,
            dah,
            commitments: self.commitments,
            original_shares,
            ods_width,
        })
    }
}

pub(super) fn prepare_block(
    cycle: u32,
    height: u64,
    batch: &OrderedBatch,
    pre_root: SettlementStateRoot,
    post_root: SettlementStateRoot,
    config: &DaCfg,
) -> Result<PreparedDa, Scenario2Err> {
    let package_count = u32::try_from(batch.items.len())
        .map_err(|_| Scenario2Err::Da("package count overflow".to_string()))?;
    let namespace = config.namespace_value()?;
    let mut payload_hasher = Sha256::new();
    let mut raw_hasher = Sha256::new();
    let mut proof_hasher = Sha256::new();
    payload_hasher.update(PAYLOAD_LABEL);
    raw_hasher.update(RAW_ROOT_LABEL);
    proof_hasher.update(PROOF_ROOT_LABEL);
    let header = da_header(
        cycle,
        height,
        package_count,
        batch.batch_id.into_bytes(),
        pre_root,
        post_root,
    );
    payload_hasher.update(&header);
    let mut payload_bytes = bytes_len(&header, "DA header")?;
    let mut proof_bytes = 0_u64;
    let mut builder = CelestiaBlockBuilder::new(
        namespace,
        config.max_blob_payload_bytes,
        config.max_ods_width,
    );
    builder.push_parts(&[&header])?;

    for item in &batch.items {
        let WorkPayload::Tx(package) = item.payload() else {
            return Err(Scenario2Err::Da(
                "scenario_2 DA block accepts only regular transactions".to_string(),
            ));
        };
        let package_bytes = JsonCodec
            .serialize(package.as_ref())
            .map_err(|error| Scenario2Err::Da(error.to_string()))?;
        let proof = JsonCodec
            .serialize(&package.tx.proof)
            .map_err(|error| Scenario2Err::Da(error.to_string()))?;
        let frame_len = bytes_len(&package_bytes, "DA package")?;
        let proof_len = bytes_len(&proof, "DA proof")?;
        let frame = frame_len.to_le_bytes();
        let proof_frame = proof_len.to_le_bytes();
        builder.push_parts(&[&frame, &package_bytes])?;
        payload_hasher.update(frame);
        payload_hasher.update(&package_bytes);
        raw_hasher.update(frame);
        raw_hasher.update(&package_bytes);
        proof_hasher.update(proof_frame);
        proof_hasher.update(&proof);
        payload_bytes = payload_bytes
            .checked_add(8)
            .and_then(|value| value.checked_add(frame_len))
            .ok_or_else(|| Scenario2Err::Da("DA byte count overflow".to_string()))?;
        proof_bytes = proof_bytes
            .checked_add(8)
            .and_then(|value| value.checked_add(proof_len))
            .ok_or_else(|| Scenario2Err::Da("DA proof byte count overflow".to_string()))?;
    }

    let square = builder.finish()?;
    let data_hash_value = square.dah.hash();
    let data_hash = celestia_hash(&data_hash_value)?;
    let blob_count = u32::try_from(square.commitments.len())
        .map_err(|_| Scenario2Err::Da("Celestia blob count overflow".to_string()))?;
    let original_shares = count_u64(square.original_shares, "original Celestia shares")?;
    let padded_ods_shares = count_u64(
        square
            .ods_width
            .checked_mul(square.ods_width)
            .ok_or_else(|| Scenario2Err::Da("Celestia ODS square overflow".to_string()))?,
        "padded Celestia ODS shares",
    )?;
    let eds_shares = count_u64(square.eds.data_square().len(), "Celestia EDS shares")?;
    let share_size = u32::try_from(SHARE_SIZE)
        .map_err(|_| Scenario2Err::Da("Celestia share size overflow".to_string()))?;
    let eds_bytes = eds_shares
        .checked_mul(u64::from(share_size))
        .ok_or_else(|| Scenario2Err::Da("Celestia EDS byte count overflow".to_string()))?;
    let ods_width = u16::try_from(square.ods_width)
        .map_err(|_| Scenario2Err::Da("Celestia ODS width overflow".to_string()))?;
    let manifest = CelestiaBlockManifest {
        schema_version: MANIFEST_VERSION,
        celestia_types_version: CELESTIA_TYPES_VERSION.to_string(),
        cycle,
        height,
        batch_id: batch.batch_id.into_bytes(),
        pre_root: *pre_root.as_bytes(),
        post_root: *post_root.as_bytes(),
        namespace: namespace
            .as_bytes()
            .try_into()
            .map_err(|_| Scenario2Err::Invariant("Celestia namespace width drift".to_string()))?,
        share_version: 0,
        blob_count,
        blob_commitments: square.commitments,
        package_count,
        payload_bytes,
        payload_digest: payload_hasher.finalize().into(),
        raw_tx_root: raw_hasher.finalize().into(),
        exact_proof_root: proof_hasher.finalize().into(),
        proof_bytes,
        original_shares,
        padded_ods_shares,
        eds_shares,
        ods_width,
        eds_width: square.eds.square_width(),
        share_size,
        eds_bytes,
        codec: square.eds.codec().to_string(),
        eds_file: EDS_FILE.to_string(),
        data_hash,
        dah: square.dah,
        pfb_included: false,
        network_published: false,
    };
    Ok(PreparedDa {
        eds: square.eds,
        manifest,
    })
}

pub(super) fn persist_block(
    run_dir: &Path,
    prepared: PreparedDa,
) -> Result<DaCommit, Scenario2Err> {
    let block_dir = run_dir
        .join("da")
        .join(format!("cycle-{:02}", prepared.manifest.cycle))
        .join(format!("block-{:05}", prepared.manifest.height));
    let eds_path = block_dir.join(EDS_FILE);
    let manifest_path = block_dir.join("manifest.json");
    atomic_write_file_streaming(&eds_path, |file| {
        let mut written = 0_u64;
        for share in prepared.eds.data_square() {
            file.write_all(share.data())?;
            written = written
                .checked_add(u64::try_from(share.data().len()).map_err(|_| {
                    IoError::Serialization("Celestia share length overflow".to_string())
                })?)
                .ok_or_else(|| {
                    IoError::Serialization("Celestia EDS byte count overflow".to_string())
                })?;
        }
        if written != prepared.manifest.eds_bytes {
            return Err(IoError::Serialization(
                "Celestia EDS persisted length mismatch".to_string(),
            ));
        }
        Ok(())
    })?;
    save_json(&manifest_path, &prepared.manifest)?;
    Ok(DaCommit {
        path: manifest_path,
        payload_commitment: prepared.manifest.data_hash,
        raw_tx_root: prepared.manifest.raw_tx_root,
        exact_proof_root: prepared.manifest.exact_proof_root,
        payload_bytes: prepared.manifest.payload_bytes,
        proof_bytes: prepared.manifest.proof_bytes,
        package_count: prepared.manifest.package_count,
        artifact_bytes: prepared.manifest.eds_bytes,
        eds_path,
    })
}

pub(super) fn verify_block(commit: &DaCommit, config: &DaCfg) -> Result<(), Scenario2Err> {
    let manifest: CelestiaBlockManifest = load_json_bounded(&commit.path, MANIFEST_MAX_BYTES)?;
    let namespace = validate_manifest(&manifest, commit, config)?;
    let bytes = read_file_bounded(&commit.eds_path, config.max_eds_bytes()?)?;
    if count_u64(bytes.len(), "persisted Celestia EDS bytes")? != manifest.eds_bytes {
        return Err(Scenario2Err::Invariant(
            "Celestia EDS file length mismatch".to_string(),
        ));
    }
    let mut chunks = bytes.chunks_exact(SHARE_SIZE);
    let shares = chunks.by_ref().map(<[u8]>::to_vec).collect::<Vec<_>>();
    if !chunks.remainder().is_empty()
        || count_u64(shares.len(), "reloaded Celestia shares")? != manifest.eds_shares
    {
        return Err(Scenario2Err::Invariant(
            "Celestia EDS share framing mismatch".to_string(),
        ));
    }
    let eds = ExtendedDataSquare::new(shares, manifest.codec.clone())
        .map_err(|error| Scenario2Err::Da(format!("Celestia EDS reload: {error}")))?;
    if eds.square_width() != manifest.eds_width {
        return Err(Scenario2Err::Invariant(
            "Celestia EDS reload width mismatch".to_string(),
        ));
    }
    let dah = DataAvailabilityHeader::from_eds(&eds);
    dah.validate_basic()
        .map_err(|error| Scenario2Err::Da(format!("Celestia DAH reload validation: {error}")))?;
    let dah_hash = dah.hash();
    if dah != manifest.dah || celestia_hash(&dah_hash)? != manifest.data_hash {
        return Err(Scenario2Err::Invariant(
            "Celestia DAH reload mismatch".to_string(),
        ));
    }
    let blobs = reconstruct_blobs(&eds, namespace, &manifest.blob_commitments)?;
    let mut payload_hasher = Sha256::new();
    payload_hasher.update(PAYLOAD_LABEL);
    let mut payload_bytes = 0_u64;
    for blob in blobs {
        payload_hasher.update(&blob.data);
        payload_bytes = payload_bytes
            .checked_add(bytes_len(&blob.data, "reloaded Celestia blob")?)
            .ok_or_else(|| Scenario2Err::Da("Celestia payload length overflow".to_string()))?;
    }
    if payload_bytes != manifest.payload_bytes
        || <[u8; 32]>::from(payload_hasher.finalize()) != manifest.payload_digest
    {
        return Err(Scenario2Err::Invariant(
            "Celestia blob payload reload mismatch".to_string(),
        ));
    }
    Ok(())
}

fn validate_manifest(
    manifest: &CelestiaBlockManifest,
    commit: &DaCommit,
    config: &DaCfg,
) -> Result<Namespace, Scenario2Err> {
    let namespace = Namespace::from_raw(&manifest.namespace)
        .map_err(|error| Scenario2Err::Da(format!("Celestia namespace reload: {error}")))?;
    let expected_namespace = config.namespace_value()?;
    let blob_count = usize::try_from(manifest.blob_count)
        .map_err(|_| Scenario2Err::Da("Celestia blob count overflow".to_string()))?;
    let ods_width = usize::from(manifest.ods_width);
    let eds_width = usize::from(manifest.eds_width);
    let padded_ods_shares = ods_width
        .checked_mul(ods_width)
        .ok_or_else(|| Scenario2Err::Da("Celestia ODS square overflow".to_string()))?;
    let eds_shares = eds_width
        .checked_mul(eds_width)
        .ok_or_else(|| Scenario2Err::Da("Celestia EDS square overflow".to_string()))?;
    let expected_eds_width = ods_width
        .checked_mul(2)
        .ok_or_else(|| Scenario2Err::Da("Celestia EDS width overflow".to_string()))?;
    let expected_eds_bytes =
        count_u64(eds_shares, "Celestia EDS shares")?
            .checked_mul(u64::try_from(SHARE_SIZE).map_err(|_| {
                Scenario2Err::Da("Celestia share size conversion failed".to_string())
            })?)
            .ok_or_else(|| Scenario2Err::Da("Celestia EDS byte count overflow".to_string()))?;
    if manifest.schema_version != MANIFEST_VERSION
        || manifest.celestia_types_version != CELESTIA_TYPES_VERSION
        || manifest.eds_file != EDS_FILE
        || manifest.codec != "Leopard"
        || manifest.pfb_included
        || manifest.network_published
        || namespace != expected_namespace
        || manifest.share_version != 0
        || blob_count == 0
        || blob_count != manifest.blob_commitments.len()
        || manifest.package_count == 0
        || manifest.payload_bytes == 0
        || ods_width == 0
        || !ods_width.is_power_of_two()
        || ods_width > config.max_ods_width
        || eds_width != expected_eds_width
        || manifest.original_shares == 0
        || manifest.original_shares > manifest.padded_ods_shares
        || manifest.padded_ods_shares != count_u64(padded_ods_shares, "padded Celestia ODS shares")?
        || manifest.eds_shares != count_u64(eds_shares, "Celestia EDS shares")?
        || manifest.share_size
            != u32::try_from(SHARE_SIZE)
                .map_err(|_| Scenario2Err::Da("Celestia share size overflow".to_string()))?
        || manifest.eds_bytes != expected_eds_bytes
        || manifest.data_hash != commit.payload_commitment
        || manifest.raw_tx_root != commit.raw_tx_root
        || manifest.exact_proof_root != commit.exact_proof_root
        || manifest.payload_bytes != commit.payload_bytes
        || manifest.proof_bytes != commit.proof_bytes
        || manifest.package_count != commit.package_count
        || manifest.eds_bytes != commit.artifact_bytes
    {
        return Err(Scenario2Err::Invariant(
            "Celestia manifest reload mismatch".to_string(),
        ));
    }
    Ok(namespace)
}

fn reconstruct_blobs(
    eds: &ExtendedDataSquare,
    namespace: Namespace,
    commitments: &[[u8; 32]],
) -> Result<Vec<Blob>, Scenario2Err> {
    let blobs = Blob::reconstruct_all(eds.data_square())
        .map_err(|error| Scenario2Err::Da(format!("Celestia EDS blob reconstruction: {error}")))?;
    if blobs.len() != commitments.len() {
        return Err(Scenario2Err::Invariant(
            "Celestia reconstructed blob count mismatch".to_string(),
        ));
    }
    for (blob, expected) in blobs.iter().zip(commitments) {
        blob.validate()
            .map_err(|error| Scenario2Err::Da(format!("Celestia blob revalidation: {error}")))?;
        if blob.namespace != namespace || *blob.commitment.hash() != *expected {
            return Err(Scenario2Err::Invariant(
                "Celestia reconstructed blob commitment mismatch".to_string(),
            ));
        }
    }
    Ok(blobs)
}

fn layout_width(blob_shares: &[Vec<Vec<u8>>], max_ods_width: usize) -> Result<usize, Scenario2Err> {
    let mut width = 1_usize;
    loop {
        let required_rows = blob_shares.iter().try_fold(0_usize, |rows, shares| {
            let blob_rows =
                shares.len().checked_add(width - 1).ok_or_else(|| {
                    Scenario2Err::Da("Celestia blob row count overflow".to_string())
                })? / width;
            rows.checked_add(blob_rows)
                .ok_or_else(|| Scenario2Err::Da("Celestia ODS row count overflow".to_string()))
        })?;
        if required_rows <= width {
            return Ok(width);
        }
        width = width
            .checked_mul(2)
            .ok_or_else(|| Scenario2Err::Da("Celestia ODS width overflow".to_string()))?;
        if width > max_ods_width {
            return Err(Scenario2Err::Da(
                "blob row padding exceeds the configured Celestia ODS width".to_string(),
            ));
        }
    }
}

fn namespace_padding_share(namespace: Namespace) -> Result<Vec<u8>, Scenario2Err> {
    let mut share = vec![0_u8; SHARE_SIZE];
    share[..NS_SIZE].copy_from_slice(namespace.as_bytes());
    share[NS_SIZE] = InfoByte::new(0, false)
        .map_err(|error| Scenario2Err::Da(format!("Celestia padding info byte: {error}")))?
        .as_u8();
    Ok(share)
}

fn celestia_hash(hash: &CelestiaHash) -> Result<[u8; 32], Scenario2Err> {
    match hash {
        CelestiaHash::Sha256(bytes) => Ok(*bytes),
        CelestiaHash::None => Err(Scenario2Err::Da(
            "Celestia commitment unexpectedly has no SHA-256 hash".to_string(),
        )),
    }
}

fn da_header(
    cycle: u32,
    height: u64,
    package_count: u32,
    batch_id: [u8; 32],
    pre_root: SettlementStateRoot,
    post_root: SettlementStateRoot,
) -> Vec<u8> {
    let mut header = Vec::with_capacity(8 + 2 + 4 + 8 + 4 + 32 * 3);
    header.extend_from_slice(&APP_PAYLOAD_MAGIC);
    header.extend_from_slice(&APP_PAYLOAD_VERSION.to_le_bytes());
    header.extend_from_slice(&cycle.to_le_bytes());
    header.extend_from_slice(&height.to_le_bytes());
    header.extend_from_slice(&package_count.to_le_bytes());
    header.extend_from_slice(&batch_id);
    header.extend_from_slice(pre_root.as_bytes());
    header.extend_from_slice(post_root.as_bytes());
    header
}

fn bytes_len(bytes: &[u8], label: &str) -> Result<u64, Scenario2Err> {
    u64::try_from(bytes.len()).map_err(|_| Scenario2Err::Da(format!("{label} length overflow")))
}

fn count_u64(value: usize, label: &str) -> Result<u64, Scenario2Err> {
    u64::try_from(value).map_err(|_| Scenario2Err::Da(format!("{label} count overflow")))
}

#[cfg(test)]
mod tests {
    use super::{
        celestia_hash, persist_block, reconstruct_blobs, verify_block, CelestiaBlockBuilder,
        CelestiaBlockManifest, PreparedDa, CELESTIA_TYPES_VERSION, EDS_FILE, MANIFEST_VERSION,
        PAYLOAD_LABEL,
    };
    use celestia_types::nmt::Namespace;
    use sha2::{Digest, Sha256};
    use z00z_utils::io::path_exists_no_follow;

    use crate::scenario_2::config::DaCfg;

    #[test]
    fn celestia_eds_roundtrips_blobs() -> Result<(), super::Scenario2Err> {
        let namespace = Namespace::new_v0(b"z00z-sim2")
            .map_err(|error| super::Scenario2Err::Da(error.to_string()))?;
        let mut builder = CelestiaBlockBuilder::new(namespace, 8, 8);
        builder.push_parts(&[b"first"])?;
        builder.push_parts(&[b"second"])?;
        let square = builder.finish()?;
        let blobs = reconstruct_blobs(&square.eds, namespace, &square.commitments)?;
        let dah_hash = square.dah.hash();

        assert_eq!(blobs.len(), 2);
        assert_eq!(blobs[0].data, b"first");
        assert_eq!(blobs[1].data, b"second");
        assert_ne!(celestia_hash(&dah_hash)?, [0_u8; 32]);
        Ok(())
    }

    #[test]
    fn celestia_artifact_reloads() -> Result<(), super::Scenario2Err> {
        let config = DaCfg {
            namespace: "z00z-sim2".to_string(),
            max_blob_payload_bytes: 8,
            max_ods_width: 8,
        };
        let namespace = config.namespace_value()?;
        let mut builder = CelestiaBlockBuilder::new(namespace, 8, 8);
        builder.push_parts(&[b"first"])?;
        builder.push_parts(&[b"second"])?;
        let square = builder.finish()?;
        let dah_hash = square.dah.hash();
        let eds_shares = u64::try_from(square.eds.data_square().len())
            .map_err(|_| super::Scenario2Err::Da("test EDS count overflow".to_string()))?;
        let mut payload_hasher = Sha256::new();
        payload_hasher.update(PAYLOAD_LABEL);
        payload_hasher.update(b"first");
        payload_hasher.update(b"second");
        let manifest = CelestiaBlockManifest {
            schema_version: MANIFEST_VERSION,
            celestia_types_version: CELESTIA_TYPES_VERSION.to_string(),
            cycle: 1,
            height: 1,
            batch_id: [1_u8; 32],
            pre_root: [2_u8; 32],
            post_root: [3_u8; 32],
            namespace: namespace
                .as_bytes()
                .try_into()
                .map_err(|_| super::Scenario2Err::Da("test namespace width drift".to_string()))?,
            share_version: 0,
            blob_count: 2,
            blob_commitments: square.commitments,
            package_count: 2,
            payload_bytes: 11,
            payload_digest: payload_hasher.finalize().into(),
            raw_tx_root: [4_u8; 32],
            exact_proof_root: [5_u8; 32],
            proof_bytes: 0,
            original_shares: u64::try_from(square.original_shares).map_err(|_| {
                super::Scenario2Err::Da("test original share count overflow".to_string())
            })?,
            padded_ods_shares: u64::try_from(square.ods_width * square.ods_width).map_err(
                |_| super::Scenario2Err::Da("test ODS share count overflow".to_string()),
            )?,
            eds_shares,
            ods_width: u16::try_from(square.ods_width)
                .map_err(|_| super::Scenario2Err::Da("test ODS width overflow".to_string()))?,
            eds_width: square.eds.square_width(),
            share_size: u32::try_from(super::SHARE_SIZE)
                .map_err(|_| super::Scenario2Err::Da("test share size overflow".to_string()))?,
            eds_bytes: eds_shares
                .checked_mul(
                    u64::try_from(super::SHARE_SIZE).map_err(|_| {
                        super::Scenario2Err::Da("test share size overflow".to_string())
                    })?,
                )
                .ok_or_else(|| super::Scenario2Err::Da("test EDS bytes overflow".to_string()))?,
            codec: square.eds.codec().to_string(),
            eds_file: EDS_FILE.to_string(),
            data_hash: celestia_hash(&dah_hash)?,
            dah: square.dah,
            pfb_included: false,
            network_published: false,
        };
        let temp =
            tempfile::tempdir().map_err(|error| super::Scenario2Err::Da(error.to_string()))?;
        let commit = persist_block(
            temp.path(),
            PreparedDa {
                eds: square.eds,
                manifest,
            },
        )?;

        verify_block(&commit, &config)?;
        assert!(path_exists_no_follow(&commit.path)?);
        assert!(path_exists_no_follow(&commit.eds_path)?);
        Ok(())
    }
}
