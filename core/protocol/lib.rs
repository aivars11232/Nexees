//! The Nexees protocol: what travels between a window and its host, and between two paired
//! hosts (SS-DOMAIN-PROTOCOL, C15).
//!
//! It carries the domain types of `nexees-domain` and adds nothing to their meaning. Its modules
//! are the slots of `docs/architecture/subsystems.lcl.txt`:
//!
//! - [`version_negotiation`]: every channel opens with a hello and refuses a mismatch, never
//!   downgrading (STOP-08);
//! - [`serialization`]: bounded, strict JSON, and the canonical form of a request that approvals
//!   bind to;
//! - [`operations`]: the closed registry of operations a peer or the local user may request;
//! - [`messages`]: the message families, kept apart, and the reconnect rules;
//! - [`admission`]: the protocol's part of admitting a remote request (EC-01 to EC-06), and the
//!   meaning of stale revisions and bindings.
//!
//! The crate defines contracts and pure rules; it opens no socket, stores nothing and runs no
//! operation. Those belong to the other schema tasks of its interfaces: the local attachment to
//! TASK-009 and TASK-057, the stores to TASK-008, the transport to TASK-060 and TASK-070,
//! synchronization to TASK-069, and the runtime that admits and executes remote requests to
//! TASK-070.

#![forbid(unsafe_code)]

pub mod admission;
pub mod messages;
pub mod operations;
pub mod serialization;
pub mod version_negotiation;
