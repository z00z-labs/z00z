//! Bounded same-origin browser resource helpers.

use std::io::{Error, ErrorKind};

use js_sys::{Reflect, Uint8Array};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::ReadableStreamDefaultReader;

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
        || path.contains(['?', '#', '%'])
        || path.chars().any(char::is_control)
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
    if response.redirected() {
        return Err(runtime_failure("browser resource redirect is forbidden"));
    }
    let stream = response
        .body()
        .ok_or_else(|| runtime_failure("browser resource body is unavailable"))?;
    let reader = ReadableStreamDefaultReader::new(&stream)
        .map_err(|_| runtime_failure("browser resource reader failed"))?;
    let mut bytes = Vec::new();
    loop {
        let item = JsFuture::from(reader.read())
            .await
            .map_err(|_| runtime_failure("browser resource body failed"))?;
        let done = Reflect::get(&item, &JsValue::from_str("done"))
            .map_err(|_| runtime_failure("browser resource body failed"))?
            .as_bool()
            .ok_or_else(|| runtime_failure("browser resource body failed"))?;
        if done {
            break;
        }
        let value = Reflect::get(&item, &JsValue::from_str("value"))
            .map_err(|_| runtime_failure("browser resource body failed"))?;
        let chunk = Uint8Array::new(&value);
        let chunk_len = usize::try_from(chunk.length())
            .map_err(|_| runtime_failure("browser resource chunk is too large"))?;
        let next_len = bytes
            .len()
            .checked_add(chunk_len)
            .ok_or_else(|| runtime_failure("browser resource size overflow"))?;
        if next_len > max_bytes {
            let _ = JsFuture::from(reader.cancel()).await;
            return Err(IoError::FileTooLarge {
                size: next_len as u64,
                max: max_bytes as u64,
            });
        }
        let start = bytes.len();
        bytes.resize(next_len, 0);
        chunk.copy_to(&mut bytes[start..]);
    }
    reader.release_lock();
    if bytes.is_empty() {
        return Err(runtime_failure("browser resource is empty"));
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
