//! Domain events: facts a host reports about its own state (IF-ATTACH, ST-REMOTE-SNAPSHOT).
//!
//! An event says what happened, such as a task status, an agent state or an import step. It
//! never carries authority. No event holds a grant, an approval record, a secret or a command, and
//! receiving one changes nothing on the receiving side except what it shows. A client keeps its
//! view current by applying events in order, and a peer shows them as last observed (A3).

use serde::{Deserialize, Serialize};

use crate::errors::{Blocker, Outcome};
use crate::handoff::{HandoffPhase, OrientationSetting};
use crate::ids::{
    AgentSessionId, DeviceId, EvidenceId, HandoffId, ImportId, OperationId, RequestId, TaskId,
    TransferId, WorkspaceId,
};
use crate::import::ImportState;
use crate::lcl::SpecRevisionState;
use crate::model_capabilities::ProviderModel;
use crate::revision::{Generation, SpecRevision};
use crate::session::TransferState;
use crate::task::{EvidenceKind, EvidenceResult, TaskStatus};
use crate::text::Note;
use crate::time::Timestamp;
use crate::workspace::{ContentAvailability, WorkspaceRecord};

/// What an agent session is doing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentRunState {
    /// Bound but not working.
    Idle,
    /// Working.
    Running,
    /// Paused by the user or by a model switch.
    Paused,
    /// Stopping; effects in flight are being settled.
    Cancelling,
    /// Stopped.
    Cancelled,
    /// Cannot continue, with the reason.
    Blocked(Blocker),
}

/// The level of a log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    /// Progress.
    Info,
    /// Something to look at.
    Warning,
    /// A failure.
    Error,
}

/// A fact a host reports. Each names the records it concerns by identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum DomainEvent {
    /// A workspace was created, renamed or changed kind.
    WorkspaceChanged {
        /// The workspace as it is now.
        workspace: WorkspaceRecord,
    },
    /// A device's replica of a workspace changed availability.
    ReplicaChanged {
        /// The workspace.
        workspace_id: WorkspaceId,
        /// The device that holds the replica.
        device_id: DeviceId,
        /// Whether the content is on that device now.
        availability: ContentAvailability,
    },
    /// A session was bound, or its model changed with a new generation.
    SessionBound {
        /// The session.
        session_id: AgentSessionId,
        /// Its workspace.
        workspace_id: WorkspaceId,
        /// Its execution device.
        execution_device_id: DeviceId,
        /// Its current generation.
        generation: Generation,
        /// Its provider and model.
        model: ProviderModel,
    },
    /// An agent session changed what it is doing.
    AgentStateChanged {
        /// The session.
        session_id: AgentSessionId,
        /// The generation the state belongs to.
        generation: Generation,
        /// What it is doing now.
        state: AgentRunState,
    },
    /// The orientation setting changed: a session's override, or the user's default.
    OrientationChanged {
        /// The session whose override changed; absent for the user's default.
        session_id: Option<AgentSessionId>,
        /// The setting now.
        setting: OrientationSetting,
    },
    /// A model handoff reached a new phase.
    HandoffChanged {
        /// The handoff.
        handoff_id: HandoffId,
        /// Its session.
        session_id: AgentSessionId,
        /// Its generation.
        generation: Generation,
        /// The phase reached.
        phase: HandoffPhase,
    },
    /// A transfer of execution to another device reached a new step.
    TransferChanged {
        /// The transfer.
        transfer_id: TransferId,
        /// Its session.
        session_id: AgentSessionId,
        /// The step reached.
        state: TransferState,
    },
    /// A task changed status.
    TaskChanged {
        /// The task.
        task_id: TaskId,
        /// Its workspace.
        workspace_id: WorkspaceId,
        /// Its status now.
        status: TaskStatus,
        /// Why it is blocked, when it is.
        blocker: Option<Blocker>,
    },
    /// Evidence was recorded for a task, including its closure records such as the cleanup.
    EvidenceRecorded {
        /// The evidence.
        evidence_id: EvidenceId,
        /// Its task.
        task_id: TaskId,
        /// What it records.
        kind: EvidenceKind,
        /// Its result.
        result: EvidenceResult,
    },
    /// A specification revision was created, validated or found invalid.
    LclRevisionChanged {
        /// The revision.
        revision: SpecRevision,
        /// Its validation state now.
        state: SpecRevisionState,
    },
    /// An import reached a new state.
    ImportChanged {
        /// The import.
        import_id: ImportId,
        /// Its target workspace.
        workspace_id: WorkspaceId,
        /// Its state now.
        state: ImportState,
    },
    /// A request on this host waits for the local user's approval.
    ApprovalRequested {
        /// The request.
        request_id: RequestId,
        /// Its operation.
        operation: OperationId,
        /// When the request expires; the approval cannot outlive it.
        expires_at: Timestamp,
    },
    /// The local user decided a pending approval.
    ApprovalDecided {
        /// The request.
        request_id: RequestId,
        /// Whether it was approved.
        approved: bool,
    },
    /// A request reached an outcome on this host.
    RequestOutcome {
        /// The request.
        request_id: RequestId,
        /// Its outcome.
        outcome: Outcome,
    },
    /// A session wrote a log line.
    Log {
        /// The session.
        session_id: AgentSessionId,
        /// Its level.
        level: LogLevel,
        /// The line, bounded and without control characters.
        text: Note,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::BlockerKind;

    fn blocked() -> DomainEvent {
        DomainEvent::AgentStateChanged {
            session_id: AgentSessionId::new("s1").unwrap(),
            generation: Generation::FIRST,
            state: AgentRunState::Blocked(Blocker {
                kind: BlockerKind::MissingCapability,
                detail: Note::new("No test backend on this host.").unwrap(),
            }),
        }
    }

    #[test]
    fn events_round_trip_with_their_kind_as_the_tag() {
        let event = blocked();
        let text = serde_json::to_string(&event).unwrap();
        assert!(text.starts_with(r#"{"agent_state_changed":{"session_id":"s1","generation":1,"#));
        assert_eq!(serde_json::from_str::<DomainEvent>(&text).unwrap(), event);
    }

    #[test]
    fn an_event_cannot_smuggle_authority_or_be_of_an_unknown_kind() {
        let granted = r#"{"approval_decided":{"request_id":"q1","approved":true,"grant":"all"}}"#;
        assert!(serde_json::from_str::<DomainEvent>(granted).is_err());
        let unknown = r#"{"grant_created":{"grant_id":"g1"}}"#;
        assert!(serde_json::from_str::<DomainEvent>(unknown).is_err());
    }

    #[test]
    fn log_lines_are_bounded_text_without_control_sequences() {
        let line = r#"{"log":{"session_id":"s1","level":"info","text":"\u001b[2Jcleared"}}"#;
        assert!(serde_json::from_str::<DomainEvent>(line).is_err());
        let fine = r#"{"log":{"session_id":"s1","level":"warning","text":"3 tests skipped"}}"#;
        assert!(serde_json::from_str::<DomainEvent>(fine).is_ok());
    }
}
