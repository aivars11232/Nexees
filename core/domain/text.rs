//! Bounded human-readable text: names and labels ([`Label`]) and longer explanations ([`Note`]).
//!
//! Text in a record is data shown to people. It is never an identity, a target or a permission,
//! whatever it says (authority_hierarchy). Both types are bounded so that every record has a
//! known maximum size (RC-23), and neither carries control characters that could hide content
//! in logs or terminals.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{DomainError, ErrorKind};

/// Largest [`Label`], in bytes of UTF-8.
pub const MAX_LABEL_BYTES: usize = 256;
/// Largest [`Note`], in bytes of UTF-8.
pub const MAX_NOTE_BYTES: usize = 16 * 1024;

/// Checks text of at most `max` bytes that is not only whitespace and holds no control
/// character except, when `multiline` is set, line feed and tab.
fn check_text(
    value: &str,
    max: usize,
    multiline: bool,
    field: &'static str,
) -> Result<(), DomainError> {
    if value.trim().is_empty() {
        return Err(DomainError::new(field, ErrorKind::Empty));
    }
    if value.len() > max {
        return Err(DomainError::new(field, ErrorKind::TooLong));
    }
    let allowed = |c: char| !c.is_control() || (multiline && matches!(c, '\n' | '\t'));
    if !value.chars().all(allowed) {
        return Err(DomainError::new(field, ErrorKind::InvalidCharacter));
    }
    Ok(())
}

macro_rules! text_type {
    ($(#[$doc:meta])* $name:ident, $max:expr, $multiline:expr) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("Checks `value` and returns it as a [`", stringify!($name), "`].")]
            pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
                let value = value.into();
                check_text(&value, $max, $multiline, stringify!($name))?;
                Ok(Self(value))
            }

            /// The text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

text_type!(
    /// A one-line name or label of at most [`MAX_LABEL_BYTES`] bytes, such as a workspace or
    /// device name. Two records may share a label; only identifiers tell them apart (TH-47).
    Label,
    MAX_LABEL_BYTES,
    false
);

text_type!(
    /// An explanation of at most [`MAX_NOTE_BYTES`] bytes, which may span lines: an objective, a
    /// summary or the detail of a blocker.
    Note,
    MAX_NOTE_BYTES,
    true
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_one_line_bounded_and_not_blank() {
        assert!(Label::new("Arch Dock").is_ok());
        assert_eq!(Label::new("   ").unwrap_err().kind, ErrorKind::Empty);
        assert_eq!(
            Label::new("two\nlines").unwrap_err().kind,
            ErrorKind::InvalidCharacter
        );
        assert_eq!(
            Label::new("x".repeat(MAX_LABEL_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::TooLong
        );
    }

    #[test]
    fn notes_allow_lines_and_tabs_but_no_other_control_characters() {
        assert!(Note::new("first\n\tsecond").is_ok());
        assert_eq!(
            Note::new("bell\u{7}").unwrap_err().kind,
            ErrorKind::InvalidCharacter
        );
        assert_eq!(
            Note::new("escape\u{1b}[2J").unwrap_err().kind,
            ErrorKind::InvalidCharacter
        );
    }

    #[test]
    fn decoding_applies_the_same_checks_as_construction() {
        assert!(serde_json::from_str::<Label>("\"tab\\there\"").is_err());
        let label: Label = serde_json::from_str("\"LCL — Next Project\"").unwrap();
        assert_eq!(label.as_str(), "LCL — Next Project");
    }
}
