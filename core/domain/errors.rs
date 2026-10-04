//! The shared error and outcome vocabulary of the domain.
//!
//! [`DomainError`] is what validation returns. Every constructor and every decoder in this crate
//! reports an invalid value through it instead of panicking, so a malformed input fails closed.
//! [`Outcome`] is the closed set of outcomes of a remote request (RC-12), and [`Blocker`] says
//! why a task, an import, a transfer or a handoff cannot continue.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::text::Note;

/// Why a value was rejected: the field it concerns and the kind of problem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainError {
    /// The field, type or rule the problem concerns, such as `workspace_id` or `RelativePath`.
    pub field: &'static str,
    /// What is wrong with it.
    pub kind: ErrorKind,
}

impl DomainError {
    /// A problem of `kind` with `field`.
    pub const fn new(field: &'static str, kind: ErrorKind) -> Self {
        Self { field, kind }
    }
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.kind.describe())
    }
}

impl std::error::Error for DomainError {}

/// The kinds of problem validation reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// A value that must not be empty is empty.
    Empty,
    /// A value is longer than its bound.
    TooLong,
    /// A value contains a character its type does not allow.
    InvalidCharacter,
    /// A value does not have the shape its type requires.
    InvalidFormat,
    /// A number is outside its range.
    OutOfRange,
    /// Fields that must agree do not, such as a revision of another workspace.
    Mismatch,
    /// A field required in this combination is absent.
    Missing,
    /// A field that must be absent in this combination is present.
    Unexpected,
    /// An entry occurs twice where entries must be unique.
    Duplicate,
    /// A state change that the lifecycle does not allow.
    InvalidTransition,
    /// A revision, generation, epoch or approval is not the current one.
    Stale,
    /// A request, grant or approval is past its expiry.
    Expired,
    /// A record has another schema name or an unsupported schema version.
    UnsupportedSchema,
}

impl ErrorKind {
    const fn describe(self) -> &'static str {
        match self {
            Self::Empty => "must not be empty",
            Self::TooLong => "is longer than allowed",
            Self::InvalidCharacter => "contains a character that is not allowed",
            Self::InvalidFormat => "does not have the required format",
            Self::OutOfRange => "is out of range",
            Self::Mismatch => "does not match the value it must agree with",
            Self::Missing => "is required here",
            Self::Unexpected => "must be absent here",
            Self::Duplicate => "contains a duplicate",
            Self::InvalidTransition => "cannot change to that state",
            Self::Stale => "is not current",
            Self::Expired => "has expired",
            Self::UnsupportedSchema => "has an unsupported schema name or version",
        }
    }
}

/// The explicit outcome of a remote request (RC-12, `binding.sec_outcomes`).
///
/// The serialized names are exactly those of the binding. A delivered request, a started process
/// or an exit code is not a [`Outcome::Completed`]; a lost acknowledgment is
/// [`Outcome::OutcomeUnknown`] until it is reconciled, never a failure (RC-17).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Outcome {
    /// Admitted by the destination; not yet running.
    Accepted,
    /// Running on the destination.
    Running,
    /// Finished, with evidence of the effect.
    Completed,
    /// Refused by the destination's checks.
    Denied,
    /// Not available on the destination's platform or configuration.
    Unsupported,
    /// Waiting for the destination's user, such as a confirmation the OS requires (RC-16).
    NeedsUserAction,
    /// Past its expiry; never executed later.
    Expired,
    /// The effect may or may not have happened; it must be reconciled before anything else.
    OutcomeUnknown,
}

/// Why something cannot continue, with a precise kind and a short explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Blocker {
    /// The kind of blocker.
    pub kind: BlockerKind,
    /// What exactly blocks, for the user and the evidence.
    pub detail: Note,
}

/// The kinds of blocker. A blocker is reported, never worked around (C11, CA-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockerKind {
    /// A required tool, toolchain or model capability is not available (A4, HO-08).
    MissingCapability,
    /// A mandatory file has not been read, or changed after it was read (HO-06).
    MissingRead,
    /// A revision, binding, generation, epoch or approval is no longer current.
    StaleState,
    /// The bound LCL revision is not validated, or is missing a required reference (I4, HO-06).
    InvalidSpecification,
    /// A provider quota or a context, token, read or time limit was reached (HO-07).
    LimitReached,
    /// A provider, host or peer cannot be reached.
    Unreachable,
    /// An effect's outcome must be reconciled before anything else happens (RC-17, H2).
    OutcomeUnknown,
    /// The user must decide or confirm.
    NeedsUserAction,
    /// A required check failed (C11).
    VerificationFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_serialize_with_the_exact_names_of_the_security_binding() {
        let names: Vec<String> = [
            Outcome::Accepted,
            Outcome::Running,
            Outcome::Completed,
            Outcome::Denied,
            Outcome::Unsupported,
            Outcome::NeedsUserAction,
            Outcome::Expired,
            Outcome::OutcomeUnknown,
        ]
        .iter()
        .map(|o| serde_json::to_string(o).unwrap())
        .collect();
        assert_eq!(
            names,
            [
                "\"accepted\"",
                "\"running\"",
                "\"completed\"",
                "\"denied\"",
                "\"unsupported\"",
                "\"needs-user-action\"",
                "\"expired\"",
                "\"outcome-unknown\""
            ]
        );
    }

    #[test]
    fn an_unknown_outcome_is_rejected_rather_than_guessed() {
        assert!(serde_json::from_str::<Outcome>("\"done\"").is_err());
    }

    #[test]
    fn errors_name_the_field_and_the_problem() {
        let error = DomainError::new("workspace_id", ErrorKind::Mismatch);
        assert_eq!(
            error.to_string(),
            "workspace_id: does not match the value it must agree with"
        );
    }
}
