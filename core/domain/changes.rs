//! Content changes for selected synchronization (ST-SYNC, IF-SYNC, A6, C22).
//!
//! A replica records each change of a selected workspace as a [`ContentChange`]: the path, what
//! happened to it, and the content it was made against. The receiving replica applies a change
//! only onto that base content. The same change arriving again finds its result already in place
//! and changes nothing. Any other state is a conflict that keeps both versions; there is no
//! timestamp-based last writer wins. A deletion is a tombstone, which carries the content it
//! deleted, so that an offline reconnection can neither resurrect newer work nor silently delete
//! it. Changes carry content, never authority. The outbox store is TASK-008's, the
//! synchronization itself TASK-069's; this module fixes the record and the rule for applying it.

use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{ChangeId, DeviceId, WorkspaceId};
use crate::revision::{ContentHash, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, validated_record};
use crate::time::Timestamp;
use crate::workspace::RelativePath;

/// What a change does to its path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ChangeKind {
    /// The path now holds this content: a creation when there is no base, an edit otherwise.
    Write {
        /// The new content's hash; its bytes travel separately.
        content: ContentHash,
        /// The new content's size in bytes.
        size: u64,
    },
    /// The path moved to `to`, with its content unchanged.
    Rename {
        /// The new path.
        to: RelativePath,
    },
    /// The path was deleted: a tombstone.
    Delete,
}

/// The fields of a [`ContentChange`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentChangeFields {
    /// The change; applying it twice has the effect of applying it once.
    pub change_id: ChangeId,
    /// The selected workspace.
    pub workspace_id: WorkspaceId,
    /// The path changed.
    pub path: RelativePath,
    /// What happened to it.
    pub kind: ChangeKind,
    /// The content the change was made against; absent when it created the path. A rename or a
    /// deletion always names the content it moved or deleted.
    pub base: Option<ContentHash>,
    /// The device that made the change.
    pub origin_device_id: DeviceId,
    /// The revision of `workspace_id` on that device that contains the change.
    pub origin_revision: WorkspaceRevision,
    /// When it was recorded.
    pub recorded_at: Timestamp,
}

impl Validate for ContentChangeFields {
    fn validate(&self) -> Result<(), DomainError> {
        self.origin_revision
            .ensure_of(&self.workspace_id, "origin_revision")?;
        match &self.kind {
            ChangeKind::Rename { to } if to == &self.path => {
                Err(DomainError::new("kind", ErrorKind::Duplicate))
            }
            ChangeKind::Rename { .. } | ChangeKind::Delete if self.base.is_none() => {
                Err(DomainError::new("base", ErrorKind::Missing))
            }
            ChangeKind::Write { content, .. } if self.base.as_ref() == Some(content) => {
                Err(DomainError::new("kind", ErrorKind::Duplicate))
            }
            _ => Ok(()),
        }
    }
}

validated_record!(
    /// One change of a selected workspace, as the outbox keeps it and IF-SYNC carries it.
    ContentChange,
    ContentChangeFields
);

/// What the receiving replica does with a change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Apply {
    /// The path is exactly at the change's base: apply it.
    Apply,
    /// The change's result is already in place: change nothing (idempotent delivery).
    AlreadyApplied,
    /// The path changed differently here: record a conflict that keeps both versions.
    Conflict,
}

impl ContentChange {
    /// The decision for this change, given the content the receiving replica holds at its path
    /// (`current`, absent when the path does not exist) and, for a rename, at its new path
    /// (`current_at_target`). It never decides by time or by which side is newer.
    pub fn apply_to(
        &self,
        current: Option<&ContentHash>,
        current_at_target: Option<&ContentHash>,
    ) -> Apply {
        let base = self.0.base.as_ref();
        match &self.0.kind {
            ChangeKind::Write { content, .. } if current == Some(content) => Apply::AlreadyApplied,
            ChangeKind::Delete if current.is_none() => Apply::AlreadyApplied,
            ChangeKind::Rename { .. } if current.is_none() && current_at_target == base => {
                Apply::AlreadyApplied
            }
            ChangeKind::Rename { .. } if current_at_target.is_some() => Apply::Conflict,
            _ if current == base => Apply::Apply,
            _ => Apply::Conflict,
        }
    }
}

impl Record for ContentChange {
    const SCHEMA: &'static str = "nexees.changes.content_change";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::SelectedContent;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revision::RevisionId;

    fn hash(c: char) -> ContentHash {
        ContentHash::new(c.to_string().repeat(64)).unwrap()
    }

    fn change(kind: ChangeKind, base: Option<char>) -> Result<ContentChange, DomainError> {
        ContentChange::try_from(ContentChangeFields {
            change_id: ChangeId::new("c1").unwrap(),
            workspace_id: WorkspaceId::new("lcl-b").unwrap(),
            path: RelativePath::new("rules/master.lcl.txt").unwrap(),
            kind,
            base: base.map(hash),
            origin_device_id: DeviceId::new("phone").unwrap(),
            origin_revision: WorkspaceRevision {
                workspace_id: WorkspaceId::new("lcl-b").unwrap(),
                revision: RevisionId::new("r9").unwrap(),
            },
            recorded_at: Timestamp::from_unix_millis(1),
        })
    }

    fn write(content: char, base: Option<char>) -> ContentChange {
        change(
            ChangeKind::Write {
                content: hash(content),
                size: 10,
            },
            base,
        )
        .unwrap()
    }

    #[test]
    fn a_change_applies_only_onto_its_base_and_never_twice() {
        let edit = write('b', Some('a'));
        assert_eq!(edit.apply_to(Some(&hash('a')), None), Apply::Apply);
        // Delivered again after it was applied: nothing changes.
        assert_eq!(edit.apply_to(Some(&hash('b')), None), Apply::AlreadyApplied);
        // The receiving side changed the file meanwhile: both versions are kept.
        assert_eq!(edit.apply_to(Some(&hash('c')), None), Apply::Conflict);
        // The receiving side deleted it meanwhile: still a conflict, not a resurrection.
        assert_eq!(edit.apply_to(None, None), Apply::Conflict);
    }

    #[test]
    fn creations_and_deletions_are_explicit_about_what_they_meet() {
        let create = write('b', None);
        assert_eq!(create.apply_to(None, None), Apply::Apply);
        assert_eq!(create.apply_to(Some(&hash('e')), None), Apply::Conflict);
        let delete = change(ChangeKind::Delete, Some('a')).unwrap();
        assert_eq!(delete.apply_to(Some(&hash('a')), None), Apply::Apply);
        assert_eq!(delete.apply_to(None, None), Apply::AlreadyApplied);
        // Newer work on the receiving side is never deleted silently.
        assert_eq!(delete.apply_to(Some(&hash('c')), None), Apply::Conflict);
    }

    #[test]
    fn a_rename_never_overwrites_what_is_at_its_target() {
        let rename = change(
            ChangeKind::Rename {
                to: RelativePath::new("rules/main.lcl.txt").unwrap(),
            },
            Some('a'),
        )
        .unwrap();
        assert_eq!(rename.apply_to(Some(&hash('a')), None), Apply::Apply);
        assert_eq!(
            rename.apply_to(None, Some(&hash('a'))),
            Apply::AlreadyApplied
        );
        assert_eq!(
            rename.apply_to(Some(&hash('a')), Some(&hash('f'))),
            Apply::Conflict
        );
    }

    #[test]
    fn changes_say_what_they_were_made_against() {
        assert_eq!(
            change(ChangeKind::Delete, None).unwrap_err(),
            DomainError::new("base", ErrorKind::Missing)
        );
        assert_eq!(
            change(
                ChangeKind::Rename {
                    to: RelativePath::new("rules/master.lcl.txt").unwrap()
                },
                Some('a')
            )
            .unwrap_err()
            .kind,
            ErrorKind::Duplicate
        );
        assert_eq!(
            change(
                ChangeKind::Write {
                    content: hash('a'),
                    size: 1
                },
                Some('a')
            )
            .unwrap_err()
            .kind,
            ErrorKind::Duplicate
        );
        let mut fields = write('b', Some('a')).into_fields();
        fields.origin_revision.workspace_id = WorkspaceId::new("other").unwrap();
        assert_eq!(
            ContentChange::try_from(fields).unwrap_err(),
            DomainError::new("origin_revision", ErrorKind::Mismatch)
        );
    }

    #[test]
    fn a_change_is_selected_content_and_cannot_carry_authority() {
        assert_eq!(<ContentChange as Record>::SYNC, SyncPolicy::SelectedContent);
        let mut value = serde_json::to_value(write('b', Some('a'))).unwrap();
        value["grant"] = serde_json::json!({"capability": "agent_control"});
        assert!(serde_json::from_value::<ContentChange>(value).is_err());
    }
}
