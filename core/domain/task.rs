//! Tasks, and the evidence that decides them (B9, B20, C11, ST-TASK, ST-HISTORY).
//!
//! A task is complete only through [`Task::complete`]. That requires a passing verification of
//! that task, recorded on the device that executes it, against the checkout revision that is
//! current. A model's claim never completes a task. Neither does a copied evidence record or
//! evidence for inputs that have since changed. Evidence records may be copied to another device
//! as portable records; tasks never are, so no device's completion merges into another's.

use serde::{Deserialize, Serialize};

use crate::device::DeviceKind;
use crate::errors::{Blocker, DomainError, ErrorKind};
use crate::ids::{AgentSessionId, DeviceId, EvidenceId, TaskId, WorkspaceId};
use crate::revision::{SpecRevision, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, ensure_unique, validated_record};
use crate::text::{Label, Note};
use crate::time::Timestamp;
use crate::workspace::RelativePath;

/// Where a task stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Not started.
    NotStarted,
    /// Being worked on.
    InProgress,
    /// Cannot continue; the task's blocker says why.
    Blocked,
    /// Complete: its required verification passed (B9).
    Complete,
}

/// The fields of a [`Task`]: the minimum task state of the persistent state model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskFields {
    /// The task.
    pub task_id: TaskId,
    /// The workspace the task changes.
    pub workspace_id: WorkspaceId,
    /// What the task must achieve.
    pub objective: Note,
    /// Where it stands.
    pub status: TaskStatus,
    /// Tasks that must be complete first.
    pub dependencies: Vec<TaskId>,
    /// The files to read, in order, before working on it.
    pub read_order: Vec<RelativePath>,
    /// The verification it requires.
    pub verification_requirements: Vec<Note>,
    /// Decisions that bear on it.
    pub decisions: Vec<EvidenceId>,
    /// Files it changed.
    pub changed_files: Vec<RelativePath>,
    /// Its verification results.
    pub verification_results: Vec<EvidenceId>,
    /// Why it cannot continue; present exactly when it is blocked.
    pub blocker: Option<Blocker>,
    /// The checkpoint its work last reached, a revision of `workspace_id`.
    pub checkpoint: Option<WorkspaceRevision>,
    /// The session working on it.
    pub assigned_session: Option<AgentSessionId>,
}

impl Validate for TaskFields {
    fn validate(&self) -> Result<(), DomainError> {
        if self.dependencies.contains(&self.task_id) {
            return Err(DomainError::new("dependencies", ErrorKind::Mismatch));
        }
        ensure_unique(&self.dependencies, "dependencies")?;
        ensure_unique(&self.read_order, "read_order")?;
        ensure_unique(&self.decisions, "decisions")?;
        ensure_unique(&self.changed_files, "changed_files")?;
        ensure_unique(&self.verification_results, "verification_results")?;
        match (self.status, &self.blocker) {
            (TaskStatus::Blocked, None) => {
                return Err(DomainError::new("blocker", ErrorKind::Missing));
            }
            (TaskStatus::NotStarted | TaskStatus::InProgress | TaskStatus::Complete, Some(_)) => {
                return Err(DomainError::new("blocker", ErrorKind::Unexpected));
            }
            _ => {}
        }
        if self.status == TaskStatus::Complete && self.verification_results.is_empty() {
            return Err(DomainError::new("verification_results", ErrorKind::Missing));
        }
        if let Some(checkpoint) = &self.checkpoint {
            checkpoint.ensure_of(&self.workspace_id, "checkpoint")?;
        }
        Ok(())
    }
}

validated_record!(
    /// A task and its progression (ST-TASK).
    Task,
    TaskFields
);

impl Task {
    /// The task completed by `verification`, its passing verification.
    ///
    /// Only a task in progress can complete. The evidence must be a verification of this task
    /// that passed. It must have been recorded on `execution_host`, the device that executes
    /// the task, so a copied record cannot complete it. And it must be for `current`, the
    /// task's checkout revision now: evidence for changed inputs is stale (B9, B20, C11).
    pub fn complete(
        &self,
        verification: &Evidence,
        execution_host: &DeviceId,
        current: &WorkspaceRevision,
    ) -> Result<Self, DomainError> {
        let evidence = verification.fields();
        if self.0.status != TaskStatus::InProgress {
            return Err(DomainError::new("status", ErrorKind::InvalidTransition));
        }
        if evidence.task_id != self.0.task_id {
            return Err(DomainError::new("evidence.task_id", ErrorKind::Mismatch));
        }
        if evidence.kind != EvidenceKind::Verification || evidence.result != EvidenceResult::Passed
        {
            return Err(DomainError::new("evidence.result", ErrorKind::Mismatch));
        }
        if &evidence.environment.device_id != execution_host {
            return Err(DomainError::new(
                "evidence.environment",
                ErrorKind::Mismatch,
            ));
        }
        current.ensure_of(&self.0.workspace_id, "current")?;
        if &evidence.checkout != current {
            return Err(DomainError::new("evidence.checkout", ErrorKind::Stale));
        }
        let mut fields = self.0.clone();
        fields.status = TaskStatus::Complete;
        if !fields.verification_results.contains(&evidence.evidence_id) {
            fields
                .verification_results
                .push(evidence.evidence_id.clone());
        }
        Self::try_from(fields)
    }
}

impl Record for Task {
    const SCHEMA: &'static str = "nexees.task.task";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// What a piece of evidence records. Besides decisions and verification runs these are the
/// closure records every task keeps: readability, cleanup, manual impact and the final
/// verification after cleanup (code_readability_and_cleanup RC-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// A decision and its reason.
    Decision,
    /// A verification run (C11).
    Verification,
    /// The review of comments and names against the final code.
    Readability,
    /// The scoped cleanup.
    Cleanup,
    /// Whether the manuals changed with the behaviour.
    ManualImpact,
    /// The verification of the final revision after cleanup.
    FinalVerification,
}

/// The result a piece of evidence records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceResult {
    /// Done, and checked where a check applies.
    Passed,
    /// Failed.
    Failed,
    /// Does not apply here; the summary says why.
    NotApplicable,
}

/// The environment evidence was produced in (B20): the device and the tools that ran.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    /// The device.
    pub device_id: DeviceId,
    /// The device's kind.
    pub device_kind: DeviceKind,
    /// The tools and versions that ran, such as `rustc 1.99.0`, each listed once.
    pub tools: Vec<Label>,
}

/// The fields of an [`Evidence`] record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFields {
    /// The evidence.
    pub evidence_id: EvidenceId,
    /// The task it belongs to.
    pub task_id: TaskId,
    /// What it records.
    pub kind: EvidenceKind,
    /// Its result.
    pub result: EvidenceResult,
    /// The checkout revision it was produced against.
    pub checkout: WorkspaceRevision,
    /// The specification revision it was produced against, when one applies.
    pub specification: Option<SpecRevision>,
    /// Where it was produced.
    pub environment: Environment,
    /// When it was recorded.
    pub recorded_at: Timestamp,
    /// What was done and observed.
    pub summary: Note,
}

impl Validate for EvidenceFields {
    fn validate(&self) -> Result<(), DomainError> {
        ensure_unique(&self.environment.tools, "environment.tools")?;
        let allowed = match self.kind {
            // A decision is recorded, not passed or failed.
            EvidenceKind::Decision => self.result == EvidenceResult::NotApplicable,
            // A required check is never "not applicable" (C11).
            EvidenceKind::Verification | EvidenceKind::FinalVerification => {
                self.result != EvidenceResult::NotApplicable
            }
            // An inspected no-op is allowed with its reason in the summary.
            EvidenceKind::Readability | EvidenceKind::Cleanup | EvidenceKind::ManualImpact => true,
        };
        if allowed {
            Ok(())
        } else {
            Err(DomainError::new("result", ErrorKind::Mismatch))
        }
    }
}

validated_record!(
    /// One piece of evidence, tied to the revisions and the environment it was produced under
    /// (B20, ST-HISTORY).
    Evidence,
    EvidenceFields
);

impl Record for Evidence {
    const SCHEMA: &'static str = "nexees.task.evidence";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::PortableRecord;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::BlockerKind;
    use crate::revision::RevisionId;
    use crate::schema::Versioned;

    fn revision(r: &str) -> WorkspaceRevision {
        WorkspaceRevision {
            workspace_id: WorkspaceId::new("arch-dock").unwrap(),
            revision: RevisionId::new(r).unwrap(),
        }
    }

    fn task(status: TaskStatus) -> TaskFields {
        TaskFields {
            task_id: TaskId::new("t1").unwrap(),
            workspace_id: WorkspaceId::new("arch-dock").unwrap(),
            objective: Note::new("Make the parser reject cycles.").unwrap(),
            status,
            dependencies: vec![TaskId::new("t0").unwrap()],
            read_order: vec![RelativePath::new("docs/parser.md").unwrap()],
            verification_requirements: vec![Note::new("cargo test -p parser").unwrap()],
            decisions: vec![],
            changed_files: vec![],
            verification_results: vec![],
            blocker: None,
            checkpoint: Some(revision("r7")),
            assigned_session: Some(AgentSessionId::new("s1").unwrap()),
        }
    }

    fn evidence(kind: EvidenceKind, result: EvidenceResult, device: &str, r: &str) -> Evidence {
        Evidence::try_from(EvidenceFields {
            evidence_id: EvidenceId::new("e1").unwrap(),
            task_id: TaskId::new("t1").unwrap(),
            kind,
            result,
            checkout: revision(r),
            specification: None,
            environment: Environment {
                device_id: DeviceId::new(device).unwrap(),
                device_kind: DeviceKind::Desktop,
                tools: vec![Label::new("rustc 1.99.0").unwrap()],
            },
            recorded_at: Timestamp::from_unix_millis(10),
            summary: Note::new("12 tests passed.").unwrap(),
        })
        .unwrap()
    }

    #[test]
    fn a_passing_verification_on_the_execution_host_completes_the_task() {
        let running = Task::try_from(task(TaskStatus::InProgress)).unwrap();
        let passed = evidence(
            EvidenceKind::Verification,
            EvidenceResult::Passed,
            "pc",
            "r8",
        );
        let done = running
            .complete(&passed, &DeviceId::new("pc").unwrap(), &revision("r8"))
            .unwrap();
        assert_eq!(done.fields().status, TaskStatus::Complete);
        assert_eq!(
            done.fields().verification_results,
            [EvidenceId::new("e1").unwrap()]
        );
    }

    #[test]
    fn claims_copies_failures_and_stale_evidence_never_complete_a_task() {
        let running = Task::try_from(task(TaskStatus::InProgress)).unwrap();
        let pc = DeviceId::new("pc").unwrap();
        let cases = [
            (
                evidence(
                    EvidenceKind::Verification,
                    EvidenceResult::Failed,
                    "pc",
                    "r8",
                ),
                revision("r8"),
                "evidence.result",
                ErrorKind::Mismatch,
            ),
            (
                evidence(
                    EvidenceKind::Readability,
                    EvidenceResult::Passed,
                    "pc",
                    "r8",
                ),
                revision("r8"),
                "evidence.result",
                ErrorKind::Mismatch,
            ),
            // Recorded on another device and copied here.
            (
                evidence(
                    EvidenceKind::Verification,
                    EvidenceResult::Passed,
                    "phone",
                    "r8",
                ),
                revision("r8"),
                "evidence.environment",
                ErrorKind::Mismatch,
            ),
            // The inputs changed after the verification ran.
            (
                evidence(
                    EvidenceKind::Verification,
                    EvidenceResult::Passed,
                    "pc",
                    "r8",
                ),
                revision("r9"),
                "evidence.checkout",
                ErrorKind::Stale,
            ),
        ];
        for (evidence, current, field, kind) in cases {
            assert_eq!(
                running.complete(&evidence, &pc, &current).unwrap_err(),
                DomainError::new(field, kind)
            );
        }
        let not_started = Task::try_from(task(TaskStatus::NotStarted)).unwrap();
        let passed = evidence(
            EvidenceKind::Verification,
            EvidenceResult::Passed,
            "pc",
            "r8",
        );
        assert_eq!(
            not_started
                .complete(&passed, &pc, &revision("r8"))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn task_records_keep_their_status_consistent() {
        let mut complete = task(TaskStatus::Complete);
        assert_eq!(
            Task::try_from(complete.clone()).unwrap_err(),
            DomainError::new("verification_results", ErrorKind::Missing)
        );
        complete
            .verification_results
            .push(EvidenceId::new("e1").unwrap());
        assert!(Task::try_from(complete).is_ok());

        let mut blocked = task(TaskStatus::Blocked);
        assert_eq!(
            Task::try_from(blocked.clone()).unwrap_err().kind,
            ErrorKind::Missing
        );
        blocked.blocker = Some(Blocker {
            kind: BlockerKind::MissingCapability,
            detail: Note::new("No Git on the phone.").unwrap(),
        });
        assert!(Task::try_from(blocked).is_ok());

        let mut own = task(TaskStatus::NotStarted);
        own.dependencies.push(own.task_id.clone());
        assert_eq!(Task::try_from(own).unwrap_err().kind, ErrorKind::Mismatch);
        let mut elsewhere = task(TaskStatus::NotStarted);
        elsewhere.checkpoint = Some(WorkspaceRevision {
            workspace_id: WorkspaceId::new("other").unwrap(),
            revision: RevisionId::new("r").unwrap(),
        });
        assert_eq!(
            Task::try_from(elsewhere).unwrap_err(),
            DomainError::new("checkpoint", ErrorKind::Mismatch)
        );
    }

    #[test]
    fn required_checks_are_never_not_applicable_but_inspected_no_ops_may_be() {
        let fields = evidence(
            EvidenceKind::Cleanup,
            EvidenceResult::NotApplicable,
            "pc",
            "r8",
        )
        .into_fields();
        for (kind, result, ok) in [
            (
                EvidenceKind::Verification,
                EvidenceResult::NotApplicable,
                false,
            ),
            (
                EvidenceKind::FinalVerification,
                EvidenceResult::NotApplicable,
                false,
            ),
            (EvidenceKind::Decision, EvidenceResult::Passed, false),
            (EvidenceKind::Decision, EvidenceResult::NotApplicable, true),
            (
                EvidenceKind::ManualImpact,
                EvidenceResult::NotApplicable,
                true,
            ),
        ] {
            let attempt = Evidence::try_from(EvidenceFields {
                kind,
                result,
                ..fields.clone()
            });
            assert_eq!(attempt.is_ok(), ok, "{kind:?} {result:?}");
        }
    }

    #[test]
    fn evidence_round_trips_as_a_portable_record() {
        let record = evidence(
            EvidenceKind::FinalVerification,
            EvidenceResult::Passed,
            "pc",
            "r8",
        );
        let text = serde_json::to_string(&Versioned(record.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<Evidence>>(&text)
                .unwrap()
                .0,
            record
        );
        assert!(<Evidence as Record>::SYNC.travels());
        assert!(!<Task as Record>::SYNC.travels());
    }
}
