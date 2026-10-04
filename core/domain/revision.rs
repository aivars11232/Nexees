//! Which version of something a record refers to: content hashes, revisions, session
//! generations and permission epochs.
//!
//! A revision always names the workspace it belongs to. That makes a combination which mixes
//! revisions of different workspaces detectable, and every record of this crate rejects one
//! (C21, B17, B20).

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{WorkspaceId, check_id, identifier};

/// The SHA-256 digest of some content, as 64 lowercase hexadecimal characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentHash(String);

impl ContentHash {
    /// Checks `value` and returns it as a [`ContentHash`].
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.len() != 64 {
            return Err(DomainError::new("ContentHash", ErrorKind::InvalidFormat));
        }
        if !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(DomainError::new("ContentHash", ErrorKind::InvalidCharacter));
        }
        Ok(Self(value))
    }

    /// The digest as hexadecimal text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for ContentHash {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ContentHash {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

identifier!(
    /// A revision as the revision service of its workspace issues it (IF-REVISION). It means
    /// nothing outside that workspace, which is why it travels inside [`WorkspaceRevision`] or
    /// [`SpecRevision`].
    RevisionId,
    check_id
);

/// A revision of a workspace's content: a checkout or a working tree, uncommitted edits
/// included (H2 step 3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRevision {
    /// The workspace the revision belongs to.
    pub workspace_id: WorkspaceId,
    /// The revision within that workspace.
    pub revision: RevisionId,
}

impl WorkspaceRevision {
    /// Fails with [`ErrorKind::Mismatch`] on `field` unless this revision belongs to `workspace`.
    pub fn ensure_of(
        &self,
        workspace: &WorkspaceId,
        field: &'static str,
    ) -> Result<(), DomainError> {
        if &self.workspace_id == workspace {
            Ok(())
        } else {
            Err(DomainError::new(field, ErrorKind::Mismatch))
        }
    }
}

/// A revision of an LCL specification, with the workspace that holds the specification. That
/// workspace may differ from the one a session works on.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecRevision {
    /// The workspace that holds the specification.
    pub workspace_id: WorkspaceId,
    /// The revision of the specification within that workspace.
    pub revision: RevisionId,
}

/// The generation of an agent session. It starts at 1 and increases with every model switch, so
/// that a delayed reply from an older generation can be recognized and refused (H2 step 1, B23).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u64", into = "u64")]
pub struct Generation(u64);

impl Generation {
    /// The generation of a new session.
    pub const FIRST: Self = Self(1);

    /// Checks `value`, which must be at least 1, and returns it as a [`Generation`].
    pub const fn new(value: u64) -> Result<Self, DomainError> {
        if value == 0 {
            Err(DomainError::new("Generation", ErrorKind::OutOfRange))
        } else {
            Ok(Self(value))
        }
    }

    /// The number.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The following generation; fails rather than wrapping around.
    pub const fn next(self) -> Result<Self, DomainError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(DomainError::new("Generation", ErrorKind::OutOfRange)),
        }
    }
}

impl TryFrom<u64> for Generation {
    type Error = DomainError;

    fn try_from(value: u64) -> Result<Self, DomainError> {
        Self::new(value)
    }
}

impl From<Generation> for u64 {
    fn from(generation: Generation) -> u64 {
        generation.0
    }
}

/// The permission epoch of a destination device. It increases on every revoke, stop, scope or
/// lock-policy change, which voids every request and approval made under an older epoch
/// (`type.sec_grant`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionEpoch(u64);

impl PermissionEpoch {
    /// The epoch `value`.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The number.
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The following epoch; fails rather than wrapping around.
    pub const fn next(self) -> Result<Self, DomainError> {
        match self.0.checked_add(1) {
            Some(value) => Ok(Self(value)),
            None => Err(DomainError::new("PermissionEpoch", ErrorKind::OutOfRange)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hashes_are_lowercase_sha256_hex() {
        let hex = "ab".repeat(32);
        assert!(ContentHash::new(hex.clone()).is_ok());
        assert_eq!(
            ContentHash::new("ab").unwrap_err().kind,
            ErrorKind::InvalidFormat
        );
        assert_eq!(
            ContentHash::new(hex.to_uppercase()).unwrap_err().kind,
            ErrorKind::InvalidCharacter
        );
        assert!(serde_json::from_str::<ContentHash>("\"0g\"").is_err());
    }

    #[test]
    fn a_revision_belongs_to_exactly_one_workspace() {
        let revision = WorkspaceRevision {
            workspace_id: WorkspaceId::new("w-a").unwrap(),
            revision: RevisionId::new("r1").unwrap(),
        };
        assert!(
            revision
                .ensure_of(&WorkspaceId::new("w-a").unwrap(), "checkout")
                .is_ok()
        );
        let error = revision
            .ensure_of(&WorkspaceId::new("w-b").unwrap(), "checkout")
            .unwrap_err();
        assert_eq!(error, DomainError::new("checkout", ErrorKind::Mismatch));
    }

    #[test]
    fn generations_start_at_one_and_never_wrap() {
        assert_eq!(Generation::new(0).unwrap_err().kind, ErrorKind::OutOfRange);
        assert!(serde_json::from_str::<Generation>("0").is_err());
        assert_eq!(Generation::FIRST.next().unwrap().get(), 2);
        assert!(Generation::new(u64::MAX).unwrap().next().is_err());
        assert_eq!(serde_json::to_string(&Generation::FIRST).unwrap(), "1");
    }

    #[test]
    fn permission_epochs_never_wrap() {
        assert_eq!(PermissionEpoch::new(4).next().unwrap().get(), 5);
        assert!(PermissionEpoch::new(u64::MAX).next().is_err());
    }

    #[test]
    fn revisions_reject_unknown_fields() {
        let json = r#"{"workspace_id":"w","revision":"r","grants":["all"]}"#;
        assert!(serde_json::from_str::<WorkspaceRevision>(json).is_err());
    }
}
