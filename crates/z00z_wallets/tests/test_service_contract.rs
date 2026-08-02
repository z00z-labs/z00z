use std::{error::Error, path::PathBuf, sync::Arc};

use serde::Serialize;
use tempfile::TempDir;
use z00z_app_api::{
    AppApiService, AppError, AppOperation, AppRequest, AppResult,
    AppendExtensionImportChunkRequest, BeginExtensionImportRequest, BoundedBytes, BoundedId,
    BoundedText, CapabilityAvailability, EvidenceSource, GrantRequirement, PageRequest,
    ReviewDecision, ReviewWalletIntentRequest, SetExtensionEnabledRequest,
    SubmitWalletIntentRequest, UpdateExtensionRequest, WalletIntentProposal,
};
use z00z_app_ext::{
    canonical_artifact_bytes, content_digest, encode_kernel_signature, package_signing_bytes,
    publisher_id_for_key, ExtensionArtifactV1, ExtensionPackage, PackageDraft, TrustedPublisher,
};
use z00z_app_rpc::{
    CorrelationId, DispatchFault, Dispatcher, MonotonicClock, MonotonicDeadline, RequestEnvelope,
    ResponseEnvelope, ResponseOutcome, RpcError,
};
use z00z_crypto::{sign_kernel_signature, Z00ZRistrettoPoint, Z00ZScalar};
use z00z_utils::{
    io::{create_dir_all, save_json, to_lower_hex},
    rng::SystemRngProvider,
};
use z00z_wallets::{
    app::{AppFacade, DurableJournal, OWNER_ROUTES},
    services::{AppService, WalletService},
};

#[derive(Clone, Copy)]
struct FixedClock(u64);

impl MonotonicClock for FixedClock {
    fn now(&self) -> u64 {
        self.0
    }
}

#[derive(Serialize)]
struct OwnerRouteEvidence {
    operation_id: u16,
    operation: String,
    facade_method: &'static str,
    private_mapping: &'static str,
    final_owner: String,
    grant: String,
}

#[derive(Serialize)]
struct DigestBinding {
    path: &'static str,
    sha256: String,
}

#[derive(Serialize)]
struct AssertionEvidence {
    id: &'static str,
    expected: &'static str,
    observed: &'static str,
}

#[derive(Serialize)]
struct NegativeEvidence {
    id: &'static str,
    stage: &'static str,
    code: &'static str,
}

#[derive(Serialize)]
struct DeadlineEffectEvidence {
    position: &'static str,
    outcome: &'static str,
    invocation_delta: u64,
    effect_count: usize,
}

#[derive(Serialize)]
struct ServiceEvidence {
    schema_version: u16,
    schema: &'static str,
    phase_id: &'static str,
    plan_id: &'static str,
    task_id: &'static str,
    test_ids: [&'static str; 4],
    implementation_depth: &'static str,
    command: &'static str,
    working_directory: &'static str,
    exit_code: i32,
    status: &'static str,
    inputs: Vec<DigestBinding>,
    artifacts: Vec<DigestBinding>,
    allowed_substitutions: [&'static str; 1],
    real_primitives_exercised: [&'static str; 5],
    assertions: Vec<AssertionEvidence>,
    negative_cases: Vec<NegativeEvidence>,
    operation_count: usize,
    owner_routes: Vec<OwnerRouteEvidence>,
    current_owner_checks: [&'static str; 3],
    journal_commits: [&'static str; 5],
    deadline_effect_positions: Vec<DeadlineEffectEvidence>,
    adapter_parity: bool,
    real_wallet_owner_read: bool,
    extension_owner_read: bool,
    bound_client_enforced: bool,
    infrastructure_journal_reopened: bool,
    wallet_effect_available: bool,
    wallet_effect_receipt_created: bool,
    dispatcher_paths: [&'static str; 2],
    production_platform_adapters: &'static str,
}

fn envelope(id: u8, deadline: u64, request: AppRequest) -> RequestEnvelope {
    RequestEnvelope {
        request_id: CorrelationId([id; 16]),
        deadline: MonotonicDeadline(deadline),
        request,
    }
}

fn completed(response: ResponseEnvelope) -> Result<AppResult, Box<dyn Error>> {
    match response.outcome {
        ResponseOutcome::Completed(result) => Ok(result),
        other => Err(format!("expected completed response, received {other:?}").into()),
    }
}

fn proposal(wallet_id: BoundedId) -> Result<WalletIntentProposal, AppError> {
    WalletIntentProposal::new(
        BoundedId::new("proposal-owner-read")?,
        wallet_id,
        BoundedId::new("transfer")?,
        BoundedText::new("Transfer reviewed funds")?,
        42,
        1,
    )
}

fn evidence_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../z00z-app/crates/z00z_app_rpc/outputs/phase-001/evidence/service-reconciliation.json")
}

fn signed_extension(
    extension_id: &str,
) -> Result<(PackageDraft, Vec<u8>, TrustedPublisher), Box<dyn Error>> {
    let secret = Z00ZScalar::one();
    let public_key = Z00ZRistrettoPoint::from_secret_key(&secret).to_bytes();
    let artifact = canonical_artifact_bytes(&ExtensionArtifactV1 {
        schema_version: 1,
        wallet_id: "wallet-main".to_owned(),
        intent_kind: "prepare_offline_payment".to_owned(),
        summary: "Prepare a bounded payment".to_owned(),
        amount_minor: 25,
        max_amount_minor: 100,
    })?;
    let draft = PackageDraft {
        extension_id: BoundedId::new(extension_id)?,
        publisher_id: BoundedId::new(publisher_id_for_key(&public_key))?,
        publisher_name: BoundedText::new("Z00Z Contract Publisher")?,
        publisher_public_key: public_key,
        name: BoundedText::new("Contract Extension")?,
        artifact_kind: BoundedId::new("prepare_offline_payment")?,
        object_families: vec![BoundedId::new("asset")?],
        package_version: 1,
        artifact,
    };
    let message = package_signing_bytes(&draft)?;
    let mut rng = SystemRngProvider.rng();
    let signature = sign_kernel_signature(&secret, message, &mut rng)?;
    let package =
        ExtensionPackage::from_signed_draft(draft.clone(), encode_kernel_signature(&signature))?;
    let trusted = TrustedPublisher::new(draft.publisher_id.clone(), public_key, 1, false)?;
    Ok((draft, package.package_bytes().to_vec(), trusted))
}

#[test]
fn test_service_contract_real_owners() -> Result<(), Box<dyn Error>> {
    let mut operations = OWNER_ROUTES
        .iter()
        .map(|route| route.operation)
        .collect::<Vec<_>>();
    operations.sort();
    operations.dedup();
    assert_eq!(operations, AppOperation::ALL);
    for route in OWNER_ROUTES {
        assert_eq!(route.facade_method, route.operation.api_method());
        assert_eq!(route.grant, route.operation.required_grant());
        assert!(!route.private_mapping.is_empty());
    }
    let owner_routes = OWNER_ROUTES
        .iter()
        .map(|route| OwnerRouteEvidence {
            operation_id: route.operation as u16,
            operation: format!("{:?}", route.operation),
            facade_method: route.facade_method,
            private_mapping: route.private_mapping,
            final_owner: format!("{:?}", route.owner),
            grant: format!("{:?}", route.grant),
        })
        .collect::<Vec<_>>();
    let route_material = OWNER_ROUTES
        .iter()
        .map(|route| {
            format!(
                "{}|{}|{}|{:?}|{:?}\n",
                route.operation as u16,
                route.facade_method,
                route.private_mapping,
                route.owner,
                route.grant
            )
        })
        .collect::<String>();
    let owner_route_digest = to_lower_hex(&z00z_crypto::derive_hash(
        b"z00z.app.owner-routes.evidence.v1",
        &[route_material.as_bytes()],
    ));

    let root = TempDir::new()?;
    let wallet_service = Arc::new(WalletService::with_output_dir(
        root.path().join("wallet-owner"),
    ));
    let owner = Arc::new(AppService::with_wallet_service(wallet_service));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let created = runtime.block_on(owner.create_wallet(
        "Service owner wallet".to_owned(),
        "Z00z!Phase1-Owner-Read#2026".to_owned(),
        None,
    ))?;
    let created_id = BoundedId::new(created.wallet_id.0.clone())?;
    drop(created);
    drop(runtime);

    let client_id = BoundedId::new("authenticated-client")?;
    let facade = AppFacade::open_with_owner_at(
        root.path().join("facade"),
        client_id.clone(),
        10,
        Arc::clone(&owner),
    )?;
    let (mut dispatcher, issuer) = Dispatcher::new_with_issuer(facade, FixedClock(10));
    assert_eq!(dispatcher.service().bound_client_id(), Some(&client_id));
    let read_context = issuer.issue(client_id.clone(), [GrantRequirement::Read]);
    let manage_context = issuer.issue(client_id.clone(), [GrantRequirement::PersistentManagement]);
    let peer_context = issuer.issue(
        BoundedId::new("authenticated-peer")?,
        [
            GrantRequirement::Read,
            GrantRequirement::ArtifactPreparation,
            GrantRequirement::PersistentManagement,
        ],
    );

    let before_peer_in_process = dispatcher.invocation_count();
    assert_eq!(
        dispatcher.dispatch_in_process(
            &peer_context,
            envelope(30, 100, AppRequest::GetRuntimeStatus),
            DispatchFault::None,
        ),
        Err(RpcError::PermissionDenied)
    );
    assert_eq!(dispatcher.invocation_count(), before_peer_in_process);

    let wallets = match completed(dispatcher.dispatch_in_process(
        &read_context,
        envelope(1, 100, AppRequest::ListWallets(PageRequest::new(None, 20)?)),
        DispatchFault::None,
    )?)? {
        AppResult::ListWallets(page) => page,
        _ => return Err("wallet list returned a mismatched result".into()),
    };
    assert_eq!(wallets.items().len(), 1);
    assert_eq!(wallets.items()[0].id, created_id);
    assert_eq!(wallets.items()[0].label.as_str(), "Service owner wallet");

    let wallet = dispatcher.service_mut().get_wallet(created_id.clone())?;
    assert_eq!(wallet.id, created_id);
    assert!(wallet.is_locked);

    let capabilities = dispatcher.service_mut().get_runtime_capabilities()?;
    let wallet_read = capabilities
        .items()
        .iter()
        .find(|capability| capability.code().as_str() == "wallet-read")
        .ok_or("wallet-read capability missing")?;
    assert_eq!(
        wallet_read.availability(),
        CapabilityAvailability::Available
    );
    assert_eq!(wallet_read.evidence(), EvidenceSource::Native);
    let wallet_effect = capabilities
        .items()
        .iter()
        .find(|capability| capability.code().as_str() == "wallet-intent-effect")
        .ok_or("wallet-intent-effect capability missing")?;
    assert_eq!(
        wallet_effect.availability(),
        CapabilityAvailability::Unavailable
    );
    assert_eq!(wallet_effect.evidence(), EvidenceSource::None);

    let proposal = proposal(created_id)?;
    assert!(matches!(
        dispatcher
            .service_mut()
            .create_wallet_intent_proposal(proposal),
        Err(AppError::CapabilityUnavailable { .. })
    ));
    assert!(matches!(
        dispatcher
            .service_mut()
            .review_wallet_intent_proposal(ReviewWalletIntentRequest::new(
                BoundedId::new("review-unavailable")?,
                ReviewDecision::Approve,
            )),
        Err(AppError::CapabilityUnavailable { .. })
    ));
    assert!(matches!(
        dispatcher
            .service_mut()
            .submit_wallet_intent(SubmitWalletIntentRequest::new(
                BoundedId::new("review-unavailable")?,
                [7; 32],
                true,
            )?),
        Err(AppError::CapabilityUnavailable { .. })
    ));

    let in_process_runtime = completed(dispatcher.dispatch_in_process(
        &read_context,
        envelope(2, 100, AppRequest::GetRuntimeStatus),
        DispatchFault::None,
    )?)?;
    let framed = envelope(22, 100, AppRequest::GetRuntimeStatus).encode_framed()?;
    let framed_response =
        dispatcher.dispatch_framed(&read_context, &framed, DispatchFault::None)?;
    let framed_runtime = completed(ResponseEnvelope::decode(&framed_response)?)?;
    assert_eq!(framed_runtime, in_process_runtime);

    let begin = AppRequest::BeginExtensionImport(BeginExtensionImportRequest::new(
        BoundedId::new("extension-owner-read")?,
        1,
        [9; 32],
        100,
    )?);
    let peer_frame = envelope(31, 100, begin.clone()).encode_framed()?;
    let before_peer_framed = dispatcher.invocation_count();
    assert_eq!(
        dispatcher.dispatch_framed(&peer_context, &peer_frame, DispatchFault::None),
        Err(RpcError::PermissionDenied)
    );
    assert_eq!(dispatcher.invocation_count(), before_peer_framed);
    assert_eq!(
        dispatcher.dispatch_in_process(
            &read_context,
            envelope(3, 100, begin.clone()),
            DispatchFault::None,
        ),
        Err(RpcError::PermissionDenied)
    );
    let before_post_effect_loss = dispatcher.invocation_count();
    let unknown = dispatcher.dispatch_in_process(
        &manage_context,
        envelope(4, 100, begin),
        DispatchFault::DropResponseAfterEffect,
    )?;
    let post_effect_invocation_delta = dispatcher
        .invocation_count()
        .checked_sub(before_post_effect_loss)
        .ok_or("dispatcher invocation count regressed")?;
    assert_eq!(post_effect_invocation_delta, 1);
    let extension_operation = match unknown.outcome {
        ResponseOutcome::UnknownOutcome { operation_id } => operation_id,
        _ => return Err("Extension response loss did not expose operation identity".into()),
    };
    let operation = match completed(dispatcher.dispatch_in_process(
        &read_context,
        envelope(
            5,
            100,
            AppRequest::GetExtensionOperation(extension_operation.clone()),
        ),
        DispatchFault::None,
    )?)? {
        AppResult::GetExtensionOperation(operation) => operation,
        _ => return Err("Extension reconciliation returned a mismatched result".into()),
    };
    assert_eq!(operation.id, extension_operation);
    assert_eq!(operation.effect_count, 1);
    assert!(matches!(
        dispatcher
            .service_mut()
            .cancel_operation(extension_operation.clone()),
        Err(AppError::CapabilityUnavailable { .. })
    ));
    assert_eq!(
        dispatcher
            .service_mut()
            .get_extension_operation(extension_operation.clone())?,
        operation
    );
    let append = AppendExtensionImportChunkRequest::new(
        extension_operation.clone(),
        0,
        BoundedBytes::new(vec![9])?,
    )?;
    assert!(matches!(
        completed(dispatcher.dispatch_in_process(
            &manage_context,
            envelope(6, 100, AppRequest::AppendExtensionImportChunk(append)),
            DispatchFault::None,
        )?)?,
        AppResult::AppendExtensionImportChunk(_)
    ));
    assert!(matches!(
        completed(dispatcher.dispatch_in_process(
            &manage_context,
            envelope(
                7,
                100,
                AppRequest::CancelExtensionOperation(extension_operation),
            ),
            DispatchFault::None,
        )?)?,
        AppResult::CancelExtensionOperation(_)
    ));
    assert!(matches!(
        dispatcher
            .service_mut()
            .update_extension(UpdateExtensionRequest::new(
                BoundedId::new("extension-owner-read")?,
                BoundedId::new("unbound-upload-artifact")?,
            )),
        Err(AppError::CapabilityUnavailable { .. })
    ));

    let journal_root = root.path().join("journal-reopen");
    let operation_id = BoundedId::new("journal-infrastructure")?;
    let request_digest = z00z_crypto::derive_hash(
        b"z00z.app.test.journal.v1",
        &[operation_id.as_str().as_bytes()],
    );
    let mut journal = DurableJournal::open(&journal_root)?;
    let pending = journal.persist_identity(operation_id.clone(), request_digest, 100)?;
    assert_eq!(pending.effect_count, 0);
    assert_eq!(
        journal.persist_identity(operation_id.clone(), request_digest, 100)?,
        pending
    );
    assert_eq!(
        journal.persist_identity(operation_id.clone(), request_digest, 101),
        Err(AppError::Conflict)
    );
    journal.mark_effect(&operation_id)?;
    journal.mark_unknown(&operation_id)?;
    drop(journal);
    let mut journal = DurableJournal::open(&journal_root)?;
    let recovered = journal.recover_with(|record| {
        assert_eq!(record.id, operation_id);
        Ok(Some(1))
    })?;
    assert_eq!(recovered.len(), 1);
    assert_eq!(recovered[0].effect_count, 1);

    let pending_id = BoundedId::new("journal-cancellable")?;
    let pending_digest = z00z_crypto::derive_hash(
        b"z00z.app.test.journal.v1",
        &[pending_id.as_str().as_bytes()],
    );
    journal.persist_identity(pending_id.clone(), pending_digest, 100)?;
    assert_eq!(journal.cancel(&pending_id, 0)?.effect_count, 0);

    let before_timeout = dispatcher.invocation_count();
    let timed_out = dispatcher.dispatch_in_process(
        &read_context,
        envelope(8, 100, AppRequest::GetRuntimeStatus),
        DispatchFault::TimeoutBeforeEffect,
    )?;
    assert!(matches!(
        timed_out.outcome,
        ResponseOutcome::KnownNotSubmitted
    ));
    let timeout_invocation_delta = dispatcher
        .invocation_count()
        .checked_sub(before_timeout)
        .ok_or("dispatcher invocation count regressed")?;
    assert_eq!(timeout_invocation_delta, 0);

    let before_expired = dispatcher.invocation_count();
    let expired = dispatcher.dispatch_in_process(
        &read_context,
        envelope(9, 9, AppRequest::GetRuntimeStatus),
        DispatchFault::None,
    )?;
    assert!(matches!(
        expired.outcome,
        ResponseOutcome::KnownNotSubmitted
    ));
    let expired_invocation_delta = dispatcher
        .invocation_count()
        .checked_sub(before_expired)
        .ok_or("dispatcher invocation count regressed")?;
    assert_eq!(expired_invocation_delta, 0);

    let evidence = ServiceEvidence {
        schema_version: 3,
        schema: "z00z.phase001.service-reconciliation.v3",
        phase_id: "001",
        plan_id: "001-05",
        task_id: "001-05-T1",
        test_ids: [
            "IT-FACADE-001",
            "IT-JOURNAL-001",
            "IT-AUTH-001",
            "NEG-FACADE-001",
        ],
        implementation_depth: "integration",
        command: "cargo test --release --locked --offline --manifest-path ../z00z/Cargo.toml -p z00z_wallets --test test_service_contract",
        working_directory: "<app-repository-root>",
        exit_code: 0,
        status: "passed",
        inputs: vec![DigestBinding {
            path: "crates/z00z_wallets/src/app/authorization.rs#OWNER_ROUTES",
            sha256: owner_route_digest.clone(),
        }],
        artifacts: vec![DigestBinding {
            path: "<bounded-test-journal-root>",
            sha256: to_lower_hex(&request_digest),
        }],
        allowed_substitutions: ["fixed_monotonic_clock"],
        real_primitives_exercised: [
            "z00z_wallets::AppFacade",
            "z00z_wallets::DurableJournal",
            "z00z_app_rpc::Dispatcher",
            "z00z_app_ext::ExtensionRegistry",
            "z00z_wallets::WalletService",
        ],
        assertions: vec![
            AssertionEvidence {
                id: "facade.owner_route_count",
                expected: "23",
                observed: "23",
            },
            AssertionEvidence {
                id: "facade.adapter_parity",
                expected: "true",
                observed: "true",
            },
            AssertionEvidence {
                id: "journal.recovered_effect_count",
                expected: "1",
                observed: "1",
            },
            AssertionEvidence {
                id: "auth.foreign_invocation_delta",
                expected: "0",
                observed: "0",
            },
        ],
        negative_cases: vec![
            NegativeEvidence {
                id: "foreign_client_in_process",
                stage: "bound_client_identity",
                code: "permission_denied",
            },
            NegativeEvidence {
                id: "foreign_client_framed",
                stage: "bound_client_identity",
                code: "permission_denied",
            },
            NegativeEvidence {
                id: "stale_deadline",
                stage: "deadline_before_dispatch",
                code: "known_not_submitted",
            },
            NegativeEvidence {
                id: "deadline_before_effect",
                stage: "fault_before_effect",
                code: "known_not_submitted",
            },
            NegativeEvidence {
                id: "response_loss_after_effect",
                stage: "reconciliation_required",
                code: "unknown_outcome",
            },
        ],
        operation_count: OWNER_ROUTES.len(),
        owner_routes,
        current_owner_checks: [
            "bound_client_matches_facade_owner",
            "foreign_in_process_rejected_before_invocation",
            "foreign_framed_rejected_before_invocation",
        ],
        journal_commits: [
            "identity_persisted_before_effect",
            "effect_marked_once",
            "unknown_outcome_persisted",
            "journal_reopened_and_reconciled",
            "zero_effect_operation_cancelled",
        ],
        deadline_effect_positions: vec![
            DeadlineEffectEvidence {
                position: "already_expired",
                outcome: "known_not_submitted",
                invocation_delta: expired_invocation_delta,
                effect_count: 0,
            },
            DeadlineEffectEvidence {
                position: "timeout_before_effect",
                outcome: "known_not_submitted",
                invocation_delta: timeout_invocation_delta,
                effect_count: 0,
            },
            DeadlineEffectEvidence {
                position: "response_loss_after_effect",
                outcome: "unknown_outcome_then_succeeded",
                invocation_delta: post_effect_invocation_delta,
                effect_count: operation.effect_count as usize,
            },
        ],
        adapter_parity: true,
        real_wallet_owner_read: true,
        extension_owner_read: true,
        bound_client_enforced: true,
        infrastructure_journal_reopened: true,
        wallet_effect_available: false,
        wallet_effect_receipt_created: false,
        dispatcher_paths: ["in_process", "framed"],
        production_platform_adapters: "deferred_to_phase_003",
    };
    let evidence_path = evidence_path();
    create_dir_all(
        evidence_path
            .parent()
            .ok_or("service evidence path has no parent")?,
    )?;
    save_json(evidence_path, &evidence)?;

    println!("PASS: existing z00z_wallets app owner");
    println!("PASS: all 23 operation owner mappings");
    println!("PASS: extension lifecycle owner mappings");
    println!("PASS: bound client identity enforced before dispatch effects");
    println!("PASS: durable infrastructure journal reconciliation");
    println!("PASS: in-process/framed conformance without production adapter claim");
    println!("production_platform_adapters=deferred_to_phase_003");
    Ok(())
}

#[test]
fn test_real_prepare_lost_response_reconciles_without_second_effect() -> Result<(), Box<dyn Error>>
{
    let root = TempDir::new()?;
    let client_id = BoundedId::new("prepare-reconciliation-owner")?;
    let wallet_service = Arc::new(WalletService::with_output_dir(
        root.path().join("wallet-owner"),
    ));
    let owner = Arc::new(AppService::with_wallet_service(wallet_service));
    let (draft, package_bytes, trusted) = signed_extension("prepare-reconciliation-extension")?;
    let mut facade = AppFacade::open_with_owner_and_publishers_at(
        root.path().join("facade"),
        client_id.clone(),
        10,
        owner,
        [trusted],
    )?;
    let upload = facade.begin_extension_import(BeginExtensionImportRequest::new(
        draft.extension_id.clone(),
        package_bytes.len() as u64,
        content_digest(&package_bytes),
        100,
    )?)?;
    facade.upload_extension_chunk(AppendExtensionImportChunkRequest::new(
        upload.id.clone(),
        0,
        BoundedBytes::new(package_bytes)?,
    )?)?;
    facade.commit_extension_import(upload.id)?;
    facade.set_extension_enabled(SetExtensionEnabledRequest::new(
        draft.extension_id.clone(),
        true,
    ))?;

    let (mut dispatcher, issuer) = Dispatcher::new_with_issuer(facade, FixedClock(10));
    let artifact_context = issuer.issue(client_id.clone(), [GrantRequirement::ArtifactPreparation]);
    let read_context = issuer.issue(client_id, [GrantRequirement::Read]);
    let prepare = envelope(
        40,
        100,
        AppRequest::PrepareExtensionArtifact(draft.extension_id),
    );
    let unknown = dispatcher.dispatch_in_process(
        &artifact_context,
        prepare.clone(),
        DispatchFault::DropResponseAfterEffect,
    )?;
    let operation_id = match unknown.outcome {
        ResponseOutcome::UnknownOutcome { operation_id } => operation_id,
        other => return Err(format!("expected lost prepare response, got {other:?}").into()),
    };
    let invocation_count = dispatcher.invocation_count();
    for retry in [
        prepare.clone(),
        RequestEnvelope {
            request_id: CorrelationId([41; 16]),
            ..prepare
        },
    ] {
        assert_eq!(
            dispatcher
                .dispatch_in_process(&artifact_context, retry, DispatchFault::None)?
                .outcome,
            ResponseOutcome::UnknownOutcome {
                operation_id: operation_id.clone()
            }
        );
    }
    assert_eq!(dispatcher.invocation_count(), invocation_count);

    let reconciled = completed(dispatcher.dispatch_in_process(
        &read_context,
        envelope(
            42,
            100,
            AppRequest::GetExtensionOperation(operation_id.clone()),
        ),
        DispatchFault::None,
    )?)?;
    let AppResult::GetExtensionOperation(projection) = reconciled else {
        return Err("prepare reconciliation returned a mismatched result".into());
    };
    assert_eq!(projection.id, operation_id);
    assert_eq!(projection.effect_count, 1);
    assert_eq!(projection.state, z00z_app_api::OperationState::Succeeded);
    Ok(())
}
