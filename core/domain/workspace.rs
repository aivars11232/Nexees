//! Workspaces, their device-local replicas, the paths inside them, and synchronization
//! conflicts (A3, A6, B18, ST-WORKSPACE, ST-CONTENT).
//!
//! A workspace is known by the same [`WorkspaceId`] on every participating device, but each
//! device keeps its own [`Replica`] with its own root. A root is meaningful only on its own
//! device: it is never copied to another device ([`Replica`] never travels), and asking a replica
//! for its root on any other device fails. A PC path is therefore never used as an Android path.
//! Mapping a workspace to a root grants nothing; authority is recorded only in
//! [`crate::authority`].

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{DomainError, ErrorKind};
use crate::ids::{ConflictId, DeviceId, WorkspaceId};
use crate::revision::{ContentHash, WorkspaceRevision};
use crate::schema::{Record, SyncPolicy, Validate, validated_record};
use crate::text::Label;
use crate::time::Timestamp;

/// The kind of a workspace (workspace_model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceKind {
    /// A coding project.
    Code,
    /// An LCL project.
    Lcl,
}

/// The part of a workspace that every participating device knows: its identity, kind and name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceRecord {
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// Its kind.
    pub kind: WorkspaceKind,
    /// Its name as shown to the user.
    pub name: Label,
}

impl Record for WorkspaceRecord {
    const SCHEMA: &'static str = "nexees.workspace.workspace";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::PortableRecord;
}

/// Longest [`RelativePath`], in bytes.
pub const MAX_PATH_BYTES: usize = 4096;
/// Longest segment of a [`RelativePath`], in bytes.
pub const MAX_SEGMENT_BYTES: usize = 255;

/// A path inside a workspace, relative to its root: segments joined by `/`.
///
/// It can never leave the root. It is not absolute, and it has no empty, `.` or `..` segment. It
/// contains no backslash, no colon (a drive or alternate-stream prefix on Windows) and no control
/// character. Segments are compared byte for byte. Filesystem-specific aliases, such as case or
/// Unicode normalization collisions, are checked by the code that writes to a filesystem: the
/// workspace scope (TASK-023) and import (TASK-051).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativePath(String);

impl RelativePath {
    /// Checks `value` and returns it as a [`RelativePath`].
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        const FIELD: &str = "RelativePath";
        if value.is_empty() {
            return Err(DomainError::new(FIELD, ErrorKind::Empty));
        }
        if value.len() > MAX_PATH_BYTES {
            return Err(DomainError::new(FIELD, ErrorKind::TooLong));
        }
        for segment in value.split('/') {
            if segment.is_empty() || segment == "." || segment == ".." {
                return Err(DomainError::new(FIELD, ErrorKind::InvalidFormat));
            }
            if segment.len() > MAX_SEGMENT_BYTES {
                return Err(DomainError::new(FIELD, ErrorKind::TooLong));
            }
            if segment
                .chars()
                .any(|c| c.is_control() || c == '\\' || c == ':')
            {
                return Err(DomainError::new(FIELD, ErrorKind::InvalidCharacter));
            }
        }
        Ok(Self(value))
    }

    /// The path as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RelativePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RelativePath {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RelativePath {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// The root of a replica on its own device, in that platform's form: a filesystem path on
/// Desktop, an app-private location or an authorized document tree on Android. Its meaning
/// belongs to that one device; this type only bounds it and keeps control characters out.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceRoot(String);

impl DeviceRoot {
    /// Checks `value` and returns it as a [`DeviceRoot`].
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.is_empty() {
            return Err(DomainError::new("DeviceRoot", ErrorKind::Empty));
        }
        if value.len() > MAX_PATH_BYTES {
            return Err(DomainError::new("DeviceRoot", ErrorKind::TooLong));
        }
        if value.chars().any(char::is_control) {
            return Err(DomainError::new("DeviceRoot", ErrorKind::InvalidCharacter));
        }
        Ok(Self(value))
    }

    /// The root in the device's own form.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for DeviceRoot {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for DeviceRoot {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Whether a device holds a workspace's content (A3). A cached listing is not content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentAvailability {
    /// The content lives on this device and is edited here.
    Local,
    /// Only a listing is known; the content is on another device and cannot be edited here.
    RemoteOnly,
    /// A local copy, reconciled with the other replicas by synchronization.
    SyncedCopy,
}

/// The fields of a [`Replica`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplicaFields {
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// The device that holds this replica.
    pub device_id: DeviceId,
    /// Where the content is on that device; absent exactly when it is
    /// [`ContentAvailability::RemoteOnly`].
    pub root: Option<DeviceRoot>,
    /// Whether the content is on that device.
    pub availability: ContentAvailability,
}

impl Validate for ReplicaFields {
    fn validate(&self) -> Result<(), DomainError> {
        match (self.availability, &self.root) {
            (ContentAvailability::RemoteOnly, Some(_)) => {
                Err(DomainError::new("root", ErrorKind::Unexpected))
            }
            (ContentAvailability::Local | ContentAvailability::SyncedCopy, None) => {
                Err(DomainError::new("root", ErrorKind::Missing))
            }
            _ => Ok(()),
        }
    }
}

validated_record!(
    /// One device's replica of a workspace: where its content is on that device, if anywhere
    /// (B18). It never leaves the device and confers no authority.
    Replica,
    ReplicaFields
);

impl Replica {
    /// The root of this replica, asked for on `device`.
    ///
    /// Fails with [`ErrorKind::Mismatch`] on another device, because a root is meaningful only
    /// where it was created, and with [`ErrorKind::Missing`] when only a listing is known.
    pub fn root_on(&self, device: &DeviceId) -> Result<&DeviceRoot, DomainError> {
        if &self.0.device_id != device {
            return Err(DomainError::new("device_id", ErrorKind::Mismatch));
        }
        self.0
            .root
            .as_ref()
            .ok_or(DomainError::new("root", ErrorKind::Missing))
    }

    /// Whether the content can be edited on the replica's device.
    pub fn is_editable(&self) -> bool {
        self.0.availability != ContentAvailability::RemoteOnly
    }
}

impl Record for Replica {
    const SCHEMA: &'static str = "nexees.workspace.replica";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// One side of a [`Conflict`]: a device's revision and the content it holds there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictSide {
    /// The device that made the change.
    pub device_id: DeviceId,
    /// The workspace revision that contains the change.
    pub revision: WorkspaceRevision,
    /// The content on that side; absent when that side deleted the path.
    pub content: Option<ContentHash>,
}

/// The fields of a [`Conflict`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConflictFields {
    /// The conflict.
    pub conflict_id: ConflictId,
    /// The workspace.
    pub workspace_id: WorkspaceId,
    /// The path both sides changed.
    pub path: RelativePath,
    /// The content both sides started from; absent when both created the path.
    pub base: Option<ContentHash>,
    /// The change on this device.
    pub local: ConflictSide,
    /// The change from the other device.
    pub remote: ConflictSide,
    /// When the conflict was detected.
    pub detected_at: Timestamp,
}

impl Validate for ConflictFields {
    fn validate(&self) -> Result<(), DomainError> {
        self.local
            .revision
            .ensure_of(&self.workspace_id, "local.revision")?;
        self.remote
            .revision
            .ensure_of(&self.workspace_id, "remote.revision")?;
        if self.local.device_id == self.remote.device_id {
            return Err(DomainError::new("remote.device_id", ErrorKind::Duplicate));
        }
        // Two sides that ended with the same content, or that both deleted the path, agree.
        if self.local.content == self.remote.content {
            return Err(DomainError::new("remote.content", ErrorKind::Duplicate));
        }
        Ok(())
    }
}

validated_record!(
    /// Concurrent changes of one path on two devices, with both versions preserved until the user
    /// decides (A6, C22). A deletion is a side without content, so offline reconnection can never
    /// silently resurrect or delete newer work.
    Conflict,
    ConflictFields
);

impl Record for Conflict {
    const SCHEMA: &'static str = "nexees.workspace.conflict";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revision::RevisionId;
    use crate::schema::Versioned;

    fn id(value: &str) -> WorkspaceId {
        WorkspaceId::new(value).unwrap()
    }

    fn device(value: &str) -> DeviceId {
        DeviceId::new(value).unwrap()
    }

    fn replica(
        device_id: &str,
        root: Option<&str>,
        availability: ContentAvailability,
    ) -> ReplicaFields {
        ReplicaFields {
            workspace_id: id("arch-dock"),
            device_id: device(device_id),
            root: root.map(|r| DeviceRoot::new(r).unwrap()),
            availability,
        }
    }

    #[test]
    fn relative_paths_cannot_leave_the_root_or_hide_a_prefix() {
        for good in [
            "src/main.rs",
            "rules/master.lcl.txt",
            ".gitignore",
            "a b/ü.txt",
        ] {
            assert!(RelativePath::new(good).is_ok(), "{good}");
        }
        for (bad, kind) in [
            ("", ErrorKind::Empty),
            ("/etc/passwd", ErrorKind::InvalidFormat),
            ("../outside", ErrorKind::InvalidFormat),
            ("a/../../b", ErrorKind::InvalidFormat),
            ("a//b", ErrorKind::InvalidFormat),
            ("a/./b", ErrorKind::InvalidFormat),
            ("trailing/", ErrorKind::InvalidFormat),
            ("C:/Windows", ErrorKind::InvalidCharacter),
            ("file.txt:stream", ErrorKind::InvalidCharacter),
            ("dir\\..\\x", ErrorKind::InvalidCharacter),
            ("nul\0byte", ErrorKind::InvalidCharacter),
            ("line\nbreak", ErrorKind::InvalidCharacter),
        ] {
            assert_eq!(RelativePath::new(bad).unwrap_err().kind, kind, "{bad:?}");
        }
        assert_eq!(
            RelativePath::new("x".repeat(MAX_SEGMENT_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::TooLong
        );
        assert!(serde_json::from_str::<RelativePath>("\"../x\"").is_err());
    }

    #[test]
    fn one_workspace_maps_to_a_different_root_on_each_device() {
        let pc = Replica::try_from(replica(
            "pc",
            Some("/home/u/arch-dock"),
            ContentAvailability::Local,
        ))
        .unwrap();
        let phone = Replica::try_from(replica(
            "phone",
            Some("content://tree/primary:Nexees/arch-dock"),
            ContentAvailability::SyncedCopy,
        ))
        .unwrap();
        assert_eq!(pc.fields().workspace_id, phone.fields().workspace_id);
        assert_eq!(
            pc.root_on(&device("pc")).unwrap().as_str(),
            "/home/u/arch-dock"
        );
        // The PC's root is never used on the phone, and the phone's never on the PC.
        assert_eq!(
            pc.root_on(&device("phone")).unwrap_err().kind,
            ErrorKind::Mismatch
        );
        assert_eq!(
            phone.root_on(&device("pc")).unwrap_err().kind,
            ErrorKind::Mismatch
        );
    }

    #[test]
    fn a_remote_only_listing_has_no_root_and_is_not_editable() {
        let listing =
            Replica::try_from(replica("phone", None, ContentAvailability::RemoteOnly)).unwrap();
        assert!(!listing.is_editable());
        assert_eq!(
            listing.root_on(&device("phone")).unwrap_err().kind,
            ErrorKind::Missing
        );
        assert_eq!(
            Replica::try_from(replica(
                "phone",
                Some("/x"),
                ContentAvailability::RemoteOnly
            ))
            .unwrap_err(),
            DomainError::new("root", ErrorKind::Unexpected)
        );
        assert_eq!(
            Replica::try_from(replica("phone", None, ContentAvailability::Local)).unwrap_err(),
            DomainError::new("root", ErrorKind::Missing)
        );
    }

    #[test]
    fn mapping_a_root_cannot_carry_authority_in_the_record() {
        // A replica or workspace record with a smuggled authority field is rejected, so a root
        // mapping can never grant anything by itself.
        let replica = concat!(
            r#"{"workspace_id":"w","device_id":"phone","root":"/r","#,
            r#""availability":"local","grants":["all"]}"#
        );
        assert!(serde_json::from_str::<Replica>(replica).is_err());
        let workspace = r#"{"workspace_id":"w","kind":"code","name":"W","permission_epoch":9}"#;
        assert!(serde_json::from_str::<WorkspaceRecord>(workspace).is_err());
    }

    #[test]
    fn replicas_and_workspaces_round_trip_as_versioned_records() {
        let replica =
            Replica::try_from(replica("pc", Some("/home/u/w"), ContentAvailability::Local))
                .unwrap();
        let text = serde_json::to_string(&Versioned(replica.clone())).unwrap();
        assert!(text.starts_with(r#"{"schema":"nexees.workspace.replica","version":1,"record":"#));
        assert_eq!(
            serde_json::from_str::<Versioned<Replica>>(&text).unwrap().0,
            replica
        );

        let workspace = WorkspaceRecord {
            workspace_id: id("w"),
            kind: WorkspaceKind::Lcl,
            name: Label::new("LCL — Next").unwrap(),
        };
        let text = serde_json::to_string(&Versioned(workspace.clone())).unwrap();
        assert_eq!(
            serde_json::from_str::<Versioned<WorkspaceRecord>>(&text)
                .unwrap()
                .0,
            workspace
        );
        // The same bytes under another schema name, or a future version, are refused.
        let renamed = text.replace("nexees.workspace.workspace", "nexees.workspace.replica");
        assert!(serde_json::from_str::<Versioned<WorkspaceRecord>>(&renamed).is_err());
        let newer = text.replace(r#""version":1"#, r#""version":2"#);
        assert!(serde_json::from_str::<Versioned<WorkspaceRecord>>(&newer).is_err());
    }

    fn side(device_id: &str, workspace: &str, content: Option<char>) -> ConflictSide {
        ConflictSide {
            device_id: device(device_id),
            revision: WorkspaceRevision {
                workspace_id: id(workspace),
                revision: RevisionId::new("r").unwrap(),
            },
            content: content.map(|c| ContentHash::new(c.to_string().repeat(64)).unwrap()),
        }
    }

    fn conflict(local: ConflictSide, remote: ConflictSide) -> ConflictFields {
        ConflictFields {
            conflict_id: ConflictId::new("c1").unwrap(),
            workspace_id: id("w"),
            path: RelativePath::new("rules/master.lcl.txt").unwrap(),
            base: Some(ContentHash::new("0".repeat(64)).unwrap()),
            local,
            remote,
            detected_at: Timestamp::from_unix_millis(1),
        }
    }

    #[test]
    fn conflicts_keep_both_versions_including_a_deletion() {
        let edited_versus_deleted = conflict(side("pc", "w", Some('a')), side("phone", "w", None));
        assert!(Conflict::try_from(edited_versus_deleted).is_ok());
        let both_deleted = conflict(side("pc", "w", None), side("phone", "w", None));
        assert_eq!(
            Conflict::try_from(both_deleted).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let same_device = conflict(side("pc", "w", Some('a')), side("pc", "w", Some('b')));
        assert_eq!(
            Conflict::try_from(same_device).unwrap_err().kind,
            ErrorKind::Duplicate
        );
        let other_workspace = conflict(side("pc", "w", Some('a')), side("phone", "v", Some('b')));
        assert_eq!(
            Conflict::try_from(other_workspace).unwrap_err(),
            DomainError::new("remote.revision", ErrorKind::Mismatch)
        );
    }
}
