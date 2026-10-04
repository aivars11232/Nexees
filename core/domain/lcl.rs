//! LCL specification state: the specification mode of a session, and each revision of a
//! specification from draft to adoption (B7, C22, I4, ST-LCL-REVISION).
//!
//! Every revision starts as a draft, whether it was edited here, synchronized from another
//! device or imported. Only a validation that passed makes it [`SpecRevisionState::Validated`].
//! Only a validated revision, adopted explicitly at a safe boundary, constrains a session
//! ([`Adoption`]); a draft or an invalid revision never does. Validation status and adoptions
//! never travel: a synchronized specification arrives as content and becomes a new draft on the
//! receiving device.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{AgentSessionId, UserId};
use crate::revision::SpecRevision;
use crate::schema::{Record, SyncPolicy, Validate, validated_record};
use crate::text::Label;
use crate::time::Timestamp;

/// How a session uses LCL (lcl_integration). Independent of the autonomy mode (TASK-048).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecificationMode {
    /// No LCL.
    Standard,
    /// The whole project specification is authoritative.
    FullProject,
    /// Ordered tasks are the authoritative execution units.
    TaskDecomposed,
    /// Master architecture, rules, contracts and invariants, plus ordered tasks.
    Hybrid,
}

impl SpecificationMode {
    /// Whether a session in this mode is bound to a specification revision.
    pub const fn uses_specification(self) -> bool {
        !matches!(self, Self::Standard)
    }
}

/// Where a specification revision came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionOrigin {
    /// Edited on this device.
    Edited,
    /// Received from another device by synchronization.
    Synced,
    /// Imported from files or an archive.
    Imported,
}

/// The validation state of a specification revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecRevisionState {
    /// Not validated yet; it may be edited, never executed.
    Draft,
    /// Validated by the pinned LCL core; it may be adopted.
    Validated,
    /// Failed validation; it may be kept as an inactive draft, never executed (I4).
    Invalid,
}

/// The result of validating a revision with the pinned LCL core (IF-LCL).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationSummary {
    /// Whether the revision passed.
    pub passed: bool,
    /// The LCL core version that validated it.
    pub core_version: Label,
    /// The number of diagnostics reported.
    pub diagnostics: u32,
    /// When it was validated.
    pub validated_at: Timestamp,
}

/// The fields of a [`SpecRevisionRecord`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecRevisionRecordFields {
    /// The revision.
    pub revision: SpecRevision,
    /// Where it came from.
    pub origin: RevisionOrigin,
    /// Its validation state.
    pub state: SpecRevisionState,
    /// The validation result: absent for a draft, passed for a validated revision and failed
    /// for an invalid one.
    pub validation: Option<ValidationSummary>,
    /// When the revision was created.
    pub created_at: Timestamp,
}

impl Validate for SpecRevisionRecordFields {
    fn validate(&self) -> Result<(), DomainError> {
        match (self.state, &self.validation) {
            (SpecRevisionState::Draft, None) => Ok(()),
            (SpecRevisionState::Draft, Some(_)) => {
                Err(DomainError::new("validation", ErrorKind::Unexpected))
            }
            (SpecRevisionState::Validated | SpecRevisionState::Invalid, None) => {
                Err(DomainError::new("validation", ErrorKind::Missing))
            }
            (state, Some(summary)) => {
                if summary.passed == (state == SpecRevisionState::Validated) {
                    Ok(())
                } else {
                    Err(DomainError::new("validation", ErrorKind::Mismatch))
                }
            }
        }
    }
}

validated_record!(
    /// One revision of a specification with its validation state (ST-LCL-REVISION).
    SpecRevisionRecord,
    SpecRevisionRecordFields
);

impl SpecRevisionRecord {
    /// A new revision. It is always a draft, whatever its origin.
    pub fn draft(revision: SpecRevision, origin: RevisionOrigin, created_at: Timestamp) -> Self {
        Self(SpecRevisionRecordFields {
            revision,
            origin,
            state: SpecRevisionState::Draft,
            validation: None,
            created_at,
        })
    }

    /// The draft with the result of its validation: validated if it passed, invalid if not.
    /// Only a draft can be validated; a changed specification is a new revision and a new draft.
    pub fn with_validation(self, summary: ValidationSummary) -> Result<Self, DomainError> {
        if self.0.state != SpecRevisionState::Draft {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        let state = if summary.passed {
            SpecRevisionState::Validated
        } else {
            SpecRevisionState::Invalid
        };
        Ok(Self(SpecRevisionRecordFields {
            state,
            validation: Some(summary),
            ..self.0
        }))
    }
}

impl Record for SpecRevisionRecord {
    const SCHEMA: &'static str = "nexees.lcl.spec_revision";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The fields of an [`Adoption`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionFields {
    /// The session that now executes under the revision.
    pub session_id: AgentSessionId,
    /// The adopted revision.
    pub revision: SpecRevision,
    /// The passing validation it was adopted on.
    pub validation: ValidationSummary,
    /// The user who adopted it.
    pub adopted_by: UserId,
    /// When it was adopted.
    pub adopted_at: Timestamp,
}

impl Validate for AdoptionFields {
    fn validate(&self) -> Result<(), DomainError> {
        if self.validation.passed {
            Ok(())
        } else {
            Err(DomainError::new("validation", ErrorKind::Mismatch))
        }
    }
}

validated_record!(
    /// An explicit adoption of a validated revision by a session at a safe boundary (B7, C22).
    /// Storing one is checked against the revision's own record by its owner (TASK-050).
    Adoption,
    AdoptionFields
);

impl Adoption {
    /// Adopts `record` for `session_id`. Fails unless the revision is validated.
    pub fn adopt(
        session_id: AgentSessionId,
        record: &SpecRevisionRecord,
        adopted_by: UserId,
        adopted_at: Timestamp,
    ) -> Result<Self, DomainError> {
        let fields = record.fields();
        match (&fields.state, &fields.validation) {
            (SpecRevisionState::Validated, Some(validation)) => Self::try_from(AdoptionFields {
                session_id,
                revision: fields.revision.clone(),
                validation: validation.clone(),
                adopted_by,
                adopted_at,
            }),
            _ => Err(DomainError::new("state", ErrorKind::InvalidTransition)),
        }
    }
}

impl Record for Adoption {
    const SCHEMA: &'static str = "nexees.lcl.adoption";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::WorkspaceId;
    use crate::revision::RevisionId;

    fn revision() -> SpecRevision {
        SpecRevision {
            workspace_id: WorkspaceId::new("lcl").unwrap(),
            revision: RevisionId::new("s2").unwrap(),
        }
    }

    fn summary(passed: bool) -> ValidationSummary {
        ValidationSummary {
            passed,
            core_version: Label::new("0.3.0").unwrap(),
            diagnostics: u32::from(!passed),
            validated_at: Timestamp::from_unix_millis(5),
        }
    }

    fn adopt(record: &SpecRevisionRecord) -> Result<Adoption, DomainError> {
        Adoption::adopt(
            AgentSessionId::new("s").unwrap(),
            record,
            UserId::new("u").unwrap(),
            Timestamp::from_unix_millis(9),
        )
    }

    #[test]
    fn every_new_revision_is_a_draft_whatever_its_origin() {
        for origin in [
            RevisionOrigin::Edited,
            RevisionOrigin::Synced,
            RevisionOrigin::Imported,
        ] {
            let record =
                SpecRevisionRecord::draft(revision(), origin, Timestamp::from_unix_millis(1));
            assert_eq!(record.fields().state, SpecRevisionState::Draft);
            assert_eq!(
                adopt(&record).unwrap_err().kind,
                ErrorKind::InvalidTransition
            );
        }
    }

    #[test]
    fn only_a_passing_validation_makes_a_revision_adoptable() {
        let draft = SpecRevisionRecord::draft(
            revision(),
            RevisionOrigin::Imported,
            Timestamp::from_unix_millis(1),
        );
        let invalid = draft.clone().with_validation(summary(false)).unwrap();
        assert_eq!(invalid.fields().state, SpecRevisionState::Invalid);
        assert_eq!(
            adopt(&invalid).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );

        let validated = draft.with_validation(summary(true)).unwrap();
        assert_eq!(validated.fields().state, SpecRevisionState::Validated);
        let adoption = adopt(&validated).unwrap();
        assert_eq!(adoption.fields().revision, revision());
        // A validated revision is not validated again; a change is a new draft.
        assert_eq!(
            validated.with_validation(summary(true)).unwrap_err().kind,
            ErrorKind::InvalidTransition
        );
    }

    #[test]
    fn decoded_records_must_agree_with_their_state() {
        let mut fields = SpecRevisionRecord::draft(
            revision(),
            RevisionOrigin::Synced,
            Timestamp::from_unix_millis(1),
        )
        .into_fields();
        fields.state = SpecRevisionState::Validated;
        assert_eq!(
            SpecRevisionRecord::try_from(fields.clone())
                .unwrap_err()
                .kind,
            ErrorKind::Missing
        );
        fields.validation = Some(summary(false));
        assert_eq!(
            SpecRevisionRecord::try_from(fields).unwrap_err().kind,
            ErrorKind::Mismatch
        );

        let json = concat!(
            r#"{"session_id":"s","revision":{"workspace_id":"lcl","revision":"s2"},"#,
            r#""validation":{"passed":false,"core_version":"0.3.0","diagnostics":1,"#,
            r#""validated_at":5},"adopted_by":"u","adopted_at":9}"#
        );
        assert!(serde_json::from_str::<Adoption>(json).is_err());
    }

    #[test]
    fn standard_mode_is_the_only_mode_without_a_specification() {
        assert!(!SpecificationMode::Standard.uses_specification());
        for mode in [
            SpecificationMode::FullProject,
            SpecificationMode::TaskDecomposed,
            SpecificationMode::Hybrid,
        ] {
            assert!(mode.uses_specification());
        }
    }
}
