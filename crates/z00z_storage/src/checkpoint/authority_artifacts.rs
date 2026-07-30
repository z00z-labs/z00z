//! Active recursive-checkpoint artifact identity.
//!
//! This authority input is deliberately outside the verifier source revision:
//! the verifier bundle commits that revision, while this independent pin selects
//! the complete generated bundle and prevents a source/bundle digest cycle.

use crate::CheckpointError;

/// Complete content digest of the active generation's verifier bundle.
///
/// The value is the role-framed `verifier-bundle` digest, not the raw file SHA-256.
pub(crate) const ACTIVE_VERIFIER_BUNDLE_DIGEST_V2: [u8; 32] = [
    0x98, 0x4f, 0x6a, 0x28, 0x29, 0x6e, 0x0d, 0x83, 0xbd, 0x2f, 0x38, 0x1e, 0xdd, 0x46, 0xe0, 0xfc,
    0x6a, 0x83, 0xf9, 0xa4, 0x0c, 0x17, 0x4f, 0xef, 0xd8, 0x12, 0xbd, 0x98, 0xe7, 0xe3, 0xa8, 0x19,
];

const VERIFIER_CATALOG_MAGIC_V2: [u8; 8] = *b"Z00ZVCA2";
const VERIFIER_CATALOG_WIRE_V2: u16 = 2;

/// Exact compiled verifier row accepted by historical history-proof reload.
///
/// The persisted authority bundle carries these bytes verbatim. Resolution is
/// an equality lookup against this compiled catalog; generation numbers never
/// synthesize a verifier row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Plonky3VerifierCatalogRowV2 {
    bundle_digest: [u8; 32],
    source_revision: &'static str,
    crate_version: &'static str,
    circuit_version: &'static str,
}

impl Plonky3VerifierCatalogRowV2 {
    #[must_use]
    pub(crate) const fn bundle_digest(self) -> [u8; 32] {
        self.bundle_digest
    }

    #[must_use]
    pub(crate) const fn source_revision(self) -> &'static str {
        self.source_revision
    }

    #[must_use]
    pub(crate) const fn crate_version(self) -> &'static str {
        self.crate_version
    }

    #[must_use]
    pub(crate) const fn circuit_version(self) -> &'static str {
        self.circuit_version
    }
}

pub(crate) const ACTIVE_VERIFIER_CATALOG_ROW_V2: Plonky3VerifierCatalogRowV2 =
    Plonky3VerifierCatalogRowV2 {
        bundle_digest: ACTIVE_VERIFIER_BUNDLE_DIGEST_V2,
        source_revision: ACTIVE_PLONKY3_SOURCE_REVISION_V2,
        crate_version: ACTIVE_PLONKY3_CRATE_VERSION_V2,
        circuit_version: ACTIVE_PLONKY3_CIRCUIT_VERSION_V2,
    };

pub(crate) fn active_verifier_catalog_bytes_v2() -> Vec<u8> {
    let row = ACTIVE_VERIFIER_CATALOG_ROW_V2;
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(&VERIFIER_CATALOG_MAGIC_V2);
    bytes.extend_from_slice(&VERIFIER_CATALOG_WIRE_V2.to_le_bytes());
    put_catalog_text(&mut bytes, row.source_revision);
    put_catalog_text(&mut bytes, row.crate_version);
    put_catalog_text(&mut bytes, row.circuit_version);
    bytes.extend_from_slice(&row.bundle_digest);
    bytes
}

pub(crate) fn resolve_verifier_catalog_v2(
    persisted_bytes: &[u8],
    expected_bundle_digest: [u8; 32],
) -> Result<Plonky3VerifierCatalogRowV2, CheckpointError> {
    let row = ACTIVE_VERIFIER_CATALOG_ROW_V2;
    if expected_bundle_digest == [0; 32]
        || expected_bundle_digest != row.bundle_digest
        || persisted_bytes != active_verifier_catalog_bytes_v2()
    {
        return Err(CheckpointError::Authority);
    }
    Ok(row)
}

fn put_catalog_text(bytes: &mut Vec<u8>, value: &str) {
    let len = u16::try_from(value.len()).expect("compiled verifier catalog text fits u16");
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(value.as_bytes());
}

/// Generation-bound preprocessed-common/VK authority for the canonical
/// three-replica Plan-07 recursion tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Plonky3RootCommonAuthorityV2 {
    aggregation_generation: u8,
    fixed_point_depth: u16,
    replica_root_common: [u8; 32],
    first_fold_common: [u8; 32],
    final_root_common: [u8; 32],
}

impl Plonky3RootCommonAuthorityV2 {
    #[must_use]
    pub(crate) const fn aggregation_generation(self) -> u8 {
        self.aggregation_generation
    }

    #[must_use]
    pub(crate) const fn fixed_point_depth(self) -> u16 {
        self.fixed_point_depth
    }

    #[must_use]
    pub(crate) const fn replica_root_common(self) -> [u8; 32] {
        self.replica_root_common
    }

    #[must_use]
    pub(crate) const fn first_fold_common(self) -> [u8; 32] {
        self.first_fold_common
    }

    #[must_use]
    pub(crate) const fn final_root_common(self) -> [u8; 32] {
        self.final_root_common
    }
}

/// The isolated generation-16 diagnostic derived these values from three real
/// depth-three replica trees, both ordered replica folds, and the actual pinned
/// Plonky3 verifier. A circuit/shape change must rotate this complete manifest.
pub(crate) const PLONKY3_ROOT_AUTHORITY_V2: Plonky3RootCommonAuthorityV2 =
    Plonky3RootCommonAuthorityV2 {
        aggregation_generation: 16,
        fixed_point_depth: 3,
        replica_root_common: [
            0x74, 0x06, 0x5b, 0x93, 0x02, 0x1d, 0x83, 0xc7, 0x94, 0xed, 0xae, 0xe8, 0x37, 0x22,
            0x1c, 0x30, 0x63, 0xeb, 0xaa, 0x38, 0x80, 0x56, 0xeb, 0x55, 0xad, 0xec, 0xc6, 0xed,
            0x33, 0x3c, 0x1e, 0xb9,
        ],
        first_fold_common: [
            0xed, 0x37, 0x9d, 0x64, 0xfc, 0x0f, 0x73, 0xe0, 0x2e, 0xb1, 0xba, 0x0e, 0x35, 0xcd,
            0x5d, 0x45, 0xd0, 0x12, 0xdf, 0x92, 0x5a, 0x6f, 0x33, 0x28, 0xa6, 0x5d, 0x30, 0x23,
            0x20, 0xdc, 0x10, 0x5a,
        ],
        final_root_common: [
            0xe6, 0xae, 0xdf, 0x85, 0x11, 0xff, 0x37, 0x5d, 0xe2, 0x4d, 0x88, 0xbd, 0x1a, 0x05,
            0x8c, 0x96, 0xb3, 0xa4, 0xc2, 0x8f, 0x9d, 0x3a, 0xef, 0x01, 0x13, 0xc1, 0xb4, 0x8a,
            0x2f, 0xd7, 0xe5, 0xf9,
        ],
    };

/// Exact upstream revision selected by the live Plonky3 base-proof authority.
pub(crate) const ACTIVE_PLONKY3_SOURCE_REVISION_V2: &str =
    "b36339709a7a67ee9760fb578b3d4339fd983709";

/// One canonical crates.io family for every direct Plonky3 dependency.
pub(crate) const ACTIVE_PLONKY3_CRATE_VERSION_V2: &str = "0.6.1";

/// Exact circuit/circuit-prover API family at the pinned upstream revision.
pub(crate) const ACTIVE_PLONKY3_CIRCUIT_VERSION_V2: &str = "0.1.0";
