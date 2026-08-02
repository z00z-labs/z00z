//! Adapter over the trusted declarative Extension service owner.

use std::path::Path;

use z00z_app_api::{
    AppError, AppRequest, BoundedId, ExtensionArtifactEnvelope, ExtensionId, ExtensionPage,
    ExtensionProjection, OperationId, OperationProjection, PageRequest, UpdateExtensionRequest,
};
use z00z_app_ext::{
    ExtensionGrant, ExtensionService, OwnerObservation, TrustedClientContext, TrustedPublisher,
};

use super::redaction::extension_error;

const OWNER_DEADLINE_TTL_TICKS: u64 = 600;

/// Exact trusted-service owner and server-authenticated client context.
pub struct ExtensionAdapter {
    service: ExtensionService,
    read_context: TrustedClientContext,
    artifact_context: TrustedClientContext,
    manage_context: TrustedClientContext,
}

impl ExtensionAdapter {
    pub fn open_with_publishers(
        root: impl AsRef<Path>,
        client_id: BoundedId,
        trusted_publishers: impl IntoIterator<Item = TrustedPublisher>,
    ) -> Result<Self, AppError> {
        let (service, issuer) = ExtensionService::open_with_issuer(root, trusted_publishers)
            .map_err(|error| extension_error(&error))?;
        let read_context = issuer
            .issue(client_id.clone(), [ExtensionGrant::Read])
            .map_err(|error| extension_error(&error))?;
        let artifact_context = issuer
            .issue(client_id.clone(), [ExtensionGrant::ArtifactPreparation])
            .map_err(|error| extension_error(&error))?;
        let manage_context = issuer
            .issue(client_id, [ExtensionGrant::PersistentManagement])
            .map_err(|error| extension_error(&error))?;
        Ok(Self {
            service,
            read_context,
            artifact_context,
            manage_context,
        })
    }

    pub fn owner_deadline(&self, now_monotonic: u64) -> Result<u64, AppError> {
        now_monotonic
            .checked_add(OWNER_DEADLINE_TTL_TICKS)
            .ok_or(AppError::Conflict)
    }

    pub fn reserve(
        &mut self,
        request: &AppRequest,
        deadline: u64,
        now_monotonic: u64,
    ) -> Result<OperationId, AppError> {
        self.service
            .reserve_owner_request(&self.manage_context, request, deadline, now_monotonic)
            .map_err(|error| extension_error(&error))
    }

    pub fn execute_reserved(
        &mut self,
        request: AppRequest,
        operation_id: &OperationId,
        deadline: u64,
        now_monotonic: u64,
    ) -> Result<OperationProjection, AppError> {
        self.service
            .execute_reserved_owner_request(
                &self.manage_context,
                request,
                operation_id,
                deadline,
                now_monotonic,
            )
            .map_err(|error| extension_error(&error))
    }

    pub fn fail_reserved(&mut self, operation_id: &OperationId) -> Result<(), AppError> {
        self.service
            .fail_reserved_owner_request(&self.manage_context, operation_id)
            .map_err(|error| extension_error(&error))
    }

    pub fn reserve_prepare(
        &mut self,
        id: &ExtensionId,
        deadline: u64,
    ) -> Result<OperationId, AppError> {
        self.service
            .reserve_artifact_preparation(&self.artifact_context, id, deadline)
            .map_err(|error| extension_error(&error))
    }

    pub fn execute_reserved_prepare(
        &mut self,
        id: &ExtensionId,
        operation_id: &OperationId,
        deadline: u64,
    ) -> Result<ExtensionArtifactEnvelope, AppError> {
        self.service
            .execute_reserved_artifact_preparation(
                &self.artifact_context,
                id,
                operation_id,
                deadline,
            )
            .map(|prepared| prepared.envelope)
            .map_err(|error| extension_error(&error))
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

    pub fn update(
        &mut self,
        _request: &UpdateExtensionRequest,
    ) -> Result<OperationProjection, AppError> {
        Err(AppError::CapabilityUnavailable {
            code: BoundedId::new("extension-update-binding")?,
        })
    }

    pub fn observe(&mut self, id: &OperationId) -> Result<OwnerObservation, AppError> {
        self.service
            .observe_operation(&self.read_context, id)
            .map_err(|error| extension_error(&error))
    }

    #[cfg(test)]
    pub(crate) fn active_uploads_for_test(&self) -> usize {
        self.service.active_uploads()
    }

    #[cfg(test)]
    pub(crate) fn quarantine_is_empty_for_test(&self) -> bool {
        self.service.quarantine_is_empty().unwrap_or(false)
    }
}
