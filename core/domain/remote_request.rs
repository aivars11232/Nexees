//! The remote request envelope as a domain type (RC-09, SI-12, `type.sec_request_envelope`).
//!
//! The envelope has every field of TASK-004's contract and no required field is optional. Its
//! targets are explicit and typed: what an operation targets is declared by its [`Operation`]
//! implementation, and an envelope whose workspace, agent or revisions do not fit is rejected.
//! A missing target is never filled from the foreground workspace, the current session or a
//! device name (SI-12, TH-47). The operation registry, the wire schema and the canonical hash
//! are TASK-007's; admission on the destination (the channel peer, grants, lock state and the
//! deduplication ledger) is TASK-007's and TASK-070's.

use std::fmt;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{
    AgentSessionId, ApprovalId, DeviceId, HostSessionId, OperationId, RequestId, UserId,
    WorkspaceId,
};
use crate::revision::{Generation, PermissionEpoch, RevisionId, SpecRevision, WorkspaceRevision};
use crate::schema::{Validate, ensure_unique};
use crate::time::Timestamp;

/// The ceiling on a request's lifetime, measured from its receipt on the destination
/// (`binding.sec_max_request_lifetime`). TASK-007 may lower it, never raise it.
pub const MAX_REQUEST_LIFETIME_MS: u64 = 10 * 60 * 1000;

/// How far into the destination's future a request's `issued_at` may be
/// (`binding.sec_clock_skew_allowance`).
pub const CLOCK_SKEW_ALLOWANCE_MS: u64 = 2 * 60 * 1000;

/// What an operation acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationTarget {
    /// A workspace: the envelope names the workspace and no agent.
    Workspace,
    /// An agent session: the envelope names the agent and its workspace.
    Agent,
    /// The destination device itself, such as an application launch: the envelope names neither
    /// workspace nor agent, only the device, user and session.
    Device,
}

/// One operation of the closed remote-operation registry with its typed arguments. TASK-007
/// defines the registry by implementing this trait once per operation.
pub trait Operation: Serialize + DeserializeOwned + Clone + PartialEq + Eq + fmt::Debug {
    /// The operation's registry key; it must be valid as an [`OperationId`].
    const ID: &'static str;
    /// What the operation acts on.
    const TARGET: OperationTarget;
    /// Whether it changes anything. Only operations that change nothing may expect no revisions.
    const CHANGES_STATE: bool;
}

/// A remote protocol version; at least 1. A destination refuses anything below its minimum
/// secure version (TH-42).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct ProtocolVersion(u32);

impl ProtocolVersion {
    /// Checks `value` and returns it as a [`ProtocolVersion`].
    pub const fn new(value: u32) -> Result<Self, DomainError> {
        if value == 0 {
            Err(DomainError::new("ProtocolVersion", ErrorKind::OutOfRange))
        } else {
            Ok(Self(value))
        }
    }

    /// The number.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for ProtocolVersion {
    type Error = DomainError;

    fn try_from(value: u32) -> Result<Self, DomainError> {
        Self::new(value)
    }
}

impl From<ProtocolVersion> for u32 {
    fn from(version: ProtocolVersion) -> u32 {
        version.0
    }
}

/// A revision a request expects on the destination; a stale one is rejected. The contract's
/// list of revision strings, with each entry typed by what it is a revision of.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectedRevision {
    /// A revision of the target workspace's content.
    Workspace(WorkspaceRevision),
    /// The adopted specification revision of the target.
    Specification(SpecRevision),
    /// The target agent session's generation.
    Generation(Generation),
    /// The destination's policy and allowlist revision, for operations on the device itself.
    Policy(RevisionId),
}

/// The fields of a [`RequestEnvelope`]: exactly those of `type.sec_request_envelope`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestEnvelopeFields<A> {
    /// The remote protocol version.
    pub protocol_version: ProtocolVersion,
    /// Unique per request and never reused.
    pub request_id: RequestId,
    /// The sender. The destination takes it from the authenticated channel and rejects a
    /// different value here.
    pub requesting_device_id: DeviceId,
    /// The user signed in on the sender.
    pub requesting_user_id: UserId,
    /// The intended destination; any other device rejects the request.
    pub destination_device_id: DeviceId,
    /// The destination's user.
    pub destination_user_id: UserId,
    /// The destination host session: the OS login session on Desktop, the app on Android.
    pub destination_session_id: HostSessionId,
    /// The operation's registry key; it must be `A::ID`.
    pub operation: OperationId,
    /// The operation's typed arguments. An application launch carries only an allowlist key.
    pub arguments: A,
    /// The target workspace, for workspace and agent operations only.
    pub workspace_id: Option<WorkspaceId>,
    /// The target agent session, for agent operations only.
    pub agent_id: Option<AgentSessionId>,
    /// The revisions the operation expects; empty only for operations that change nothing.
    pub expected_revisions: Vec<ExpectedRevision>,
    /// The epoch of the grant the sender relies on.
    pub permission_epoch: PermissionEpoch,
    /// When the sender issued the request, on its clock.
    pub issued_at: Timestamp,
    /// The sender's expiry; after `issued_at`.
    pub expires_at: Timestamp,
    /// The approval this request executes, when it executes one.
    pub approval_id: Option<ApprovalId>,
}

impl<A: Operation> Validate for RequestEnvelopeFields<A> {
    fn validate(&self) -> Result<(), DomainError> {
        if self.operation.as_str() != A::ID {
            return Err(DomainError::new("operation", ErrorKind::Mismatch));
        }
        if self.requesting_device_id == self.destination_device_id {
            return Err(DomainError::new(
                "destination_device_id",
                ErrorKind::Duplicate,
            ));
        }
        check_targets::<A>(
            self.workspace_id.as_ref(),
            self.agent_id.as_ref(),
            &self.expected_revisions,
        )?;
        if self.expires_at <= self.issued_at {
            return Err(DomainError::new("expires_at", ErrorKind::OutOfRange));
        }
        Ok(())
    }
}

/// Fails unless the explicit targets and expected revisions fit operation `A` (SI-12): a
/// workspace operation names its workspace and no agent; an agent operation names the agent and
/// its workspace; a device operation names neither. Revisions are expected exactly when the
/// operation changes something, each once; workspace revisions belong to the named workspace, and
/// a generation is expected only of an agent. Remote requests and a client's local intents share
/// these rules.
pub fn check_targets<A: Operation>(
    workspace_id: Option<&WorkspaceId>,
    agent_id: Option<&AgentSessionId>,
    expected_revisions: &[ExpectedRevision],
) -> Result<(), DomainError> {
    let (workspace, agent) = match A::TARGET {
        OperationTarget::Workspace => (true, false),
        OperationTarget::Agent => (true, true),
        OperationTarget::Device => (false, false),
    };
    expect_presence(workspace_id.is_some(), workspace, "workspace_id")?;
    expect_presence(agent_id.is_some(), agent, "agent_id")?;
    if A::CHANGES_STATE && expected_revisions.is_empty() {
        return Err(DomainError::new("expected_revisions", ErrorKind::Empty));
    }
    if !A::CHANGES_STATE && !expected_revisions.is_empty() {
        return Err(DomainError::new(
            "expected_revisions",
            ErrorKind::Unexpected,
        ));
    }
    ensure_unique(expected_revisions, "expected_revisions")?;
    for expected in expected_revisions {
        match (expected, workspace_id) {
            (ExpectedRevision::Workspace(revision), Some(target)) => {
                revision.ensure_of(target, "expected_revisions")?;
            }
            (ExpectedRevision::Workspace(_) | ExpectedRevision::Specification(_), None) => {
                return Err(DomainError::new(
                    "expected_revisions",
                    ErrorKind::Unexpected,
                ));
            }
            (ExpectedRevision::Generation(_), _) if A::TARGET != OperationTarget::Agent => {
                return Err(DomainError::new(
                    "expected_revisions",
                    ErrorKind::Unexpected,
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn expect_presence(present: bool, required: bool, field: &'static str) -> Result<(), DomainError> {
    match (present, required) {
        (false, true) => Err(DomainError::new(field, ErrorKind::Missing)),
        (true, false) => Err(DomainError::new(field, ErrorKind::Unexpected)),
        _ => Ok(()),
    }
}

/// A validated remote request envelope for the operation `A` (RC-09).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestEnvelope<A>(RequestEnvelopeFields<A>);

impl<A: Operation> RequestEnvelope<A> {
    /// The validated fields.
    pub fn fields(&self) -> &RequestEnvelopeFields<A> {
        &self.0
    }

    /// The fields, to build a changed request from; the change is validated again.
    pub fn into_fields(self) -> RequestEnvelopeFields<A> {
        self.0
    }

    /// The moment the request expires on a destination that received it at `received_at`: the
    /// earlier of the sender's expiry and the lifetime ceiling counted from receipt.
    pub fn effective_expiry(&self, received_at: Timestamp) -> Timestamp {
        self.0
            .expires_at
            .min(received_at.saturating_add_millis(MAX_REQUEST_LIFETIME_MS))
    }

    /// Fails unless the request, received at `received_at`, may still run at `now`. It must not
    /// claim to be issued further in the future than the skew allowance, and it must not have
    /// expired. An expired request is never executed later (RC-17).
    pub fn check_fresh(&self, received_at: Timestamp, now: Timestamp) -> Result<(), DomainError> {
        if self.0.issued_at > received_at.saturating_add_millis(CLOCK_SKEW_ALLOWANCE_MS) {
            return Err(DomainError::new("issued_at", ErrorKind::OutOfRange));
        }
        if now >= self.effective_expiry(received_at) {
            return Err(DomainError::new("expires_at", ErrorKind::Expired));
        }
        Ok(())
    }
}

impl<A: Operation> TryFrom<RequestEnvelopeFields<A>> for RequestEnvelope<A> {
    type Error = DomainError;

    fn try_from(fields: RequestEnvelopeFields<A>) -> Result<Self, DomainError> {
        fields.validate()?;
        Ok(Self(fields))
    }
}

impl<A: Operation> Serialize for RequestEnvelope<A> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, A: Operation> Deserialize<'de> for RequestEnvelope<A> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = RequestEnvelopeFields::<A>::deserialize(deserializer)?;
        Self::try_from(fields).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::AllowlistKey;
    use crate::workspace::RelativePath;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct WriteFile {
        path: RelativePath,
    }
    impl Operation for WriteFile {
        const ID: &'static str = "file_write";
        const TARGET: OperationTarget = OperationTarget::Workspace;
        const CHANGES_STATE: bool = true;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct PauseAgent {}
    impl Operation for PauseAgent {
        const ID: &'static str = "agent_pause";
        const TARGET: OperationTarget = OperationTarget::Agent;
        const CHANGES_STATE: bool = true;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LaunchApp {
        app: AllowlistKey,
    }
    impl Operation for LaunchApp {
        const ID: &'static str = "app_launch";
        const TARGET: OperationTarget = OperationTarget::Device;
        const CHANGES_STATE: bool = true;
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ReadStatus {}
    impl Operation for ReadStatus {
        const ID: &'static str = "status_read";
        const TARGET: OperationTarget = OperationTarget::Workspace;
        const CHANGES_STATE: bool = false;
    }

    fn workspace_revision(workspace: &str) -> ExpectedRevision {
        ExpectedRevision::Workspace(WorkspaceRevision {
            workspace_id: WorkspaceId::new(workspace).unwrap(),
            revision: RevisionId::new("r7").unwrap(),
        })
    }

    fn envelope<A: Operation>(arguments: A) -> RequestEnvelopeFields<A> {
        RequestEnvelopeFields {
            protocol_version: ProtocolVersion::new(1).unwrap(),
            request_id: RequestId::new("q1").unwrap(),
            requesting_device_id: DeviceId::new("phone").unwrap(),
            requesting_user_id: UserId::new("u1").unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            destination_user_id: UserId::new("u1").unwrap(),
            destination_session_id: HostSessionId::new("login-2").unwrap(),
            operation: OperationId::new(A::ID).unwrap(),
            arguments,
            workspace_id: None,
            agent_id: None,
            expected_revisions: vec![],
            permission_epoch: PermissionEpoch::new(3),
            issued_at: Timestamp::from_unix_millis(1_000_000),
            expires_at: Timestamp::from_unix_millis(1_060_000),
            approval_id: None,
        }
    }

    fn write_file() -> RequestEnvelopeFields<WriteFile> {
        RequestEnvelopeFields {
            workspace_id: Some(WorkspaceId::new("arch-dock").unwrap()),
            expected_revisions: vec![workspace_revision("arch-dock")],
            ..envelope(WriteFile {
                path: RelativePath::new("src/main.rs").unwrap(),
            })
        }
    }

    #[test]
    fn a_workspace_request_names_its_workspace_and_revisions_of_it() {
        assert!(RequestEnvelope::try_from(write_file()).is_ok());
        let missing = RequestEnvelopeFields {
            workspace_id: None,
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(missing).unwrap_err(),
            DomainError::new("workspace_id", ErrorKind::Missing)
        );
        let with_agent = RequestEnvelopeFields {
            agent_id: Some(AgentSessionId::new("s1").unwrap()),
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(with_agent).unwrap_err(),
            DomainError::new("agent_id", ErrorKind::Unexpected)
        );
        let other_revision = RequestEnvelopeFields {
            expected_revisions: vec![workspace_revision("lcl-next")],
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(other_revision).unwrap_err(),
            DomainError::new("expected_revisions", ErrorKind::Mismatch)
        );
        let no_revision = RequestEnvelopeFields {
            expected_revisions: vec![],
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(no_revision).unwrap_err().kind,
            ErrorKind::Empty
        );
    }

    #[test]
    fn an_agent_request_names_the_agent_and_its_workspace() {
        let pause = RequestEnvelopeFields {
            workspace_id: Some(WorkspaceId::new("arch-dock").unwrap()),
            agent_id: Some(AgentSessionId::new("s1").unwrap()),
            expected_revisions: vec![ExpectedRevision::Generation(Generation::FIRST)],
            ..envelope(PauseAgent {})
        };
        assert!(RequestEnvelope::try_from(pause.clone()).is_ok());
        let no_agent = RequestEnvelopeFields {
            agent_id: None,
            ..pause.clone()
        };
        assert_eq!(
            RequestEnvelope::try_from(no_agent).unwrap_err(),
            DomainError::new("agent_id", ErrorKind::Missing)
        );
        let no_workspace = RequestEnvelopeFields {
            workspace_id: None,
            ..pause
        };
        assert_eq!(
            RequestEnvelope::try_from(no_workspace).unwrap_err(),
            DomainError::new("workspace_id", ErrorKind::Missing)
        );
    }

    #[test]
    fn a_device_request_never_carries_an_ambient_workspace() {
        let launch = RequestEnvelopeFields {
            expected_revisions: vec![ExpectedRevision::Policy(RevisionId::new("p4").unwrap())],
            ..envelope(LaunchApp {
                app: AllowlistKey::new("hoptodesk").unwrap(),
            })
        };
        assert!(RequestEnvelope::try_from(launch.clone()).is_ok());
        let ambient = RequestEnvelopeFields {
            workspace_id: Some(WorkspaceId::new("arch-dock").unwrap()),
            ..launch.clone()
        };
        assert_eq!(
            RequestEnvelope::try_from(ambient).unwrap_err(),
            DomainError::new("workspace_id", ErrorKind::Unexpected)
        );
        let generation = RequestEnvelopeFields {
            expected_revisions: vec![ExpectedRevision::Generation(Generation::FIRST)],
            ..launch
        };
        assert_eq!(
            RequestEnvelope::try_from(generation).unwrap_err().kind,
            ErrorKind::Unexpected
        );
    }

    #[test]
    fn a_read_only_request_expects_no_revisions() {
        let read = RequestEnvelopeFields {
            workspace_id: Some(WorkspaceId::new("arch-dock").unwrap()),
            ..envelope(ReadStatus {})
        };
        assert!(RequestEnvelope::try_from(read.clone()).is_ok());
        let expecting = RequestEnvelopeFields {
            expected_revisions: vec![workspace_revision("arch-dock")],
            ..read
        };
        assert_eq!(
            RequestEnvelope::try_from(expecting).unwrap_err().kind,
            ErrorKind::Unexpected
        );
    }

    #[test]
    fn envelopes_reject_self_targets_mislabelled_operations_and_backwards_expiry() {
        let to_itself = RequestEnvelopeFields {
            destination_device_id: DeviceId::new("phone").unwrap(),
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(to_itself).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let relabelled = RequestEnvelopeFields {
            operation: OperationId::new("status_read").unwrap(),
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(relabelled).unwrap_err(),
            DomainError::new("operation", ErrorKind::Mismatch)
        );
        let backwards = RequestEnvelopeFields {
            expires_at: Timestamp::from_unix_millis(1_000_000),
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(backwards).unwrap_err().kind,
            ErrorKind::OutOfRange
        );
        let twice = RequestEnvelopeFields {
            expected_revisions: vec![
                workspace_revision("arch-dock"),
                workspace_revision("arch-dock"),
            ],
            ..write_file()
        };
        assert_eq!(
            RequestEnvelope::try_from(twice).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        assert!(ProtocolVersion::new(0).is_err());
    }

    #[test]
    fn requests_expire_at_the_earlier_of_their_expiry_and_the_ceiling() {
        let long = RequestEnvelopeFields {
            expires_at: Timestamp::from_unix_millis(9_000_000),
            ..write_file()
        };
        let request = RequestEnvelope::try_from(long).unwrap();
        let received = Timestamp::from_unix_millis(1_000_500);
        assert_eq!(
            request.effective_expiry(received).unix_millis(),
            1_000_500 + MAX_REQUEST_LIFETIME_MS
        );
        assert!(request.check_fresh(received, received).is_ok());
        let late = received.saturating_add_millis(MAX_REQUEST_LIFETIME_MS);
        assert_eq!(
            request.check_fresh(received, late).unwrap_err().kind,
            ErrorKind::Expired
        );
        // Issued further in the destination's future than the skew allowance allows.
        let early_receipt = Timestamp::from_unix_millis(1_000_000 - CLOCK_SKEW_ALLOWANCE_MS - 1);
        assert_eq!(
            request
                .check_fresh(early_receipt, early_receipt)
                .unwrap_err()
                .field,
            "issued_at"
        );
    }

    #[test]
    fn envelopes_round_trip_and_reject_unknown_or_malformed_fields() {
        let request = RequestEnvelope::try_from(write_file()).unwrap();
        let text = serde_json::to_string(&request).unwrap();
        assert_eq!(
            serde_json::from_str::<RequestEnvelope<WriteFile>>(&text).unwrap(),
            request
        );
        for (from, to) in [
            (
                "\"approval_id\":null",
                "\"approval_id\":null,\"elevate\":true",
            ),
            (
                "\"path\":\"src/main.rs\"",
                "\"path\":\"src/main.rs\",\"shell\":\"rm -rf /\"",
            ),
            ("\"path\":\"src/main.rs\"", "\"path\":\"/etc/passwd\""),
            ("\"workspace_id\":\"arch-dock\"", "\"workspace_id\":\"\""),
            ("\"protocol_version\":1", "\"protocol_version\":0"),
        ] {
            let tampered = text.replace(from, to);
            assert_ne!(tampered, text, "{from}");
            assert!(
                serde_json::from_str::<RequestEnvelope<WriteFile>>(&tampered).is_err(),
                "{to}"
            );
        }
        // Dropping the workspace decodes to a request without a target, which is refused.
        let untargeted = text.replace("\"workspace_id\":\"arch-dock\"", "\"workspace_id\":null");
        assert!(serde_json::from_str::<RequestEnvelope<WriteFile>>(&untargeted).is_err());
    }
}
