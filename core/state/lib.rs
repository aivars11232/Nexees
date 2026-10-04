//! The Nexees state store: each host's one authoritative, transactional store (SS-STATE,
//! IF-STORE, PD-PERSISTENCE).
//!
//! Every subsystem that persists records does so through this store and writes only the
//! entities it owns; none keeps a second store. Its modules are the slots of
//! `docs/architecture/subsystems.lcl.txt`:
//!
//! - [`state_store`]: the store itself. It is SQLite in WAL mode with full sync, held by one
//!   process at a time. Records are versioned and decoded strictly on every read. It keeps the
//!   outbox of content changes, the integrity checks and each session's recovery metadata.
//! - [`migrations::migration_registry`]: the ordered schema migrations, applied all at once or
//!   not at all. A migration may not change a record it does not declare.
//! - [`operation_journal`]: the write-ahead record of every effect. An interrupted effect
//!   becomes outcome-unknown and is never replayed blindly (RC-17).
//!
//! Workspace files live under their workspace roots, never in the store, and secrets live in
//! the platform credential store (R17). Checkpoints (TASK-026), the project-state entities
//! (TASK-036) and crash-safe resume (TASK-038) build on this crate. Each entity's own rules
//! belong to the subsystem that owns it.

#![forbid(unsafe_code)]

pub mod migrations {
    //! The schema migrations of the state store.
    pub mod migration_registry;
}
pub mod operation_journal;
pub mod state_store;
