//! Bounded same-origin browser resource helpers.

use std::io::{Error, ErrorKind};

use super::IoError;

fn invalid_input(message: &'static str) -> IoError {
    IoError::Io(Error::new(ErrorKind::InvalidInput, message))
}

fn runtime_failure(message: &'static str) -> IoError {
    IoError::Io(Error::other(message))
}

fn validate_same_origin_path(path: &str) -> Result<(), IoError> {
    if !path.starts_with('/')
        || path.starts_with("//")
        || path.contains("://")
        || path.contains('\\')
        || path.split('/').any(|segment| matches!(segment, "." | ".."))
    {
        return Err(invalid_input("browser resource path is not same-origin"));
    }
    Ok(())
}

/// Yield for the requested browser timer duration.
pub async fn delay_ms(milliseconds: u32) {
    gloo_timers::future::TimeoutFuture::new(milliseconds).await;
}

/// Read one non-empty same-origin browser resource with a caller-owned byte cap.
pub async fn read_web_resource_bounded(path: &str, max_bytes: usize) -> Result<Vec<u8>, IoError> {
    validate_same_origin_path(path)?;
    if max_bytes == 0 {
        return Err(invalid_input("browser resource byte cap is zero"));
    }
    let response = gloo_net::http::Request::get(path)
        .send()
        .await
        .map_err(|_| runtime_failure("browser resource request failed"))?;
    if !response.ok() {
        return Err(runtime_failure("browser resource status is not successful"));
    }
    let bytes = response
        .binary()
        .await
        .map_err(|_| runtime_failure("browser resource body failed"))?;
    if bytes.is_empty() {
        return Err(runtime_failure("browser resource is empty"));
    }
    if bytes.len() > max_bytes {
        return Err(IoError::FileTooLarge {
            size: bytes.len() as u64,
            max: max_bytes as u64,
        });
    }
    Ok(bytes)
}

/// Return the active browser pathname without exposing the full URL.
pub fn web_location_pathname() -> Result<String, IoError> {
    web_sys::window()
        .ok_or_else(|| runtime_failure("browser window is unavailable"))?
        .location()
        .pathname()
        .map_err(|_| runtime_failure("browser pathname is unavailable"))
}
