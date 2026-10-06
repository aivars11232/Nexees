//! The Nexees workspaces (SS-WORKSPACES): what a device knows of its workspaces, and what keeps
//! what the user sees apart from what an agent may change (C1 to C4).
//!
//! Its modules are the slots of `docs/architecture/subsystems.lcl.txt`:
//!
//! - [`workspace_registry`]: create, open, close and list workspaces, each with a stable ID, a
//!   kind (CODE or LCL), a name, a device-local root and its content availability (C1, B18).
//!   Opening and closing set a client's foreground workspace and nothing else (C2).
//!
//! Agent bindings (TASK-015), content revisions, the file tree model and the view state of each
//! workspace (TASK-016) are slots of this subsystem that later tasks implement. The records the
//! registry keeps are the shared domain model's ([`nexees_domain::workspace`]), in the device's
//! one state store.

#![forbid(unsafe_code)]

pub mod workspace_registry;
