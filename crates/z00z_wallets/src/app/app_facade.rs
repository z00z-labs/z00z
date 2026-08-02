//! Platform-neutral application facade over accountable final owners.

use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Instant,
};

use z00z_app_api::{
    AppApiService, AppError, AppOperation, AppRequest, AppendExtensionImportChunkRequest,
    BeginExtensionImportRequest, BoundedId, BoundedText, BuildMetadata, CapabilityAvailability,
    CapabilityFreshness, CapabilityMaturity, CapabilityPage, EvidenceSource,
    ExtensionArtifactEnvelope, ExtensionId, ExtensionPage, ExtensionProjection, OperationId,
    OperationProjection, PageRequest, PresentationMode, ReviewProjection,
    ReviewWalletIntentRequest, RuntimeCapability, RuntimeStatus, SetExtensionEnabledRequest,
    SetExtensionLocalBlockRequest, SubmitWalletIntentRequest, SubscriptionAck, SubscriptionRequest,
    UpdateExtensionRequest, WalletId, WalletIntentProposal, WalletPage, WalletProjection,
    APP_API_SCHEMA_VERSION,
};
use z00z_app_ext::{canonical_owner_request_digest, TrustedPublisher};
use z00z_utils::{
    io::SecureDir,
    time::{SystemTimeProvider, TimeProvider},
};

use crate::services::{AppService, WalletService};

use super::{
    action_adapter::ActionAdapter, authorization::owner_route, extension_adapter::ExtensionAdapter,
    journal::DurableJournal, redaction::internal_error, wallet_adapter::WalletAdapter,
};

const CLOCK_FLOOR_FILE: &str = "monotonic-high-water.v1";
const MAX_CLOCK_FLOOR_BYTES: u64 = 64;
const MAX_FACADE_ROOT_ENTRIES: usize = 16;

/// One service implementation shared by in-process and framed transports.
pub struct AppFacade {
    root_guard: SecureDir,
    bound_client_id: BoundedId,
    wallets: WalletAdapter,
    actions: ActionAdapter,
    extensions: ExtensionAdapter,
    journal: DurableJournal,
    clock: Arc<dyn FacadeMonotonicClock>,
    last_clock_sample: u64,
    next_subscription: u64,
}

/// Trusted advancing time source sampled at every facade operation boundary.
pub trait FacadeMonotonicClock: Send + Sync {
    /// Return a nonzero monotonic tick in the application's persisted deadline scale.
    fn now_monotonic(&self) -> Result<u64, AppError>;
}

struct SystemMonotonicClock {
    started: Instant,
    origin: u64,
    last: AtomicU64,
}

impl SystemMonotonicClock {
    fn new() -> Result<Self, AppError> {
        let origin = SystemTimeProvider
            .try_unix_timestamp()
            .map_err(|_| internal_error("monotonic-clock-origin"))?
            .max(1);
        Ok(Self {
            started: Instant::now(),
            origin,
            last: AtomicU64::new(origin),
        })
    }
}

impl FacadeMonotonicClock for SystemMonotonicClock {
    fn now_monotonic(&self) -> Result<u64, AppError> {
        let sampled = self
            .origin
            .checked_add(self.started.elapsed().as_secs())
            .ok_or(AppError::Conflict)?;
        let mut previous = self.last.load(Ordering::Acquire);
        loop {
            let next = sampled.max(previous);
            match self.last.compare_exchange_weak(
                previous,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(next),
                Err(actual) => previous = actual,
            }
        }
    }
}

struct FixedMonotonicClock(u64);

impl FacadeMonotonicClock for FixedMonotonicClock {
    fn now_monotonic(&self) -> Result<u64, AppError> {
        Ok(self.0)
    }
}

fn load_clock_floor(directory: &SecureDir) -> Result<Option<u64>, AppError> {
    let names = directory
        .read_dir_bounded(MAX_FACADE_ROOT_ENTRIES)
        .map_err(|_| internal_error("clock-floor-list"))?;
    if !names.iter().any(|name| name == CLOCK_FLOOR_FILE) {
        return Ok(None);
    }
    let mut file = directory
        .open_lock(CLOCK_FLOOR_FILE)
        .map_err(|_| internal_error("clock-floor-open"))?;
    let length = file
        .metadata()
        .map_err(|_| internal_error("clock-floor-metadata"))?
        .len();
    if length == 0 || length > MAX_CLOCK_FLOOR_BYTES {
        return Err(AppError::IntegrityFailure);
    }
    let mut bytes =
        Vec::with_capacity(usize::try_from(length).map_err(|_| AppError::IntegrityFailure)?);
    file.read_to_end(&mut bytes)
        .map_err(|_| internal_error("clock-floor-read"))?;
    let value = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|text| text.strip_prefix("z00z-app-clock-v1\n"))
        .and_then(|text| text.strip_suffix('\n'))
        .and_then(|text| text.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .ok_or(AppError::IntegrityFailure)?;
    Ok(Some(value))
}

fn persist_clock_floor(directory: &SecureDir, value: u64) -> Result<(), AppError> {
    let bytes = format!("z00z-app-clock-v1\n{value}\n");
    let mut file = directory
        .open_lock(CLOCK_FLOOR_FILE)
        .map_err(|_| internal_error("clock-floor-open"))?;
    file.set_len(0)
        .map_err(|_| internal_error("clock-floor-truncate"))?;
    file.seek(SeekFrom::Start(0))
        .map_err(|_| internal_error("clock-floor-seek"))?;
    file.write_all(bytes.as_bytes())
        .map_err(|_| internal_error("clock-floor-write"))?;
    file.sync_all()
        .map_err(|_| internal_error("clock-floor-file-sync"))?;
    directory
        .sync()
        .map_err(|_| internal_error("clock-floor-sync"))
}

impl AppFacade {
    /// Open real wallet, journal, and Extension owners beneath a bounded root.
    pub fn open(root: impl AsRef<Path>, client_id: BoundedId) -> Result<Self, AppError> {
        let root = root.as_ref();
        let _root_guard =
            SecureDir::ensure_private(root).map_err(|_| internal_error("app-root"))?;
        let wallet_service = Arc::new(WalletService::with_output_dir(root.join("wallets")));
        let owner = Arc::new(AppService::with_wallet_service(wallet_service));
        Self::open_with_owner_and_publishers_clock(
            root,
            client_id,
            Arc::new(SystemMonotonicClock::new()?),
            owner,
            [],
        )
    }

    /// Open real owners at a deterministic monotonic instant.
    pub fn open_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
    ) -> Result<Self, AppError> {
        let root = root.as_ref();
        let _root_guard =
            SecureDir::ensure_private(root).map_err(|_| internal_error("app-root"))?;
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
        Self::open_with_owner_and_publishers_clock(
            root,
            client_id,
            Arc::new(SystemMonotonicClock::new()?),
            owner,
            [],
        )
    }

    /// Open with an injected owner and deterministic monotonic instant.
    pub fn open_with_owner_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
        owner: Arc<AppService>,
    ) -> Result<Self, AppError> {
        Self::open_with_owner_and_publishers_at(root, client_id, now_monotonic, owner, [])
    }

    /// Open with explicit wallet and Extension trust composition owners.
    pub fn open_with_owner_and_publishers_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
        owner: Arc<AppService>,
        trusted_publishers: impl IntoIterator<Item = TrustedPublisher>,
    ) -> Result<Self, AppError> {
        if now_monotonic == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        Self::open_with_owner_and_publishers_clock(
            root,
            client_id,
            Arc::new(FixedMonotonicClock(now_monotonic)),
            owner,
            trusted_publishers,
        )
    }

    /// Open with an injected trusted clock sampled independently for every call.
    pub fn open_with_owner_and_publishers_clock(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        clock: Arc<dyn FacadeMonotonicClock>,
        owner: Arc<AppService>,
        trusted_publishers: impl IntoIterator<Item = TrustedPublisher>,
    ) -> Result<Self, AppError> {
        let root = root.as_ref().to_path_buf();
        let root_guard =
            SecureDir::ensure_private(&root).map_err(|_| internal_error("app-root"))?;
        let initial_clock_sample = clock.now_monotonic()?;
        if initial_clock_sample == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        if load_clock_floor(&root_guard)?.is_some_and(|floor| initial_clock_sample < floor) {
            return Err(AppError::IntegrityFailure);
        }
        persist_clock_floor(&root_guard, initial_clock_sample)?;
        Ok(Self {
            root_guard,
            bound_client_id: client_id.clone(),
            wallets: WalletAdapter::new(owner)?,
            actions: ActionAdapter::new(),
            extensions: ExtensionAdapter::open_with_publishers(
                root.join("extensions"),
                client_id.clone(),
                trusted_publishers,
            )?,
            journal: DurableJournal::open_for_client(root.join("journal"), client_id)?,
            clock,
            last_clock_sample: initial_clock_sample,
            next_subscription: 1,
        })
    }

    /// Return the durable infrastructure journal for integration verification.
    pub fn journal_mut(&mut self) -> &mut DurableJournal {
        &mut self.journal
    }

    fn check(&mut self, operation: AppOperation) -> Result<u64, AppError> {
        owner_route(operation).ok_or(AppError::UnknownOperation)?;
        let now = self.clock.now_monotonic()?;
        if now == 0 || now < self.last_clock_sample {
            return Err(AppError::IntegrityFailure);
        }
        if now > self.last_clock_sample {
            persist_clock_floor(&self.root_guard, now)?;
        }
        self.last_clock_sample = now;
        Ok(now)
    }

    fn owner_error_after_binding(
        &mut self,
        operation_id: &OperationId,
        request_digest: [u8; 32],
        deadline: u64,
        error: AppError,
    ) -> AppError {
        if matches!(error, AppError::UnknownOutcome { .. }) {
            return error;
        }
        match self
            .journal
            .discard_binding(operation_id, request_digest, deadline)
        {
            Ok(()) => error,
            Err(_) => AppError::IntegrityFailure,
        }
    }

    fn execute_owner_request(
        &mut self,
        request: AppRequest,
        deadline: u64,
        now_monotonic: u64,
    ) -> Result<OperationProjection, AppError> {
        let request_digest =
            canonical_owner_request_digest(&request).map_err(|_| AppError::IntegrityFailure)?;
        let existing = self.journal.bound_operation(request_digest)?;
        let was_existing = existing.is_some();
        let (operation_id, deadline) = match existing {
            Some(binding) => binding,
            None => {
                let operation_id = self.extensions.reserve(&request, deadline, now_monotonic)?;
                if let Err(error) =
                    self.journal
                        .bind_request(operation_id.clone(), request_digest, deadline)
                {
                    return match self.extensions.fail_reserved(&operation_id) {
                        Ok(()) => Err(error),
                        Err(_) => Err(AppError::IntegrityFailure),
                    };
                }
                (operation_id, deadline)
            }
        };

        if was_existing {
            let observed = self.extensions.observe(&operation_id)?;
            if observed.projection.state != z00z_app_api::OperationState::Pending
                || observed.projection.effect_count != 0
            {
                return self.mirror_owner_observation(&request, deadline, observed);
            }
        }

        let projection = match self.extensions.execute_reserved(
            request.clone(),
            &operation_id,
            deadline,
            now_monotonic,
        ) {
            Ok(projection) => projection,
            Err(error) => {
                return Err(self.owner_error_after_binding(
                    &operation_id,
                    request_digest,
                    deadline,
                    error,
                ));
            }
        };
        if projection.id != operation_id {
            return Err(AppError::UnknownOutcome { operation_id });
        }
        self.persist_owner_result(&request, deadline, projection)
    }

    fn persist_owner_result(
        &mut self,
        request: &AppRequest,
        deadline: u64,
        projection: OperationProjection,
    ) -> Result<OperationProjection, AppError> {
        let operation_id = projection.id.clone();
        let observed =
            self.extensions
                .observe(&operation_id)
                .map_err(|_| AppError::UnknownOutcome {
                    operation_id: operation_id.clone(),
                })?;
        if observed.projection != projection
            || observed.request_digest
                != canonical_owner_request_digest(request)
                    .map_err(|_| AppError::IntegrityFailure)?
            || observed.deadline != deadline
        {
            return Err(AppError::UnknownOutcome { operation_id });
        }
        self.mirror_owner_observation(request, deadline, observed)
    }

    fn mirror_owner_observation(
        &mut self,
        request: &AppRequest,
        deadline: u64,
        observed: z00z_app_ext::OwnerObservation,
    ) -> Result<OperationProjection, AppError> {
        let operation_id = observed.projection.id.clone();
        let request_digest =
            canonical_owner_request_digest(request).map_err(|_| AppError::IntegrityFailure)?;
        if observed.request_digest != request_digest || observed.deadline != deadline {
            return Err(AppError::IntegrityFailure);
        }
        let result = match self.journal.record(&operation_id) {
            Ok(identity) => {
                if identity.request_digest != request_digest || identity.deadline != deadline {
                    return Err(AppError::IntegrityFailure);
                }
                self.journal
                    .update_projection(observed.projection, request_digest, deadline)
            }
            Err(AppError::NotFound) => self.journal.persist_projection_from_binding(
                observed.projection,
                request_digest,
                deadline,
            ),
            Err(_) => return Err(AppError::UnknownOutcome { operation_id }),
        };
        match result {
            Ok(projection) => Ok(projection),
            Err(_) => Err(AppError::UnknownOutcome { operation_id }),
        }
    }

    fn capability(
        &self,
        now_monotonic: u64,
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
                CapabilityFreshness::Timestamp(now_monotonic)
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
            now_monotonic,
        )
    }
}

impl AppApiService for AppFacade {
    fn bound_client_id(&self) -> Option<&BoundedId> {
        Some(&self.bound_client_id)
    }

    fn get_runtime_status(&mut self) -> Result<RuntimeStatus, AppError> {
        let now = self.check(AppOperation::GetRuntimeStatus)?;
        Ok(RuntimeStatus {
            is_ready: true,
            revision: now,
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
        let now = self.check(AppOperation::GetRuntimeCapabilities)?;
        let wallet_read = self.wallets.probe().is_ok();
        let extension_read = self.extensions.probe().is_ok();
        CapabilityPage::new(vec![
            self.capability(now, "wallet-read", wallet_read, "wallet-owner-unavailable")?,
            self.capability(
                now,
                "extension-read",
                extension_read,
                "extension-owner-unavailable",
            )?,
            self.capability(
                now,
                "wallet-intent-effect",
                false,
                "phase-006-owner-unavailable",
            )?,
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

    fn cancel_operation(&mut self, _id: OperationId) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::CancelOperation)?;
        Err(AppError::CapabilityUnavailable {
            code: BoundedId::new("operation-cancel-owner")?,
        })
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
        let now = self.check(AppOperation::BeginExtensionImport)?;
        let canonical_request = AppRequest::BeginExtensionImport(request.clone());
        let deadline = request.expires_at_monotonic();
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn upload_extension_chunk(
        &mut self,
        request: AppendExtensionImportChunkRequest,
    ) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::AppendExtensionImportChunk)?;
        let canonical_request = AppRequest::AppendExtensionImportChunk(request.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn commit_extension_import(
        &mut self,
        id: OperationId,
    ) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::FinishExtensionImport)?;
        let canonical_request = AppRequest::FinishExtensionImport(id.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn get_extension_operation(
        &mut self,
        id: OperationId,
    ) -> Result<OperationProjection, AppError> {
        self.check(AppOperation::GetExtensionOperation)?;
        let observed = self.extensions.observe(&id)?;
        match self.journal.record(&id) {
            Ok(identity) => {
                if identity.request_digest != observed.request_digest
                    || identity.deadline != observed.deadline
                {
                    return Err(AppError::IntegrityFailure);
                }
                self.journal.update_projection(
                    observed.projection,
                    observed.request_digest,
                    observed.deadline,
                )
            }
            Err(AppError::NotFound) => self.journal.persist_projection_from_binding(
                observed.projection,
                observed.request_digest,
                observed.deadline,
            ),
            Err(error) => Err(error),
        }
    }

    fn abort_extension_import(&mut self, id: OperationId) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::CancelExtensionOperation)?;
        let canonical_request = AppRequest::CancelExtensionOperation(id.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn update_extension(
        &mut self,
        request: UpdateExtensionRequest,
    ) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::UpdateExtension)?;
        let _ = now;
        self.extensions.update(&request)
    }

    fn set_extension_enabled(
        &mut self,
        request: SetExtensionEnabledRequest,
    ) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::SetExtensionEnabled)?;
        let canonical_request = AppRequest::SetExtensionEnabled(request.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn set_extension_local_block(
        &mut self,
        request: SetExtensionLocalBlockRequest,
    ) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::SetExtensionLocalBlock)?;
        let canonical_request = AppRequest::SetExtensionLocalBlock(request.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn remove_extension(&mut self, id: ExtensionId) -> Result<OperationProjection, AppError> {
        let now = self.check(AppOperation::RemoveExtension)?;
        let canonical_request = AppRequest::RemoveExtension(id.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        self.execute_owner_request(canonical_request, deadline, now)
    }

    fn prepare_extension_artifact(
        &mut self,
        id: ExtensionId,
    ) -> Result<ExtensionArtifactEnvelope, AppError> {
        let now = self.check(AppOperation::PrepareExtensionArtifact)?;
        let canonical_request = AppRequest::PrepareExtensionArtifact(id.clone());
        let deadline = self.extensions.owner_deadline(now)?;
        let request_digest = canonical_owner_request_digest(&canonical_request)
            .map_err(|_| AppError::IntegrityFailure)?;
        if let Some((operation_id, bound_deadline)) =
            self.journal.bound_operation(request_digest)?
        {
            let observed = self.extensions.observe(&operation_id)?;
            self.mirror_owner_observation(&canonical_request, bound_deadline, observed)?;
            return Err(AppError::UnknownOutcome { operation_id });
        }
        let operation_id = self.extensions.reserve_prepare(&id, deadline)?;
        if let Err(error) =
            self.journal
                .bind_request(operation_id.clone(), request_digest, deadline)
        {
            return match self.extensions.fail_reserved(&operation_id) {
                Ok(()) => Err(error),
                Err(_) => Err(AppError::IntegrityFailure),
            };
        }
        let envelope = match self
            .extensions
            .execute_reserved_prepare(&id, &operation_id, deadline)
        {
            Ok(envelope) => envelope,
            Err(error) => {
                return Err(self.owner_error_after_binding(
                    &operation_id,
                    request_digest,
                    deadline,
                    error,
                ));
            }
        };
        let observed = self
            .extensions
            .observe(&envelope.operation_id)
            .map_err(|_| AppError::UnknownOutcome {
                operation_id: envelope.operation_id.clone(),
            })?;
        self.mirror_owner_observation(&canonical_request, deadline, observed)
            .map_err(|_| AppError::UnknownOutcome {
                operation_id: envelope.operation_id.clone(),
            })?;
        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_app_api::{
        AppRequest, AppendExtensionImportChunkRequest, BeginExtensionImportRequest, BoundedBytes,
        OperationState, ReconciliationState,
    };
    use z00z_app_ext::{
        canonical_artifact_bytes, content_digest, encode_kernel_signature, package_signing_bytes,
        publisher_id_for_key, ExtensionArtifactV1, ExtensionPackage, PackageDraft,
    };
    use z00z_crypto::{sign_kernel_signature, Z00ZRistrettoPoint, Z00ZScalar};
    use z00z_utils::io::{create_dir_all, TemporaryDirectory};
    use z00z_utils::rng::SystemRngProvider;

    const PER_CLIENT_UPLOAD_LIMIT: usize = 4;
    const GLOBAL_UPLOAD_LIMIT: usize = 64;

    struct AdvancingClock(AtomicU64);

    impl AdvancingClock {
        fn set(&self, now: u64) {
            self.0.store(now, Ordering::Release);
        }
    }

    impl FacadeMonotonicClock for AdvancingClock {
        fn now_monotonic(&self) -> Result<u64, AppError> {
            Ok(self.0.load(Ordering::Acquire))
        }
    }

    fn signed_extension(
        extension_id: &str,
    ) -> Result<(PackageDraft, Vec<u8>, TrustedPublisher), Box<dyn std::error::Error>> {
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
            publisher_name: BoundedText::new("Z00Z Facade Test Publisher")?,
            publisher_public_key: public_key,
            name: BoundedText::new("Facade Test Extension")?,
            artifact_kind: BoundedId::new("prepare_offline_payment")?,
            object_families: vec![BoundedId::new("asset")?],
            package_version: 1,
            artifact,
        };
        let message = package_signing_bytes(&draft)?;
        let mut rng = SystemRngProvider.rng();
        let signature = sign_kernel_signature(&secret, message, &mut rng)?;
        let package = ExtensionPackage::from_signed_draft(
            draft.clone(),
            encode_kernel_signature(&signature),
        )?;
        let trusted = TrustedPublisher::new(draft.publisher_id.clone(), public_key, 1, false)?;
        Ok((draft, package.package_bytes().to_vec(), trusted))
    }

    fn succeeded(id: &str) -> Result<OperationProjection, AppError> {
        Ok(OperationProjection {
            id: BoundedId::new(id)?,
            state: OperationState::Succeeded,
            reconciliation: ReconciliationState::Settled,
            revision: 2,
            effect_count: 1,
        })
    }

    #[test]
    fn journal_failures_preserve_owner_ids() -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let app_root = root.path().to_path_buf();
        let mut facade = AppFacade::open_at(&app_root, BoundedId::new("journal-fault-owner")?, 10)?;
        let upload_id = BoundedId::new("owner-upload")?;
        let extension_id = BoundedId::new("owner-extension")?;
        let requests = [
            AppRequest::BeginExtensionImport(BeginExtensionImportRequest::new(
                extension_id.clone(),
                1,
                [1; 32],
                100,
            )?),
            AppRequest::AppendExtensionImportChunk(AppendExtensionImportChunkRequest::new(
                upload_id.clone(),
                0,
                BoundedBytes::new(vec![1])?,
            )?),
            AppRequest::FinishExtensionImport(upload_id),
            AppRequest::SetExtensionEnabled(SetExtensionEnabledRequest::new(
                extension_id.clone(),
                true,
            )),
            AppRequest::SetExtensionLocalBlock(SetExtensionLocalBlockRequest::new(
                extension_id.clone(),
                true,
            )),
            AppRequest::RemoveExtension(extension_id),
        ];

        for (index, request) in requests.into_iter().enumerate() {
            let operation_id = BoundedId::new(format!("owner-request-{index}"))?;
            let digest = canonical_owner_request_digest(&request)?;
            facade
                .journal
                .bind_request(operation_id.clone(), digest, 100)?;
            create_dir_all(
                app_root
                    .join("journal")
                    .join(format!("{}.record", operation_id.as_str())),
            )?;
            let projection = OperationProjection {
                id: operation_id.clone(),
                state: OperationState::Succeeded,
                reconciliation: ReconciliationState::Settled,
                revision: 2,
                effect_count: 1,
            };
            let frame = request.encode()?.encode()?;
            let observed = z00z_app_ext::OwnerObservation {
                projection,
                request_digest: z00z_crypto::hash::sha256_256(
                    "z00z.app.extension.owner.v1",
                    "canonical-request",
                    &[&frame],
                ),
                deadline: 100,
            };
            assert_eq!(
                facade.mirror_owner_observation(&request, 100, observed),
                Err(AppError::UnknownOutcome { operation_id })
            );
        }
        Ok(())
    }

    #[test]
    fn owner_identity_is_recomputed_before_journal_mutation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let mut facade = AppFacade::open_at(root.path(), BoundedId::new("identity-owner")?, 10)?;
        let request = AppRequest::SetExtensionEnabled(SetExtensionEnabledRequest::new(
            BoundedId::new("identity-extension")?,
            true,
        ));
        let projection = succeeded("identity-operation")?;
        let digest = canonical_owner_request_digest(&request)?;

        assert_eq!(
            facade.mirror_owner_observation(
                &request,
                610,
                z00z_app_ext::OwnerObservation {
                    projection: projection.clone(),
                    request_digest: [7; 32],
                    deadline: 610,
                },
            ),
            Err(AppError::IntegrityFailure)
        );
        assert_eq!(facade.journal.get(&projection.id), Err(AppError::NotFound));

        assert_eq!(
            facade.mirror_owner_observation(
                &request,
                610,
                z00z_app_ext::OwnerObservation {
                    projection: projection.clone(),
                    request_digest: digest,
                    deadline: 611,
                },
            ),
            Err(AppError::IntegrityFailure)
        );
        assert_eq!(facade.journal.get(&projection.id), Err(AppError::NotFound));
        Ok(())
    }

    #[test]
    fn existing_journal_identity_conflicts_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let mut facade = AppFacade::open_at(root.path(), BoundedId::new("conflict-owner")?, 10)?;
        let request = AppRequest::SetExtensionEnabled(SetExtensionEnabledRequest::new(
            BoundedId::new("conflict-extension")?,
            true,
        ));
        let projection = succeeded("conflict-operation")?;
        let digest = canonical_owner_request_digest(&request)?;
        facade
            .journal_mut()
            .persist_projection(projection.clone(), [9; 32], 610)?;

        assert_eq!(
            facade.mirror_owner_observation(
                &request,
                610,
                z00z_app_ext::OwnerObservation {
                    projection,
                    request_digest: digest,
                    deadline: 610,
                },
            ),
            Err(AppError::IntegrityFailure)
        );
        Ok(())
    }

    #[test]
    fn matching_owner_identity_survives_facade_reopen() -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let request = AppRequest::SetExtensionLocalBlock(SetExtensionLocalBlockRequest::new(
            BoundedId::new("reopen-extension")?,
            true,
        ));
        let projection = succeeded("reopen-operation")?;
        let digest = canonical_owner_request_digest(&request)?;
        let observation = z00z_app_ext::OwnerObservation {
            projection: projection.clone(),
            request_digest: digest,
            deadline: 610,
        };
        {
            let mut facade = AppFacade::open_at(root.path(), BoundedId::new("reopen-owner")?, 10)?;
            facade
                .journal
                .bind_request(projection.id.clone(), digest, 610)?;
            assert_eq!(
                facade.mirror_owner_observation(&request, 610, observation.clone())?,
                projection
            );
        }
        let mut reopened = AppFacade::open_at(root.path(), BoundedId::new("reopen-owner")?, 10)?;
        assert_eq!(
            reopened.mirror_owner_observation(&request, 610, observation)?,
            projection
        );
        Ok(())
    }

    #[test]
    fn first_journal_write_failure_recovers_original_owner_identity_after_reopen(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let facade_root = root.path().join("facade");
        let client_id = BoundedId::new("first-write-recovery-client")?;
        let wallet_service = Arc::new(WalletService::with_output_dir(root.path().join("wallet")));
        let owner = Arc::new(AppService::with_wallet_service(wallet_service));
        let (draft, package_bytes, trusted) = signed_extension("first-write-extension")?;
        let mut facade = AppFacade::open_with_owner_and_publishers_at(
            &facade_root,
            client_id.clone(),
            10,
            owner,
            [trusted.clone()],
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

        let canonical_request = AppRequest::PrepareExtensionArtifact(draft.extension_id.clone());
        let expected_digest = canonical_owner_request_digest(&canonical_request)?;
        facade.journal.fail_next_record_write_for_test();
        let operation_id = match facade.prepare_extension_artifact(draft.extension_id) {
            Err(AppError::UnknownOutcome { operation_id }) => operation_id,
            other => {
                return Err(format!("expected first-write unknown outcome, got {other:?}").into());
            }
        };
        let restored = facade.get_extension_operation(operation_id.clone())?;
        assert_eq!(restored.state, OperationState::Succeeded);
        assert_eq!(restored.effect_count, 1);
        let identity = facade.journal.record(&operation_id)?;
        assert_eq!(identity.client_id, client_id);
        assert_eq!(identity.request_digest, expected_digest);
        assert_eq!(identity.deadline, 610);
        drop(facade);

        let reopened_owner = Arc::new(AppService::with_wallet_service(Arc::new(
            WalletService::with_output_dir(root.path().join("wallet-reopened")),
        )));
        let mut reopened = AppFacade::open_with_owner_and_publishers_at(
            &facade_root,
            client_id,
            10,
            reopened_owner,
            [trusted],
        )?;
        let after_reopen = reopened.get_extension_operation(operation_id.clone())?;
        assert_eq!(after_reopen, restored);
        assert_eq!(after_reopen.effect_count, 1);
        let identity = reopened.journal.record(&operation_id)?;
        assert_eq!(identity.request_digest, expected_digest);
        assert_eq!(identity.deadline, 610);
        Ok(())
    }

    #[test]
    fn advancing_clock_expires_upload_at_exact_deadline_and_cleans_quarantine(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let clock = Arc::new(AdvancingClock(AtomicU64::new(10)));
        let owner = Arc::new(AppService::with_wallet_service(Arc::new(
            WalletService::with_output_dir(root.path().join("wallet")),
        )));
        let mut facade = AppFacade::open_with_owner_and_publishers_clock(
            root.path().join("facade"),
            BoundedId::new("advancing-clock-client")?,
            clock.clone(),
            owner,
            [],
        )?;
        assert_eq!(facade.get_runtime_status()?.revision, 10);
        let upload = facade.begin_extension_import(BeginExtensionImportRequest::new(
            BoundedId::new("advancing-clock-extension")?,
            1,
            [11; 32],
            20,
        )?)?;
        assert_eq!(facade.extensions.active_uploads_for_test(), 1);

        clock.set(20);
        assert_eq!(facade.get_runtime_status()?.revision, 20);
        assert_eq!(
            facade.upload_extension_chunk(AppendExtensionImportChunkRequest::new(
                upload.id,
                0,
                BoundedBytes::new(vec![1])?,
            )?),
            Err(AppError::TimeoutKnownNotSubmitted)
        );
        assert_eq!(facade.extensions.active_uploads_for_test(), 0);
        assert!(facade.extensions.quarantine_is_empty_for_test());
        clock.set(19);
        assert_eq!(facade.get_runtime_status(), Err(AppError::IntegrityFailure));
        Ok(())
    }

    #[test]
    fn crash_before_response_retries_exact_reserved_owner_operation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let facade_root = root.path().join("facade");
        let client_id = BoundedId::new("crash-before-response-client")?;
        let request = BeginExtensionImportRequest::new(
            BoundedId::new("crash-before-response-extension")?,
            1,
            [31; 32],
            100,
        )?;
        let canonical = AppRequest::BeginExtensionImport(request.clone());
        let digest = canonical_owner_request_digest(&canonical)?;
        let operation_id;
        let owner_projection;
        {
            let mut facade = AppFacade::open_at(&facade_root, client_id.clone(), 10)
                .expect("open initial crash facade");
            operation_id = facade.extensions.reserve(&canonical, 100, 10)?;
            facade
                .journal
                .bind_request(operation_id.clone(), digest, 100)?;
            owner_projection =
                facade
                    .extensions
                    .execute_reserved(canonical.clone(), &operation_id, 100, 10)?;
            assert_eq!(facade.journal.get(&operation_id), Err(AppError::NotFound));
        }

        let mut reopened =
            AppFacade::open_at(&facade_root, client_id, 10).expect("reopen crash facade");
        let retried = reopened.begin_extension_import(request)?;
        assert_eq!(retried, owner_projection);
        assert_eq!(retried.id, operation_id);
        assert_eq!(reopened.extensions.active_uploads_for_test(), 1);
        assert_eq!(reopened.journal.get(&operation_id)?, owner_projection);
        Ok(())
    }

    #[test]
    fn identical_requests_at_one_deadline_keep_distinct_owner_ids(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let mut facade =
            AppFacade::open_at(root.path(), BoundedId::new("same-request-client")?, 10)?;
        let request = BeginExtensionImportRequest::new(
            BoundedId::new("same-request-extension")?,
            1,
            [32; 32],
            100,
        )?;
        let first = facade.begin_extension_import(request.clone())?;
        let second = facade.begin_extension_import(request)?;
        assert_ne!(first.id, second.id);
        assert_eq!(facade.journal.get(&first.id)?, first);
        assert_eq!(facade.journal.get(&second.id)?, second);
        assert_eq!(facade.extensions.active_uploads_for_test(), 2);
        Ok(())
    }

    #[test]
    fn retry_after_clock_advance_uses_bound_owner_deadline(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let facade_root = root.path().join("facade");
        let client_id = BoundedId::new("advanced-retry-client")?;
        let request;
        let operation_id;
        let owner_projection;
        {
            let mut facade = AppFacade::open_at(&facade_root, client_id.clone(), 10)?;
            let upload = facade.begin_extension_import(BeginExtensionImportRequest::new(
                BoundedId::new("advanced-retry-extension")?,
                1,
                content_digest(&[71]),
                100,
            )?)?;
            request =
                AppendExtensionImportChunkRequest::new(upload.id, 0, BoundedBytes::new(vec![71])?)?;
            let canonical = AppRequest::AppendExtensionImportChunk(request.clone());
            let digest = canonical_owner_request_digest(&canonical)?;
            operation_id = facade.extensions.reserve(&canonical, 610, 10)?;
            facade
                .journal
                .bind_request(operation_id.clone(), digest, 610)?;
            owner_projection =
                facade
                    .extensions
                    .execute_reserved(canonical, &operation_id, 610, 10)?;
        }

        let mut reopened = AppFacade::open_at(&facade_root, client_id, 11)?;
        let retried = reopened.upload_extension_chunk(request)?;
        assert_eq!(retried, owner_projection);
        assert_eq!(retried.id, operation_id);
        assert_eq!(reopened.journal.record(&operation_id)?.deadline, 610);
        Ok(())
    }

    #[test]
    fn expired_uploads_release_per_client_and_global_facade_admission(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let per_client_root = TemporaryDirectory::new()?;
        let clock = Arc::new(AdvancingClock(AtomicU64::new(10)));
        let owner = Arc::new(AppService::with_wallet_service(Arc::new(
            WalletService::with_output_dir(per_client_root.path().join("wallet")),
        )));
        let mut facade = AppFacade::open_with_owner_and_publishers_clock(
            per_client_root.path().join("facade"),
            BoundedId::new("expired-per-client")?,
            clock.clone(),
            owner,
            [],
        )?;
        for index in 0..PER_CLIENT_UPLOAD_LIMIT {
            facade.begin_extension_import(BeginExtensionImportRequest::new(
                BoundedId::new(format!("expired-per-client-{index}"))?,
                1,
                [u8::try_from(index + 1)?; 32],
                20,
            )?)?;
        }
        clock.set(20);
        facade.begin_extension_import(BeginExtensionImportRequest::new(
            BoundedId::new("replacement-per-client")?,
            1,
            [40; 32],
            30,
        )?)?;
        assert_eq!(facade.extensions.active_uploads_for_test(), 1);

        let global_root = TemporaryDirectory::new()?;
        let facade_root = global_root.path().join("facade");
        for client in 0..(GLOBAL_UPLOAD_LIMIT / PER_CLIENT_UPLOAD_LIMIT) {
            let mut client_facade = AppFacade::open_at(
                &facade_root,
                BoundedId::new(format!("expired-global-client-{client}"))?,
                10,
            )?;
            for upload in 0..PER_CLIENT_UPLOAD_LIMIT {
                client_facade.begin_extension_import(BeginExtensionImportRequest::new(
                    BoundedId::new(format!("expired-global-{client}-{upload}"))?,
                    1,
                    [u8::try_from(upload + 1)?; 32],
                    20,
                )?)?;
            }
        }
        let global_clock = Arc::new(AdvancingClock(AtomicU64::new(10)));
        let global_owner = Arc::new(AppService::with_wallet_service(Arc::new(
            WalletService::with_output_dir(global_root.path().join("wallet")),
        )));
        let mut global_facade = AppFacade::open_with_owner_and_publishers_clock(
            &facade_root,
            BoundedId::new("expired-global-replacement")?,
            global_clock.clone(),
            global_owner,
            [],
        )?;
        global_clock.set(20);
        global_facade.begin_extension_import(BeginExtensionImportRequest::new(
            BoundedId::new("replacement-global")?,
            1,
            [41; 32],
            30,
        )?)?;
        assert_eq!(global_facade.extensions.active_uploads_for_test(), 1);
        Ok(())
    }

    #[test]
    fn durable_clock_floor_rejects_rollback_without_extending_expiry(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let facade_root = root.path().join("facade");
        let client_id = BoundedId::new("durable-clock-client")?;
        {
            let mut facade = AppFacade::open_at(&facade_root, client_id.clone(), 10)?;
            facade.begin_extension_import(BeginExtensionImportRequest::new(
                BoundedId::new("durable-clock-expiring")?,
                1,
                [51; 32],
                20,
            )?)?;
            let clock = Arc::new(AdvancingClock(AtomicU64::new(10)));
            facade.clock = clock.clone();
            clock.set(20);
            assert_eq!(facade.get_runtime_status()?.revision, 20);
        }
        assert!(matches!(
            AppFacade::open_at(&facade_root, client_id.clone(), 19),
            Err(AppError::IntegrityFailure)
        ));
        let mut reopened = AppFacade::open_at(&facade_root, client_id, 20)?;
        reopened.begin_extension_import(BeginExtensionImportRequest::new(
            BoundedId::new("durable-clock-replacement")?,
            1,
            [52; 32],
            30,
        )?)?;
        assert_eq!(reopened.extensions.active_uploads_for_test(), 1);
        Ok(())
    }

    #[test]
    fn torn_clock_floor_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
        let root = TemporaryDirectory::new()?;
        let facade_root = root.path().join("facade");
        let client_id = BoundedId::new("torn-clock-client")?;
        drop(AppFacade::open_at(&facade_root, client_id.clone(), 10)?);
        std::fs::OpenOptions::new()
            .write(true)
            .open(facade_root.join(CLOCK_FLOOR_FILE))?
            .set_len(0)?;
        assert!(matches!(
            AppFacade::open_at(&facade_root, client_id, 10),
            Err(AppError::IntegrityFailure)
        ));
        Ok(())
    }
}
