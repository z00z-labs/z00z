//! Sanitized application boundary errors.

use z00z_app_api::{AppError, BoundedId};

/// Convert an internal failure into an opaque incident identifier.
pub fn internal_error(code: &str) -> AppError {
    match BoundedId::new(format!("incident-{code}")) {
        Ok(incident_id) => AppError::Internal { incident_id },
        Err(_) => AppError::IntegrityFailure,
    }
}

/// Map a trusted Extension owner error without paths, keys, or raw package data.
pub fn extension_error(error: &z00z_app_ext::ServiceError) -> AppError {
    use z00z_app_ext::ServiceError;

    match error {
        ServiceError::AuthorizationDenied => AppError::AuthorizationDenied,
        ServiceError::NotFound => AppError::NotFound,
        ServiceError::Conflict | ServiceError::Rollback => AppError::Conflict,
        ServiceError::UploadExpired => AppError::TimeoutKnownNotSubmitted,
        ServiceError::Revoked | ServiceError::TrustRejected => AppError::IntegrityFailure,
        ServiceError::Bounds | ServiceError::ChunkOrder | ServiceError::DigestMismatch => {
            AppError::Validation(z00z_app_api::ValidationCode::Bounds)
        }
        ServiceError::UnknownOutcome { operation_id } => AppError::UnknownOutcome {
            operation_id: operation_id.clone(),
        },
        _ => internal_error("extension-owner"),
    }
}
