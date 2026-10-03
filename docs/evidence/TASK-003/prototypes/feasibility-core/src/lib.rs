//! TASK-003 feasibility core: evidence only, not Nexees product code.
//!
//! Each module answers one feasibility question with the smallest code that
//! exercises the candidate component for real, on Desktop and on Android:
//!
//! - [`lcl_bridge`]: the existing LCL engine, linked as a library (no second
//!   parser or validator).
//! - [`store`]: durable phone-local state in SQLite.
//! - [`import`]: bounded, staged ZIP import followed by LCL validation.
//! - [`js_backend`]: an on-device code/test backend with no ambient authority.
//! - [`help`]: passive offline help with active content removed.
//! - [`remote`]: one dispatcher for typed device requests over mutually
//!   authenticated TLS 1.3, with directional grants checked on the executing
//!   device.
//! - [`probe`]: runs every local check and reports the result.

pub mod help;
pub mod import;
pub mod js_backend;
pub mod lcl_bridge;
pub mod probe;
pub mod remote;
pub mod store;

#[cfg(target_os = "android")]
mod android;

/// Lowercase hex SHA-256 of `bytes`.
pub fn sha256_hex(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Seconds since the Unix epoch, for request expiry.
pub fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
