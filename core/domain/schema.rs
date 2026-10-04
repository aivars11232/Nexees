//! Versioned records, and how each kind of record may travel between devices.
//!
//! Every record that a host stores or sends implements [`Record`]: a schema name that is unique
//! across Nexees, a schema version, and a [`SyncPolicy`]. [`Versioned`] carries a record together
//! with its schema name and version and refuses anything else on decoding: another schema, an
//! unsupported version or an unknown field. An older version is migrated by the store (TASK-008)
//! before it is decoded, never guessed.
//!
//! The sync policies keep the three planes apart (A9, C22, AD-05, AD-06):
//!
//! - **control:** grants, approvals and remote requests are decided on and for one device and
//!   never travel as synchronized records;
//! - **sync:** workspace content and the portable records a user selected;
//! - **UI focus:** what a client shows, which never travels and never targets anything.
//!
//! A synchronization layer can refuse a record that must not travel at compile time, for example
//! with `const { assert!(T::SYNC.travels()) }` in a generic function.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::errors::{DomainError, ErrorKind};

/// How a record may travel between devices: the `sync_policy` values of the architecture's
/// state catalogue (`docs/architecture/state.lcl.txt`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncPolicy {
    /// Content the user selected for synchronization, reconciled with revisions and conflicts.
    SelectedContent,
    /// A record that may be copied to another device as a record. A copy changes no state on
    /// the receiving device by itself, such as a task's completion (B20).
    PortableRecord,
    /// A preference that travels only when the user explicitly asks.
    ExplicitPreference,
    /// Shown on another device as a last-observed view, never as live or authoritative state.
    ViewOnly,
    /// Never leaves the device that holds it.
    Never,
}

impl SyncPolicy {
    /// Whether a record with this policy may leave its device at all. How it travels, as selected
    /// content, a portable record, an explicit preference or a last-observed view, is the policy.
    pub const fn travels(self) -> bool {
        !matches!(self, Self::Never)
    }
}

/// A record that a host stores or sends.
pub trait Record: Serialize + DeserializeOwned {
    /// The schema name, unique across Nexees: `nexees.<module>.<record>`.
    const SCHEMA: &'static str;
    /// The schema version this crate reads and writes.
    const VERSION: u32;
    /// How the record may travel between devices.
    const SYNC: SyncPolicy;
}

/// A record with its schema name and version, as hosts store and send it.
///
/// It is written as `{"schema": ..., "version": ..., "record": {...}}`. Decoding fails unless
/// the schema name and version are exactly those of `T`, and unless the record itself passes
/// `T`'s validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Versioned<T: Record>(pub T);

#[derive(Serialize)]
struct Header<'a, T> {
    schema: &'static str,
    version: u32,
    record: &'a T,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Body<T> {
    schema: String,
    version: u32,
    record: T,
}

impl<T: Record> Serialize for Versioned<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Header {
            schema: T::SCHEMA,
            version: T::VERSION,
            record: &self.0,
        }
        .serialize(serializer)
    }
}

impl<'de, T: Record> Deserialize<'de> for Versioned<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let body = Body::<T>::deserialize(deserializer)?;
        if body.schema != T::SCHEMA || body.version != T::VERSION {
            return Err(serde::de::Error::custom(DomainError::new(
                T::SCHEMA,
                ErrorKind::UnsupportedSchema,
            )));
        }
        Ok(Self(body.record))
    }
}

/// Defines a record that is valid only as a whole: its fields are public in a plain `Fields`
/// struct, and the record exists only after [`Validate::validate`] accepted them. Construction
/// (`TryFrom`) and decoding run the same validation; the record is immutable afterwards.
macro_rules! validated_record {
    ($(#[$doc:meta])* $name:ident, $fields:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name($fields);

        impl $name {
            /// The validated fields.
            pub fn fields(&self) -> &$fields {
                &self.0
            }

            /// The fields, to build a changed record from; the change is validated again.
            pub fn into_fields(self) -> $fields {
                self.0
            }
        }

        impl TryFrom<$fields> for $name {
            type Error = $crate::errors::DomainError;

            fn try_from(fields: $fields) -> Result<Self, $crate::errors::DomainError> {
                $crate::schema::Validate::validate(&fields)?;
                Ok(Self(fields))
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                let fields = <$fields as ::serde::Deserialize>::deserialize(deserializer)?;
                Self::try_from(fields).map_err(::serde::de::Error::custom)
            }
        }
    };
}
pub(crate) use validated_record;

/// The rules a record's fields must satisfy together.
pub trait Validate {
    /// Checks the fields; the first problem found is returned.
    fn validate(&self) -> Result<(), DomainError>;
}

/// Fails with [`ErrorKind::Duplicate`] on `field` if `items` contains an entry twice.
pub(crate) fn ensure_unique<T: Ord>(items: &[T], field: &'static str) -> Result<(), DomainError> {
    let mut sorted: Vec<&T> = items.iter().collect();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        Err(DomainError::new(field, ErrorKind::Duplicate))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::{Approval, PermissionGrant, RemoteGrant};
    use crate::client::ClientState;
    use crate::device::{HostCapabilities, PeerDevice};
    use crate::handoff::Handoff;
    use crate::import::ImportTransaction;
    use crate::lcl::{Adoption, SpecRevisionRecord};
    use crate::session::{AgentSession, ExecutionTransfer};
    use crate::task::{Evidence, Task};
    use crate::workspace::{Conflict, Replica, WorkspaceRecord};

    /// Every record type of the crate with its schema name and sync policy.
    fn records() -> Vec<(&'static str, u32, SyncPolicy)> {
        fn entry<T: Record>() -> (&'static str, u32, SyncPolicy) {
            (T::SCHEMA, T::VERSION, T::SYNC)
        }
        vec![
            entry::<WorkspaceRecord>(),
            entry::<Replica>(),
            entry::<Conflict>(),
            entry::<PeerDevice>(),
            entry::<HostCapabilities>(),
            entry::<ClientState>(),
            entry::<AgentSession>(),
            entry::<ExecutionTransfer>(),
            entry::<Task>(),
            entry::<Evidence>(),
            entry::<SpecRevisionRecord>(),
            entry::<Adoption>(),
            entry::<ImportTransaction>(),
            entry::<Handoff>(),
            entry::<PermissionGrant>(),
            entry::<RemoteGrant>(),
            entry::<Approval>(),
        ]
    }

    #[test]
    fn every_record_has_its_own_schema_so_no_state_is_stored_twice() {
        let records = records();
        let mut names: Vec<&str> = records.iter().map(|r| r.0).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(
            names.len(),
            records.len(),
            "two record types share a schema name"
        );
        for (name, version, _) in &records {
            assert!(
                name.starts_with("nexees.") && name.split('.').count() == 3,
                "{name}"
            );
            assert_eq!(*version, 1, "{name}");
        }
    }

    #[test]
    fn sync_policies_follow_the_architecture_state_catalogue() {
        let travels: Vec<&str> = records()
            .into_iter()
            .filter(|r| r.2.travels())
            .map(|r| r.0)
            .collect();
        // ST-WORKSPACE's portable part and ST-HISTORY are portable records; everything else of
        // this crate stays on its device: authority, sessions, tasks, revisions, imports,
        // handoffs, trust, capabilities, conflicts, roots and UI focus (state.lcl.txt).
        assert_eq!(
            travels,
            ["nexees.workspace.workspace", "nexees.task.evidence"]
        );
    }
}
