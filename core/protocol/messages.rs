//! The message families of the protocol, kept apart (IF-ATTACH, IF-REMOTE, IF-SYNC, C22, AD-05).
//!
//! - A window and its own host exchange [`ClientMessage`] and [`HostMessage`] over local IPC:
//!   intents, acknowledgments, status and ordered domain events, the panel layout the host keeps
//!   for the window (ST-VIEW), and the workspaces of the host's device, which the window lists,
//!   creates, opens and closes (ST-WORKSPACE).
//! - Two paired hosts exchange [`PeerMessage`], which holds exactly one family per message:
//!   [`RemoteMessage`] for requests, acknowledgments, reconciliation, status and execution
//!   transfers, or [`SyncMessage`] for content changes and portable records. The sync family has no
//!   variant that can hold a request, an approval, a grant or a secret.
//!
//! Every channel opens with a [`Hello`] and refuses a version mismatch.
//!
//! **Reconnect semantics.**
//!
//! - **Between hosts:** after the hellos, the sender reconciles its pending requests by request ID
//!   with a [`ReconcileQuery`]. Each one is then settled, resent under the same ID if the
//!   destination never received it and it has not expired, or dropped; nothing is replayed blindly
//!   (RC-17, [`nexees_domain::requests`]). Synchronization resumes; changes are idempotent by their
//!   base content.
//! - **Between a window and its host:** the window asks to [`ClientMessage::Resync`] after the last
//!   event it applied. It applies only events that continue its sequence, and on a gap it shows a
//!   fresh status first. Reattaching starts no executor and replays no intent (RC-04, RC-T14).
//!
//! Neither reconnect touches bindings, ownership or focus (RC-20).

use nexees_domain::changes::ContentChange;
use nexees_domain::client::PanelLayout;
use nexees_domain::errors::{Blocker, DomainError, ErrorKind, Outcome};
use nexees_domain::events::{AgentRunState, DomainEvent};
use nexees_domain::ids::{
    AgentSessionId, ChangeId, ConflictId, DeviceId, HostSessionId, OperationId, RequestId, TaskId,
    UserId, WorkspaceId,
};
use nexees_domain::model_capabilities::ProviderModel;
use nexees_domain::remote_request::{
    ExpectedRevision, MAX_REQUEST_LIFETIME_MS, Operation, ProtocolVersion, RequestEnvelope,
    check_targets,
};
use nexees_domain::revision::Generation;
use nexees_domain::schema::{Record, Validate};
use nexees_domain::session::{ExecutionTransfer, TransferState};
use nexees_domain::task::{Evidence, TaskStatus};
use nexees_domain::text::{Label, Note};
use nexees_domain::time::Timestamp;
use nexees_domain::workspace::{ContentAvailability, DeviceRoot, WorkspaceKind, WorkspaceRecord};
use serde::{Deserialize, Serialize};

use crate::operations::{LocalIntent, RemoteRequest};
use crate::serialization::{
    Base64Bytes, MAX_FRAME_BYTES, MAX_IPC_MESSAGE_BYTES, ProtocolError, decode,
};
use crate::version_negotiation::{Hello, LAYOUT_SINCE_VERSION, WORKSPACES_SINCE_VERSION};

/// Most request IDs one reconciliation may name.
pub const MAX_RECONCILE: usize = 256;
/// Most sessions, and most tasks, one status may list.
pub const MAX_STATUS_ENTRIES: usize = 64;
/// Largest content one chunk may carry, in bytes before base64.
pub const MAX_CHUNK_BYTES: usize = 32 * 1024;

fn ensure_unique<T: Ord>(items: &[&T], field: &'static str) -> Result<(), DomainError> {
    let mut sorted = items.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(DomainError::new(field, ErrorKind::Duplicate))
    } else {
        Ok(())
    }
}

/// What every request shares, whatever its operation.
pub trait Header {
    /// The request.
    fn request_id(&self) -> &RequestId;
    /// Its operation.
    fn operation(&self) -> &OperationId;
    /// Fails unless the request, received at `received_at`, may still run at `now`.
    fn check_fresh(&self, received_at: Timestamp, now: Timestamp) -> Result<(), DomainError>;
}

/// What a remote request adds: who sent it, to whom, under which protocol version.
pub trait RemoteHeader: Header {
    /// The protocol version it was sent under.
    fn protocol_version(&self) -> ProtocolVersion;
    /// The device that sent it.
    fn requesting_device_id(&self) -> &DeviceId;
    /// The destination: device, user and host session.
    fn destination(&self) -> (&DeviceId, &UserId, &HostSessionId);
}

impl<A: Operation> Header for RequestEnvelope<A> {
    fn request_id(&self) -> &RequestId {
        &self.fields().request_id
    }

    fn operation(&self) -> &OperationId {
        &self.fields().operation
    }

    fn check_fresh(&self, received_at: Timestamp, now: Timestamp) -> Result<(), DomainError> {
        RequestEnvelope::check_fresh(self, received_at, now)
    }
}

impl<A: Operation> RemoteHeader for RequestEnvelope<A> {
    fn protocol_version(&self) -> ProtocolVersion {
        self.fields().protocol_version
    }

    fn requesting_device_id(&self) -> &DeviceId {
        &self.fields().requesting_device_id
    }

    fn destination(&self) -> (&DeviceId, &UserId, &HostSessionId) {
        let f = self.fields();
        (
            &f.destination_device_id,
            &f.destination_user_id,
            &f.destination_session_id,
        )
    }
}

/// The fields of an [`Intent`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntentFields<A> {
    /// Unique per intent; the host's ledger deduplicates by it, as for remote requests.
    pub request_id: RequestId,
    /// The operation's registry key; it must be `A::ID`.
    pub operation: OperationId,
    /// The operation's typed arguments.
    pub arguments: A,
    /// The target workspace, for workspace and agent operations only; never taken from focus.
    pub workspace_id: Option<WorkspaceId>,
    /// The target agent session, for agent operations only.
    pub agent_id: Option<AgentSessionId>,
    /// The revisions the operation expects; empty only for operations that change nothing.
    pub expected_revisions: Vec<ExpectedRevision>,
    /// When the window issued it.
    pub issued_at: Timestamp,
    /// When it expires; after `issued_at`.
    pub expires_at: Timestamp,
}

impl<A: Operation> Validate for IntentFields<A> {
    fn validate(&self) -> Result<(), DomainError> {
        if self.operation.as_str() != A::ID {
            return Err(DomainError::new("operation", ErrorKind::Mismatch));
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

/// A validated intent from a window to its own host: the local counterpart of a request
/// envelope, with the same explicit targets, expected revisions and expiry (IF-ATTACH). The host
/// authorizes it like any other request; the window is never authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Intent<A>(IntentFields<A>);

impl<A: Operation> Intent<A> {
    /// The validated fields.
    pub fn fields(&self) -> &IntentFields<A> {
        &self.0
    }
}

impl<A: Operation> TryFrom<IntentFields<A>> for Intent<A> {
    type Error = DomainError;

    fn try_from(fields: IntentFields<A>) -> Result<Self, DomainError> {
        fields.validate()?;
        Ok(Self(fields))
    }
}

impl<A: Operation> Serialize for Intent<A> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, A: Operation> Deserialize<'de> for Intent<A> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = IntentFields::<A>::deserialize(deserializer)?;
        Self::try_from(fields).map_err(serde::de::Error::custom)
    }
}

impl<A: Operation> Header for Intent<A> {
    fn request_id(&self) -> &RequestId {
        &self.0.request_id
    }

    fn operation(&self) -> &OperationId {
        &self.0.operation
    }

    /// Window and host share one clock, so only the expiry and the lifetime ceiling apply.
    fn check_fresh(&self, _received_at: Timestamp, now: Timestamp) -> Result<(), DomainError> {
        let ceiling = self
            .0
            .issued_at
            .saturating_add_millis(MAX_REQUEST_LIFETIME_MS);
        if now >= self.0.expires_at.min(ceiling) {
            return Err(DomainError::new("expires_at", ErrorKind::Expired));
        }
        Ok(())
    }
}

/// The answer to a request or an intent: its explicit outcome (RC-12). A delivered request is not
/// a completed one, and a lost acknowledgment leaves the outcome unknown until reconciled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acknowledgment {
    /// The request answered.
    pub request_id: RequestId,
    /// Its outcome on the destination.
    pub outcome: Outcome,
    /// When the destination recorded it.
    pub recorded_at: Timestamp,
    /// What the user should know, such as why it was denied.
    pub detail: Option<Note>,
}

/// A sender's question after reconnecting: what does the ledger record for these requests?
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ReconcileQueryFields")]
pub struct ReconcileQuery {
    request_ids: Vec<RequestId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconcileQueryFields {
    request_ids: Vec<RequestId>,
}

impl ReconcileQuery {
    /// A query for 1 to [`MAX_RECONCILE`] distinct request IDs.
    pub fn new(request_ids: Vec<RequestId>) -> Result<Self, DomainError> {
        if request_ids.is_empty() {
            return Err(DomainError::new("request_ids", ErrorKind::Empty));
        }
        if request_ids.len() > MAX_RECONCILE {
            return Err(DomainError::new("request_ids", ErrorKind::TooLong));
        }
        ensure_unique(&request_ids.iter().collect::<Vec<_>>(), "request_ids")?;
        Ok(Self { request_ids })
    }

    /// The request IDs asked about.
    pub fn request_ids(&self) -> &[RequestId] {
        &self.request_ids
    }
}

impl TryFrom<ReconcileQueryFields> for ReconcileQuery {
    type Error = DomainError;

    fn try_from(fields: ReconcileQueryFields) -> Result<Self, DomainError> {
        Self::new(fields.request_ids)
    }
}

/// What the destination's ledger records for one request: an outcome, or nothing when the
/// request never arrived.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconcileEntry {
    /// The request.
    pub request_id: RequestId,
    /// The recorded outcome; absent when the destination has no record of the request.
    pub recorded: Option<Outcome>,
}

/// The destination's answer to a [`ReconcileQuery`]: one entry per request asked about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ReconcileReplyFields")]
pub struct ReconcileReply {
    entries: Vec<ReconcileEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconcileReplyFields {
    entries: Vec<ReconcileEntry>,
}

impl ReconcileReply {
    /// A reply with 1 to [`MAX_RECONCILE`] entries for distinct requests.
    pub fn new(entries: Vec<ReconcileEntry>) -> Result<Self, DomainError> {
        if entries.is_empty() {
            return Err(DomainError::new("entries", ErrorKind::Empty));
        }
        if entries.len() > MAX_RECONCILE {
            return Err(DomainError::new("entries", ErrorKind::TooLong));
        }
        ensure_unique(
            &entries.iter().map(|e| &e.request_id).collect::<Vec<_>>(),
            "entries",
        )?;
        Ok(Self { entries })
    }

    /// The entries.
    pub fn entries(&self) -> &[ReconcileEntry] {
        &self.entries
    }

    /// The recorded state of `request_id`: `Ok(None)` when the destination has no record of it,
    /// and an error when this reply does not answer for it at all, which must never be read as
    /// "never received".
    pub fn recorded(&self, request_id: &RequestId) -> Result<Option<Outcome>, DomainError> {
        self.entries
            .iter()
            .find(|e| &e.request_id == request_id)
            .map(|e| e.recorded)
            .ok_or(DomainError::new("entries", ErrorKind::Missing))
    }
}

impl TryFrom<ReconcileReplyFields> for ReconcileReply {
    type Error = DomainError;

    fn try_from(fields: ReconcileReplyFields) -> Result<Self, DomainError> {
        Self::new(fields.entries)
    }
}

/// One session in a [`HostStatus`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionStatus {
    /// The session.
    pub session_id: AgentSessionId,
    /// Its workspace.
    pub workspace_id: WorkspaceId,
    /// Its generation.
    pub generation: Generation,
    /// Its provider and model.
    pub model: ProviderModel,
    /// What it is doing.
    pub state: AgentRunState,
}

/// One task in a [`HostStatus`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskStatusEntry {
    /// The task.
    pub task_id: TaskId,
    /// Its workspace.
    pub workspace_id: WorkspaceId,
    /// Its status.
    pub status: TaskStatus,
    /// Why it is blocked, when it is.
    pub blocker: Option<Blocker>,
}

/// A host's status as it observed it at one moment: its sessions and tasks (ST-REMOTE-SNAPSHOT).
/// Another device shows it as live only while its session with the host confirms it, otherwise
/// as last observed (A3, RC-22). It is a view, never authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "HostStatusFields")]
pub struct HostStatus {
    host_device_id: DeviceId,
    observed_at: Timestamp,
    sessions: Vec<SessionStatus>,
    tasks: Vec<TaskStatusEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostStatusFields {
    host_device_id: DeviceId,
    observed_at: Timestamp,
    sessions: Vec<SessionStatus>,
    tasks: Vec<TaskStatusEntry>,
}

/// How a device may present a status it received.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Observation {
    /// The session with the host confirms it now.
    Live,
    /// Only known as of this time.
    LastObserved(Timestamp),
}

impl HostStatus {
    /// A status with at most [`MAX_STATUS_ENTRIES`] sessions and tasks, each listed once.
    pub fn new(
        host_device_id: DeviceId,
        observed_at: Timestamp,
        sessions: Vec<SessionStatus>,
        tasks: Vec<TaskStatusEntry>,
    ) -> Result<Self, DomainError> {
        if sessions.len() > MAX_STATUS_ENTRIES || tasks.len() > MAX_STATUS_ENTRIES {
            return Err(DomainError::new("HostStatus", ErrorKind::TooLong));
        }
        ensure_unique(
            &sessions.iter().map(|s| &s.session_id).collect::<Vec<_>>(),
            "sessions",
        )?;
        ensure_unique(
            &tasks.iter().map(|t| &t.task_id).collect::<Vec<_>>(),
            "tasks",
        )?;
        Ok(Self {
            host_device_id,
            observed_at,
            sessions,
            tasks,
        })
    }

    /// The host that reported it.
    pub fn host_device_id(&self) -> &DeviceId {
        &self.host_device_id
    }

    /// The sessions.
    pub fn sessions(&self) -> &[SessionStatus] {
        &self.sessions
    }

    /// The tasks.
    pub fn tasks(&self) -> &[TaskStatusEntry] {
        &self.tasks
    }

    /// How to present it: live only while `session_confirms` that the host is connected and this
    /// is its latest status; otherwise as last observed.
    pub fn observation(&self, session_confirms: bool) -> Observation {
        if session_confirms {
            Observation::Live
        } else {
            Observation::LastObserved(self.observed_at)
        }
    }
}

impl TryFrom<HostStatusFields> for HostStatus {
    type Error = DomainError;

    fn try_from(f: HostStatusFields) -> Result<Self, DomainError> {
        Self::new(f.host_device_id, f.observed_at, f.sessions, f.tasks)
    }
}

/// The position of an event in a host's event stream to one window, from 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct EventSeq(u64);

impl EventSeq {
    /// The first position.
    pub const FIRST: Self = Self(1);

    /// The position `value`, which must be at least 1.
    pub fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            Err(DomainError::new("EventSeq", ErrorKind::OutOfRange))
        } else {
            Ok(Self(value))
        }
    }

    /// The number.
    pub fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for EventSeq {
    type Error = DomainError;

    fn try_from(value: u64) -> Result<Self, DomainError> {
        Self::new(value)
    }
}

impl From<EventSeq> for u64 {
    fn from(seq: EventSeq) -> u64 {
        seq.0
    }
}

/// A domain event with its position in the stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SequencedEvent {
    /// Its position.
    pub seq: EventSeq,
    /// The fact.
    pub event: DomainEvent,
}

/// What a window does with an event it receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// It continues the sequence: apply it.
    Apply,
    /// Already applied, as after a resync: ignore it.
    Duplicate,
    /// Events were missed: do not apply it; ask to resync and show a fresh status first.
    Gap,
}

/// A window's position in its host's event stream, which makes reconnecting safe: no event is
/// applied twice, and none is applied after a gap.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EventCursor {
    last: Option<EventSeq>,
}

impl EventCursor {
    /// The last event applied, which a [`ClientMessage::Resync`] names.
    pub fn last(&self) -> Option<EventSeq> {
        self.last
    }

    /// Whether to apply the event at `seq`; the cursor advances only when it is applied.
    pub fn accept(&mut self, seq: EventSeq) -> Delivery {
        let expected = self.last.map_or(1, |last| last.0.saturating_add(1));
        if seq.0 < expected {
            Delivery::Duplicate
        } else if seq.0 > expected {
            Delivery::Gap
        } else {
            self.last = Some(seq);
            Delivery::Apply
        }
    }

    /// The cursor after a fresh status that is current up to `seq`.
    pub fn reset_to(&mut self, seq: Option<EventSeq>) {
        self.last = seq;
    }
}

/// A chunk of the content of one change, in order; the chunks of a change together are the
/// content whose hash the change names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ContentChunkFields")]
pub struct ContentChunk {
    change_id: ChangeId,
    offset: u64,
    data: Base64Bytes,
    last: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContentChunkFields {
    change_id: ChangeId,
    offset: u64,
    data: Base64Bytes,
    last: bool,
}

impl ContentChunk {
    /// A chunk of at most [`MAX_CHUNK_BYTES`] bytes at `offset` of a change's content.
    pub fn new(
        change_id: ChangeId,
        offset: u64,
        data: Vec<u8>,
        last: bool,
    ) -> Result<Self, DomainError> {
        if data.len() > MAX_CHUNK_BYTES {
            return Err(DomainError::new("data", ErrorKind::TooLong));
        }
        Ok(Self {
            change_id,
            offset,
            data: Base64Bytes(data),
            last,
        })
    }

    /// The change.
    pub fn change_id(&self) -> &ChangeId {
        &self.change_id
    }

    /// Where the bytes start in the content.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// The bytes.
    pub fn data(&self) -> &[u8] {
        &self.data.0
    }

    /// Whether this is the last chunk.
    pub fn is_last(&self) -> bool {
        self.last
    }
}

impl TryFrom<ContentChunkFields> for ContentChunk {
    type Error = DomainError;

    fn try_from(f: ContentChunkFields) -> Result<Self, DomainError> {
        Self::new(f.change_id, f.offset, f.data.0, f.last)
    }
}

/// What the receiving replica did with a change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ApplyResult {
    /// Applied onto its base.
    Applied,
    /// Its result was already in place; nothing changed.
    AlreadyApplied,
    /// The path changed differently here; both versions are kept as this conflict.
    Conflict {
        /// The conflict recorded.
        conflict_id: ConflictId,
    },
    /// Refused, with the reason, such as a path outside the selection or a failed validation.
    Rejected {
        /// Why.
        reason: Blocker,
    },
}

/// The receiving replica's answer to one change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeAck {
    /// The change.
    pub change_id: ChangeId,
    /// What happened to it.
    pub result: ApplyResult,
}

/// Defines [`PortableRecord`] from one list of records. Each record must declare that it may
/// travel ([`Record::SYNC`]); one that must stay on its device fails to compile.
macro_rules! portable_records {
    ($($(#[$doc:meta])* $variant:ident($record:ty),)*) => {
        /// A record that may travel by synchronization: only portable records, never authority.
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum PortableRecord {
            $($(#[$doc])* $variant($record),)*
        }

        $(const _: () = assert!(<$record as Record>::SYNC.travels());)*
    };
}

portable_records!(
    /// A workspace's portable record.
    Workspace(WorkspaceRecord),
    /// A piece of evidence; a copy never completes a task on the receiving device (B20).
    Evidence(Evidence),
);

// The sync family's changes are selected content; this fails to compile if their record ever stops
// declaring that it travels.
const _: () = assert!(<ContentChange as Record>::SYNC.travels());

/// What a host does with an execution-transfer step it receives (A7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferStep {
    /// A valid next step, or a new transfer request: hold this transfer from now on.
    Applied(Box<ExecutionTransfer>),
    /// The step this host already holds, delivered again: change nothing.
    AlreadyApplied,
    /// A skipped, repeated-out-of-order or altered step: refuse it, and ownership stays put.
    Refused,
}

/// The step `incoming` applied to the transfer this host holds (`held`, absent when it holds
/// none). A new transfer starts only as requested. A later step is accepted only as the next one
/// of the same transfer, with every other field unchanged, so a duplicate, a skipped step or an
/// altered record never moves ownership.
pub fn receive_transfer(
    held: Option<&ExecutionTransfer>,
    incoming: &ExecutionTransfer,
) -> TransferStep {
    let next = &incoming.fields().state;
    let Some(held) = held else {
        return if *next == TransferState::Requested {
            TransferStep::Applied(Box::new(incoming.clone()))
        } else {
            TransferStep::Refused
        };
    };
    if held == incoming {
        return TransferStep::AlreadyApplied;
    }
    let mut expected = held.fields().clone();
    expected.state = next.clone();
    if &expected != incoming.fields() {
        return TransferStep::Refused;
    }
    held.advance(next.clone())
        .map_or(TransferStep::Refused, |next| {
            TransferStep::Applied(Box::new(next))
        })
}

/// A workspace as a window is told about it: its ID, kind and name, its root on the host's device
/// and whether its content is there (core/workspaces/workspace_registry). It is for showing: a
/// window that acts on a workspace names it by its ID or by its root, never by its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceEntry {
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// Its kind.
    pub kind: WorkspaceKind,
    /// Its name.
    pub name: Label,
    /// Its root on the host's device; absent when only a listing of it is known there.
    pub root: Option<DeviceRoot>,
    /// Whether its content is on the host's device.
    pub availability: ContentAvailability,
}

/// A workspace a window asks its host to create on the host's device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewWorkspace {
    /// Its kind.
    pub kind: WorkspaceKind,
    /// Its name.
    pub name: Label,
    /// The folder that holds its content.
    pub root: DeviceRoot,
}

/// The workspace a window asks to open: by its ID, or the one whose root is a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceTarget {
    /// The workspace with this ID.
    WorkspaceId(WorkspaceId),
    /// The workspace whose root is this folder, however the path spells it.
    Root(DeviceRoot),
}

/// Why a host refused a window's workspace request. A refused request changed nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceRefusal {
    /// No workspace on the host's device has that ID, or that root.
    NotFound,
    /// The root is not an absolute path.
    RootNotAbsolute,
    /// The root does not exist or cannot be read.
    RootMissing,
    /// The root is not a folder.
    RootNotAFolder,
    /// The root's path is not text, or is too long.
    RootUnusable,
    /// The recorded root was moved, or replaced by a link.
    RootMoved,
    /// The root shares files with another workspace's root.
    Overlaps,
    /// Only a listing of the workspace is known on the host's device (A3).
    NotHere,
    /// The client does not show that workspace.
    NotOpen,
}

/// A host's answer to a window's workspace request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkspaceAnswer {
    /// The workspaces of the host's device after the window's cursor, in ID order, as many as one
    /// message holds.
    Page {
        /// The workspaces.
        entries: Vec<WorkspaceEntry>,
        /// Whether more follow after the last of them.
        more: bool,
        /// The workspace the client shows, if any.
        foreground: Option<WorkspaceId>,
    },
    /// The request was carried out.
    Done {
        /// The workspace it concerned, when it created or opened one.
        workspace: Option<WorkspaceEntry>,
        /// The workspace the client shows now, if any.
        foreground: Option<WorkspaceId>,
    },
    /// The request was refused.
    Refused {
        /// Why.
        reason: WorkspaceRefusal,
        /// The other workspace, when a root overlaps its root.
        workspace_id: Option<WorkspaceId>,
    },
}

/// Messages from a window to its own host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ClientMessage {
    /// The window's hello.
    Hello(Hello),
    /// An intent, answered by an [`Acknowledgment`].
    Intent(LocalIntent),
    /// After reconnecting: a fresh status, then the events after `after`.
    Resync {
        /// The last event the window applied; absent when it has none.
        after: Option<EventSeq>,
    },
    /// Asks for the panel layout the host keeps for this client; answered by
    /// [`HostMessage::Layout`]. From protocol version 2.
    LoadLayout {},
    /// The client's panels as they are now, for the host to keep as its view state (ST-VIEW);
    /// answered by [`HostMessage::Layout`] with what the host then keeps. It names no client:
    /// the host keeps it for the client on this channel. From protocol version 2.
    StoreLayout(PanelLayout),
    /// Asks for the workspaces of the host's device in ID order, after `after` when it is given;
    /// answered by [`HostMessage::Workspaces`] with a page. From protocol version 3.
    ListWorkspaces {
        /// The last workspace of the page before, if any.
        after: Option<WorkspaceId>,
    },
    /// Asks the host to create a workspace on its device; answered by
    /// [`HostMessage::Workspaces`]. From protocol version 3.
    CreateWorkspace(NewWorkspace),
    /// Makes a workspace the foreground workspace of the client on this channel: the one it
    /// shows (C2, B2). It names no client and binds no agent; answered by
    /// [`HostMessage::Workspaces`]. From protocol version 3.
    OpenWorkspace(WorkspaceTarget),
    /// Closes the workspace the client on this channel shows; answered by
    /// [`HostMessage::Workspaces`]. From protocol version 3.
    CloseWorkspace {
        /// The workspace the client shows.
        workspace_id: WorkspaceId,
    },
}

impl ClientMessage {
    /// The first protocol version that has this message. A host refuses a message newer than
    /// the version it agreed on with the window.
    pub const fn since(&self) -> u32 {
        match self {
            Self::Hello(_) | Self::Intent(_) | Self::Resync { .. } => 1,
            Self::LoadLayout {} | Self::StoreLayout(_) => LAYOUT_SINCE_VERSION,
            Self::ListWorkspaces { .. }
            | Self::CreateWorkspace(_)
            | Self::OpenWorkspace(_)
            | Self::CloseWorkspace { .. } => WORKSPACES_SINCE_VERSION,
        }
    }
}

/// Messages from a host to one of its windows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostMessage {
    /// The host's hello.
    Hello(Hello),
    /// The outcome of an intent.
    Acknowledgment(Acknowledgment),
    /// The host's status.
    Status(HostStatus),
    /// The next event of the stream.
    Event(SequencedEvent),
    /// The panel layout the host keeps for this client, or none: the answer to
    /// [`ClientMessage::LoadLayout`] and [`ClientMessage::StoreLayout`]. It is presentation
    /// only, never an instruction (ST-VIEW). From protocol version 2.
    Layout(Option<PanelLayout>),
    /// The answer to a workspace request. From protocol version 3.
    Workspaces(WorkspaceAnswer),
}

/// The remote-control family between paired hosts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteMessage {
    /// A request for an operation on the receiving host.
    Request(RemoteRequest),
    /// The outcome of a request.
    Acknowledgment(Acknowledgment),
    /// What the receiver's ledger records for some requests.
    ReconcileQuery(ReconcileQuery),
    /// The answer to a reconciliation.
    ReconcileReply(ReconcileReply),
    /// The sender's status, for example in answer to `status_read`.
    Status(HostStatus),
    /// A step of a transfer of execution between the two hosts; each step is echoed back once
    /// applied, and only the full sequence moves ownership (A7).
    Transfer(ExecutionTransfer),
}

/// The synchronization family between paired hosts: content and portable records only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMessage {
    /// A change of a selected workspace.
    Change(ContentChange),
    /// A chunk of a change's content.
    Chunk(ContentChunk),
    /// The receiver's answer to a change.
    Applied(ChangeAck),
    /// A portable record.
    Record(PortableRecord),
}

/// A message between two paired hosts: exactly one family per message (AD-05).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeerMessage {
    /// A hello.
    Hello(Hello),
    /// The remote-control family.
    Remote(RemoteMessage),
    /// The synchronization family.
    Sync(SyncMessage),
}

/// A window's message, decoded within the local IPC limit.
pub fn decode_client(bytes: &[u8]) -> Result<ClientMessage, ProtocolError> {
    decode(bytes, MAX_IPC_MESSAGE_BYTES)
}

/// A host's message to a window, decoded within the local IPC limit.
pub fn decode_host(bytes: &[u8]) -> Result<HostMessage, ProtocolError> {
    decode(bytes, MAX_IPC_MESSAGE_BYTES)
}

/// A peer's message, decoded within the frame limit.
pub fn decode_peer(bytes: &[u8]) -> Result<PeerMessage, ProtocolError> {
    decode(bytes, MAX_FRAME_BYTES)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::admission::{Admission, HostIdentity, admit};
    use crate::operations::{
        AgentPause, AppLaunch, ApprovalDecide, FileText, FileWrite, StatusRead,
    };
    use crate::serialization::encode;
    use crate::version_negotiation::MIN_SECURE_VERSION;
    use nexees_domain::changes::{ChangeKind, ContentChangeFields};
    use nexees_domain::ids::{AllowlistKey, TransferId};
    use nexees_domain::remote_request::RequestEnvelopeFields;
    use nexees_domain::requests::{PendingRequest, PendingRequestFields, PendingState, Reconnect};
    use nexees_domain::revision::{ContentHash, PermissionEpoch, RevisionId, WorkspaceRevision};
    use nexees_domain::session::ExecutionTransferFields;
    use nexees_domain::workspace::RelativePath;

    pub(crate) fn at(ms: u64) -> Timestamp {
        Timestamp::from_unix_millis(ms)
    }

    pub(crate) fn host() -> HostIdentity {
        HostIdentity {
            device_id: DeviceId::new("pc").unwrap(),
            user_id: UserId::new("u1").unwrap(),
            session_id: HostSessionId::new("login-2").unwrap(),
        }
    }

    pub(crate) fn hash(c: char) -> ContentHash {
        ContentHash::new(c.to_string().repeat(64)).unwrap()
    }

    /// A valid request from the phone to the PC's login session 2, issued at 1 s, expiring at 61 s.
    pub(crate) fn envelope<A: Operation>(
        id: &str,
        arguments: A,
        workspace: Option<&str>,
        agent: Option<&str>,
        expected_revisions: Vec<ExpectedRevision>,
    ) -> RequestEnvelopeFields<A> {
        RequestEnvelopeFields {
            protocol_version: ProtocolVersion::new(1).unwrap(),
            request_id: RequestId::new(id).unwrap(),
            requesting_device_id: DeviceId::new("phone").unwrap(),
            requesting_user_id: UserId::new("u1").unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            destination_user_id: UserId::new("u1").unwrap(),
            destination_session_id: HostSessionId::new("login-2").unwrap(),
            operation: OperationId::new(A::ID).unwrap(),
            arguments,
            workspace_id: workspace.map(|w| WorkspaceId::new(w).unwrap()),
            agent_id: agent.map(|a| AgentSessionId::new(a).unwrap()),
            expected_revisions,
            permission_epoch: PermissionEpoch::new(3),
            issued_at: at(1_000),
            expires_at: at(61_000),
            approval_id: None,
        }
    }

    pub(crate) fn workspace_revision(w: &str, r: &str) -> ExpectedRevision {
        ExpectedRevision::Workspace(WorkspaceRevision {
            workspace_id: WorkspaceId::new(w).unwrap(),
            revision: RevisionId::new(r).unwrap(),
        })
    }

    pub(crate) fn file_write(id: &str) -> RemoteRequest {
        RemoteRequest::FileWrite(
            RequestEnvelope::try_from(envelope(
                id,
                FileWrite {
                    path: RelativePath::new("notes.md").unwrap(),
                    text: FileText::new("hello").unwrap(),
                },
                Some("w"),
                None,
                vec![workspace_revision("w", "r1")],
            ))
            .unwrap(),
        )
    }

    fn app_launch(id: &str) -> RemoteRequest {
        RemoteRequest::AppLaunch(
            RequestEnvelope::try_from(envelope(
                id,
                AppLaunch {
                    app: AllowlistKey::new("hoptodesk").unwrap(),
                },
                None,
                None,
                vec![ExpectedRevision::Policy(RevisionId::new("p4").unwrap())],
            ))
            .unwrap(),
        )
    }

    fn change() -> ContentChange {
        ContentChange::try_from(ContentChangeFields {
            change_id: ChangeId::new("c1").unwrap(),
            workspace_id: WorkspaceId::new("w").unwrap(),
            path: RelativePath::new("notes.md").unwrap(),
            kind: ChangeKind::Write {
                content: hash('b'),
                size: 5,
            },
            base: Some(hash('a')),
            origin_device_id: DeviceId::new("phone").unwrap(),
            origin_revision: WorkspaceRevision {
                workspace_id: WorkspaceId::new("w").unwrap(),
                revision: RevisionId::new("r2").unwrap(),
            },
            recorded_at: at(5),
        })
        .unwrap()
    }

    #[test]
    fn peer_messages_round_trip_one_family_at_a_time() {
        for message in [
            PeerMessage::Hello(Hello::current()),
            PeerMessage::Remote(RemoteMessage::Request(file_write("q1"))),
            PeerMessage::Remote(RemoteMessage::Acknowledgment(Acknowledgment {
                request_id: RequestId::new("q1").unwrap(),
                outcome: Outcome::Completed,
                recorded_at: at(2_000),
                detail: None,
            })),
            PeerMessage::Sync(SyncMessage::Change(change())),
            PeerMessage::Sync(SyncMessage::Chunk(
                ContentChunk::new(ChangeId::new("c1").unwrap(), 0, b"hello".to_vec(), true)
                    .unwrap(),
            )),
        ] {
            let bytes = encode(&message, MAX_FRAME_BYTES).unwrap();
            assert_eq!(decode_peer(&bytes).unwrap(), message);
        }
        // A second family alongside the first is refused, not ignored.
        let hello = serde_json::to_string(&Hello::current()).unwrap();
        let change = serde_json::to_string(&change()).unwrap();
        let both = format!(r#"{{"hello":{hello},"sync":{{"change":{change}}}}}"#);
        assert!(decode_peer(both.as_bytes()).is_err());
    }

    #[test]
    fn the_sync_family_cannot_carry_requests_approvals_or_grants() {
        let request = serde_json::to_value(file_write("q1")).unwrap();
        for smuggled in [
            serde_json::json!({"sync": {"request": request}}),
            serde_json::json!({"sync": {"record": {"request": request}}}),
            serde_json::json!({"sync": {"record": {"remote_grant": {"grant_id": "g1"}}}}),
            serde_json::json!({"sync": {"approval": {"approval_id": "a1"}}}),
            // And a change cannot pass as a remote message.
            serde_json::json!({"remote": {"change": serde_json::to_value(change()).unwrap()}}),
        ] {
            let text = smuggled.to_string();
            assert!(
                serde_json::from_value::<PeerMessage>(smuggled).is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn requests_decode_strictly_and_only_for_registered_operations() {
        let text = serde_json::to_string(&file_write("q1")).unwrap();
        assert!(serde_json::from_str::<RemoteRequest>(&text).is_ok());
        let unknown = text.replace(
            "\"operation\":\"file_write\"",
            "\"operation\":\"shell_run\"",
        );
        assert!(serde_json::from_str::<RemoteRequest>(&unknown).is_err());
        let duplicated = text.replacen(
            "\"workspace_id\":\"w\"",
            "\"workspace_id\":\"w\",\"workspace_id\":\"other\"",
            1,
        );
        assert_ne!(duplicated, text);
        assert!(serde_json::from_str::<RemoteRequest>(&duplicated).is_err());
        let extra = text.replacen("\"request_id\"", "\"priority\":1,\"request_id\"", 1);
        assert!(serde_json::from_str::<RemoteRequest>(&extra).is_err());
        let mismatched = text.replace(
            "\"operation\":\"file_write\"",
            "\"operation\":\"app_launch\"",
        );
        assert!(serde_json::from_str::<RemoteRequest>(&mismatched).is_err());
    }

    #[test]
    fn local_only_operations_are_refused_from_peers() {
        let fields = IntentFields {
            request_id: RequestId::new("i1").unwrap(),
            operation: OperationId::new("approval_decide").unwrap(),
            arguments: ApprovalDecide {
                request_id: RequestId::new("q9").unwrap(),
                approve: true,
            },
            workspace_id: None,
            agent_id: None,
            expected_revisions: vec![ExpectedRevision::Policy(RevisionId::new("p4").unwrap())],
            issued_at: at(1_000),
            expires_at: at(2_000),
        };
        let intent = LocalIntent::ApprovalDecide(Intent::try_from(fields).unwrap());
        let text = serde_json::to_string(&intent).unwrap();
        assert_eq!(serde_json::from_str::<LocalIntent>(&text).unwrap(), intent);
        assert!(serde_json::from_str::<RemoteRequest>(&text).is_err());
    }

    #[test]
    fn intents_name_their_targets_and_expire() {
        let fields = IntentFields {
            request_id: RequestId::new("i1").unwrap(),
            operation: OperationId::new("agent_pause").unwrap(),
            arguments: AgentPause {},
            workspace_id: Some(WorkspaceId::new("w").unwrap()),
            agent_id: Some(AgentSessionId::new("s1").unwrap()),
            expected_revisions: vec![ExpectedRevision::Generation(Generation::FIRST)],
            issued_at: at(1_000),
            expires_at: at(2_000),
        };
        let intent = Intent::try_from(fields.clone()).unwrap();
        assert!(Header::check_fresh(&intent, at(1_500), at(1_999)).is_ok());
        assert_eq!(
            Header::check_fresh(&intent, at(1_500), at(2_000))
                .unwrap_err()
                .kind,
            ErrorKind::Expired
        );
        // Focus is never a target: an agent intent without its workspace is refused.
        let untargeted = IntentFields {
            workspace_id: None,
            ..fields.clone()
        };
        assert_eq!(
            Intent::try_from(untargeted).unwrap_err().field,
            "workspace_id"
        );
        let read = IntentFields {
            request_id: RequestId::new("i2").unwrap(),
            operation: OperationId::new("status_read").unwrap(),
            arguments: StatusRead {},
            workspace_id: Some(WorkspaceId::new("w").unwrap()),
            agent_id: None,
            expected_revisions: vec![],
            issued_at: at(1_000),
            expires_at: at(2_000),
        };
        assert!(Intent::try_from(read).is_ok());
    }

    #[test]
    fn reconciliation_is_bounded_and_answers_for_every_request() {
        assert_eq!(
            ReconcileQuery::new(vec![]).unwrap_err().kind,
            ErrorKind::Empty
        );
        let ids: Vec<RequestId> = (0..=MAX_RECONCILE)
            .map(|i| RequestId::new(format!("q{i}")).unwrap())
            .collect();
        assert_eq!(
            ReconcileQuery::new(ids).unwrap_err().kind,
            ErrorKind::TooLong
        );
        let q = RequestId::new("q1").unwrap();
        assert_eq!(
            ReconcileQuery::new(vec![q.clone(), q.clone()])
                .unwrap_err()
                .kind,
            ErrorKind::Duplicate
        );
        let reply = ReconcileReply::new(vec![ReconcileEntry {
            request_id: q.clone(),
            recorded: None,
        }])
        .unwrap();
        assert_eq!(reply.recorded(&q), Ok(None));
        // A reply that does not mention a request never counts as "never received".
        assert_eq!(
            reply
                .recorded(&RequestId::new("q2").unwrap())
                .unwrap_err()
                .kind,
            ErrorKind::Missing
        );
    }

    #[test]
    fn events_apply_once_in_order_and_never_across_a_gap() {
        let seq = |n| EventSeq::new(n).unwrap();
        let mut cursor = EventCursor::default();
        assert_eq!(cursor.accept(seq(1)), Delivery::Apply);
        assert_eq!(cursor.accept(seq(2)), Delivery::Apply);
        // Redelivered after a reconnect: ignored, not applied twice.
        assert_eq!(cursor.accept(seq(2)), Delivery::Duplicate);
        // Missed events: nothing after the gap is applied until a fresh status.
        assert_eq!(cursor.accept(seq(5)), Delivery::Gap);
        assert_eq!(cursor.last(), Some(seq(2)));
        cursor.reset_to(Some(seq(4)));
        assert_eq!(cursor.accept(seq(5)), Delivery::Apply);
        assert!(EventSeq::new(0).is_err());
    }

    #[test]
    fn status_is_live_only_while_its_session_confirms_it() {
        let status = HostStatus::new(DeviceId::new("pc").unwrap(), at(7), vec![], vec![]).unwrap();
        assert_eq!(status.observation(true), Observation::Live);
        assert_eq!(status.observation(false), Observation::LastObserved(at(7)));
    }

    #[test]
    fn channels_bound_their_messages() {
        let big = ContentChunk::new(
            ChangeId::new("c1").unwrap(),
            0,
            vec![0; MAX_CHUNK_BYTES + 1],
            false,
        );
        assert_eq!(big.unwrap_err().kind, ErrorKind::TooLong);
        let full = SyncMessage::Chunk(
            ContentChunk::new(
                ChangeId::new("c1").unwrap(),
                0,
                vec![7; MAX_CHUNK_BYTES],
                true,
            )
            .unwrap(),
        );
        assert!(encode(&PeerMessage::Sync(full), MAX_FRAME_BYTES).is_ok());
        // A window's message larger than the local IPC limit is refused unread.
        let oversized = vec![b' '; MAX_IPC_MESSAGE_BYTES + 1];
        assert!(matches!(
            decode_client(&oversized),
            Err(ProtocolError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_windows_layout_messages_are_small_strict_and_of_protocol_version_2() {
        let panels = concat!(
            r#"{"left":{"shown":true,"size":203,"selected":"explorer-view-container"},"#,
            r#""right":{"shown":false,"size":354,"selected":"nexees-area-tasks"},"#,
            r#""bottom":{"shown":true,"size":167,"selected":null}}"#
        );
        let layout: PanelLayout = serde_json::from_str(panels).unwrap();
        let store = ClientMessage::StoreLayout(layout.clone());
        let bytes = encode(&store, MAX_IPC_MESSAGE_BYTES).unwrap();
        assert_eq!(
            String::from_utf8(bytes.clone()).unwrap(),
            format!(r#"{{"store_layout":{panels}}}"#)
        );
        assert_eq!(decode_client(&bytes).unwrap(), store);
        assert_eq!(
            decode_client(br#"{"load_layout":{}}"#).unwrap(),
            ClientMessage::LoadLayout {}
        );
        let kept = encode(&HostMessage::Layout(Some(layout)), MAX_IPC_MESSAGE_BYTES).unwrap();
        assert_eq!(
            String::from_utf8(kept).unwrap(),
            format!(r#"{{"layout":{panels}}}"#)
        );
        assert_eq!(
            decode_host(br#"{"layout":null}"#).unwrap(),
            HostMessage::Layout(None)
        );
        // A window names no client and no target, and a size that is none is refused before the
        // host looks at anything else.
        for bad in [
            r#"{"load_layout":{"client_id":"another-window"}}"#.to_owned(),
            format!(r#"{{"store_layout":{panels},"client_id":"another-window"}}"#),
            format!(r#"{{"store_layout":{}}}"#, panels.replace("203", "0")),
            format!(
                r#"{{"store_layout":{}}}"#,
                panels.replace(r#""size":203"#, r#""workspace":"w","size":203"#)
            ),
        ] {
            assert!(decode_client(bad.as_bytes()).is_err(), "{bad}");
        }
        // Both messages belong to protocol version 2; everything older is of version 1.
        assert_eq!(store.since(), LAYOUT_SINCE_VERSION);
        assert_eq!(ClientMessage::LoadLayout {}.since(), LAYOUT_SINCE_VERSION);
        assert_eq!(ClientMessage::Resync { after: None }.since(), 1);
        assert_eq!(ClientMessage::Hello(Hello::current()).since(), 1);
    }

    #[test]
    fn a_windows_workspace_messages_are_strict_and_of_protocol_version_3() {
        let wire = [
            (
                r#"{"list_workspaces":{"after":null}}"#,
                ClientMessage::ListWorkspaces { after: None },
            ),
            (
                concat!(
                    r#"{"create_workspace":{"kind":"lcl","name":"LCL — Next","#,
                    r#""root":"/home/u/lcl-next"}}"#
                ),
                ClientMessage::CreateWorkspace(NewWorkspace {
                    kind: WorkspaceKind::Lcl,
                    name: Label::new("LCL — Next").unwrap(),
                    root: DeviceRoot::new("/home/u/lcl-next").unwrap(),
                }),
            ),
            (
                r#"{"open_workspace":{"workspace_id":"ws-1"}}"#,
                ClientMessage::OpenWorkspace(WorkspaceTarget::WorkspaceId(
                    WorkspaceId::new("ws-1").unwrap(),
                )),
            ),
            (
                r#"{"open_workspace":{"root":"/home/u/arch-dock"}}"#,
                ClientMessage::OpenWorkspace(WorkspaceTarget::Root(
                    DeviceRoot::new("/home/u/arch-dock").unwrap(),
                )),
            ),
            (
                r#"{"close_workspace":{"workspace_id":"ws-1"}}"#,
                ClientMessage::CloseWorkspace {
                    workspace_id: WorkspaceId::new("ws-1").unwrap(),
                },
            ),
        ];
        for (text, message) in &wire {
            assert_eq!(&decode_client(text.as_bytes()).unwrap(), message, "{text}");
            assert_eq!(
                String::from_utf8(encode(message, MAX_IPC_MESSAGE_BYTES).unwrap()).unwrap(),
                *text
            );
            // Every one of them belongs to protocol version 3.
            assert_eq!(message.since(), WORKSPACES_SINCE_VERSION, "{text}");
        }
        let entry = WorkspaceEntry {
            workspace_id: WorkspaceId::new("ws-1").unwrap(),
            kind: WorkspaceKind::Code,
            name: Label::new("Arch Dock").unwrap(),
            root: Some(DeviceRoot::new("/home/u/arch-dock").unwrap()),
            availability: ContentAvailability::Local,
        };
        for (text, answer) in [
            (
                concat!(
                    r#"{"workspaces":{"page":{"entries":[{"workspace_id":"ws-1","kind":"code","#,
                    r#""name":"Arch Dock","root":"/home/u/arch-dock","availability":"local"}],"#,
                    r#""more":false,"foreground":"ws-1"}}}"#
                ),
                WorkspaceAnswer::Page {
                    entries: vec![entry.clone()],
                    more: false,
                    foreground: Some(entry.workspace_id.clone()),
                },
            ),
            (
                r#"{"workspaces":{"done":{"workspace":null,"foreground":null}}}"#,
                WorkspaceAnswer::Done {
                    workspace: None,
                    foreground: None,
                },
            ),
            (
                r#"{"workspaces":{"refused":{"reason":"overlaps","workspace_id":"ws-1"}}}"#,
                WorkspaceAnswer::Refused {
                    reason: WorkspaceRefusal::Overlaps,
                    workspace_id: Some(entry.workspace_id.clone()),
                },
            ),
        ] {
            let message = HostMessage::Workspaces(answer);
            assert_eq!(decode_host(text.as_bytes()).unwrap(), message, "{text}");
            assert_eq!(
                String::from_utf8(encode(&message, MAX_IPC_MESSAGE_BYTES).unwrap()).unwrap(),
                text
            );
        }
        // A window names no client and no agent, carries no authority, and a root or a name that
        // is no valid value is refused before the host looks at anything else.
        for bad in [
            r#"{"list_workspaces":{"after":null,"client_id":"another-window"}}"#,
            r#"{"open_workspace":{"workspace_id":"ws-1","agent_session":"claude-1"}}"#,
            r#"{"open_workspace":{"workspace_id":"ws-1","root":"/x"}}"#,
            r#"{"create_workspace":{"kind":"code","name":"W","root":"/w","grants":["all"]}}"#,
            r#"{"create_workspace":{"kind":"android","name":"W","root":"/w"}}"#,
            r#"{"create_workspace":{"kind":"code","name":"two\nlines","root":"/w"}}"#,
            r#"{"create_workspace":{"kind":"code","name":"W","root":"/w\u0000x"}}"#,
            r#"{"close_workspace":{}}"#,
        ] {
            assert!(decode_client(bad.as_bytes()).is_err(), "{bad}");
        }
        assert!(
            decode_host(
                br#"{"workspaces":{"done":{"workspace":null,"foreground":null,"bound":"s"}}}"#
            )
            .is_err()
        );
    }

    fn transfer(state: TransferState) -> ExecutionTransfer {
        ExecutionTransfer::try_from(ExecutionTransferFields {
            transfer_id: TransferId::new("t1").unwrap(),
            session_id: AgentSessionId::new("s1").unwrap(),
            workspace_id: WorkspaceId::new("w").unwrap(),
            from_device_id: DeviceId::new("pc").unwrap(),
            to_device_id: DeviceId::new("phone").unwrap(),
            checkpoint: WorkspaceRevision {
                workspace_id: WorkspaceId::new("w").unwrap(),
                revision: RevisionId::new("r7").unwrap(),
            },
            specification: None,
            state,
            requested_at: at(1),
        })
        .unwrap()
    }

    #[test]
    fn transfer_steps_apply_once_in_order_and_never_skip_quiescence() {
        let requested = transfer(TransferState::Requested);
        assert_eq!(
            receive_transfer(None, &requested),
            TransferStep::Applied(Box::new(requested.clone()))
        );
        // A transfer cannot begin anywhere but at its request.
        assert_eq!(
            receive_transfer(None, &transfer(TransferState::Transferred)),
            TransferStep::Refused
        );
        // The same step delivered twice changes nothing.
        assert_eq!(
            receive_transfer(Some(&requested), &requested),
            TransferStep::AlreadyApplied
        );
        // Skipping the owner's acknowledgment never moves ownership.
        assert_eq!(
            receive_transfer(Some(&requested), &transfer(TransferState::Transferred)),
            TransferStep::Refused
        );
        let quiesced = transfer(TransferState::OwnerQuiesced);
        assert!(matches!(
            receive_transfer(Some(&requested), &quiesced),
            TransferStep::Applied(_)
        ));
        // An altered record under the same transfer is refused.
        let mut altered = quiesced.clone().into_fields();
        altered.to_device_id = DeviceId::new("tablet").unwrap();
        let altered = ExecutionTransfer::try_from(altered).unwrap();
        assert_eq!(
            receive_transfer(Some(&requested), &altered),
            TransferStep::Refused
        );
    }

    #[test]
    fn rc_t09_a_lost_acknowledgment_never_executes_the_action_twice() {
        let request = file_write("q7");
        let peer = DeviceId::new("phone").unwrap();
        let mut ledger = None;
        let mut executions = 0;
        // The destination admits and runs the request; its acknowledgment is then lost.
        if let Admission::Admit(entry) = admit(
            &request,
            hash('a'),
            &host(),
            &peer,
            ledger.as_ref(),
            at(1_100),
            at(1_100),
            MIN_SECURE_VERSION,
        ) {
            executions += 1;
            ledger = Some(entry.record(Outcome::Completed, at(1_200)).unwrap());
        }
        let pending = PendingRequest::try_from(PendingRequestFields {
            request_id: RequestId::new("q7").unwrap(),
            request_hash: hash('a'),
            destination_device_id: DeviceId::new("pc").unwrap(),
            operation: OperationId::new("file_write").unwrap(),
            expires_at: at(61_000),
            state: PendingState::Sent,
        })
        .unwrap();
        // After reconnecting, the sender reconciles by request ID and learns the outcome.
        let query = ReconcileQuery::new(vec![RequestId::new("q7").unwrap()]).unwrap();
        let reply = ReconcileReply::new(
            query
                .request_ids()
                .iter()
                .map(|id| ReconcileEntry {
                    request_id: id.clone(),
                    recorded: ledger
                        .as_ref()
                        .filter(|e| &e.fields().request_id == id)
                        .map(|e| e.fields().state),
                })
                .collect(),
        )
        .unwrap();
        let recorded = reply.recorded(&RequestId::new("q7").unwrap()).unwrap();
        let (settled, action) = pending.reconnect(recorded, at(3_000));
        assert_eq!(action, Reconnect::Settled(Outcome::Completed));
        assert!(settled.completed());
        // Even a blind resend of the same request is answered from the ledger, never run again.
        let again = admit(
            &request,
            hash('a'),
            &host(),
            &peer,
            ledger.as_ref(),
            at(3_100),
            at(3_100),
            MIN_SECURE_VERSION,
        );
        assert_eq!(again, Admission::Duplicate(Outcome::Completed));
        assert_eq!(executions, 1);
    }

    #[test]
    fn rc_t10_an_expired_queued_command_is_not_replayed_while_content_still_syncs() {
        let queued = PendingRequest::try_from(PendingRequestFields {
            request_id: RequestId::new("q8").unwrap(),
            request_hash: hash('a'),
            destination_device_id: DeviceId::new("pc").unwrap(),
            operation: OperationId::new("app_launch").unwrap(),
            expires_at: at(61_000),
            state: PendingState::Unsent,
        })
        .unwrap();
        // Reconnecting after its expiry: the destination never received it, and it is dropped.
        assert_eq!(queued.reconnect(None, at(90_000)).1, Reconnect::Drop);
        // Even if it were sent anyway, the destination refuses it as expired.
        let late = admit(
            &app_launch("q8"),
            hash('a'),
            &host(),
            &DeviceId::new("phone").unwrap(),
            None,
            at(90_000),
            at(90_000),
            MIN_SECURE_VERSION,
        );
        assert!(matches!(
            late,
            Admission::Refuse {
                outcome: Outcome::Expired,
                ..
            }
        ));
        // File content keeps synchronizing; the sync family carries content, never the command or
        // its grant (see the_sync_family_cannot_carry_requests_approvals_or_grants).
        let content = PeerMessage::Sync(SyncMessage::Change(change()));
        assert!(encode(&content, MAX_FRAME_BYTES).is_ok());
    }
}
