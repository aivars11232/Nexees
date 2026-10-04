//! The Linux Desktop adapters of Nexees (SS-PLATFORM-DESKTOP): what the Desktop host and window
//! need from the operating system. Nothing here decides authority; it enforces what the shared
//! core and the host decided. Its modules are the slots of `docs/architecture/subsystems.lcl.txt`:
//!
//! - [`transport::local_ipc`]: the window-to-host channel, a Unix socket in a folder private to
//!   the user, with the peer's user checked by the kernel and bounded frames (PD-LOCAL-IPC,
//!   RC-23).
//! - [`lifecycle::application_lifecycle`]: one host per user, the host's lifetime, and the
//!   user-level start at login that exists only while the user has enabled it (RC-04, RC-05,
//!   RC-24).
//!
//! The other adapters of this subsystem (process execution, credentials, scoped files,
//! notifications and the peer transport) arrive with the tasks that own them.

#![forbid(unsafe_code)]

pub mod lifecycle {
    //! The Desktop host's lifetime.
    pub mod application_lifecycle;
}
pub mod transport {
    //! The Desktop's channels.
    pub mod local_ipc;
}
