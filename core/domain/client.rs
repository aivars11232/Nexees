//! What a UI client shows: its foreground workspace (ST-VIEW).
//!
//! UI focus is authoritative for presentation only. No type in this crate takes a
//! [`ClientState`] as a target, an agent binding, an import destination or a model-switch
//! session. Every one of those names its workspace explicitly, so switching the visible
//! workspace can never rebind or redirect work (C2, R11, RC-20, AD-06). The layout details of a
//! client, such as tabs, tree expansion and panels, are added by TASK-012 and TASK-016.

use serde::{Deserialize, Serialize};

use crate::ids::{ClientId, DeviceId, WorkspaceId};
use crate::schema::{Record, SyncPolicy};

/// One UI client's foreground selection. Each client has its own; another client, another
/// device or a synchronized change never moves it (A3, A6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientState {
    /// The client.
    pub client_id: ClientId,
    /// The device the client runs on.
    pub device_id: DeviceId,
    /// The workspace the client shows, if any.
    pub foreground_workspace: Option<WorkspaceId>,
}

impl Record for ClientState {
    const SCHEMA: &'static str = "nexees.client.state";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Versioned;

    #[test]
    fn client_focus_round_trips_and_rejects_smuggled_targets() {
        let state = ClientState {
            client_id: ClientId::new("window-1").unwrap(),
            device_id: DeviceId::new("pc").unwrap(),
            foreground_workspace: Some(WorkspaceId::new("lcl-next").unwrap()),
        };
        let text = serde_json::to_string(&Versioned(state.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<ClientState>>(&text)
                .unwrap()
                .0,
            state
        );
        let with_binding =
            r#"{"client_id":"w","device_id":"pc","foreground_workspace":null,"agent_session":"s"}"#;
        assert!(serde_json::from_str::<ClientState>(with_binding).is_err());
    }
}
