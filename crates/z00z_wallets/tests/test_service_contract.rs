use std::{error::Error, path::PathBuf, sync::Arc};

use serde::Serialize;
use tempfile::TempDir;
use z00z_app_api::{
    AppApiService, AppError, AppOperation, AppRequest, AppResult,
    AppendExtensionImportChunkRequest, BeginExtensionImportRequest, BoundedBytes, BoundedId,
    BoundedText, CapabilityAvailability, EvidenceSource, GrantRequirement, PageRequest,
    ReviewDecision, ReviewWalletIntentRequest, SubmitWalletIntentRequest, UpdateExtensionRequest,
    WalletIntentProposal,
};
use z00z_app_rpc::{
    CorrelationId, DispatchFault, Dispatcher, MonotonicClock, MonotonicDeadline, RequestEnvelope,
    ResponseEnvelope, ResponseOutcome, RpcError, ServerContext,
};
use z00z_utils::io::{create_dir_all, save_json};
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
struct ServiceEvidence {
    schema: &'static str,
    status: &'static str,
    operation_count: usize,
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
        .join("../../../z00z-app/target/phase-001-evidence/001-05/service-reconciliation.json")
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
    let mut dispatcher = Dispatcher::new(facade, FixedClock(10));
    assert_eq!(dispatcher.service().bound_client_id(), Some(&client_id));
    let read_context = ServerContext::new(client_id.clone(), [GrantRequirement::Read]);
    let manage_context =
        ServerContext::new(client_id.clone(), [GrantRequirement::PersistentManagement]);
    let peer_context = ServerContext::new(
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
    assert!(wallet.locked);

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

    let framed = envelope(2, 100, AppRequest::GetRuntimeStatus).encode_framed()?;
    let framed_response =
        dispatcher.dispatch_framed(&read_context, &framed, DispatchFault::None)?;
    assert!(matches!(
        ResponseEnvelope::decode(&framed_response)?.outcome,
        ResponseOutcome::Completed(AppResult::GetRuntimeStatus(_))
    ));

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
    let unknown = dispatcher.dispatch_in_process(
        &manage_context,
        envelope(4, 100, begin),
        DispatchFault::DropResponseAfterEffect,
    )?;
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
    assert_eq!(operation.effect_count, 0);
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

    let timed_out = dispatcher.dispatch_in_process(
        &read_context,
        envelope(8, 100, AppRequest::GetRuntimeStatus),
        DispatchFault::TimeoutBeforeEffect,
    )?;
    assert!(matches!(
        timed_out.outcome,
        ResponseOutcome::KnownNotSubmitted
    ));

    let evidence = ServiceEvidence {
        schema: "z00z.phase001.service-reconciliation.v2",
        status: "pass",
        operation_count: OWNER_ROUTES.len(),
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
