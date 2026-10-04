//! Model handoff: switching the model of a running session, and the read-only orientation that
//! may follow (B23, H1–H4, ST-HANDOFF).
//!
//! A [`Handoff`] pins the session, its new generation and every binding at the moment of the
//! switch. With orientation [`OrientationSetting::On`] the new model reads the required files
//! first, and the handoff becomes ready only when the runtime observed a read receipt for each.
//! Orientation is a reading phase, not an authority toggle: turning it off skips only the extra
//! reads, never the permission, policy or generation checks (H1). The switch transaction is
//! TASK-044's and the orientation TASK-045's; this module fixes the record they share.

use serde::{Deserialize, Serialize};

use crate::errors::{Blocker, DomainError, ErrorKind};
use crate::ids::{AgentSessionId, DeviceId, HandoffId, TaskId, WorkspaceId};
use crate::revision::{ContentHash, Generation, SpecRevision, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, ensure_unique, validated_record};
use crate::session::AgentSession;
use crate::time::Timestamp;
use crate::workspace::RelativePath;

/// Whether a model switch is followed by the read-only orientation phase. **On** by default
/// (H1, v0.3 auth_and_settings).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrientationSetting {
    /// The new model reads the required files, read-only, before it may write.
    #[default]
    On,
    /// The extra reading phase is skipped; every other gate still applies.
    Off,
}

impl OrientationSetting {
    /// The setting in effect: the session's override if it has one, otherwise the user's
    /// default (H1).
    pub fn effective(user_default: Self, session_override: Option<Self>) -> Self {
        session_override.unwrap_or(user_default)
    }
}

/// The steps of a handoff, as the user sees them (H4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum HandoffPhase {
    /// The old model receives no new work; its requests are cancelled or drained.
    PausingOldModel,
    /// The checkpoint is saved and the new model's access and capabilities are checked.
    PreparingState,
    /// The new model reads the required files, read-only (orientation on only).
    ReadingContext,
    /// Ready to continue with the session's normal permissions.
    Ready,
    /// Continuing the unfinished task. Final.
    Continuing,
    /// Paused visibly, with the precise reason; another explicit model choice is a new
    /// handoff. Final.
    Blocked(Blocker),
    /// Cancelled; the checkpoint is kept. Final.
    Cancelled,
}

impl HandoffPhase {
    const fn is_final(&self) -> bool {
        matches!(self, Self::Continuing | Self::Blocked(_) | Self::Cancelled)
    }
}

/// A read the runtime observed during orientation: which file, with which content, when.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadReceipt {
    /// The file read.
    pub path: RelativePath,
    /// The hash of the content returned.
    pub content: ContentHash,
    /// When it was read.
    pub read_at: Timestamp,
}

/// The fields of a [`Handoff`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffFields {
    /// The handoff.
    pub handoff_id: HandoffId,
    /// The session whose model changes.
    pub session_id: AgentSessionId,
    /// The new generation, which the switch created: always after the first.
    pub generation: Generation,
    /// The session's execution device, unchanged by the switch.
    pub execution_device_id: DeviceId,
    /// The session's workspace, unchanged by the switch.
    pub workspace_id: WorkspaceId,
    /// The unfinished task.
    pub task_id: TaskId,
    /// The checkpoint of the working tree, uncommitted edits included.
    pub checkpoint: WorkspaceRevision,
    /// The adopted specification revision, when the session uses one.
    pub specification: Option<SpecRevision>,
    /// The orientation setting in effect for this handoff.
    pub orientation: OrientationSetting,
    /// The step reached.
    pub phase: HandoffPhase,
    /// The files to read in order, each once.
    pub required_reads: Vec<RelativePath>,
    /// The reads observed so far, at most one per required file.
    pub receipts: Vec<ReadReceipt>,
}

impl HandoffFields {
    fn orientation_complete(&self) -> bool {
        self.required_reads
            .iter()
            .all(|path| self.receipts.iter().any(|r| &r.path == path))
    }
}

impl Validate for HandoffFields {
    fn validate(&self) -> Result<(), DomainError> {
        if self.generation <= Generation::FIRST {
            return Err(DomainError::new("generation", ErrorKind::OutOfRange));
        }
        self.checkpoint
            .ensure_of(&self.workspace_id, "checkpoint")?;
        ensure_unique(&self.required_reads, "required_reads")?;
        let read: Vec<&RelativePath> = self.receipts.iter().map(|r| &r.path).collect();
        ensure_unique(&read, "receipts")?;
        if read.iter().any(|path| !self.required_reads.contains(path)) {
            return Err(DomainError::new("receipts", ErrorKind::Unexpected));
        }
        match self.orientation {
            OrientationSetting::Off if self.phase == HandoffPhase::ReadingContext => {
                Err(DomainError::new("phase", ErrorKind::InvalidTransition))
            }
            OrientationSetting::On
                if matches!(self.phase, HandoffPhase::Ready | HandoffPhase::Continuing)
                    && !self.orientation_complete() =>
            {
                Err(DomainError::new("receipts", ErrorKind::Missing))
            }
            _ => Ok(()),
        }
    }
}

validated_record!(
    /// One model handoff (B23): `handoff -> session + generation + execution device + workspace
    /// + task + checkpoint revision + specification revision`.
    Handoff,
    HandoffFields
);

impl Handoff {
    /// The handoff at its next step. Pausing leads to preparing; preparing leads to reading the
    /// context when orientation is on and straight to ready when it is off; ready leads to
    /// continuing. Any unfinished handoff may block or be cancelled. A ready handoff with
    /// orientation on needs a receipt for every required read.
    pub fn advance(&self, next: HandoffPhase) -> Result<Self, DomainError> {
        let on = self.0.orientation == OrientationSetting::On;
        let allowed = match (&self.0.phase, &next) {
            (current, HandoffPhase::Blocked(_) | HandoffPhase::Cancelled) => !current.is_final(),
            (HandoffPhase::PausingOldModel, HandoffPhase::PreparingState) => true,
            (HandoffPhase::PreparingState, HandoffPhase::ReadingContext) => on,
            (HandoffPhase::PreparingState, HandoffPhase::Ready) => !on,
            (HandoffPhase::ReadingContext, HandoffPhase::Ready) => true,
            (HandoffPhase::Ready, HandoffPhase::Continuing) => true,
            _ => false,
        };
        if !allowed {
            return Err(DomainError::new("phase", ErrorKind::InvalidTransition));
        }
        Self::try_from(HandoffFields {
            phase: next,
            ..self.0.clone()
        })
    }

    /// The handoff with one more observed read. Reads are recorded only while reading the
    /// context, only for required files, and once per file.
    pub fn record_read(&self, receipt: ReadReceipt) -> Result<Self, DomainError> {
        if self.0.phase != HandoffPhase::ReadingContext {
            return Err(DomainError::new("phase", ErrorKind::InvalidTransition));
        }
        let mut fields = self.0.clone();
        fields.receipts.push(receipt);
        Self::try_from(fields)
    }

    /// Fails unless the handoff still matches `session`'s current binding and generation. The
    /// binding is checked again before the new model may write, because a newly adopted rule or
    /// another switch can invalidate an orientation (B23, H3).
    pub fn check_binding(&self, session: &AgentSession) -> Result<(), DomainError> {
        let (h, s) = (&self.0, session.fields());
        if h.session_id != s.session_id {
            return Err(DomainError::new("session_id", ErrorKind::Mismatch));
        }
        session.check_generation(h.generation)?;
        if h.execution_device_id != s.execution_device_id {
            return Err(DomainError::new("execution_device_id", ErrorKind::Mismatch));
        }
        if h.workspace_id != s.workspace_id {
            return Err(DomainError::new("workspace_id", ErrorKind::Mismatch));
        }
        if h.specification != s.specification {
            return Err(DomainError::new("specification", ErrorKind::Stale));
        }
        Ok(())
    }
}

impl Record for Handoff {
    const SCHEMA: &'static str = "nexees.handoff.handoff";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::BlockerKind;
    use crate::ids::{ModelId, ProviderId};
    use crate::model_capabilities::ProviderModel;
    use crate::revision::RevisionId;
    use crate::schema::Versioned;
    use crate::session::tests::session_fields;
    use crate::text::Note;

    fn switched_session() -> AgentSession {
        AgentSession::try_from(session_fields())
            .unwrap()
            .switch_model(ProviderModel {
                provider_id: ProviderId::new("openai").unwrap(),
                model_id: ModelId::new("gpt-5.1").unwrap(),
            })
            .unwrap()
    }

    fn handoff(orientation: OrientationSetting, phase: HandoffPhase) -> HandoffFields {
        let s = switched_session().into_fields();
        HandoffFields {
            handoff_id: HandoffId::new("h1").unwrap(),
            session_id: s.session_id,
            generation: s.generation,
            execution_device_id: s.execution_device_id,
            workspace_id: s.workspace_id,
            task_id: TaskId::new("t1").unwrap(),
            checkpoint: s.checkout,
            specification: s.specification,
            orientation,
            phase,
            required_reads: vec![
                RelativePath::new("AGENTS.md").unwrap(),
                RelativePath::new("rules/master.lcl.txt").unwrap(),
            ],
            receipts: vec![],
        }
    }

    fn receipt(path: &str) -> ReadReceipt {
        ReadReceipt {
            path: RelativePath::new(path).unwrap(),
            content: ContentHash::new("d".repeat(64)).unwrap(),
            read_at: Timestamp::from_unix_millis(3),
        }
    }

    #[test]
    fn orientation_is_on_by_default_and_in_fresh_records() {
        assert_eq!(OrientationSetting::default(), OrientationSetting::On);
        assert_eq!(
            OrientationSetting::effective(OrientationSetting::default(), None),
            OrientationSetting::On
        );
        assert_eq!(
            OrientationSetting::effective(OrientationSetting::On, Some(OrientationSetting::Off)),
            OrientationSetting::Off
        );
    }

    #[test]
    fn with_orientation_on_every_required_read_is_observed_before_ready() {
        let start = Handoff::try_from(handoff(
            OrientationSetting::On,
            HandoffPhase::PausingOldModel,
        ))
        .unwrap();
        let reading = start
            .advance(HandoffPhase::PreparingState)
            .and_then(|h| h.advance(HandoffPhase::ReadingContext))
            .unwrap();
        assert_eq!(
            reading.advance(HandoffPhase::Ready).unwrap_err(),
            DomainError::new("receipts", ErrorKind::Missing)
        );
        let half = reading.record_read(receipt("AGENTS.md")).unwrap();
        assert_eq!(
            half.advance(HandoffPhase::Ready).unwrap_err().kind,
            ErrorKind::Missing
        );
        let ready = half
            .record_read(receipt("rules/master.lcl.txt"))
            .and_then(|h| h.advance(HandoffPhase::Ready))
            .unwrap();
        assert!(ready.advance(HandoffPhase::Continuing).is_ok());
        // Preparing cannot skip the reads while orientation is on.
        let preparing = start.advance(HandoffPhase::PreparingState).unwrap();
        assert_eq!(
            preparing.advance(HandoffPhase::Ready).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn reads_are_only_of_required_files_once_each_and_only_while_reading() {
        let reading = Handoff::try_from(handoff(
            OrientationSetting::On,
            HandoffPhase::ReadingContext,
        ))
        .unwrap();
        assert_eq!(
            reading
                .record_read(receipt("src/secret_plan.md"))
                .unwrap_err(),
            DomainError::new("receipts", ErrorKind::Unexpected)
        );
        let once = reading.record_read(receipt("AGENTS.md")).unwrap();
        assert_eq!(
            once.record_read(receipt("AGENTS.md")).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let preparing = Handoff::try_from(handoff(
            OrientationSetting::On,
            HandoffPhase::PreparingState,
        ))
        .unwrap();
        assert_eq!(
            preparing
                .record_read(receipt("AGENTS.md"))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn with_orientation_off_only_the_extra_reading_phase_is_skipped() {
        let preparing = Handoff::try_from(handoff(
            OrientationSetting::Off,
            HandoffPhase::PreparingState,
        ))
        .unwrap();
        assert_eq!(
            preparing
                .advance(HandoffPhase::ReadingContext)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
        let ready = preparing.advance(HandoffPhase::Ready).unwrap();
        assert!(ready.check_binding(&switched_session()).is_ok());
    }

    #[test]
    fn a_handoff_is_fenced_to_the_current_binding_and_generation() {
        let handoff = Handoff::try_from(handoff(
            OrientationSetting::On,
            HandoffPhase::ReadingContext,
        ))
        .unwrap();
        let session = switched_session();
        assert!(handoff.check_binding(&session).is_ok());
        // Another switch makes this handoff's generation old.
        let again = session
            .switch_model(ProviderModel {
                provider_id: ProviderId::new("local").unwrap(),
                model_id: ModelId::new("llama3.1:8b").unwrap(),
            })
            .unwrap();
        assert_eq!(
            handoff.check_binding(&again).unwrap_err(),
            DomainError::new("generation", ErrorKind::Stale)
        );
        // A newly adopted rule revision invalidates the orientation.
        let mut fields = session.into_fields();
        fields.specification = Some(SpecRevision {
            workspace_id: WorkspaceId::new("lcl-arch").unwrap(),
            revision: RevisionId::new("s4").unwrap(),
        });
        let readopted = AgentSession::try_from(fields).unwrap();
        assert_eq!(
            handoff.check_binding(&readopted).unwrap_err(),
            DomainError::new("specification", ErrorKind::Stale)
        );
    }

    #[test]
    fn a_handoff_belongs_to_a_switch_and_to_its_workspace() {
        let mut first = handoff(OrientationSetting::On, HandoffPhase::PausingOldModel);
        first.generation = Generation::FIRST;
        assert_eq!(
            Handoff::try_from(first).unwrap_err().kind,
            ErrorKind::OutOfRange
        );
        let mut elsewhere = handoff(OrientationSetting::On, HandoffPhase::PausingOldModel);
        elsewhere.checkpoint = WorkspaceRevision {
            workspace_id: WorkspaceId::new("other").unwrap(),
            revision: RevisionId::new("r").unwrap(),
        };
        assert_eq!(
            Handoff::try_from(elsewhere).unwrap_err(),
            DomainError::new("checkpoint", ErrorKind::Mismatch)
        );
    }

    #[test]
    fn a_blocked_handoff_stays_blocked_and_round_trips() {
        let blocked = Handoff::try_from(handoff(
            OrientationSetting::On,
            HandoffPhase::ReadingContext,
        ))
        .unwrap()
        .advance(HandoffPhase::Blocked(Blocker {
            kind: BlockerKind::LimitReached,
            detail: Note::new("The new provider's quota is exhausted.").unwrap(),
        }))
        .unwrap();
        assert_eq!(
            blocked.advance(HandoffPhase::Ready).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
        let text = serde_json::to_string(&Versioned(blocked.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<Handoff>>(&text).unwrap().0,
            blocked
        );
    }
}
