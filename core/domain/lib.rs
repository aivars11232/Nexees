//! Shared Nexees domain types, used unchanged by the Desktop and the Android host.
//!
//! This crate is the vocabulary the rest of Nexees speaks: workspaces and their device-local
//! roots, agent sessions and what they are bound to, tasks with their evidence, model
//! capabilities, domain events, and the shared error and outcome vocabulary. It belongs to the
//! subsystem SS-DOMAIN-PROTOCOL of `docs/architecture/subsystems.lcl.txt`, which also lists one
//! module per slot: `workspace`, `session`, `task`, `model_capabilities`, `events` and `errors`.
//!
//! The crate holds types, their validation and their serialization only. Behaviour, storage
//! and transport live in other crates, so that both hosts share one definition of every domain
//! type and neither can drift from the other (C15).

#![forbid(unsafe_code)]
