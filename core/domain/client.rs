//! What a UI client shows (ST-VIEW): its foreground workspace, and how the user left the panels
//! of its window.
//!
//! Both are authoritative for presentation only. No type in this crate takes a [`ClientState`]
//! or a [`ClientLayout`] as a target, an agent binding, an import destination or a model-switch
//! session. Every one of those names its workspace explicitly, so switching the visible
//! workspace or rearranging a window can never rebind or redirect work (C2, R11, RC-20, AD-06).
//!
//! The panels ([`PanelLayout`]) are one layout per client: which panels are shown, how large
//! they are and which view each has selected (TASK-012). What a client keeps per workspace,
//! such as tabs and tree expansion, is added by TASK-016.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{ClientId, DeviceId, ViewId, WorkspaceId};
use crate::schema::{Record, SyncPolicy, Validate, validated_record};

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

/// The largest size a panel is remembered with, in density-independent pixels (CSS pixels on
/// Desktop). No screen is that large; a larger number is not a size.
pub const MAX_PANEL_SIZE: u32 = 16_384;

/// One panel of a client's window, as the user last left it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelState {
    /// Whether the panel is shown.
    pub shown: bool,
    /// Its size when it was last shown, in density-independent pixels: the width of a sidebar,
    /// the height of the bottom panel. Absent only for a panel that has never been shown.
    pub size: Option<u32>,
    /// The view selected in it when it was last shown, so that a hidden panel comes back with
    /// that view. Absent when the panel held no view.
    pub selected: Option<ViewId>,
}

/// The fields of a [`PanelLayout`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelLayoutFields {
    /// The left sidebar.
    pub left: PanelState,
    /// The right sidebar.
    pub right: PanelState,
    /// The bottom panel.
    pub bottom: PanelState,
}

impl Validate for PanelLayoutFields {
    fn validate(&self) -> Result<(), DomainError> {
        for (field, panel) in [
            ("left", &self.left),
            ("right", &self.right),
            ("bottom", &self.bottom),
        ] {
            match panel.size {
                Some(size) if size == 0 || size > MAX_PANEL_SIZE => {
                    return Err(DomainError::new(field, ErrorKind::OutOfRange));
                }
                // A panel on the screen has a size; a layout that shows one without it was not
                // measured on a window.
                None if panel.shown => return Err(DomainError::new(field, ErrorKind::Missing)),
                _ => {}
            }
        }
        Ok(())
    }
}

validated_record!(
    /// The panels of a Desktop window as the user last left them: the three panels that the
    /// Desktop UI contract places around the editor. Each size is between 1 and
    /// [`MAX_PANEL_SIZE`], and a shown panel has one.
    PanelLayout,
    PanelLayoutFields
);

/// The panel layout a host keeps for one UI client, so that the client finds its window as the
/// user left it after it was closed or stopped unexpectedly (ST-VIEW, FD-UI-CLIENT). The host
/// writes it only on behalf of that client and reads no instruction from it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientLayout {
    /// The client.
    pub client_id: ClientId,
    /// The device the client runs on.
    pub device_id: DeviceId,
    /// Its panels.
    pub panels: PanelLayout,
}

impl Record for ClientLayout {
    const SCHEMA: &'static str = "nexees.client.layout";
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

    fn panel(shown: bool, size: Option<u32>, selected: Option<&str>) -> PanelState {
        PanelState {
            shown,
            size,
            selected: selected.map(|view| ViewId::new(view).unwrap()),
        }
    }

    fn panels(left: PanelState) -> Result<PanelLayout, DomainError> {
        PanelLayout::try_from(PanelLayoutFields {
            left,
            right: panel(false, Some(354), Some("nexees-area-tasks")),
            bottom: panel(false, None, None),
        })
    }

    #[test]
    fn a_client_layout_round_trips_with_a_hidden_panels_size_and_view() {
        let layout = ClientLayout {
            client_id: ClientId::new("desktop-window").unwrap(),
            device_id: DeviceId::new("pc").unwrap(),
            panels: panels(panel(true, Some(203), Some("explorer-view-container"))).unwrap(),
        };
        let text = serde_json::to_string(&Versioned(layout.clone())).unwrap();
        assert!(text.starts_with(r#"{"schema":"nexees.client.layout","version":1,"#));
        let read = serde_json::from_str::<Versioned<ClientLayout>>(&text)
            .unwrap()
            .0;
        assert_eq!(read, layout);
        // The hidden right sidebar keeps what it comes back with.
        let right = &read.panels.fields().right;
        assert_eq!((right.shown, right.size), (false, Some(354)));
        assert_eq!(
            right.selected.as_ref().unwrap().as_str(),
            "nexees-area-tasks"
        );
    }

    #[test]
    fn a_panel_size_that_is_no_size_is_refused() {
        for (left, kind) in [
            (panel(true, Some(0), None), ErrorKind::OutOfRange),
            (panel(false, Some(0), None), ErrorKind::OutOfRange),
            (
                panel(true, Some(MAX_PANEL_SIZE + 1), None),
                ErrorKind::OutOfRange,
            ),
            // Shown, but never measured.
            (panel(true, None, None), ErrorKind::Missing),
        ] {
            let error = panels(left).unwrap_err();
            assert_eq!((error.field, error.kind), ("left", kind));
        }
        assert!(panels(panel(true, Some(MAX_PANEL_SIZE), None)).is_ok());
        assert!(panels(panel(true, Some(1), None)).is_ok());
    }

    #[test]
    fn decoding_a_layout_refuses_what_construction_refuses_and_any_unknown_field() {
        let good = concat!(
            r#"{"left":{"shown":true,"size":203,"selected":"explorer"},"#,
            r#""right":{"shown":false,"size":null,"selected":null},"#,
            r#""bottom":{"shown":false,"size":120,"selected":null}}"#
        );
        assert!(serde_json::from_str::<PanelLayout>(good).is_ok());
        for bad in [
            // A size out of range, and a shown panel without one.
            good.replace("203", "0"),
            good.replace("203", "16385"),
            good.replace("203", "null"),
            good.replace("203", "-1"),
            good.replace("203", "203.5"),
            // A view name that is no identifier.
            good.replace("explorer", "../explorer"),
            good.replace("explorer", ""),
            // Nothing rides along: not in a panel, not beside the panels.
            good.replace(r#""shown":true,"#, r#""shown":true,"command":"x","#),
            good.replace(r#"{"left""#, r#"{"agent_session":"s","left""#),
            // A panel is missing.
            good.replace(
                r#","bottom":{"shown":false,"size":120,"selected":null}"#,
                "",
            ),
        ] {
            assert!(serde_json::from_str::<PanelLayout>(&bad).is_err(), "{bad}");
        }
        let smuggled = format!(
            r#"{{"client_id":"w","device_id":"pc","panels":{good},"foreground_workspace":"w1"}}"#
        );
        assert!(serde_json::from_str::<ClientLayout>(&smuggled).is_err());
    }
}
