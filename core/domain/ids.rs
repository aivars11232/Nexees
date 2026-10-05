//! Identifiers shared by every record of the domain.
//!
//! Each kind of identity has its own type, so a device can never be passed where a workspace is
//! expected, and a request always names its targets with explicit types (SI-12, TH-47). There are
//! three shapes:
//!
//! - **Opaque identifiers**, such as [`DeviceId`] or [`WorkspaceId`]: 1 to 128 ASCII characters,
//!   letters, digits and `-`, `_`, `.`, `:`, starting with a letter or digit. They are compared
//!   byte for byte, without case folding or Unicode normalization, so two different identities
//!   can never compare equal.
//! - **Model identifiers** ([`ModelId`]) also allow `/`, `@` and `+`, because providers name
//!   their models that way.
//! - **Registry keys**, such as [`OperationId`]: 1 to 64 characters of lowercase `snake_case`.
//!   Each names an entry of a closed registry that its owning task defines. The type only
//!   guarantees the shape; the owner rejects any key that its registry does not list.
//!
//! A display name is never an identity: two devices may share a name, never an identifier.

use crate::errors::{DomainError, ErrorKind};

/// Longest opaque identifier, in bytes.
pub const MAX_ID_BYTES: usize = 128;
/// Longest model identifier, in bytes.
pub const MAX_MODEL_ID_BYTES: usize = 256;
/// Longest registry key, in bytes.
pub const MAX_KEY_BYTES: usize = 64;

fn check_ascii(
    value: &str,
    max: usize,
    first: fn(u8) -> bool,
    rest: fn(u8) -> bool,
    field: &'static str,
) -> Result<(), DomainError> {
    let bytes = value.as_bytes();
    let Some(&head) = bytes.first() else {
        return Err(DomainError::new(field, ErrorKind::Empty));
    };
    if bytes.len() > max {
        return Err(DomainError::new(field, ErrorKind::TooLong));
    }
    if !first(head) || !bytes.iter().all(|&b| rest(b)) {
        return Err(DomainError::new(field, ErrorKind::InvalidCharacter));
    }
    Ok(())
}

/// Checks an opaque identifier.
pub(crate) fn check_id(value: &str, field: &'static str) -> Result<(), DomainError> {
    check_ascii(
        value,
        MAX_ID_BYTES,
        |b| b.is_ascii_alphanumeric(),
        |b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'),
        field,
    )
}

fn check_model_id(value: &str, field: &'static str) -> Result<(), DomainError> {
    check_ascii(
        value,
        MAX_MODEL_ID_BYTES,
        |b| b.is_ascii_alphanumeric(),
        |b| {
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':' | b'/' | b'@' | b'+')
        },
        field,
    )
}

/// Checks a registry key.
pub(crate) fn check_key(value: &str, field: &'static str) -> Result<(), DomainError> {
    check_ascii(
        value,
        MAX_KEY_BYTES,
        |b| b.is_ascii_lowercase(),
        |b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_',
        field,
    )
}

/// Defines an identifier type: a validated string with its own name, compared byte for byte.
/// Construction and decoding run the same check, so no invalid identifier exists.
macro_rules! identifier {
    ($(#[$doc:meta])* $name:ident, $check:path) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            #[doc = concat!("Checks `value` and returns it as a [`", stringify!($name), "`].")]
            pub fn new(value: impl Into<String>) -> Result<Self, $crate::errors::DomainError> {
                let value = value.into();
                $check(&value, stringify!($name))?;
                Ok(Self(value))
            }

            /// The identifier as text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let value = <String as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::new(value).map_err(::serde::de::Error::custom)
            }
        }
    };
}
pub(crate) use identifier;

identifier!(
    /// A device: the fingerprint of its pairing key (PD-TRANSPORT), as the pairing task
    /// (TASK-060) derives it. A device's display name is a [`crate::text::Label`], never this.
    DeviceId,
    check_id
);
identifier!(
    /// A Nexees account user.
    UserId,
    check_id
);
identifier!(
    /// A host session on a device: the OS login session on Desktop, the app instance on Android
    /// (the destination session of RC-09).
    HostSessionId,
    check_id
);
identifier!(
    /// A workspace. It is the same on every device that has a replica of the workspace; each
    /// device maps it to its own root (A3, B18).
    WorkspaceId,
    check_id
);
identifier!(
    /// An agent session (B17). Remote requests name it as their agent (RC-09).
    AgentSessionId,
    check_id
);
identifier!(
    /// A task.
    TaskId,
    check_id
);
identifier!(
    /// A UI client instance: one Desktop window or one Android activity attached to its host.
    ClientId,
    check_id
);
identifier!(
    /// A view of a UI client as the client itself names it, such as a sidebar view or a
    /// bottom-panel view of the Desktop window. A host keeps the name and gives it back; it
    /// reads no meaning into it.
    ViewId,
    check_id
);
identifier!(
    /// A remote request: unique per request and never reused (RC-17).
    RequestId,
    check_id
);
identifier!(
    /// An approval.
    ApprovalId,
    check_id
);
identifier!(
    /// A permission grant or a remote grant.
    GrantId,
    check_id
);
identifier!(
    /// An import transaction (B22).
    ImportId,
    check_id
);
identifier!(
    /// A model handoff (B23).
    HandoffId,
    check_id
);
identifier!(
    /// A transfer of a session's execution to another device (A7).
    TransferId,
    check_id
);
identifier!(
    /// An evidence record: a decision, a verification run or a closure record.
    EvidenceId,
    check_id
);
identifier!(
    /// A synchronization conflict.
    ConflictId,
    check_id
);
identifier!(
    /// One content change of a replica, the unit synchronization applies once (A6).
    ChangeId,
    check_id
);
identifier!(
    /// An editor extension, which is a principal of its own (AD-10).
    ExtensionId,
    check_id
);
identifier!(
    /// A configured provider.
    ProviderId,
    check_id
);
identifier!(
    /// A model as its provider names it, such as `claude-opus-5-5` or `meta-llama/Llama-3.1-8B`.
    ModelId,
    check_model_id
);
identifier!(
    /// An operation of the closed remote-operation registry (RC-09), defined by TASK-007.
    OperationId,
    check_key
);
identifier!(
    /// A capability of the closed remote-grant registry, such as `app_launch` or `agent_control`
    /// (`type.sec_grant`).
    GrantCapability,
    check_key
);
identifier!(
    /// A capability an execution host offers, such as a shell, Git or a test backend (A4). These
    /// are separate from model capabilities (provider_and_agent_model v0.2).
    HostCapability,
    check_key
);
identifier!(
    /// A tool action of the closed action registry that the permission engine (TASK-022) defines.
    ActionId,
    check_key
);
identifier!(
    /// The key of an application in a device's launch allowlist (RC-11); never a command line.
    AllowlistKey,
    check_key
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_identifiers_accept_their_alphabet_only() {
        for good in ["a", "W-01", "dev_1.local:7", "0f9e"] {
            assert!(WorkspaceId::new(good).is_ok(), "{good}");
        }
        for (bad, kind) in [
            ("", ErrorKind::Empty),
            ("-leading", ErrorKind::InvalidCharacter),
            ("with space", ErrorKind::InvalidCharacter),
            ("slash/inside", ErrorKind::InvalidCharacter),
            ("caf\u{e9}", ErrorKind::InvalidCharacter),
            ("dev\u{0430}ce", ErrorKind::InvalidCharacter),
            ("nul\0", ErrorKind::InvalidCharacter),
        ] {
            assert_eq!(WorkspaceId::new(bad).unwrap_err().kind, kind, "{bad:?}");
        }
        assert_eq!(
            DeviceId::new("a".repeat(MAX_ID_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::TooLong
        );
        assert!(DeviceId::new("a".repeat(MAX_ID_BYTES)).is_ok());
    }

    #[test]
    fn identifiers_compare_byte_for_byte_without_case_folding() {
        assert_ne!(
            DeviceId::new("Phone").unwrap(),
            DeviceId::new("phone").unwrap()
        );
    }

    #[test]
    fn model_identifiers_allow_provider_naming() {
        for good in [
            "claude-opus-5-5",
            "meta-llama/Llama-3.1-8B",
            "llama3.1:8b",
            "model@2026-10-01",
        ] {
            assert!(ModelId::new(good).is_ok(), "{good}");
        }
        assert!(ModelId::new("../escape").is_err());
        assert!(ModelId::new("two words").is_err());
    }

    #[test]
    fn registry_keys_are_lowercase_snake_case() {
        assert!(OperationId::new("agent_pause").is_ok());
        assert!(AllowlistKey::new("hoptodesk").is_ok());
        for bad in ["AgentPause", "agent-pause", "_x", "9lives", "rm -rf /", ""] {
            assert!(OperationId::new(bad).is_err(), "{bad:?}");
        }
        assert_eq!(
            GrantCapability::new("a".repeat(MAX_KEY_BYTES + 1))
                .unwrap_err()
                .kind,
            ErrorKind::TooLong
        );
    }

    #[test]
    fn decoding_rejects_what_construction_rejects() {
        assert!(serde_json::from_str::<DeviceId>("\"dev 1\"").is_err());
        assert!(serde_json::from_str::<DeviceId>("42").is_err());
        let id: DeviceId = serde_json::from_str("\"dev-1\"").unwrap();
        assert_eq!(serde_json::to_string(&id).unwrap(), "\"dev-1\"");
    }
}
