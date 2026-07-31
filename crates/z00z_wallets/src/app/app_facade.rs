//! Platform-neutral application facade over accountable final owners.

use std::{path::Path, sync::Arc};

use z00z_app_api::{
    AppApiService, AppError, AppOperation, AppendExtensionImportChunkRequest,
    BeginExtensionImportRequest, BoundedId, BoundedText, BuildMetadata, CapabilityAvailability,
    CapabilityFreshness, CapabilityMaturity, CapabilityPage, EvidenceSource,
    ExtensionArtifactEnvelope, ExtensionId, ExtensionPage, ExtensionProjection, OperationId,
    OperationProjection, PageRequest, PresentationMode, ReviewProjection,
    ReviewWalletIntentRequest, RuntimeCapability, RuntimeStatus, SetExtensionEnabledRequest,
    SetExtensionLocalBlockRequest, SubmitWalletIntentRequest, SubscriptionAck, SubscriptionRequest,
    UpdateExtensionRequest, WalletId, WalletIntentProposal, WalletPage, WalletProjection,
    APP_API_SCHEMA_VERSION,
};
use z00z_utils::io::SecureDir;

use crate::services::{AppService, WalletService};

use super::{
    action_adapter::ActionAdapter, authorization::owner_route, extension_adapter::ExtensionAdapter,
    journal::DurableJournal, redaction::internal_error, wallet_adapter::WalletAdapter,
};

const JOURNAL_REQUEST_DOMAIN: &[u8] = b"z00z.app.journal.request.v1";

/// One service implementation shared by in-process and framed transports.
pub struct AppFacade {
    _root_guard: SecureDir,
    bound_client_id: BoundedId,
    wallets: WalletAdapter,
    actions: ActionAdapter,
    extensions: ExtensionAdapter,
    journal: DurableJournal,
    now_monotonic: u64,
    next_subscription: u64,
}

impl AppFacade {
    /// Open real wallet, journal, and Extension owners beneath a bounded root.
    pub fn open(root: impl AsRef<Path>, client_id: BoundedId) -> Result<Self, AppError> {
        Self::open_at(root, client_id, 1)
    }

    /// Open real owners at a deterministic monotonic instant.
    pub fn open_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
    ) -> Result<Self, AppError> {
        let root = root.as_ref();
        let wallet_service = Arc::new(WalletService::with_output_dir(root.join("wallets")));
        let owner = Arc::new(AppService::with_wallet_service(wallet_service));
        Self::open_with_owner_at(root, client_id, now_monotonic, owner)
    }

    /// Open with an injected real application service owner.
    pub fn open_with_owner(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        owner: Arc<AppService>,
    ) -> Result<Self, AppError> {
        Self::open_with_owner_at(root, client_id, 1, owner)
    }

    /// Open with an injected owner and deterministic monotonic instant.
    pub fn open_with_owner_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
        owner: Arc<AppService>,
    ) -> Result<Self, AppError> {
        if now_monotonic == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        let root = root.as_ref();
        let root_guard = SecureDir::ensure_private(root).map_err(|_| internal_error("app-root"))?;
        Ok(Self {
            _root_guard: root_guard,
            bound_client_id: client_id.clone(),
            wallets: WalletAdapter::new(owner)?,
            actions: ActionAdapter::new(),
            extensions: ExtensionAdapter::open_at(
                root.join("extensions"),
                client_id,
                now_monotonic,
            )?,
            journal: DurableJournal::open(root.join("journal"))?,
            now_monotonic,
            next_subscription: 1,
        })
    }

    /// Return the durable infrastructure journal for integration verification.
    pub fn journal_mut(&mut self) -> &mut DurableJournal {
        &mut self.journal
    }

    fn check(&self, operation: AppOperation) -> Result<(), AppError> {
        owner_route(operation)
            .map(|_| ())
            .ok_or(AppError::UnknownOperation)
    }

    fn persist_owner_result(
        &mut self,
        operation: AppOperation,
        projection: OperationProjection,
        owner_deadline: u64,
    ) -> Result<OperationProjection, AppError> {
        if self.journal.get(&projection.id).is_ok() {
            self.journal.update_projection(projection)
        } else {
            let digest = request_digest(operation, projection.id.as_str().as_bytes());
            self.journal
                .persist_projection(projection, digest, owner_deadline)
        }
    }

    fn capability(
        &self,
        code: &str,
        is_available: bool,
        reason: &str,
    ) -> Result<RuntimeCapability, AppError> {
        RuntimeCapability::new(
            BoundedId::new(code)?,
            if is_available {
                CapabilityMaturity::Live
            } else {
                CapabilityMaturity::Target
            },
            if is_available {
                CapabilityAvailability::Available
            } else {
                CapabilityAvailability::Unavailable
            },
            if is_available {
                EvidenceSource::Native
            } else {
                EvidenceSource::None
            },
            if is_available {
                CapabilityFreshness::Timestamp(self.now_monotonic)
            } else {
                CapabilityFreshness::NotApplicable
            },
            PresentationMode::Product,
            if is_available {
                None
            } else {
                Some(BoundedId::new(reason)?)
            },
            APP_API_SCHEMA_VERSION,
            self.now_monotonic,
        )
    }
}

impl AppApiService for AppFacade {
    fn bound_client_id(&self) -> Option<&BoundedId> {
        Some(&self.bound_client_id)
    }

    fn get_runtime_status(&mut self) -> Result<RuntimeStatus, AppError> {
        self.check(AppOperation::GetRuntimeStatus)?;
        Ok(RuntimeStatus {
            ready: true,
            revision: self.now_monotonic,
        })
    }

    fn get_build_metadata(&mut self) -> Result<BuildMetadata, AppError> {
        self.check(AppOperation::GetBuildMetadata)?;
        Ok(BuildMetadata {
            version: BoundedText::new(env!("CARGO_PKG_VERSION"))?,
            target: BoundedText::new("platform-neutral-service")?,
        })
    }

    fn get_runtime_capabilities(&mut self) -> Result<CapabilityPage, AppError> {
        self.check(AppOperation::GetRuntimeCapabilities)?;
        let wallet_read = self.wallets.probe().is_ok();
        let extension_read = self.extensions.probe().is_ok();
        CapabilityPage::new(vec![
            self.capability("wallet-read", wallet_read, "wallet-owner-unavailable")?,
            self.capability(
                "extension-read",
                extension_read,
                "extension-owner-unavailable",
            )?,
            self.capability("wallet-intent-effect", false, "phase-006-owner-unavailable")?,
        ])
    }

    fn list_wallets(&mut self, request: PageRequest) -> Result<WalletPage, AppError> {
        self.check(AppOperation::ListWallets)?;
        self.wallets.list(&request)
    }

    fn get_wallet(&mut self, id: WalletId) -> Result<WalletProjection, AppError> {
        self.check(AppOperation::GetWallet)?;
        self.wallets.get(&id)
    }

    fn create_wallet_intent_proposal(
        &mut self,
        proposal: WalletIntentProposal,
    ) -> Result<WalletIntentProposal, AppError> {
        self.check(AppOperation::CreateWalletIntentProposal)?;
        self.actions.propose(proposal)
    }

    fn review_wallet_intent_proposal(
        &mut self,
        request: ReviewWalletIntentRequest,
    ) -> Result<ReviewProjection, AppError> {
        self.check(AppOperation::ReviewWalletIntentProposal)?;
        self.actions.review(&request)
    }

    fn submit_wallet_intent(
        &mut self,
        request: SubmitWalletIntentRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::SubmitWalletIntent)?;
        self.actions.submit(&request)
    }

    fn get_operation(&mut self, id: OperationId) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::GetOperation)?;
        self.journal.get(&id)
    }

    fn cancel_operation(&mut self, id: OperationId) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::CancelOperation)?;
        self.journal.cancel(&id, 0)
    }

    fn subscribe_operation_events(
        &mut self,
        request: SubscriptionRequest,
    ) -> Result<SubscriptionAck, AppError> {
        self.check(AppOperation::SubscribeOperationEvents)?;
        self.journal.get(request.operation_id())?;
        let subscription_id = self.next_subscription;
        self.next_subscription = self
            .next_subscription
            .checked_add(1)
            .ok_or(AppError::Conflict)?;
        Ok(SubscriptionAck {
            subscription_id,
            operation_id: request.operation_id().clone(),
            after_sequence: request.after_sequence(),
        })
    }

    fn list_extensions(&mut self, request: PageRequest) -> Result<ExtensionPage, AppError> {
        self.check(AppOperation::ListExtensions)?;
        self.extensions.list(&request)
    }

    fn inspect_extension(&mut self, id: ExtensionId) -> Result<ExtensionProjection, AppError> {
        self.check(AppOperation::GetExtension)?;
        self.extensions.inspect(&id)
    }

    fn begin_extension_import(
        &mut self,
        request: BeginExtensionImportRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::BeginExtensionImport)?;
        let owner_deadline = request.expires_at_monotonic();
        let projection = self.extensions.begin(request)?;
        self.persist_owner_result(
            AppOperation::BeginExtensionImport,
            projection,
            owner_deadline,
        )
    }

    fn upload_extension_chunk(
        &mut self,
        request: AppendExtensionImportChunkRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::AppendExtensionImportChunk)?;
        let projection = self.extensions.append(request)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(
            AppOperation::AppendExtensionImportChunk,
            projection,
            owner_deadline,
        )
    }

    fn commit_extension_import(
        &mut self,
        id: OperationId,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::FinishExtensionImport)?;
        let projection = self.extensions.commit(&id)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(
            AppOperation::FinishExtensionImport,
            projection,
            owner_deadline,
        )
    }

    fn get_extension_operation(
        &mut self,
        id: OperationId,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::GetExtensionOperation)?;
        self.journal.get(&id)
    }

    fn abort_extension_import(&mut self, id: OperationId) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::CancelExtensionOperation)?;
        let projection = self.extensions.abort(&id)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(
            AppOperation::CancelExtensionOperation,
            projection,
            owner_deadline,
        )
    }

    fn update_extension(
        &mut self,
        request: UpdateExtensionRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::UpdateExtension)?;
        let projection = self.extensions.update(&request)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(AppOperation::UpdateExtension, projection, owner_deadline)
    }

    fn set_extension_enabled(
        &mut self,
        request: SetExtensionEnabledRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::SetExtensionEnabled)?;
        let projection = self.extensions.set_enabled(&request)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(
            AppOperation::SetExtensionEnabled,
            projection,
            owner_deadline,
        )
    }

    fn set_extension_local_block(
        &mut self,
        request: SetExtensionLocalBlockRequest,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::SetExtensionLocalBlock)?;
        let projection = self.extensions.set_local_block(&request)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(
            AppOperation::SetExtensionLocalBlock,
            projection,
            owner_deadline,
        )
    }

    fn remove_extension(&mut self, id: ExtensionId) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::RemoveExtension)?;
        let projection = self.extensions.remove(&id)?;
        let owner_deadline = self.extensions.owner_deadline()?;
        self.persist_owner_result(AppOperation::RemoveExtension, projection, owner_deadline)
    }

    fn prepare_extension_artifact(
        &mut self,
        id: ExtensionId,
    ) -> Result<ExtensionArtifactEnvelope, AppError> {
        self.check(AppOperation::PrepareExtensionArtifact)?;
        self.extensions.prepare(&id)
    }
}

fn request_digest(operation: AppOperation, payload: &[u8]) -> [u8; 32] {
    let operation_bytes = (operation as u16).to_be_bytes();
    z00z_crypto::derive_hash(JOURNAL_REQUEST_DOMAIN, &[&operation_bytes, payload])
}
