use jmt::{KeyHash, ValueHash};
use serde::Serialize;
use sha2::{Digest, Sha256};
use z00z_core::assets::AssetLeaf;
use z00z_crypto::{expert::hash_domain, hash_zk::hash_zk};
use z00z_storage::settlement::{
    derive_journal_digest_v1, derive_witness_root_v1, BatchPathEntryV1, BatchProofBlobV1,
    BatchProofFamilyTagV1, BatchProofHeaderV1, BatchProofLimits, BucketPolicy, BucketRootLeaf,
    DefinitionId, DefinitionRootLeaf, DeletionFactV1, InclusionOpeningV1, LeafFamilyTagV1,
    NodeDomainTagV1, NonExistenceOpeningV1, OpeningEntryV1, PathWitnessRefV1, PriorProofContextV1,
    ProofBlob, ProofChkErr, SerialId, SerialRootLeaf, SettlementLeaf, SettlementLeafFamily,
    SettlementPath, SettlementStateRoot, SettlementStore, SiblingSideTagV1, StoreItem,
    TerminalFamilyTagV1, TerminalId, TerminalLeaf, WitnessNodeV1,
};

const JMT_PLACEHOLDER_HASH: [u8; 32] = *b"SPARSE_MERKLE_PLACEHOLDER_HASH__";
const LEAF_DOMAIN_SEPARATOR: &[u8] = b"JMT::LeafNode";
const INTERNAL_DOMAIN_SEPARATOR: &[u8] = b"JMT::IntrnalNode";

hash_domain!(TestDefKeyDom, "z00z.storage.key.definition.v1", 1);
hash_domain!(TestSerKeyDom, "z00z.storage.key.serial.v1", 1);
hash_domain!(TestProofBindDom, "z00z.storage.proof.bind", 1);
hash_domain!(TestBatchProofDom, "z00z.storage.batch.proof", 1);

const WITNESS_CHUNK_LABEL: &str = "checkpoint_witness_chunk_v1";
const WITNESS_PAYLOAD_LABEL: &str = "checkpoint_witness_payload_v1";
const WITNESS_ROOT_LABEL: &str = "checkpoint_witness_root_v1";

#[derive(Serialize)]
struct CheckpointWitnessChunkV1 {
    version: u8,
    ordinal: u32,
    chunk_kind: u8,
    encoding_version: u8,
    byte_length: u32,
    content_digest: [u8; 32],
    linked_terminal_ids: Vec<TerminalId>,
}

impl CheckpointWitnessChunkV1 {
    fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&[self.version]));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&self.ordinal.to_le_bytes()));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&[self.chunk_kind]));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&[self.encoding_version]));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&self.byte_length.to_le_bytes()));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(&self.content_digest));
        bytes.extend_from_slice(&z00z_crypto::frame_bytes(
            &(self.linked_terminal_ids.len() as u32).to_le_bytes(),
        ));
        for terminal_id in &self.linked_terminal_ids {
            bytes.extend_from_slice(&z00z_crypto::frame_bytes(terminal_id.as_bytes()));
        }
        bytes
    }
}

pub fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

pub fn sample_policy() -> BucketPolicy {
    BucketPolicy::default_fixed()
}

pub fn sample_settlement_root() -> SettlementStateRoot {
    SettlementStateRoot::settlement_v1(bytes(17))
}

pub fn sample_journal_digest() -> [u8; 32] {
    bytes(18)
}

pub fn sample_path() -> SettlementPath {
    SettlementPath::new(
        DefinitionId::new(bytes(1)),
        SerialId::new(7),
        TerminalId::new(bytes(2)),
    )
}

pub fn terminal_leaf(path: SettlementPath) -> TerminalLeaf {
    let mut core = AssetLeaf::dummy_for_scan(path.serial_id.get());
    core.asset_id = path.terminal_id.into_bytes();
    TerminalLeaf::from(core)
}

pub fn sample_batch() -> BatchProofBlobV1 {
    let path = sample_path();
    let leaf = SettlementLeaf::Terminal(terminal_leaf(path));
    let key = terminal_key(path.terminal_id);
    let witness = terminal_witness(key, 0, bytes(3));
    let opening_table = vec![OpeningEntryV1::from_inclusion(
        InclusionOpeningV1::new(&leaf).expect("leaf bytes"),
    )];
    let witness_dag = vec![witness];
    let reference_table = vec![PathWitnessRefV1 {
        witness_indexes: vec![0],
    }];
    let backend_root =
        compute_current_backend_root(path, opening_hash_inclusion(path, &leaf), &witness_dag);
    let header = BatchProofHeaderV1::new(
        BatchProofFamilyTagV1::Inclusion,
        sample_settlement_root(),
        backend_root,
        vec![LeafFamilyTagV1::Asset],
        sample_policy(),
        Some(1),
        sample_journal_digest(),
        BatchProofLimits::v1(),
    );
    let path_table = vec![BatchPathEntryV1 {
        path,
        terminal_family: TerminalFamilyTagV1::Asset,
        leaf_family: LeafFamilyTagV1::Asset,
        shard_id: None,
        routing_generation: None,
        opening_index: 0,
        reference_index: 0,
    }];
    BatchProofBlobV1::new(
        header,
        path_table,
        witness_dag,
        opening_table,
        reference_table,
    )
}

pub fn sample_nonexistence_batch() -> BatchProofBlobV1 {
    let path = sample_path();
    let marker = SettlementLeafFamily::Terminal.marker_leaf(path);
    let key = terminal_key(path.terminal_id);
    let witness = terminal_witness(key, 0, bytes(4));
    let opening_table = vec![OpeningEntryV1::from_nonexistence(
        NonExistenceOpeningV1::new(&marker).expect("marker bytes"),
    )];
    let witness_dag = vec![witness];
    let reference_table = vec![PathWitnessRefV1 {
        witness_indexes: vec![0],
    }];
    let backend_root = compute_current_backend_root(path, JMT_PLACEHOLDER_HASH, &witness_dag);
    let header = BatchProofHeaderV1::new(
        BatchProofFamilyTagV1::NonExistence,
        sample_settlement_root(),
        backend_root,
        vec![LeafFamilyTagV1::Asset],
        sample_policy(),
        Some(1),
        sample_journal_digest(),
        BatchProofLimits::v1(),
    );
    let path_table = vec![BatchPathEntryV1 {
        path,
        terminal_family: TerminalFamilyTagV1::Asset,
        leaf_family: LeafFamilyTagV1::Asset,
        shard_id: None,
        routing_generation: None,
        opening_index: 0,
        reference_index: 0,
    }];
    BatchProofBlobV1::new(
        header,
        path_table,
        witness_dag,
        opening_table,
        reference_table,
    )
}

pub fn sample_voucher_nonexistence_batch() -> BatchProofBlobV1 {
    let path = sample_path();
    let marker = SettlementLeafFamily::Voucher.marker_leaf(path);
    let key = terminal_key(path.terminal_id);
    let witness = terminal_witness(key, 0, bytes(14));
    let opening_table = vec![OpeningEntryV1::from_nonexistence(
        NonExistenceOpeningV1::new(&marker).expect("marker bytes"),
    )];
    let witness_dag = vec![witness];
    let reference_table = vec![PathWitnessRefV1 {
        witness_indexes: vec![0],
    }];
    let backend_root = compute_current_backend_root(path, JMT_PLACEHOLDER_HASH, &witness_dag);
    let header = BatchProofHeaderV1::new(
        BatchProofFamilyTagV1::NonExistence,
        sample_settlement_root(),
        backend_root,
        vec![LeafFamilyTagV1::Voucher],
        sample_policy(),
        Some(1),
        sample_journal_digest(),
        BatchProofLimits::v1(),
    );
    let path_table = vec![BatchPathEntryV1 {
        path,
        terminal_family: TerminalFamilyTagV1::Voucher,
        leaf_family: LeafFamilyTagV1::Voucher,
        shard_id: None,
        routing_generation: None,
        opening_index: 0,
        reference_index: 0,
    }];
    BatchProofBlobV1::new(
        header,
        path_table,
        witness_dag,
        opening_table,
        reference_table,
    )
}

#[allow(dead_code)]
pub fn sample_deletion_batch(
    path: SettlementPath,
    leaf: SettlementLeaf,
    prior_context: PriorProofContextV1,
) -> BatchProofBlobV1 {
    let key = terminal_key(path.terminal_id);
    let witness = terminal_witness(key, 0, bytes(5));
    let opening_table = vec![OpeningEntryV1::from_deletion(
        DeletionFactV1::new(&leaf, prior_context).expect("deletion bytes"),
    )];
    let witness_dag = vec![witness];
    let reference_table = vec![PathWitnessRefV1 {
        witness_indexes: vec![0],
    }];
    let backend_root = compute_current_backend_root(path, JMT_PLACEHOLDER_HASH, &witness_dag);
    let header = BatchProofHeaderV1::new(
        BatchProofFamilyTagV1::Deletion,
        sample_settlement_root(),
        backend_root,
        vec![LeafFamilyTagV1::Asset],
        sample_policy(),
        Some(1),
        sample_journal_digest(),
        BatchProofLimits::v1(),
    );
    let path_table = vec![BatchPathEntryV1 {
        path,
        terminal_family: TerminalFamilyTagV1::Asset,
        leaf_family: LeafFamilyTagV1::Asset,
        shard_id: None,
        routing_generation: None,
        opening_index: 0,
        reference_index: 0,
    }];
    BatchProofBlobV1::new(
        header,
        path_table,
        witness_dag,
        opening_table,
        reference_table,
    )
}

#[allow(dead_code)]
pub fn live_prior_context_from_blob(prior_blob: &ProofBlob) -> PriorProofContextV1 {
    let journal_digest = prior_blob
        .hjmt_journal_digest()
        .expect("prior blob journal digest");
    let journal_checkpoint = prior_blob
        .hjmt_journal_checkpoint()
        .expect("prior blob journal checkpoint");
    PriorProofContextV1 {
        version: 1,
        prior_hjmt_version: journal_checkpoint,
        prior_settlement_root: prior_blob.item().settlement_root(),
        prior_backend_root: prior_blob.backend_root(),
        prior_root_bind_version: prior_blob.root_bind_ver(),
        prior_root_bind: prior_blob.root_bind(),
        prior_journal_digest: journal_digest,
        prior_checkpoint_bind: checkpoint_bind(
            prior_blob.item().settlement_root(),
            prior_blob.backend_root(),
            journal_checkpoint,
            journal_digest,
        ),
        definition_root_leaf_bytes: prior_blob.item().def_leaf().encode(),
        serial_root_leaf_bytes: prior_blob.item().ser_leaf().encode(),
        bucket_root_leaf_bytes: prior_blob
            .hjmt_bucket_root_leaf()
            .expect("prior blob bucket leaf")
            .encode(),
        definition_proof_bytes: prior_blob.definition_proof().to_vec(),
        serial_proof_bytes: prior_blob.serial_proof().to_vec(),
        bucket_proof_bytes: prior_blob
            .hjmt_bucket_proof()
            .expect("prior blob bucket proof")
            .to_vec(),
        prior_terminal_proof_bytes: prior_blob.terminal_proof().to_vec(),
    }
}

pub fn definition_key(definition_id: DefinitionId) -> KeyHash {
    KeyHash(hash_zk::<TestDefKeyDom>("", &[definition_id.as_bytes()]))
}

pub fn serial_key(definition_id: DefinitionId, serial_id: SerialId) -> KeyHash {
    let serial = serial_id.get().to_le_bytes();
    KeyHash(hash_zk::<TestSerKeyDom>(
        "",
        &[definition_id.as_bytes(), &serial],
    ))
}

pub fn terminal_key(terminal_id: TerminalId) -> KeyHash {
    KeyHash(terminal_id.into_bytes())
}

pub fn bucket_key(bucket_id: z00z_storage::settlement::BucketId) -> KeyHash {
    KeyHash(bucket_id.into_bytes())
}

pub fn compute_current_backend_root(
    path: SettlementPath,
    start_hash: [u8; 32],
    witness_dag: &[WitnessNodeV1],
) -> [u8; 32] {
    let policy = sample_policy();
    let mut current_hash = start_hash;
    let mut current_key = terminal_key(path.terminal_id);

    for witness in witness_dag {
        let bit = key_bit_from_leaf(current_key, witness.tree_level);
        current_hash = if bit == 0 {
            internal_node_hash(current_hash, witness.hash_material[0])
        } else {
            internal_node_hash(witness.hash_material[0], current_hash)
        };
    }

    let bucket_id = policy.derive_bucket_id(path);
    let bucket_leaf = BucketRootLeaf {
        definition_id: path.definition_id,
        serial_id: path.serial_id,
        bucket_id,
        terminal_jmt_root: current_hash,
        bucket_policy_id: policy.bucket_policy_id(),
    };
    current_key = bucket_key(bucket_id);
    current_hash = leaf_node_hash(current_key, &bucket_leaf.encode());

    let serial_leaf = SerialRootLeaf {
        definition_id: path.definition_id,
        serial_id: path.serial_id,
        serial_root: current_hash,
    };
    current_key = serial_key(path.definition_id, path.serial_id);
    current_hash = leaf_node_hash(current_key, &serial_leaf.encode());

    let def_leaf = DefinitionRootLeaf {
        definition_id: path.definition_id,
        definition_root: current_hash,
    };
    current_key = definition_key(path.definition_id);
    current_hash = leaf_node_hash(current_key, &def_leaf.encode());

    current_hash
}

pub fn opening_hash_inclusion(path: SettlementPath, leaf: &SettlementLeaf) -> [u8; 32] {
    leaf_node_hash(
        terminal_key(path.terminal_id),
        &leaf.encode().expect("encode settlement leaf"),
    )
}

pub fn terminal_witness(key: KeyHash, level: u16, sibling_hash: [u8; 32]) -> WitnessNodeV1 {
    let child_index = key_bit_from_leaf(key, level);
    let sibling_side = if child_index == 0 {
        SiblingSideTagV1::RightSibling
    } else {
        SiblingSideTagV1::LeftSibling
    };
    WitnessNodeV1 {
        tree_level: level,
        node_domain: NodeDomainTagV1::Terminal,
        child_index,
        sibling_side,
        subtree_marker: false,
        hash_material: vec![sibling_hash],
    }
}

pub fn key_bit_from_leaf(key: KeyHash, level: u16) -> u8 {
    let bit_index = 255usize.saturating_sub(level as usize);
    let byte = key.0[bit_index / 8];
    let shift = 7 - (bit_index % 8);
    u8::from(((byte >> shift) & 1) != 0)
}

pub fn leaf_node_hash(key: KeyHash, value_bytes: &[u8]) -> [u8; 32] {
    let value_hash = ValueHash::with::<Sha256>(value_bytes);
    let mut hasher = Sha256::new();
    hasher.update(LEAF_DOMAIN_SEPARATOR);
    hasher.update(key.0);
    hasher.update(value_hash.0);
    hasher.finalize().into()
}

pub fn internal_node_hash(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(INTERNAL_DOMAIN_SEPARATOR);
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().into()
}

#[allow(dead_code)]
fn checkpoint_bind(
    root: SettlementStateRoot,
    backend_root: [u8; 32],
    prior_hjmt_version: u64,
    journal_digest: [u8; 32],
) -> [u8; 32] {
    let generation = [root.generation_version()];
    let root_bytes = root.into_bytes();
    let version = prior_hjmt_version.to_be_bytes();
    hash_zk::<TestProofBindDom>(
        "proof_hjmt_checkpoint_bind_v1",
        &[
            &generation,
            &root_bytes,
            &backend_root,
            &version,
            &journal_digest,
        ],
    )
}

fn manual_witness_root(batches: &[BatchProofBlobV1]) -> [u8; 32] {
    let mut root_bytes = z00z_crypto::frame_bytes(&(batches.len() as u32).to_le_bytes());
    for (ordinal, batch) in batches.iter().enumerate() {
        let batch_bytes = batch.encode().expect("encode batch");
        let content_digest =
            hash_zk::<TestBatchProofDom>(WITNESS_PAYLOAD_LABEL, &[batch_bytes.as_slice()]);
        let chunk = CheckpointWitnessChunkV1 {
            version: 1,
            ordinal: ordinal as u32,
            chunk_kind: 1,
            encoding_version: batch.header.encoding_version,
            byte_length: batch_bytes.len() as u32,
            content_digest,
            linked_terminal_ids: batch
                .path_table
                .iter()
                .map(|entry| entry.path.terminal_id())
                .collect(),
        };
        let chunk_bytes = chunk.canonical_bytes();
        let chunk_hash =
            hash_zk::<TestBatchProofDom>(WITNESS_CHUNK_LABEL, &[chunk_bytes.as_slice()]);
        root_bytes.extend_from_slice(&z00z_crypto::frame_bytes(&chunk_hash));
    }
    hash_zk::<TestBatchProofDom>(WITNESS_ROOT_LABEL, &[root_bytes.as_slice()])
}

fn live_batches() -> [BatchProofBlobV1; 2] {
    let mut store = SettlementStore::new();
    let path_a = sample_path();
    let path_b = SettlementPath::new(
        DefinitionId::new(bytes(9)),
        SerialId::new(8),
        TerminalId::new(bytes(10)),
    );
    store
        .put_settlement_item(StoreItem::new(path_a, terminal_leaf(path_a)).expect("item a"))
        .expect("put item a");
    store
        .put_settlement_item(StoreItem::new(path_b, terminal_leaf(path_b)).expect("item b"))
        .expect("put item b");

    [
        store
            .settlement_inclusion_batch_v1(&[path_a])
            .expect("batch a"),
        store
            .settlement_inclusion_batch_v1(&[path_b])
            .expect("batch b"),
    ]
}

#[test]
fn test_witness_root_v1_matches_manual_vector_and_batch_order() {
    let batches = live_batches();
    let actual = derive_witness_root_v1(&batches).expect("witness root");
    let expected = manual_witness_root(&batches);
    let swapped = derive_witness_root_v1(&[batches[1].clone(), batches[0].clone()])
        .expect("swapped witness root");

    assert_eq!(actual, expected);
    assert_ne!(actual, swapped);
}

#[test]
fn test_journal_digest_v1_requires_checkpoint_and_consistent_lineage() {
    let batches = live_batches();
    let actual = derive_journal_digest_v1(&batches).expect("journal digest");
    assert_eq!(actual, batches[0].header.journal_digest);

    let mut missing = batches[1].clone();
    missing.header.journal_checkpoint = None;
    let err = derive_journal_digest_v1(&[batches[0].clone(), missing])
        .expect_err("missing checkpoint must reject");
    assert!(matches!(err, ProofChkErr::BatchCheckpointMix));

    let mut drifted = batches[1].clone();
    drifted.header.journal_digest = bytes(99);
    let err = derive_journal_digest_v1(&[batches[0].clone(), drifted])
        .expect_err("journal drift must reject");
    assert!(matches!(err, ProofChkErr::BatchCheckpointMix));
}
