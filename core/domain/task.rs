//! Tasks, and the evidence that decides them (B9, B20, B24, C11, ST-TASK, ST-HISTORY).
//!
//! A task becomes complete only through [`Task::complete`], with a [`HostVerification`]: the final
//! verification after cleanup, which the device executing the task ran itself and which passed
//! against the checkout revision that is current.
//!
//! An [`Evidence`] record is a claim. It may travel to other devices as a portable record, and
//! whatever device and revision it names, the record alone proves neither. So a record never
//! completes a task: not when a model declared it, a peer synced it, it was copied or imported,
//! or it was read back from storage. Neither does a verification from before the cleanup, nor
//! evidence for inputs that have since changed. Tasks never travel, so no device's completion
//! merges into another's.
//!
//! The gates of `core/tasking` decide when that final verification runs. The verification gate
//! (TASK-054) runs the task's configured checks itself, and the cleanup gate first requires the
//! readability review and the scoped cleanup (FL-TASK-CLOSURE). This module refuses what a record
//! or a single run cannot prove.

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
    /// The task completed by its final verification.
    ///
    /// `verification` is the final verification after cleanup that `execution_host`, the device
    /// executing the task, ran itself, and `evidence` is the record the task will cite. Only a
    /// task in progress can complete, and:
    ///
    /// - the verification must be the execution host's own run;
    /// - `evidence` must be exactly the record that run made. A record that only claims the
    ///   host, the task and the revision is refused, even while the host holds a passing run;
    /// - it must verify this task after the cleanup, and have passed (B24);
    /// - it must be for `current`, the task's checkout revision now: a verification of changed
    ///   inputs is stale (B9, B20, C11).
    pub fn complete(
        &self,
        evidence: &Evidence,
        verification: &HostVerification,
        execution_host: &DeviceId,
        current: &WorkspaceRevision,
    ) -> Result<Self, DomainError> {
        if self.0.status != TaskStatus::InProgress {
            return Err(DomainError::new("status", ErrorKind::InvalidTransition));
        }
        if verification.host() != execution_host {
            return Err(DomainError::new("verification.host", ErrorKind::Mismatch));
        }
        if evidence != verification.evidence() {
            return Err(DomainError::new("evidence", ErrorKind::Mismatch));
        }
        let evidence = evidence.fields();
        if evidence.task_id != self.0.task_id {
            return Err(DomainError::new("evidence.task_id", ErrorKind::Mismatch));
        }
        if evidence.kind != EvidenceKind::FinalVerification {
            return Err(DomainError::new("evidence.kind", ErrorKind::Mismatch));
        }
        if evidence.result != EvidenceResult::Passed {
            return Err(DomainError::new("evidence.result", ErrorKind::Mismatch));
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
    /// A verification run (C11), such as the one before the cleanup. It never completes a task.
    Verification,
    /// The review of comments and names against the final code.
    Readability,
    /// The scoped cleanup.
    Cleanup,
    /// Whether the manuals changed with the behaviour.
    ManualImpact,
    /// The verification of the final revision after cleanup: the only kind that completes a
    /// task, and only as the execution host's own run ([`HostVerification`]).
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
    /// One piece of evidence, tied to the revisions and the environment it names (B20,
    /// ST-HISTORY). As a record it is a claim: only a [`HostVerification`] completes a task.
    Evidence,
    EvidenceFields
);

impl Record for Evidence {
    const SCHEMA: &'static str = "nexees.task.evidence";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::PortableRecord;
}

/// A verification the execution host ran itself, and the evidence record it made of the run: the
/// only proof that completes a task (C11, B24).
///
/// It exists only as a value in the memory of the host that ran the checks. It has no serialized
/// form, so it is never stored, sent or synced, and nothing received, imported or read back
/// decodes as one. A record, by contrast, decodes from text received from anywhere, and is only a
/// claim:
///
/// ```
/// use nexees_domain::task::{Evidence, HostVerification};
///
/// // A record decodes from text, as a claim; this one is refused as incomplete.
/// assert!(serde_json::from_str::<Evidence>("{}").is_err());
/// // A host verification is only ever made in memory.
/// assert!(std::mem::size_of::<HostVerification>() > 0);
/// ```
///
/// ```compile_fail
/// use nexees_domain::task::{Evidence, HostVerification};
///
/// assert!(serde_json::from_str::<Evidence>("{}").is_err());
/// // Nothing decodes as a host verification: it has no serialized form.
/// assert!(serde_json::from_str::<HostVerification>("{}").is_err());
/// ```
///
/// The execution host's verification gate (`core/tasking/verification_gate`, TASK-054) makes one
/// with [`HostVerification::new`], from checks it runs itself on the checkout it has itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostVerification(Evidence);

impl HostVerification {
    /// The verification `host`, this device, has just run itself, with `record` the evidence it
    /// made of the run.
    ///
    /// Only the host's verification gate calls this, with what it observed itself: never with
    /// the fields of a record that it received, read back or was told about. The record must
    /// name `host` and be a verification run.
    pub fn new(host: &DeviceId, record: EvidenceFields) -> Result<Self, DomainError> {
        if &record.environment.device_id != host {
            return Err(DomainError::new(
                "environment.device_id",
                ErrorKind::Mismatch,
            ));
        }
        if !matches!(
            record.kind,
            EvidenceKind::Verification | EvidenceKind::FinalVerification
        ) {
            return Err(DomainError::new("kind", ErrorKind::Mismatch));
        }
        Evidence::try_from(record).map(Self)
    }

    /// The device that ran it.
    pub fn host(&self) -> &DeviceId {
        &self.0.fields().environment.device_id
    }

    /// The evidence record it made: the one to store, and the only one a task may cite.
    pub fn evidence(&self) -> &Evidence {
        &self.0
    }
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

    /// A verification that `device` ran itself, of `t1` against `r`.
    fn run(device: &str, kind: EvidenceKind, result: EvidenceResult, r: &str) -> HostVerification {
        let record = evidence(kind, result, device, r).into_fields();
        HostVerification::new(&DeviceId::new(device).unwrap(), record).unwrap()
    }

    #[test]
    fn the_execution_hosts_own_final_verification_completes_the_task() {
        let running = Task::try_from(task(TaskStatus::InProgress)).unwrap();
        let final_run = run(
            "pc",
            EvidenceKind::FinalVerification,
            EvidenceResult::Passed,
            "r8",
        );
        let pc = DeviceId::new("pc").unwrap();
        let done = running
            .complete(final_run.evidence(), &final_run, &pc, &revision("r8"))
            .unwrap();
        assert_eq!(done.fields().status, TaskStatus::Complete);
        assert_eq!(
            done.fields().verification_results,
            [EvidenceId::new("e1").unwrap()]
        );
    }

    #[test]
    fn a_record_that_claims_the_host_and_revision_never_completes_a_task() {
        // The execution host holds its own passing final verification of t1 against r8. Every
        // record below names that host, task, revision and result as well, and passes; none is
        // the record the host made. Without a host verification at all a record has no way in:
        // `complete` needs one, and nothing decodes as one (see `HostVerification`).
        let running = Task::try_from(task(TaskStatus::InProgress)).unwrap();
        let pc = DeviceId::new("pc").unwrap();
        let final_run = run(
            "pc",
            EvidenceKind::FinalVerification,
            EvidenceResult::Passed,
            "r8",
        );
        let genuine = final_run.evidence().clone().into_fields();
        // Fabricated: a passing record of a run that never happened.
        let fabricated = EvidenceFields {
            evidence_id: EvidenceId::new("e9").unwrap(),
            summary: Note::new("All 12 checks passed.").unwrap(),
            ..genuine.clone()
        };
        // Self-attested: a model's own report, filed under the host's name.
        let self_attested = EvidenceFields {
            summary: Note::new("I ran the tests myself and they all pass.").unwrap(),
            ..genuine.clone()
        };
        // Copied: another device's passing run, edited to name this host.
        let copied = EvidenceFields {
            environment: Environment {
                device_id: pc.clone(),
                device_kind: DeviceKind::Android,
                tools: vec![Label::new("rustc 1.98.0").unwrap()],
            },
            ..genuine.clone()
        };
        // Synced: the host's own record as a peer sent it back, changed on the way.
        let synced = {
            let changed = Evidence::try_from(EvidenceFields {
                recorded_at: Timestamp::from_unix_millis(11),
                ..genuine.clone()
            })
            .unwrap();
            let text = serde_json::to_string(&Versioned(changed)).unwrap();
            serde_json::from_str::<Versioned<Evidence>>(&text)
                .unwrap()
                .0
                .into_fields()
        };
        for claim in [fabricated, self_attested, copied, synced] {
            let claim = Evidence::try_from(claim).unwrap();
            assert_eq!(
                running
                    .complete(&claim, &final_run, &pc, &revision("r8"))
                    .unwrap_err(),
                DomainError::new("evidence", ErrorKind::Mismatch)
            );
        }
    }

    #[test]
    fn only_the_hosts_passing_final_verification_of_the_current_revision_completes_a_task() {
        let running = Task::try_from(task(TaskStatus::InProgress)).unwrap();
        let pc = DeviceId::new("pc").unwrap();
        let (final_verification, passed) =
            (EvidenceKind::FinalVerification, EvidenceResult::Passed);
        let cases = [
            // The phone ran it, on its own replica: that proves nothing about the PC's.
            (
                run("phone", final_verification, passed, "r8"),
                revision("r8"),
                "verification.host",
                ErrorKind::Mismatch,
            ),
            // The verification before the cleanup (B24).
            (
                run("pc", EvidenceKind::Verification, passed, "r8"),
                revision("r8"),
                "evidence.kind",
                ErrorKind::Mismatch,
            ),
            // The verification after the cleanup failed.
            (
                run("pc", final_verification, EvidenceResult::Failed, "r8"),
                revision("r8"),
                "evidence.result",
                ErrorKind::Mismatch,
            ),
            // The inputs changed after it ran.
            (
                run("pc", final_verification, passed, "r8"),
                revision("r9"),
                "evidence.checkout",
                ErrorKind::Stale,
            ),
        ];
        for (verification, current, field, kind) in cases {
            assert_eq!(
                running
                    .complete(verification.evidence(), &verification, &pc, &current)
                    .unwrap_err(),
                DomainError::new(field, kind)
            );
        }
        let other_task = HostVerification::new(
            &pc,
            EvidenceFields {
                task_id: TaskId::new("t2").unwrap(),
                ..evidence(final_verification, passed, "pc", "r8").into_fields()
            },
        )
        .unwrap();
        assert_eq!(
            running
                .complete(other_task.evidence(), &other_task, &pc, &revision("r8"))
                .unwrap_err(),
            DomainError::new("evidence.task_id", ErrorKind::Mismatch)
        );
        let not_started = Task::try_from(task(TaskStatus::NotStarted)).unwrap();
        let final_run = run("pc", final_verification, passed, "r8");
        assert_eq!(
            not_started
                .complete(final_run.evidence(), &final_run, &pc, &revision("r8"))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn a_host_verification_is_a_verification_run_that_names_its_host() {
        let pc = DeviceId::new("pc").unwrap();
        let passed = EvidenceResult::Passed;
        let cases = [
            (
                evidence(EvidenceKind::FinalVerification, passed, "phone", "r8"),
                "environment.device_id",
            ),
            (
                evidence(EvidenceKind::Readability, passed, "pc", "r8"),
                "kind",
            ),
        ];
        for (record, field) in cases {
            assert_eq!(
                HostVerification::new(&pc, record.into_fields()).unwrap_err(),
                DomainError::new(field, ErrorKind::Mismatch)
            );
        }
        // Its record is a valid one: a required check is never "not applicable".
        let mut record =
            evidence(EvidenceKind::FinalVerification, passed, "pc", "r8").into_fields();
        record.result = EvidenceResult::NotApplicable;
        assert_eq!(
            HostVerification::new(&pc, record).unwrap_err(),
            DomainError::new("result", ErrorKind::Mismatch)
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
