use std::sync::{Mutex, OnceLock};

use z00z_storage::settlement::{
    AdaptiveBucket, BucketEpoch, BucketOccupancyEvidence, BucketOccupancyMetric, DefinitionId,
    FeeEnvelope, MergeProof, PolicyTransitionProof, RightClass, RightLeaf, RootGeneration,
    SerialId, SettlementLeaf, SettlementPath, SettlementStateRoot, SplitProof, TerminalId,
    TerminalLeaf, VoucherLeaf,
};

use z00z_storage::fixture_support::guardrail::{
    assert_absent, assert_all_absent, assert_all_present, assert_each_absent, assert_present,
};
use z00z_utils::io::write_file;

const ASSET_MOD: &str = include_str!("../src/settlement/mod.rs");
const BACKEND_ERROR: &str = include_str!("../src/backend/error.rs");
const BACKEND_MOD: &str = include_str!("../src/backend/mod.rs");
const CRATE_LIB: &str = include_str!("../src/lib.rs");
const SERIALIZATION_MOD: &str = include_str!("../src/serialization/mod.rs");
const TYPES_IDENTITY: &str = include_str!("../src/settlement/identity.rs");
const TYPES_QUERY: &str = include_str!("../src/settlement/query.rs");
const TYPES_RECORD: &str = include_str!("../src/settlement/record.rs");
const PROOF_RS: &str = include_str!("../src/settlement/proof.rs");
const PROOF_BATCH_RS: &str = include_str!("../src/settlement/proof_batch.rs");
const PROOF_BATCH_VERIFY_RS: &str = include_str!("../src/settlement/proof_batch_verify.rs");
const SNAPSHOT_STORE: &str = include_str!("../src/snapshot/store.rs");
const CHECKPOINT_BUILD: &str = include_str!("../src/checkpoint/build.rs");
const CHECKPOINT_BUILD_STATE: &str = include_str!("../src/checkpoint/build_state.rs");
const ROOT_ERROR: &str = include_str!("../src/error.rs");
const ROOT_CRATE_README: &str = include_str!("../README.md");
const ROOT_TYPES_DOC: &str = include_str!("../src/settlement/root_types.md");
const README_DOC: &str = include_str!("../src/settlement/README.md");
const BENCHMARKS_DOC: &str = include_str!("../../../docs/tech-papers/benchmarks.md");
const DESIGN_DOC: &str = include_str!("../../../docs/tech-papers/done/Z00Z-HJMT-Design.md");
const PHASE0_SOURCE_DOC: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/GAPS.md");
const PHASE_SOURCE_DOC: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/GAPS.md");
const PHASE_EXEC_TODO: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-TODO.md");
const PHASE_062_THIN_DOC: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/Z00Z-Thin-Transaction-Mode.md");
const PHASE_062_PLAN_03: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-03-PLAN.md");
const PHASE_062_PLAN_13: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-13-PLAN.md");
const PHASE_062_PLAN_15: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-15-PLAN.md");
const PHASE_062_PLAN_16: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-16-PLAN.md");
const PHASE_062_PLAN_17: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-17-PLAN.md");
const PHASE_062_PLAN_18: &str =
    include_str!("../../../.planning/phases/000/062-Gaps-Closing-2/062-18-PLAN.md");
const META_FLOW: &str = include_str!("../../../.github/workflows/security-hygiene-guards.yml");
const AUDIT_SECRET_TYPE: &str = include_str!("../../../scripts/audit/audit_secret_type_hygiene.sh");
const AUDIT_SECRET_EQ: &str = include_str!("../../../scripts/audit/audit_secret_eq_hygiene.sh");
const AUDIT_CRYPTO_RNG: &str = include_str!("../../../scripts/audit/audit_crypto_rng_hygiene.sh");
const AUDIT_BOUNDARY_PANIC: &str =
    include_str!("../../../scripts/audit/audit_boundary_panic_hygiene.sh");
const AUDIT_LOG_REDACT: &str =
    include_str!("../../../scripts/audit/audit_log_redaction_hygiene.sh");
const STORE_MOD: &str = include_str!("../src/settlement/store.rs");
const STORE_HJMT_CACHE: &str = include_str!("../src/settlement/hjmt_cache.rs");
const STORE_HJMT_COMMIT: &str = include_str!("../src/settlement/hjmt_commit.rs");
const STORE_HJMT_JOURNAL: &str = include_str!("../src/settlement/hjmt_journal.rs");
const STORE_HJMT_PLAN: &str = include_str!("../src/settlement/hjmt_plan.rs");
const STORE_HJMT_POLICY: &str = include_str!("../src/settlement/hjmt_policy.rs");
const STORE_HJMT_STORE: &str = include_str!("../src/settlement/hjmt_store.rs");

const VALIDATOR_FLOW: &str = include_str!("../../z00z_runtime/validators/src/checkpoint.rs");
const VALIDATOR_VERDICTS: &str = include_str!("../../z00z_runtime/validators/src/verdict.rs");
const VALIDATOR_ENGINE: &str = include_str!("../../z00z_runtime/validators/src/engine.rs");
const WATCHER_PUBLICATION: &str = include_str!("../../z00z_runtime/watchers/src/publication.rs");
const WATCHER_EVIDENCE: &str = include_str!("../../z00z_runtime/watchers/src/evidence_export.rs");
const WATCHER_ENGINE: &str = include_str!("../../z00z_runtime/watchers/src/engine.rs");
const AGGREGATOR_TYPES: &str = include_str!("../../z00z_runtime/aggregators/src/types.rs");
const WALLET_AUDIT: &str = include_str!("../../z00z_wallets/src/tx/commit_audit.rs");
const WALLET_PROOF: &str = include_str!("../../z00z_wallets/src/tx/claim_tx_verify_proof.rs");
const WALLET_WITNESS: &str = include_str!("../../z00z_wallets/src/tx/state_witness.rs");
const WALLET_RESOLVED_INPUT: &str =
    include_str!("../../z00z_wallets/src/tx/state_resolved_input.rs");
const WALLET_SPEND_BACKEND: &str = include_str!("../../z00z_wallets/src/tx/spend_proof_backend.rs");
const SIM_STAGE4_PREP: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_6/tx_preparation_core.rs");
const SIM_STAGE11_SCAN: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_11/jmt_wallet_scan.rs");
const SIM_STAGE11_CHARLIE: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_11/charlie.rs");
const SIM_STAGE13_STORAGE: &str =
    include_str!("../../z00z_simulator/src/scenario_1/stage_13/storage.rs");
const SIM_UNIFIED_GATE: &str =
    include_str!("../../z00z_simulator/tests/scenario_1/test_scenario1_unified_gate.rs");

fn bytes(value: u8) -> [u8; 32] {
    [value; 32]
}

fn section<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .unwrap_or_else(|| panic!("missing section start {start:?}"))
        .split(end)
        .next()
        .unwrap_or_else(|| panic!("missing section end {end:?}"))
}

fn assert_runtime_owner_crate_absent(plan_name: &str, plan_source: &str) {
    assert!(
        !plan_source.contains("owner_crate: `z00z_runtime`"),
        "{plan_name} must map runtime artifacts to real Cargo packages"
    );
}

fn env_case_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarReset {
    key: &'static str,
    prior: Option<String>,
}

impl Drop for EnvVarReset {
    fn drop(&mut self) {
        match &self.prior {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

fn set_env_var(key: &'static str, value: String) -> EnvVarReset {
    let prior = std::env::var(key).ok();
    std::env::set_var(key, value);
    EnvVarReset { key, prior }
}

#[test]
fn test_seam_below_semantic_surface() {
    assert_present("crate lib", CRATE_LIB, "pub mod backend;");
    assert_all_present(
        "backend mod",
        BACKEND_MOD,
        &[
            "pub trait ReadTxn",
            "pub trait WriteTxn",
            "pub trait StorageBackend",
            "pub trait JournalBackend",
        ],
    );
    assert_all_absent(
        "backend mod",
        BACKEND_MOD,
        &[
            "SettlementStore",
            "SettlementStateRoot",
            "SettlementPath",
            "ProofBlob",
            "RightLeaf",
            "FeeEnvelope",
            "backend_root",
        ],
    );
    assert_absent(
        "root error",
        ROOT_ERROR,
        "pub(crate) use crate::backend::error::StoreBackendError;",
    );
    assert_absent(
        "root error",
        ROOT_ERROR,
        "pub(crate) enum StoreBackendError",
    );
    assert_present(
        "backend error",
        BACKEND_ERROR,
        "pub(crate) enum StoreBackendError",
    );
}

#[test]
fn test_live_contract_exports_compile() {
    let root = SettlementStateRoot::settlement_v1(bytes(1));
    assert_eq!(root.generation(), RootGeneration::SettlementV1);
    assert_eq!(root.into_bytes(), bytes(1));
    assert_eq!(
        RootGeneration::from_version(1),
        Some(RootGeneration::SettlementV1)
    );
    let v2_root = SettlementStateRoot::settlement_v2(bytes(2));
    assert_eq!(v2_root.generation(), RootGeneration::SettlementV2);
    assert_eq!(v2_root.into_bytes(), bytes(2));
    assert_eq!(
        RootGeneration::from_version(2),
        Some(RootGeneration::SettlementV2)
    );

    let terminal_id = TerminalId::new(bytes(2));
    let path = SettlementPath::new(DefinitionId::new(bytes(3)), SerialId::new(7), terminal_id);
    assert_eq!(path.terminal_id.into_bytes(), bytes(2));

    let right = RightLeaf {
        version: 1,
        terminal_id,
        right_class: RightClass::MachineCapability,
        issuer_scope: bytes(5),
        provider_scope: bytes(6),
        holder_commitment: bytes(7),
        control_commitment: bytes(8),
        beneficiary_commitment: bytes(9),
        payload_commitment: bytes(10),
        valid_from: 10,
        valid_until: 20,
        challenge_from: 12,
        challenge_until: 18,
        use_nonce: bytes(11),
        revocation_policy_id: bytes(12),
        transition_policy_id: bytes(13),
        challenge_policy_id: bytes(14),
        disclosure_policy_id: bytes(15),
        retention_policy_id: bytes(16),
    };
    let leaf = SettlementLeaf::Right(right);
    assert!(matches!(leaf, SettlementLeaf::Right(_)));
    assert!(matches!(
        SettlementLeaf::Terminal(TerminalLeaf::default()),
        SettlementLeaf::Terminal(_)
    ));
    assert!(matches!(
        SettlementLeaf::Voucher(VoucherLeaf::marker(path)),
        SettlementLeaf::Voucher(_)
    ));

    let _fee = FeeEnvelope {
        version: 1,
        payer_commitment: bytes(17),
        sponsor_commitment: bytes(18),
        budget_units: 1,
        budget_commitment: bytes(19),
        domain_id: bytes(20),
        expires_at: 30,
        nonce: bytes(21),
        transition_id: bytes(22),
        replay_key: bytes(23),
        support_ref: Some(bytes(25)),
        failure_policy_id: bytes(24),
    };
    let _bucket_epoch = BucketEpoch::new(1);
    let _ = core::mem::size_of::<AdaptiveBucket>();
    let _ = core::mem::size_of::<BucketOccupancyEvidence>();
    let _ = core::mem::size_of::<BucketOccupancyMetric>();
    let _ = core::mem::size_of::<SplitProof>();
    let _ = core::mem::size_of::<MergeProof>();
    let _ = core::mem::size_of::<PolicyTransitionProof>();
}

#[test]
fn test_phase62_cutover_terms_live() {
    assert!(
        PHASE_062_THIN_DOC.contains("canonical `SettlementPath`"),
        "thin-mode phase paper must use SettlementPath as the live canonical path family"
    );
    assert!(
        PHASE_062_THIN_DOC.contains("`SettlementStateRoot` is the live semantic settlement root."),
        "thin-mode phase paper must keep SettlementStateRoot as the live root vocabulary"
    );
    assert!(
        !PHASE_062_THIN_DOC.contains("canonical `AssetPath`"),
        "thin-mode phase paper must not present AssetPath as the live canonical path family"
    );
    assert!(
        !PHASE_062_THIN_DOC.contains("AssetStateRoot` is the live asset-centric semantic root"),
        "thin-mode phase paper must not revive AssetStateRoot as live settlement truth"
    );
    assert!(
        !PHASE_062_THIN_DOC.contains("future generalized vocabulary"),
        "thin-mode phase paper must not demote SettlementStateRoot back into future vocabulary"
    );
}

#[test]
fn test_phase62_owner_packages() {
    for (plan_name, plan_source) in [
        ("062-03", PHASE_062_PLAN_03),
        ("062-13", PHASE_062_PLAN_13),
        ("062-15", PHASE_062_PLAN_15),
        ("062-16", PHASE_062_PLAN_16),
        ("062-17", PHASE_062_PLAN_17),
        ("062-18", PHASE_062_PLAN_18),
    ] {
        assert_runtime_owner_crate_absent(plan_name, plan_source);
    }
}

#[test]
fn test_proof_surface_additive() {
    assert_all_present(
        "settlement mod",
        ASSET_MOD,
        &[
            "mod proof_batch;",
            "mod proof_batch_verify;",
            "BatchProofBlobV1",
            "BatchProofHeaderV1",
            "check_batch_contract_v1",
        ],
    );
    assert_present("proof rs", PROOF_RS, "pub struct ProofBlob");
    assert_present("proof batch", PROOF_BATCH_RS, "pub struct BatchProofBlobV1");
    assert_present(
        "proof batch verify",
        PROOF_BATCH_VERIFY_RS,
        "pub fn check_batch_contract_v1",
    );
    assert_absent("proof batch", PROOF_BATCH_RS, "pub type ProofBlob");
    assert_absent("proof batch", PROOF_BATCH_RS, "type ProofBlob =");
}

#[test]
fn test_source_needs_live_names() {
    for term in [
        "SettlementStateRoot",
        "RootGeneration",
        "SettlementPath",
        "TerminalId",
        "SettlementLeaf",
        "RightLeaf",
        "FeeEnvelope",
        "AdaptiveBucket",
        "BucketEpoch",
        "BucketOccupancyEvidence",
        "BucketOccupancyMetric",
        "SplitProof",
        "MergeProof",
        "PolicyTransitionProof",
        "chk_blob_settlement",
        "chk_item_settlement",
    ] {
        assert_present("assets module", ASSET_MOD, term);
        if !(TYPES_IDENTITY.contains(term) || TYPES_RECORD.contains(term)) {
            assert_present("storage docs", README_DOC, term);
        }
    }
}

#[test]
fn test_source_rejects_public_exports() {
    assert_all_absent(
        "assets module",
        ASSET_MOD,
        &[
            "RootApi",
            "RootRec",
            "CompatRoot",
            "chk_blob,",
            "chk_blob_item,",
            "chk_item,",
            "CompatProofFamily",
            "COMPAT_PROOF_ENVELOPE_VERSION",
            "check_compat_proof_family",
        ],
    );

    assert_absent("assets module", ASSET_MOD, "pub mod compat");

    assert!(
        !SERIALIZATION_MOD.contains("#[cfg(feature = \"test-params-fast\")]")
            && SERIALIZATION_MOD.contains("pub use self::build::build_artifact;")
            && SERIALIZATION_MOD.contains("intentional test surface"),
        "live serialization builder must stay documented as intentional test surface without feature gating"
    );

    assert_all_absent(
        "proof module",
        PROOF_RS,
        &[
            "pub(crate) fn chk_blob(",
            "pub(crate) fn chk_blob_item(",
            "pub(crate) fn chk_item(",
            "pub(crate) const fn compat_envelope_version",
            "pub(crate) const fn proof_family(",
        ],
    );

    assert_all_absent(
        "query module",
        TYPES_QUERY,
        &[
            "impl From<SettlementLookup> for AssetLookup",
            "impl From<SettlementScope> for AssetScope",
            "impl From<SettlementPageTok> for AssetPageTok",
            "impl From<AssetPageTok> for SettlementPageTok",
            "impl From<SettlementListReq> for AssetListReq",
            "impl From<AssetPage> for SettlementPage",
            "enum AssetLookup",
            "enum AssetScope",
            "struct AssetPageTok",
            "struct AssetListReq",
            "struct AssetPage",
        ],
    );
}

#[test]
fn test_fake_alias_shapes_reject() {
    assert_absent(
        "identity types",
        TYPES_IDENTITY,
        "pub type SettlementStateRoot",
    );
    assert_absent("record types", TYPES_RECORD, "pub type RightLeaf");
    assert_absent("record types", TYPES_RECORD, "pub type FeeEnvelope");
    assert_absent(
        "record types",
        TYPES_RECORD,
        "pub struct RightLeaf(TerminalLeaf",
    );
    assert_absent(
        "record types",
        TYPES_RECORD,
        "pub struct RightLeaf(z00z_core",
    );
    assert_absent("record types", TYPES_RECORD, "MigrationProof");

    let root_decl = TYPES_IDENTITY
        .split("pub struct SettlementStateRoot")
        .nth(1)
        .expect("settlement root declaration")
        .split("impl SettlementStateRoot")
        .next()
        .expect("settlement root impl boundary");
    assert!(root_decl.contains("generation: RootGeneration"));
    assert!(root_decl.contains("root: [u8; 32]"));
    assert_absent("settlement root declaration", root_decl, "AssetStateRoot");

    let right_decl = TYPES_RECORD
        .split("pub struct RightLeaf")
        .nth(1)
        .expect("right leaf declaration")
        .split("/// Separate processing-support envelope")
        .next()
        .expect("right leaf boundary");
    assert_absent("right leaf declaration", right_decl, "TerminalLeaf");
    assert_absent("right leaf declaration", right_decl, "FeeEnvelope");
    assert_absent("right leaf declaration", right_decl, "payer");
    assert_absent("right leaf declaration", right_decl, "sponsor");
    assert_absent("right leaf declaration", right_decl, "relay");
    assert_absent("right leaf declaration", right_decl, "budget");
}

#[test]
fn test_store_backend_paths_canon() {
    assert_each_absent(
        &[
            ("store mod", STORE_MOD),
            ("hjmt cache", STORE_HJMT_CACHE),
            ("hjmt commit", STORE_HJMT_COMMIT),
            ("hjmt journal", STORE_HJMT_JOURNAL),
            ("hjmt plan", STORE_HJMT_PLAN),
            ("hjmt policy", STORE_HJMT_POLICY),
            ("hjmt store", STORE_HJMT_STORE),
        ],
        &[
            "store_codec::",
            "store_roots::",
            "store_mem::",
            " as store_codec",
            " as store_roots",
            " as store_mem",
        ],
    );
}

#[test]
fn test_handoff_surface_semantic() {
    assert_all_present(
        "settlement mod",
        ASSET_MOD,
        &[
            "SettlementExecHandoff",
            "SettlementRouteCtx",
            "ScopeFlow",
            "ScopeFlowItem",
            "ScopeLeafKind",
        ],
    );
    assert_all_present(
        "store mod",
        STORE_MOD,
        &[
            "pub struct SettlementExecHandoff",
            "pub struct SettlementRouteCtx",
            "pub struct ScopeFlow",
            "pub fn apply_exec_handoff(",
        ],
    );
    assert_all_absent(
        "store mod",
        STORE_MOD,
        &["ShardRouteTable", "lookup_live(", "HjmtTreeId"],
    );
    assert_all_present(
        "storage README",
        README_DOC,
        &["SettlementExecHandoff", "apply_exec_handoff", "ScopeFlow"],
    );
}

#[test]
fn test_recovery_exports_lineage() {
    assert_all_present(
        "settlement mod",
        ASSET_MOD,
        &[
            "SettlementRecoveryState",
            "SettlementRouteCtx",
            "SettlementTreeBackend",
        ],
    );
    assert_all_present(
        "store mod",
        STORE_MOD,
        &[
            "pub struct SettlementRecoveryState",
            "pub fn recovery_state(&self) -> Result<SettlementRecoveryState, SettlementStoreError>",
            "pub journal_lineage: [u8; 32]",
            "crate::backend::JournalBackend::recover_journal(&backend)?;",
        ],
    );
    assert_all_present(
        "storage README",
        README_DOC,
        &[
            "SettlementRecoveryState",
            "SettlementRouteCtx",
            "SettlementStore::recovery_state()",
            "journal_lineage",
        ],
    );
}

#[test]
fn test_journal_baseline_stays_local() {
    assert_all_present(
        "backend mod",
        BACKEND_MOD,
        &[
            "pub trait JournalBackend",
            "local durable journal as the baseline implementation",
            "must not become independent protocol truth",
        ],
    );
    assert_present(
        "redb backend",
        include_str!("../src/backend/redb/mod.rs"),
        "impl JournalBackend for StoragePlane",
    );
    assert_all_present(
        "storage README",
        README_DOC,
        &[
            "RedB-backed local durable journal as the baseline implementation",
            "A shared cross-aggregator WAL is not live protocol truth.",
            "Recovery metadata must not become a second semantic authority beside the active settlement root.",
        ],
    );
}

#[test]
fn test_sources_keep_layout_private() {
    assert_each_absent(
        &[
            ("validator flow", VALIDATOR_FLOW),
            ("validator verdict", VALIDATOR_VERDICTS),
            ("validator engine", VALIDATOR_ENGINE),
            ("wallet audit", WALLET_AUDIT),
            ("wallet proof", WALLET_PROOF),
            ("scenario_1 stage4 prep", SIM_STAGE4_PREP),
            ("scenario_1 stage11 scan", SIM_STAGE11_SCAN),
            ("scenario_1 stage13 storage", SIM_STAGE13_STORAGE),
        ],
        &[
            "TreeId",
            "HjmtTreeId",
            "HjmtTreeId",
            "ns_key",
            "store_internal",
            "StorDefNsDom",
            "StorSerNsDom",
            "StorTerminalNsDom",
            "namespace prefix",
            "branch ordering",
            "physical key",
            "flat_root_hash",
            "flat_root_hex",
            "redb_key",
            "cache_state",
        ],
    );
}

#[test]
fn test_downstream_binding_stays_shared() {
    let publication_binding_block = section(
        AGGREGATOR_TYPES,
        "pub struct PublicationBinding {",
        "impl PublicationBinding {",
    );

    assert_present(
        "validator flow",
        VALIDATOR_FLOW,
        "bind_publication_contract(",
    );
    assert_present(
        "validator flow",
        VALIDATOR_FLOW,
        "publication: bind_publication_contract(",
    );
    assert_present(
        "watcher publication",
        WATCHER_PUBLICATION,
        "if !binding.matches_pub_in(&published.pub_in)",
    );
    assert_present(
        "watcher evidence",
        WATCHER_EVIDENCE,
        ".map(PublicationBinding::binding_digest)",
    );
    assert_present(
        "watcher engine",
        WATCHER_ENGINE,
        "publication.publication.binding_digest()",
    );
    for field in [
        "pub batch_id:",
        "pub checkpoint_id:",
        "pub route_table_digest:",
        "pub prev_settlement_root:",
        "pub new_settlement_root:",
        "pub claim_root:",
        "pub spent_count:",
        "pub created_count:",
        "pub pub_in_digest:",
        "pub binding_digest:",
    ] {
        assert_absent(
            "aggregator publication binding",
            publication_binding_block,
            field,
        );
    }

    assert_each_absent(
        &[
            ("validator flow", VALIDATOR_FLOW),
            ("validator verdict", VALIDATOR_VERDICTS),
            ("validator engine", VALIDATOR_ENGINE),
            ("watcher publication", WATCHER_PUBLICATION),
            ("watcher evidence", WATCHER_EVIDENCE),
            ("watcher engine", WATCHER_ENGINE),
        ],
        &[
            "PublicationBinding::new(",
            "digest_binding(",
            "digest_pub_in(",
            "Sha256",
            "CheckpointPublicationV1::new(",
            "ShardRootLeafV1::new(",
        ],
    );
}

#[test]
fn test_docs_promote_settle_terms() {
    assert_all_present(
        "root crate README",
        ROOT_CRATE_README,
        &[
            "live settlement-first contract",
            "SettlementPath",
            "SettlementStateRoot",
        ],
    );
    assert_absent(
        "root crate README",
        ROOT_CRATE_README,
        "canonical consensus path is `AssetPath`",
    );
    assert_absent(
        "root crate README",
        ROOT_CRATE_README,
        "only `AssetStateRoot` is exported as the state-root type",
    );

    for doc in [ROOT_TYPES_DOC, README_DOC] {
        assert_all_present(
            "storage docs",
            doc,
            &[
                "live settlement",
                "SettlementStateRoot",
                "RightLeaf",
                "VoucherLeaf",
                "FeeEnvelope",
            ],
        );
        assert_all_absent(
            "storage docs",
            doc,
            &[
                "Future-Only Terms",
                "future terminology only",
                "remain future",
                "not exported by this asset-centric",
            ],
        );
    }

    assert_all_absent(
        "storage README",
        README_DOC,
        &[
            "CompatRoot",
            "RootApi",
            "RootRec",
            "CompatProofFamily",
            "COMPAT_PROOF_ENVELOPE_VERSION",
            "check_compat_proof_family",
            "chk_blob(bytes",
            "chk_blob_item(",
            "chk_item(",
            "store.check_root()",
            "store.asset_root()",
            "put_item(item)",
            "apply_ops(Vec<StoreOp>)",
            "AssetLookup::",
            "AssetListReq",
            "AssetPageTok",
            "AssetPath remains the older asset-centric address during ordered cutover tasks",
            "AssetStateRoot` is the canonical storage state commitment",
            "P[AssetPath]",
            "ASR[AssetStateRoot]",
            "derived from `AssetStateRoot`",
            "compat::AssetPath",
            "compat::AssetStateRoot",
        ],
    );

    assert_all_present(
        "storage README",
        README_DOC,
        &[
            "chk_blob_settlement",
            "chk_item_settlement",
            "asset-path adapters",
            "runtime or canonical test surface",
        ],
    );

    assert_all_present(
        "design doc operator notes",
        DESIGN_DOC,
        &[
            "Z00Z_SETTLEMENT_BACKEND_MODE",
            "Z00Z_SETTLEMENT_BUCKET_BITS",
            "Z00Z_STORAGE_SCHED_CPU",
            "Z00Z_STORAGE_SCHED_QUEUE",
        ],
    );
    assert_all_absent(
        "design doc operator notes",
        DESIGN_DOC,
        &["Z00Z_ASSET_BACKEND_MODE", "Z00Z_ASSET_BUCKET_BITS"],
    );
}

#[test]
fn test_phase0_promotes_authority() {
    let phase0_block = section(
        PHASE0_SOURCE_DOC,
        "### TASK-001 - 0. Storage Migration Boundary, Authority Facade, And Forest Backend Rollout",
        "### TASK-005 - 9. Storage Claim-Root And Checkpoint Authority Closure",
    );
    assert_all_present(
        "phase source section 0",
        phase0_block,
        &[
            "SettlementStateRoot",
            "SettlementPath",
            "backend_root",
            "Closed as normalized/superseded by live settlement root and HJMT backend.",
            "Replace any closure wording that treats `AssetStateRoot` or `AssetPath` as the live public runtime root.",
        ],
    );
    assert_all_absent(
        "phase source section 0",
        phase0_block,
        &[
            "AssetPath { definition_id, serial_id, asset_id }",
            "terminal `AssetLeaf` as the storage-owned consensus path",
        ],
    );
}

#[test]
fn test_phase36_closeout_canonical() {
    assert_all_present(
        "phase source closeout gate",
        PHASE_SOURCE_DOC,
        &[
            "### TASK-063 - 36. Spec-Gap Normalization And Residual Hardening Gate",
            "### TASK-065 - 36. Spec-Gap Normalization And Residual Hardening Gate",
            "### TASK-066 - 36. Spec-Gap Normalization And Residual Hardening Gate",
            "Detailed gap closure execution plan: .planning/phases/TODO-gaps.md",
            "active execution plan for `👍` sections.",
            "Closeout status: Bounded closed",
            "Residual gap register",
            "Recursive proof backend",
            "Linked Liability",
            "OnionNet",
            "live external DA",
            "live cross-chain bridge",
            "field-native/Poseidon2 parity",
            "useful-work scenario",
        ],
    );
    assert_all_absent(
        "phase source closeout gate",
        PHASE_SOURCE_DOC,
        &["OnionNet transport is live"],
    );
    assert_all_present(
        "phase execution todo",
        PHASE_EXEC_TODO,
        &[
            ".planning/phases/062-Gaps-Closing-2/GAPS.md",
            ".planning/phases/TODO-gaps.md",
            "legacy unless intentionally retained as a historical pointer",
        ],
    );
}

#[test]
fn test_wallet_witness_settle_first() {
    assert_all_present(
        "wallet witness",
        WALLET_WITNESS,
        &["SettlementPath", "SettlementStateRoot"],
    );
    assert_present(
        "wallet resolved input",
        WALLET_RESOLVED_INPUT,
        "SettlementPath",
    );
    assert_present(
        "wallet spend backend",
        WALLET_SPEND_BACKEND,
        "SettlementPath",
    );

    assert_each_absent(
        &[
            ("wallet witness", WALLET_WITNESS),
            ("wallet resolved input", WALLET_RESOLVED_INPUT),
            ("wallet spend backend", WALLET_SPEND_BACKEND),
        ],
        &["AssetPath", "AssetStateRoot"],
    );
}

#[test]
fn test_artifact_lane_settle_first() {
    let store_item_block = section(
        TYPES_RECORD,
        "pub struct StoreItem",
        "/// Snapshot-facing record",
    );
    assert_present(
        "store item live API",
        store_item_block,
        "path: SettlementPath",
    );
    assert_absent("store item live API", store_item_block, "asset_path(");

    let snap_decl = section(TYPES_RECORD, "pub struct SnapItem", "impl SnapItem");
    assert_present("snap item declaration", snap_decl, "path: SettlementPath");
    assert_absent("snap item declaration", snap_decl, "path: AssetPath");

    let proof_decl = section(TYPES_RECORD, "pub struct ProofItem", "impl ProofItem");
    assert_all_present(
        "proof item declaration",
        proof_decl,
        &[
            "settlement_state_root: SettlementStateRoot",
            "path: SettlementPath",
        ],
    );
    assert_all_absent(
        "proof item declaration",
        proof_decl,
        &["asset_state_root:", "path: AssetPath", "AssetStateRoot"],
    );
    assert_all_present(
        "record types",
        TYPES_RECORD,
        &[
            "pub fn new_settlement(",
            "pub const fn root(&self) -> SettlementStateRoot",
            "pub const fn path(&self) -> SettlementPath",
        ],
    );

    let checkpoint_artifacts = section(
        CHECKPOINT_BUILD,
        "pub struct MemberWit",
        "/// Minimal tx package summary",
    );
    assert_all_present(
        "checkpoint artifact lane",
        checkpoint_artifacts,
        &[
            "pub fn proof_root(&self) -> SettlementStateRoot",
            "root: SettlementStateRoot",
            "path: &SettlementPath",
            "path: SettlementPath",
            "pub const fn path(&self) -> SettlementPath",
            "ResolvedInput",
            "MemberWit",
        ],
    );
    assert_all_absent(
        "checkpoint artifact lane",
        checkpoint_artifacts,
        &["AssetPath", "AssetStateRoot", "assets::compat"],
    );

    assert_all_present(
        "checkpoint build state",
        CHECKPOINT_BUILD_STATE,
        &[
            "pub(super) fn proof_root(prev_root: CheckRoot) -> SettlementStateRoot",
            "SettlementStateRoot::settlement_v1(prev_root.into_bytes())",
            "let path = SettlementPath::new(",
        ],
    );
    assert_all_absent(
        "checkpoint build state",
        CHECKPOINT_BUILD_STATE,
        &["AssetPath", "AssetStateRoot", "assets::compat"],
    );

    assert_all_present(
        "snapshot store",
        SNAPSHOT_STORE,
        &[
            "chk_blob_settlement_inclusion(",
            "if proof_path.terminal_id() != entry.path().terminal_id()",
            "&entry.path(),",
        ],
    );
    assert_all_absent(
        "snapshot store",
        SNAPSHOT_STORE,
        &["AssetPath", "AssetStateRoot", "assets::compat"],
    );

    let stage4_artifacts = section(
        SIM_STAGE4_PREP,
        "fn claim_store_snap_items",
        "pub(crate) fn build_canon_snapshot",
    );
    assert_all_present(
        "stage4 artifact lane",
        stage4_artifacts,
        &[
            ".settlement_proof_blob(&path)",
            "SnapItem::new(path, item.leaf().clone(), wit)",
        ],
    );
    assert_all_absent(
        "stage4 artifact lane",
        stage4_artifacts,
        &["AssetPath", "AssetStateRoot", "assets::compat"],
    );
}

#[test]
fn test_stage11_labels_settle_proof() {
    let new_label = "proof_blob+chk_blob_settlement before runtime ownership detection";
    let old_label = "proof_blob+chk_blob before runtime ownership detection";

    assert!(
        SIM_STAGE11_CHARLIE.contains(new_label),
        "stage11 charlie artifact must publish settlement proof validation"
    );
    assert_absent("stage11 charlie artifact", SIM_STAGE11_CHARLIE, old_label);
    assert!(
        SIM_UNIFIED_GATE.contains(new_label),
        "simulator unified gate must assert the settlement proof label"
    );
}

#[test]
fn test_meta_gate_sources() {
    assert_absent("store mod", STORE_MOD, "settlement store open failed");
    assert_absent("benchmarks doc", BENCHMARKS_DOC, "RedbBackend::default()");
    assert!(
        BENCHMARKS_DOC.contains("SettlementStore::new()` now uses a managed local RedB root"),
        "benchmarks doc must describe the live managed SettlementStore::new() contract"
    );

    for needle in [
        "bash scripts/audit/audit_secret_type_hygiene.sh",
        "bash scripts/audit/audit_secret_eq_hygiene.sh",
        "bash scripts/audit/audit_crypto_rng_hygiene.sh",
        "bash scripts/audit/audit_boundary_panic_hygiene.sh",
        "bash scripts/audit/audit_log_redaction_hygiene.sh",
    ] {
        assert!(
            META_FLOW.contains(needle),
            "meta-gate workflow must run `{needle}`"
        );
    }

    assert!(AUDIT_SECRET_TYPE.contains("SessionToken must not derive Debug"));
    assert!(AUDIT_SECRET_EQ.contains("ct_eq_token(&session.token_hex, &token.token)"));
    assert!(AUDIT_CRYPTO_RNG.contains("crypto RNG hygiene audit passed"));
    assert!(AUDIT_BOUNDARY_PANIC.contains("boundary panic hygiene audit passed"));
    assert!(AUDIT_LOG_REDACT.contains("log redaction hygiene audit passed"));
}

#[test]
fn test_store_new_no_panic() {
    let _guard = env_case_lock().lock().expect("env lock");
    let temp = tempfile::tempdir().expect("temp dir");
    let bad_root = temp.path().join("bad-root");
    write_file(&bad_root, b"not-a-dir").expect("write root marker");

    let _root = set_env_var(
        "Z00Z_STORAGE_REDB_ROOT",
        bad_root.to_string_lossy().into_owned(),
    );
    let _bits = set_env_var("Z00Z_SETTLEMENT_BUCKET_BITS", String::from("bad"));

    assert!(
        z00z_storage::settlement::SettlementStore::try_new().is_err(),
        "fallible startup must fail closed on invalid env"
    );

    let store = z00z_storage::settlement::SettlementStore::new();
    let _ = store.settlement_root().expect("managed root");
}
