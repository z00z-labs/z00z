//! Adapter over the trusted declarative Extension service owner.

use std::path::Path;

use z00z_app_api::{
    AppError, AppendExtensionImportChunkRequest, BeginExtensionImportRequest, BoundedId,
    ExtensionArtifactEnvelope, ExtensionId, ExtensionPage, ExtensionProjection, OperationId,
    OperationProjection, PageRequest, SetExtensionEnabledRequest, SetExtensionLocalBlockRequest,
    UpdateExtensionRequest,
};
use z00z_app_ext::{ExtensionGrant, ExtensionService, TrustedClientContext};

use super::redaction::extension_error;

const OWNER_DEADLINE_TTL_TICKS: u64 = 600;

/// Exact trusted-service owner and server-authenticated client context.
pub struct ExtensionAdapter {
    service: ExtensionService,
    read_context: TrustedClientContext,
    artifact_context: TrustedClientContext,
    manage_context: TrustedClientContext,
    now_monotonic: u64,
}

impl ExtensionAdapter {
    pub fn open_at(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        now_monotonic: u64,
    ) -> Result<Self, AppError> {
        if now_monotonic == 0 {
            return Err(AppError::Validation(z00z_app_api::ValidationCode::Bounds));
        }
        let service = ExtensionService::open(root, []).map_err(|error| extension_error(&error))?;
        let read_context = TrustedClientContext::from_authenticated_dispatch(
            client_id.clone(),
            [ExtensionGrant::Read],
        );
        let artifact_context = TrustedClientContext::from_authenticated_dispatch(
            client_id.clone(),
            [ExtensionGrant::ArtifactPreparation],
        );
        let manage_context = TrustedClientContext::from_authenticated_dispatch(
            client_id,
            [ExtensionGrant::PersistentManagement],
        );
        Ok(Self {
            service,
            read_context,
            artifact_context,
            manage_context,
            now_monotonic,
        })
    }

    pub fn owner_deadline(&self) -> Result<u64, AppError> {
        self.now_monotonic
            .checked_add(OWNER_DEADLINE_TTL_TICKS)
            .ok_or(AppError::Conflict)
    }

    pub fn list(&self, request: &PageRequest) -> Result<ExtensionPage, AppError> {
        let items = self
            .service
            .list_extensions(&self.read_context)
            .map_err(|error| extension_error(&error))?;
        let start = match request.cursor() {
            None => 0,
            Some(cursor) => items
                .iter()
                .position(|extension| extension.id.as_str() == cursor.as_str())
                .map(|index| index + 1)
                .ok_or(AppError::NotFound)?,
        };
        let limit = usize::from(request.limit());
        let end = start.saturating_add(limit).min(items.len());
        let page = items[start..end].to_vec();
        let next_cursor = if end < items.len() {
            page.last().map(|extension| extension.id.clone())
        } else {
            None
        };
        ExtensionPage::new(page, next_cursor)
    }

    pub fn probe(&self) -> Result<(), AppError> {
        self.service
            .list_extensions(&self.read_context)
            .map(|_| ())
            .map_err(|error| extension_error(&error))
    }

    pub fn inspect(&self, id: &ExtensionId) -> Result<ExtensionProjection, AppError> {
        self.service
            .inspect_extension(&self.read_context, id)
            .map_err(|error| extension_error(&error))
    }

    pub fn begin(
        &mut self,
        request: BeginExtensionImportRequest,
    ) -> Result<OperationProjection, AppError> {
        self.service
            .begin_import(&self.manage_context, request, self.now_monotonic)
            .map_err(|error| extension_error(&error))
    }

    pub fn append(
        &mut self,
        request: AppendExtensionImportChunkRequest,
    ) -> Result<OperationProjection, AppError> {
        self.service
            .append_chunk(&self.manage_context, request, self.now_monotonic)
            .map_err(|error| extension_error(&error))
    }

    pub fn commit(&mut self, id: &OperationId) -> Result<OperationProjection, AppError> {
        self.service
            .commit_import(&self.manage_context, id, self.now_monotonic)
            .map_err(|error| extension_error(&error))
    }

    pub fn abort(&mut self, id: &OperationId) -> Result<OperationProjection, AppError> {
        self.service
            .abort_import(&self.manage_context, id)
            .map_err(|error| extension_error(&error))
    }

    pub fn update(
        &mut self,
        _request: &UpdateExtensionRequest,
    ) -> Result<OperationProjection, AppError> {
        Err(AppError::CapabilityUnavailable {
            code: BoundedId::new("extension-update-binding")?,
        })
    }

    pub fn set_enabled(
        &mut self,
        request: &SetExtensionEnabledRequest,
    ) -> Result<OperationProjection, AppError> {
        self.service
            .set_extension_enabled(
                &self.manage_context,
                request.extension_id(),
                request.enabled(),
            )
            .map_err(|error| extension_error(&error))
    }

    pub fn set_local_block(
        &mut self,
        request: &SetExtensionLocalBlockRequest,
    ) -> Result<OperationProjection, AppError> {
        self.service
            .set_extension_local_block(
                &self.manage_context,
                request.extension_id(),
                request.blocked(),
            )
            .map_err(|error| extension_error(&error))
    }

    pub fn remove(&mut self, id: &ExtensionId) -> Result<OperationProjection, AppError> {
        self.service
            .remove_extension(&self.manage_context, id)
            .map_err(|error| extension_error(&error))
    }

    pub fn prepare(&mut self, id: &ExtensionId) -> Result<ExtensionArtifactEnvelope, AppError> {
        self.service
            .prepare_extension_artifact(&self.artifact_context, id)
            .map(|prepared| prepared.envelope)
            .map_err(|error| extension_error(&error))
    }
}
