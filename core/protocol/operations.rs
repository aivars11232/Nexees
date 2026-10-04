//! The closed registry of operations a client or a peer may request (RC-09, EC-02).
//!
//! Each operation is a type with typed arguments that declares, through
//! [`nexees_domain::remote_request::Operation`], what it targets and whether it changes anything.
//! The envelope therefore checks its explicit targets and expected revisions. The registry is
//! closed: a request decodes only into a registered operation, and an unknown operation is refused
//! before anything else happens.
//!
//! Two sets exist. [`RemoteRequest`] holds what a paired peer may request. [`LocalIntent`]
//! adds what only the local user of a host may request through its own window: deciding an
//! approval, committing or cancelling an import, adopting an LCL revision (RC-18). Operations that
//! may never be requested remotely (`binding.sec_never_remote`) are in neither set. Admission and
//! authorization stay with the destination (RC-10); a registered operation is a shape, not a
//! permission.

use std::fmt;

use nexees_domain::errors::{DomainError, ErrorKind};
use nexees_domain::handoff::OrientationSetting;
use nexees_domain::ids::{AllowlistKey, ImportId, OperationId, RequestId};
use nexees_domain::model_capabilities::ProviderModel;
use nexees_domain::remote_request::{Operation, OperationTarget, RequestEnvelope};
use nexees_domain::revision::SpecRevision;
use nexees_domain::workspace::RelativePath;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;

use crate::messages::{Header, Intent, RemoteHeader};

/// Largest file content one `file_write` request may carry, in bytes. Larger content travels by
/// synchronization, which transfers it in chunks.
pub const MAX_FILE_TEXT_BYTES: usize = 32 * 1024;

/// The UTF-8 text a `file_write` writes, at most [`MAX_FILE_TEXT_BYTES`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileText(String);

impl FileText {
    /// Checks `value` and returns it as [`FileText`].
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        if value.len() > MAX_FILE_TEXT_BYTES {
            return Err(DomainError::new("FileText", ErrorKind::TooLong));
        }
        Ok(Self(value))
    }

    /// The text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for FileText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for FileText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Defines one operation: its argument type, registry key, target and whether it changes state.
macro_rules! operation {
    ($(#[$doc:meta])* $name:ident { $($(#[$fdoc:meta])* $field:ident: $ty:ty),* $(,)? },
     $id:literal, $target:ident, $changes:literal) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(deny_unknown_fields)]
        pub struct $name {
            $($(#[$fdoc])* pub $field: $ty,)*
        }

        impl Operation for $name {
            const ID: &'static str = $id;
            const TARGET: OperationTarget = OperationTarget::$target;
            const CHANGES_STATE: bool = $changes;
        }
    };
}

operation!(
    /// Launch an allowlisted application on the destination device (RC-11). It names the
    /// allowlist key only, never a command line, and expects the destination's policy revision.
    AppLaunch {
        /// The application's allowlist key.
        app: AllowlistKey,
    },
    "app_launch", Device, true
);
operation!(
    /// Write UTF-8 text to a file of the target workspace, at the expected workspace revision.
    FileWrite {
        /// The file.
        path: RelativePath,
        /// Its new content.
        text: FileText,
    },
    "file_write", Workspace, true
);
operation!(
    /// Read the status of the target workspace: its sessions and tasks. Changes nothing.
    StatusRead {},
    "status_read", Workspace, false
);
operation!(
    /// Pause the target agent at its next safe step.
    AgentPause {},
    "agent_pause", Agent, true
);
operation!(
    /// Resume the paused target agent.
    AgentResume {},
    "agent_resume", Agent, true
);
operation!(
    /// Cancel the target agent; effects in flight are settled, never abandoned unknown.
    AgentCancel {},
    "agent_cancel", Agent, true
);
operation!(
    /// Switch the target agent's model: the generation advances, every binding stays (H2).
    AgentSwitchModel {
        /// The new provider and model.
        model: ProviderModel,
        /// The orientation setting for this switch, when the user overrides the default (H1).
        orientation: Option<OrientationSetting>,
    },
    "agent_switch_model", Agent, true
);
operation!(
    /// Set the target agent's orientation override (H1). It applies at a safe boundary and is not
    /// a permission.
    AgentSetOrientation {
        /// The setting.
        setting: OrientationSetting,
    },
    "agent_set_orientation", Agent, true
);
operation!(
    /// Validate a specification revision of the target workspace with the pinned LCL core.
    LclValidate {
        /// The revision.
        revision: SpecRevision,
    },
    "lcl_validate", Workspace, true
);
operation!(
    /// Decide a request that waits for the local user's approval (RC-17, RC-18). Local only; the
    /// decision is revalidated against the request before it is used.
    ApprovalDecide {
        /// The waiting request.
        request_id: RequestId,
        /// Whether the user approves it.
        approve: bool,
    },
    "approval_decide", Device, true
);
operation!(
    /// Commit a previewed import of the target workspace, at the reviewed base revision (B22).
    /// Local only.
    ImportCommit {
        /// The import.
        import_id: ImportId,
    },
    "import_commit", Workspace, true
);
operation!(
    /// Cancel an import of the target workspace. Local only.
    ImportCancel {
        /// The import.
        import_id: ImportId,
    },
    "import_cancel", Workspace, true
);
operation!(
    /// Adopt a validated specification revision for the target agent at a safe boundary (B7).
    /// Local only.
    LclAdopt {
        /// The revision to adopt.
        revision: SpecRevision,
    },
    "lcl_adopt", Agent, true
);

/// Why a message names an operation its registry does not hold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownOperation(pub OperationId);

impl fmt::Display for UnknownOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "operation {} is not in this registry", self.0)
    }
}

/// The part of a request that names its operation, read first to choose the type that decodes
/// the whole request strictly.
#[derive(Deserialize)]
struct OperationName {
    operation: OperationId,
}

/// Defines a closed registry: an enum with one variant per operation, each holding `$envelope`
/// of that operation. It serializes as the inner envelope. It decodes by reading the operation
/// name first and then the whole envelope, strictly, as that operation; an unknown name is
/// refused.
macro_rules! registry {
    ($(#[$doc:meta])* $name:ident, $envelope:ident, $header:ident, [$($op:ident),* $(,)?]) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name {
            $(
                #[doc = concat!("A `", stringify!($op), "` operation.")]
                $op($envelope<$op>),
            )*
        }

        impl $name {
            /// The registry keys of this set, in order.
            pub const OPERATIONS: &'static [&'static str] = &[$(<$op as Operation>::ID),*];

            /// What the request shares with every other, whatever its operation.
            pub fn header(&self) -> &dyn $header {
                match self {
                    $(Self::$op(inner) => inner,)*
                }
            }

            /// The operation's registry key.
            pub fn operation(&self) -> &OperationId {
                self.header().operation()
            }

            /// The request's identifier.
            pub fn request_id(&self) -> &RequestId {
                self.header().request_id()
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                match self {
                    $(Self::$op(inner) => inner.serialize(serializer),)*
                }
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                use serde::de::Error;
                let raw = <Box<RawValue>>::deserialize(deserializer)?;
                let name: OperationName =
                    serde_json::from_str(raw.get()).map_err(D::Error::custom)?;
                match name.operation.as_str() {
                    $(<$op as Operation>::ID => serde_json::from_str(raw.get())
                        .map(Self::$op)
                        .map_err(D::Error::custom),)*
                    _ => Err(D::Error::custom(UnknownOperation(name.operation))),
                }
            }
        }
    };
}

registry!(
    /// A request from a paired peer, for any operation a peer may request (RC-09).
    RemoteRequest,
    RequestEnvelope,
    RemoteHeader,
    [AppLaunch, FileWrite, StatusRead, AgentPause, AgentResume, AgentCancel, AgentSwitchModel,
     AgentSetOrientation, LclValidate]
);

registry!(
    /// An intent from a window to its own host: every remote operation, and the operations only
    /// the local user may request.
    LocalIntent,
    Intent,
    Header,
    [AppLaunch, FileWrite, StatusRead, AgentPause, AgentResume, AgentCancel, AgentSwitchModel,
     AgentSetOrientation, LclValidate, ApprovalDecide, ImportCommit, ImportCancel, LclAdopt]
);

#[cfg(test)]
mod tests {
    use super::*;
    use nexees_domain::ids::OperationId;

    #[test]
    fn every_registry_key_is_valid_and_unique() {
        for set in [RemoteRequest::OPERATIONS, LocalIntent::OPERATIONS] {
            let mut keys = set.to_vec();
            for key in &keys {
                assert!(OperationId::new(*key).is_ok(), "{key}");
            }
            keys.sort_unstable();
            keys.dedup();
            assert_eq!(keys.len(), set.len());
        }
    }

    #[test]
    fn local_only_operations_are_not_remote_and_never_remote_ones_exist_nowhere() {
        for local_only in [
            "approval_decide",
            "import_commit",
            "import_cancel",
            "lcl_adopt",
        ] {
            assert!(!RemoteRequest::OPERATIONS.contains(&local_only));
            assert!(LocalIntent::OPERATIONS.contains(&local_only));
        }
        for remote in RemoteRequest::OPERATIONS {
            assert!(LocalIntent::OPERATIONS.contains(remote));
        }
        // binding.sec_never_remote: refused from any remote request, and not even shaped here.
        for never in [
            "disable_device_lock",
            "grant_control_rights",
            "reveal_or_export_secrets",
            "change_startup_policy",
            "change_receiving_policy",
            "weaken_authentication",
            "change_trusted_devices",
        ] {
            assert!(!LocalIntent::OPERATIONS.contains(&never), "{never}");
        }
    }

    #[test]
    fn file_text_is_bounded() {
        assert!(FileText::new("x".repeat(MAX_FILE_TEXT_BYTES)).is_ok());
        assert_eq!(
            FileText::new("x".repeat(MAX_FILE_TEXT_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::TooLong
        );
    }

    #[test]
    fn operation_arguments_reject_unknown_fields() {
        assert!(serde_json::from_str::<AgentPause>(r#"{"force":true}"#).is_err());
        assert!(serde_json::from_str::<AppLaunch>(r#"{"app":"hoptodesk","args":"--x"}"#).is_err());
        assert!(serde_json::from_str::<AppLaunch>(r#"{"app":"/usr/bin/sh"}"#).is_err());
    }
}
