//! Agent sessions and what they are bound to, and the transfer of a session's execution to
//! another device (B17, C21, A5, A7, ST-SESSION).
//!
//! An [`AgentSession`] binds an agent to an execution device, a workspace checkout and, when LCL
//! is used, the adopted specification revision; it also records the provider and model. UI
//! focus, a provider switch, connectivity and synchronization never change that binding:
//! switching the model changes only the model and advances the generation
//! ([`AgentSession::switch_model`]). Execution moves to another device only through an
//! [`ExecutionTransfer`] that passes an acknowledged checkpoint. An unreachable owner is never
//! replaced, so there are never two authoritative executors for one checkout.

use serde::{Deserialize, Serialize};

use crate::errors::{Blocker, DomainError, ErrorKind};
use crate::handoff::OrientationSetting;
use crate::ids::{AgentSessionId, DeviceId, TransferId, WorkspaceId};
use crate::lcl::SpecificationMode;
use crate::model_capabilities::ProviderModel;
use crate::revision::{Generation, SpecRevision, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, validated_record};
use crate::time::Timestamp;

/// How much an agent does before it asks (project charter, "Autonomy modes"). Independent of the
/// specification mode; the policy behind each mode is the permission engine's (TASK-022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyMode {
    /// The user approves each consequential step.
    Supervised,
    /// Routine steps proceed; risky ones ask.
    Balanced,
    /// Proceeds within its grants until done or blocked.
    Autonomous,
    /// A policy the user configured.
    Custom,
}

/// The fields of an [`AgentSession`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSessionFields {
    /// The session.
    pub session_id: AgentSessionId,
    /// The workspace the agent may operate on.
    pub workspace_id: WorkspaceId,
    /// The device that executes the session's tools.
    pub execution_device_id: DeviceId,
    /// The checkout or working tree it works on, a revision of `workspace_id`.
    pub checkout: WorkspaceRevision,
    /// How the session uses LCL.
    pub specification_mode: SpecificationMode,
    /// The adopted specification revision; present exactly when the mode uses one.
    pub specification: Option<SpecRevision>,
    /// The provider and model.
    pub model: ProviderModel,
    /// The current generation; it advances with every model switch.
    pub generation: Generation,
    /// The autonomy mode.
    pub autonomy: AutonomyMode,
    /// The session's own orientation setting, which overrides the user's default (H1).
    pub orientation_override: Option<OrientationSetting>,
}

impl Validate for AgentSessionFields {
    fn validate(&self) -> Result<(), DomainError> {
        self.checkout.ensure_of(&self.workspace_id, "checkout")?;
        match (
            self.specification_mode.uses_specification(),
            &self.specification,
        ) {
            (true, None) => Err(DomainError::new("specification", ErrorKind::Missing)),
            (false, Some(_)) => Err(DomainError::new("specification", ErrorKind::Unexpected)),
            _ => Ok(()),
        }
    }
}

validated_record!(
    /// An agent session and its binding (B17): `session -> execution device + workspace +
    /// checkout + specification revision`, plus its model and generation.
    AgentSession,
    AgentSessionFields
);

impl AgentSession {
    /// Fails with [`ErrorKind::Stale`] unless `generation` is the session's current one. A reply
    /// or tool call from an older model generation can never change state (B23, HO-05).
    pub fn check_generation(&self, generation: Generation) -> Result<(), DomainError> {
        if generation == self.0.generation {
            Ok(())
        } else {
            Err(DomainError::new("generation", ErrorKind::Stale))
        }
    }

    /// The same session with another model: the generation advances and every binding stays as
    /// it was. A model switch is not a device handoff and grants nothing (H2 step 4).
    pub fn switch_model(&self, model: ProviderModel) -> Result<Self, DomainError> {
        let generation = self.0.generation.next()?;
        Self::try_from(AgentSessionFields {
            model,
            generation,
            ..self.0.clone()
        })
    }

    /// The orientation setting that applies to this session: its override, otherwise the user's
    /// default (H1).
    pub fn effective_orientation(&self, user_default: OrientationSetting) -> OrientationSetting {
        OrientationSetting::effective(user_default, self.0.orientation_override)
    }
}

impl Record for AgentSession {
    const SCHEMA: &'static str = "nexees.session.agent_session";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The steps of an [`ExecutionTransfer`], in order (A7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum TransferState {
    /// Requested; the current owner still executes.
    Requested,
    /// The current owner acknowledged that it stopped at the checkpoint.
    OwnerQuiesced,
    /// Files and state are reconciled on the destination.
    Reconciled,
    /// The destination's capabilities and permissions are validated for the session.
    DestinationValidated,
    /// The destination owns the execution. Final.
    Transferred,
    /// Refused or abandoned, with the reason; the previous owner keeps the session. Final.
    Refused(Blocker),
}

impl TransferState {
    const fn is_final(&self) -> bool {
        matches!(self, Self::Transferred | Self::Refused(_))
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::Requested => 0,
            Self::OwnerQuiesced => 1,
            Self::Reconciled => 2,
            Self::DestinationValidated => 3,
            Self::Transferred => 4,
            Self::Refused(_) => 5,
        }
    }
}

/// The fields of an [`ExecutionTransfer`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionTransferFields {
    /// The transfer.
    pub transfer_id: TransferId,
    /// The session whose execution moves.
    pub session_id: AgentSessionId,
    /// The session's workspace.
    pub workspace_id: WorkspaceId,
    /// The device that executes the session now.
    pub from_device_id: DeviceId,
    /// The device that is to execute it.
    pub to_device_id: DeviceId,
    /// The checkpoint the execution resumes from, a revision of `workspace_id`.
    pub checkpoint: WorkspaceRevision,
    /// The adopted specification revision, when the session uses one.
    pub specification: Option<SpecRevision>,
    /// The step reached.
    pub state: TransferState,
    /// When the transfer was requested.
    pub requested_at: Timestamp,
}

impl Validate for ExecutionTransferFields {
    fn validate(&self) -> Result<(), DomainError> {
        self.checkpoint
            .ensure_of(&self.workspace_id, "checkpoint")?;
        if self.from_device_id == self.to_device_id {
            return Err(DomainError::new("to_device_id", ErrorKind::Duplicate));
        }
        Ok(())
    }
}

validated_record!(
    /// The transfer of a session's execution to another device at an acknowledged checkpoint
    /// (A7, C21, AN-11). Live processes, terminal handles, secrets and hidden provider context
    /// are not part of it.
    ExecutionTransfer,
    ExecutionTransferFields
);

impl ExecutionTransfer {
    /// The transfer at its next step. The steps happen in order, so the destination never owns
    /// the execution before the previous owner acknowledged that it stopped; any unfinished
    /// transfer may be refused. A final transfer does not change.
    pub fn advance(&self, next: TransferState) -> Result<Self, DomainError> {
        let current = &self.0.state;
        let allowed = !current.is_final()
            && (matches!(next, TransferState::Refused(_)) || next.rank() == current.rank() + 1);
        if !allowed {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        Self::try_from(ExecutionTransferFields {
            state: next,
            ..self.0.clone()
        })
    }
}

impl Record for ExecutionTransfer {
    const SCHEMA: &'static str = "nexees.session.execution_transfer";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::errors::BlockerKind;
    use crate::ids::{ModelId, ProviderId};
    use crate::revision::RevisionId;
    use crate::schema::Versioned;
    use crate::text::Note;

    fn revision(workspace: &str, revision: &str) -> WorkspaceRevision {
        WorkspaceRevision {
            workspace_id: WorkspaceId::new(workspace).unwrap(),
            revision: RevisionId::new(revision).unwrap(),
        }
    }

    fn model(name: &str) -> ProviderModel {
        ProviderModel {
            provider_id: ProviderId::new("anthropic").unwrap(),
            model_id: ModelId::new(name).unwrap(),
        }
    }

    pub(crate) fn session_fields() -> AgentSessionFields {
        AgentSessionFields {
            session_id: AgentSessionId::new("s1").unwrap(),
            workspace_id: WorkspaceId::new("arch-dock").unwrap(),
            execution_device_id: DeviceId::new("pc").unwrap(),
            checkout: revision("arch-dock", "r7"),
            specification_mode: SpecificationMode::Hybrid,
            specification: Some(SpecRevision {
                workspace_id: WorkspaceId::new("lcl-arch").unwrap(),
                revision: RevisionId::new("s3").unwrap(),
            }),
            model: model("claude-opus-5-5"),
            generation: Generation::FIRST,
            autonomy: AutonomyMode::Balanced,
            orientation_override: None,
        }
    }

    #[test]
    fn a_checkout_of_another_workspace_is_an_invalid_combination() {
        let mut fields = session_fields();
        fields.checkout = revision("other", "r7");
        assert_eq!(
            AgentSession::try_from(fields).unwrap_err(),
            DomainError::new("checkout", ErrorKind::Mismatch)
        );
    }

    #[test]
    fn the_specification_is_bound_exactly_when_the_mode_uses_one() {
        let mut fields = session_fields();
        fields.specification = None;
        assert_eq!(
            AgentSession::try_from(fields.clone()).unwrap_err().kind,
            ErrorKind::Missing
        );
        fields.specification_mode = SpecificationMode::Standard;
        assert!(AgentSession::try_from(fields.clone()).is_ok());
        fields.specification = session_fields().specification;
        assert_eq!(
            AgentSession::try_from(fields).unwrap_err().kind,
            ErrorKind::Unexpected
        );
    }

    #[test]
    fn switching_the_model_keeps_every_binding_and_fences_the_old_generation() {
        let before = AgentSession::try_from(session_fields()).unwrap();
        let after = before.switch_model(model("gpt-5.1")).unwrap();
        let (b, a) = (before.fields(), after.fields());
        assert_eq!(
            (
                &a.workspace_id,
                &a.execution_device_id,
                &a.checkout,
                &a.specification
            ),
            (
                &b.workspace_id,
                &b.execution_device_id,
                &b.checkout,
                &b.specification
            )
        );
        assert_eq!(a.generation.get(), 2);
        assert_eq!(
            after.check_generation(Generation::FIRST).unwrap_err().kind,
            ErrorKind::Stale
        );
        assert!(after.check_generation(a.generation).is_ok());
    }

    #[test]
    fn orientation_is_on_by_default_and_a_session_override_wins() {
        let session = AgentSession::try_from(session_fields()).unwrap();
        assert_eq!(
            session.effective_orientation(OrientationSetting::default()),
            OrientationSetting::On
        );
        assert_eq!(
            session.effective_orientation(OrientationSetting::Off),
            OrientationSetting::Off
        );
        let mut fields = session_fields();
        fields.orientation_override = Some(OrientationSetting::On);
        let session = AgentSession::try_from(fields).unwrap();
        assert_eq!(
            session.effective_orientation(OrientationSetting::Off),
            OrientationSetting::On
        );
    }

    #[test]
    fn sessions_round_trip_and_refuse_smuggled_fields() {
        let session = AgentSession::try_from(session_fields()).unwrap();
        let text = serde_json::to_string(&Versioned(session.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<AgentSession>>(&text)
                .unwrap()
                .0,
            session
        );
        let mut value = serde_json::to_value(&session).unwrap();
        value["foreground_workspace"] = serde_json::json!("lcl-next");
        assert!(serde_json::from_value::<AgentSession>(value).is_err());
    }

    fn transfer(state: TransferState) -> ExecutionTransfer {
        ExecutionTransfer::try_from(ExecutionTransferFields {
            transfer_id: TransferId::new("t1").unwrap(),
            session_id: AgentSessionId::new("s1").unwrap(),
            workspace_id: WorkspaceId::new("arch-dock").unwrap(),
            from_device_id: DeviceId::new("pc").unwrap(),
            to_device_id: DeviceId::new("phone").unwrap(),
            checkpoint: revision("arch-dock", "r7"),
            specification: None,
            state,
            requested_at: Timestamp::from_unix_millis(1),
        })
        .unwrap()
    }

    #[test]
    fn execution_moves_only_after_the_owner_acknowledged_the_checkpoint() {
        let requested = transfer(TransferState::Requested);
        // An unreachable owner never acknowledges, so the transfer cannot jump ahead.
        for skipped in [
            TransferState::Reconciled,
            TransferState::DestinationValidated,
            TransferState::Transferred,
        ] {
            assert_eq!(
                requested.advance(skipped).unwrap_err().kind,
                ErrorKind::InvalidTransition
            );
        }
        let done = requested
            .advance(TransferState::OwnerQuiesced)
            .and_then(|t| t.advance(TransferState::Reconciled))
            .and_then(|t| t.advance(TransferState::DestinationValidated))
            .and_then(|t| t.advance(TransferState::Transferred))
            .unwrap();
        assert_eq!(
            done.advance(TransferState::Requested).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn an_unfinished_transfer_can_be_refused_and_then_stays_refused() {
        let refusal = TransferState::Refused(Blocker {
            kind: BlockerKind::MissingCapability,
            detail: Note::new("The phone has no Git capability.").unwrap(),
        });
        let refused = transfer(TransferState::Reconciled)
            .advance(refusal.clone())
            .unwrap();
        assert_eq!(
            refused
                .advance(TransferState::Transferred)
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
        assert_eq!(
            refused.advance(refusal).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn a_transfer_needs_two_devices_and_a_checkpoint_of_its_workspace() {
        let mut fields = transfer(TransferState::Requested).into_fields();
        fields.to_device_id = fields.from_device_id.clone();
        assert_eq!(
            ExecutionTransfer::try_from(fields.clone())
                .unwrap_err()
                .kind,
            ErrorKind::Duplicate
        );
        fields.to_device_id = DeviceId::new("phone").unwrap();
        fields.checkpoint = revision("other", "r7");
        assert_eq!(
            ExecutionTransfer::try_from(fields).unwrap_err(),
            DomainError::new("checkpoint", ErrorKind::Mismatch)
        );
    }
}
