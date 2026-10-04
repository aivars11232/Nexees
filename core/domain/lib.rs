//! Shared Nexees domain types, used unchanged by the Desktop and the Android host.
//!
//! This crate is the vocabulary the rest of Nexees speaks (C15, B11). It belongs to the
//! subsystem SS-DOMAIN-PROTOCOL of `docs/architecture/subsystems.lcl.txt`, which lists one
//! module per slot:
//!
//! - identity and values: [`ids`], [`text`], [`time`], [`revision`];
//! - records and their travel rules: [`schema`], and the shared error and outcome vocabulary in
//!   [`errors`];
//! - workspaces and devices: [`workspace`], [`device`], [`client`], and the content changes that
//!   synchronization applies, [`changes`];
//! - work: [`session`], [`task`], [`model_capabilities`], [`lcl`], [`import`], [`handoff`], and
//!   the facts a host reports about them, [`events`];
//! - authority: [`authority`], [`remote_request`], and the records of remote requests,
//!   [`requests`].
//!
//! The crate holds types, their validation and their serialization only. Behaviour, storage and
//! transport live in other crates, so that both hosts share one definition of every domain type
//! and neither can drift from the other. The messages that carry these types are
//! `nexees-protocol`'s.
//!
//! What the types guarantee:
//!
//! - **Validated values.** Every identifier, path, hash and text is checked when it is built and
//!   again when it is decoded. Records whose fields must agree, such as a session and its
//!   checkout revision, exist only after their whole validation passed. A malformed or unknown
//!   field is rejected, never ignored or guessed (TASK-006 security requirements).
//! - **Versioned records.** Every stored or sent record has a unique schema name and a version
//!   ([`schema::Versioned`]); another name or version is refused.
//! - **Explicit bindings.** Sessions, tasks, imports, handoffs and requests name their device,
//!   workspace and revisions. UI focus ([`client::ClientState`]) is never one of their inputs,
//!   and a revision of another workspace is rejected (C2, C21, R11).
//! - **Separate planes.** Control records (grants, approvals, requests), synchronized records and
//!   UI focus are different types, and each record declares whether it may travel
//!   ([`schema::SyncPolicy`]); authority never does (C22, AD-04, AD-05).

#![forbid(unsafe_code)]

pub mod authority;
pub mod changes;
pub mod client;
pub mod device;
pub mod errors;
pub mod events;
pub mod handoff;
pub mod ids;
pub mod import;
pub mod lcl;
pub mod model_capabilities;
pub mod remote_request;
pub mod requests;
pub mod revision;
pub mod schema;
pub mod session;
pub mod task;
pub mod text;
pub mod time;
pub mod workspace;
