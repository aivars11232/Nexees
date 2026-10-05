//! The state store: each host's one transactional store (SS-STATE, IF-STORE, A3, R17).
//!
//! - **One process.** The store is a SQLite database in WAL mode with full sync, opened in
//!   exclusive locking mode. While a host holds it, no other process can open it, which is what
//!   makes the restart rules safe to apply when it opens.
//! - **Durable before saved.** [`StateStore::write`] runs a closure in one transaction and
//!   returns only once the commit is on disk. An error, a panic or a crash before the commit
//!   leaves nothing of it behind (A3).
//! - **Versioned records, checked on read.** Each record is stored with its schema name and
//!   version. It is read back only when it decodes strictly as its type, at its type's version,
//!   under its own key; anything else is an integrity error, never a guess.
//! - **No secrets.** The records are domain types, which hold no secret material (R17). As a
//!   second line, the store refuses a body that has a field named like a secret.
//! - **The outbox** keeps each peer replica's outgoing content changes, in order, until the
//!   peer acknowledges them. It holds [`ContentChange`]s only: a request, an approval or a
//!   grant has no way into it (C22, RC-17).
//! - **Recovery metadata.** Opening records whether the previous session closed and applies the
//!   restart rules of the operation journal and the request ledger. It checks the whole store
//!   after a migration or an interrupted session. Resuming work is TASK-038's.

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use nexees_domain::authority::{Approval, PermissionGrant, RemoteGrant};
use nexees_domain::changes::ContentChange;
use nexees_domain::client::{ClientLayout, ClientState};
use nexees_domain::device::{HostCapabilities, PeerDevice};
use nexees_domain::errors::{DomainError, ErrorKind};
use nexees_domain::handoff::Handoff;
use nexees_domain::ids::{ChangeId, DeviceId};
use nexees_domain::import::ImportTransaction;
use nexees_domain::lcl::{Adoption, SpecRevisionRecord};
use nexees_domain::requests::{LedgerEntry, PendingRequest};
use nexees_domain::schema::{Record, Versioned};
use nexees_domain::session::{AgentSession, ExecutionTransfer};
use nexees_domain::task::{Evidence, Task};
use nexees_domain::time::Timestamp;
use nexees_domain::workspace::{Conflict, Replica, WorkspaceRecord};
use rusqlite::{
    Connection, ErrorCode, OptionalExtension, Transaction, TransactionBehavior, params,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use serde_json::error::Category;

use crate::migrations::migration_registry::{MIGRATIONS, Migration, Plan, plan, upgrade};
use crate::operation_journal::{Interrupted, mark_interrupted, verify_journal};

/// How long opening waits for another process to release the store before it reports
/// [`StateError::InUse`].
const OPEN_WAIT: Duration = Duration::from_secs(2);

/// Field names that only secret material has; a body with one is refused (R17).
const SECRET_FIELDS: &[&str] = &[
    "access_token",
    "api_key",
    "auth_token",
    "bearer_token",
    "client_secret",
    "credential",
    "credentials",
    "passphrase",
    "password",
    "private_key",
    "recovery_code",
    "recovery_codes",
    "refresh_token",
    "secret",
    "secret_key",
];

/// Endings of field names that only secret material has.
const SECRET_SUFFIXES: &[&str] = &["_api_key", "_password", "_private_key", "_secret"];

/// Why the store refused or failed an operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    /// SQLite failed. The text is SQLite's message, which names tables and columns, not values.
    Database(String),
    /// Another process holds the store.
    InUse,
    /// The store was written by a newer build; it was left untouched.
    NewerSchema {
        /// The store's schema version.
        found: u32,
        /// This build's schema version.
        supported: u32,
    },
    /// A migration step failed or was refused; the store kept its previous version.
    Migration {
        /// The step.
        version: u32,
        /// Why.
        reason: String,
    },
    /// A stored row is not what it claims to be.
    Integrity {
        /// Its schema, or `sqlite` for SQLite's own check.
        schema: String,
        /// Its key.
        id: String,
        /// What is wrong, without the row's content.
        reason: String,
    },
    /// A record failed validation, or a change of its state is not allowed.
    Domain(DomainError),
    /// A record has a field named like a secret (R17); nothing was written.
    SecretField {
        /// The record's schema.
        schema: String,
        /// The field.
        field: String,
    },
    /// The key is taken; an identifier is never reused.
    Duplicate {
        /// The schema.
        schema: String,
        /// The key.
        id: String,
    },
    /// Nothing is stored under the key.
    Missing {
        /// The schema.
        schema: String,
        /// The key.
        id: String,
    },
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(message) => write!(f, "state store: {message}"),
            Self::InUse => f.write_str("the state store is held by another process"),
            Self::NewerSchema { found, supported } => write!(
                f,
                "the state store is at schema {found}, newer than this build's {supported}"
            ),
            Self::Migration { version, reason } => {
                write!(f, "migration to schema {version} refused: {reason}")
            }
            Self::Integrity { schema, id, reason } => {
                write!(
                    f,
                    "stored {schema} {id:?} failed its integrity check: {reason}"
                )
            }
            Self::Domain(error) => write!(f, "{error}"),
            Self::SecretField { schema, field } => {
                write!(f, "{schema} has a field named like a secret: {field}")
            }
            Self::Duplicate { schema, id } => write!(f, "{schema} {id:?} already exists"),
            Self::Missing { schema, id } => write!(f, "no {schema} {id:?} is stored"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<rusqlite::Error> for StateError {
    fn from(error: rusqlite::Error) -> Self {
        match error.sqlite_error_code() {
            Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) => Self::InUse,
            _ => Self::Database(error.to_string()),
        }
    }
}

impl From<DomainError> for StateError {
    fn from(error: DomainError) -> Self {
        Self::Domain(error)
    }
}

/// A record the store keeps under a stable key within its schema.
pub trait Stored: Record + Serialize + DeserializeOwned {
    /// The record's key: its identifier, or its identifiers joined by `/`, which no identifier
    /// contains.
    fn key(&self) -> String;
}

/// A record schema this build stores, with the check that decodes a stored body of it.
#[derive(Clone, Copy)]
pub(crate) struct KnownRecord {
    pub(crate) schema: &'static str,
    pub(crate) version: u32,
    /// Decodes a stored body and returns its key, or why it is not a valid record.
    pub(crate) decode: fn(&str) -> Result<String, String>,
}

fn decode_key<T: Stored>(body: &str) -> Result<String, String> {
    serde_json::from_str::<Versioned<T>>(body)
        .map(|record| record.0.key())
        .map_err(|error| decode_reason(&error))
}

/// Implements [`Stored`] for each record and lists them in [`RECORDS`].
macro_rules! stored {
    ($($record:ty => |$r:ident| $key:expr;)*) => {
        $(impl Stored for $record {
            fn key(&self) -> String {
                let $r = self;
                $key
            }
        })*

        /// Every record schema this build stores. Changing the list, or a listed record's
        /// version, needs a migration (see the migration registry).
        pub(crate) const RECORDS: &[KnownRecord] = &[$(KnownRecord {
            schema: <$record as Record>::SCHEMA,
            version: <$record as Record>::VERSION,
            decode: decode_key::<$record>,
        }),*];
    };
}

stored! {
    WorkspaceRecord => |r| r.workspace_id.as_str().into();
    Replica => |r| join(&[r.fields().workspace_id.as_str(), r.fields().device_id.as_str()]);
    Conflict => |r| r.fields().conflict_id.as_str().into();
    PeerDevice => |r| r.fields().device_id.as_str().into();
    HostCapabilities => |r| r.fields().device_id.as_str().into();
    ClientState => |r| r.client_id.as_str().into();
    ClientLayout => |r| r.client_id.as_str().into();
    PermissionGrant => |r| r.fields().grant_id.as_str().into();
    RemoteGrant => |r| r.fields().grant_id.as_str().into();
    Approval => |r| r.fields().approval_id.as_str().into();
    LedgerEntry => |r| r.fields().request_id.as_str().into();
    PendingRequest => |r| r.fields().request_id.as_str().into();
    AgentSession => |r| r.fields().session_id.as_str().into();
    ExecutionTransfer => |r| r.fields().transfer_id.as_str().into();
    Handoff => |r| r.fields().handoff_id.as_str().into();
    Task => |r| r.fields().task_id.as_str().into();
    Evidence => |r| r.fields().evidence_id.as_str().into();
    SpecRevisionRecord => |r| {
        let revision = &r.fields().revision;
        join(&[revision.workspace_id.as_str(), revision.revision.as_str()])
    };
    Adoption => |r| {
        let (session, revision) = (&r.fields().session_id, &r.fields().revision);
        join(&[session.as_str(), revision.workspace_id.as_str(), revision.revision.as_str()])
    };
    ImportTransaction => |r| r.fields().import_id.as_str().into();
}

fn join(parts: &[&str]) -> String {
    parts.join("/")
}

/// What opening a store found and did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// The store's schema version now.
    pub schema_version: u32,
    /// The version it was migrated from; 0 for a new store, `None` when it was current.
    pub migrated_from: Option<u32>,
    /// How the previous session ended.
    pub previous_session: PreviousSession,
    /// The effects and remote requests the restart rules turned outcome-unknown.
    pub interrupted: Interrupted,
}

/// How the previous session of a store ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviousSession {
    /// There was none: the store is new.
    None,
    /// It was closed.
    Closed,
    /// It ended without closing: the host stopped or crashed.
    Interrupted,
}

impl PreviousSession {
    /// The name the recovery metadata records.
    fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Closed => "closed",
            Self::Interrupted => "interrupted",
        }
    }
}

/// A content change waiting for a peer replica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxEntry {
    /// Its position: changes go out in this order.
    pub seq: i64,
    /// The change.
    pub change: ContentChange,
    /// When it was queued.
    pub enqueued_at: Timestamp,
}

/// A host's state store, held by this process until it is closed or dropped.
pub struct StateStore {
    conn: Connection,
    path: PathBuf,
}

impl StateStore {
    /// Opens the store at `path`, creating it when it does not exist, and migrates it to this
    /// build's schema. It applies the restart rules, so it must be opened once, by the host.
    pub fn open(path: &Path, now: Timestamp) -> Result<(Self, Opened), StateError> {
        Self::open_with(path, now, MIGRATIONS)
    }

    pub(crate) fn open_with(
        path: &Path,
        now: Timestamp,
        steps: &[Migration],
    ) -> Result<(Self, Opened), StateError> {
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(OPEN_WAIT)?;
        // Set before the first access, so the lock is held from it until the store is closed.
        conn.pragma_update(None, "locking_mode", "EXCLUSIVE")?;
        // A newer store is refused before anything, even its journal mode, is written.
        let found: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        if let Plan::Newer { found, supported } = plan(found, steps) {
            return Err(StateError::NewerSchema { found, supported });
        }
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // FULL keeps a commit through power loss, not only through a process crash.
        conn.pragma_update(None, "synchronous", "FULL")?;
        // Nothing stored in the schema runs functions on the store's behalf.
        conn.pragma_update(None, "trusted_schema", "OFF")?;
        let opened = {
            let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let migrated_from = match plan(found, steps) {
                Plan::Upgrade { from, .. } => {
                    upgrade(&tx, steps, from, now)?;
                    Some(from)
                }
                Plan::Current | Plan::Newer { .. } => None,
            };
            let previous_session = match meta(&tx, "session")?.as_deref() {
                None => PreviousSession::None,
                Some("closed") => PreviousSession::Closed,
                Some(_) => PreviousSession::Interrupted,
            };
            let interrupted = mark_interrupted(&tx, now)?;
            if migrated_from.is_none() && previous_session == PreviousSession::Interrupted {
                // After a crash, check everything; a migration has checked already.
                verify(&tx)?;
            }
            let summary = serde_json::json!({
                "opened_at": millis(now)?,
                "previous_session": previous_session.name(),
                "migrated_from": migrated_from,
                "interrupted_effects": interrupted.effects,
                "interrupted_requests": interrupted.requests,
            });
            set_meta(&tx, "session", "open")?;
            set_meta(&tx, "last_open", &summary.to_string())?;
            tx.commit()?;
            Opened {
                schema_version: steps.last().map_or(0, |step| step.version),
                migrated_from,
                previous_session,
                interrupted,
            }
        };
        let store = Self {
            conn,
            path: path.to_owned(),
        };
        Ok((store, opened))
    }

    /// Closes the store and records that the session ended in order. A host that stops
    /// without closing leaves a session the next open treats as interrupted.
    pub fn close(mut self, now: Timestamp) -> Result<(), StateError> {
        {
            let tx = self.conn.transaction()?;
            set_meta(&tx, "session", "closed")?;
            set_meta(&tx, "closed_at", &millis(now)?.to_string())?;
            tx.commit()?;
        }
        self.conn.close().map_err(|(_, error)| error.into())
    }

    /// Runs `change` in one transaction, at `now`, and commits it. Returns only once the commit
    /// is durable; when `change` fails, nothing of it is kept.
    pub fn write<R>(
        &mut self,
        now: Timestamp,
        change: impl FnOnce(&mut Write<'_>) -> Result<R, StateError>,
    ) -> Result<R, StateError> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut write = Write { tx, now };
        let result = change(&mut write)?;
        write.tx.commit()?;
        Ok(result)
    }

    /// The record of type `T` under `key`, checked as it is read.
    pub fn get<T: Stored>(&self, key: &str) -> Result<Option<T>, StateError> {
        read(&self.conn, key)
    }

    /// Every record of type `T`, by key.
    pub fn all<T: Stored>(&self) -> Result<Vec<T>, StateError> {
        read_where(&self.conn, "1")
    }

    /// The content changes waiting for `destination`'s replica, in order.
    pub fn outbox(&self, destination: &DeviceId) -> Result<Vec<OutboxEntry>, StateError> {
        let mut statement = self.conn.prepare(
            "SELECT seq, change_id, body, enqueued_at FROM outbox
             WHERE destination_device_id = ?1 ORDER BY seq",
        )?;
        let rows = statement.query_map([destination.as_str()], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        let mut entries = Vec::new();
        for row in rows {
            let (seq, change_id, body, enqueued_at) = row?;
            let change = decode_change(&change_id, &body)?;
            let enqueued_at = u64::try_from(enqueued_at).map_err(|_| {
                integrity(ContentChange::SCHEMA, &change_id, "a negative time".into())
            })?;
            entries.push(OutboxEntry {
                seq,
                change,
                enqueued_at: Timestamp::from_unix_millis(enqueued_at),
            });
        }
        Ok(entries)
    }

    /// Fails unless SQLite finds the database sound and every stored row decodes, at its
    /// schema's version, under its own key.
    pub fn check_integrity(&self) -> Result<(), StateError> {
        verify(&self.conn)
    }

    /// This device's own identity, as its host recorded it when it first ran; absent before
    /// that. It never changes ([`Write::set_device_id`]), so whatever names the device stays
    /// valid. A stored value that is not a valid identity is an integrity error.
    pub fn device_id(&self) -> Result<Option<DeviceId>, StateError> {
        meta(&self.conn, DEVICE_ID)?
            .map(|text| {
                DeviceId::new(text)
                    .map_err(|_| integrity("store_meta", DEVICE_ID, "not a device identity".into()))
            })
            .transpose()
    }

    /// The files the store owns: the database and the files SQLite keeps beside it. Uninstalling
    /// removes these and nothing else; workspace content lives under the workspace roots.
    pub fn owned_files(&self) -> Vec<PathBuf> {
        ["", "-wal", "-shm", "-journal"]
            .iter()
            .map(|suffix| {
                let mut name = OsString::from(self.path.as_os_str());
                name.push(suffix);
                PathBuf::from(name)
            })
            .collect()
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }
}

/// One open transaction of [`StateStore::write`].
pub struct Write<'s> {
    tx: Transaction<'s>,
    now: Timestamp,
}

impl<'s> Write<'s> {
    /// The time this transaction records its changes at.
    pub fn now(&self) -> Timestamp {
        self.now
    }

    /// The record of type `T` under `key`, as this transaction sees it.
    pub fn get<T: Stored>(&self, key: &str) -> Result<Option<T>, StateError> {
        read(&self.tx, key)
    }

    /// Every record of type `T`, as this transaction sees them.
    pub fn all<T: Stored>(&self) -> Result<Vec<T>, StateError> {
        read_where(&self.tx, "1")
    }

    /// Stores `record`, replacing the record under its key if there is one.
    pub fn put<T: Stored>(&mut self, record: &T) -> Result<(), StateError> {
        store(&self.tx, record, self.now, true)
    }

    /// Stores a new record. Fails when its key is taken, so an identifier is never reused for
    /// another record, such as an old task ID for a new task.
    pub fn insert<T: Stored>(&mut self, record: &T) -> Result<(), StateError> {
        store(&self.tx, record, self.now, false)
    }

    /// Queues `change` for `destination`'s replica and returns its position. Queuing the same
    /// change again changes nothing; another change under a queued change ID is refused.
    pub fn enqueue(
        &mut self,
        destination: &DeviceId,
        change: &ContentChange,
    ) -> Result<i64, StateError> {
        let body = encode(change)?;
        let change_id = change.fields().change_id.as_str();
        let queued = self
            .tx
            .query_row(
                "SELECT seq, body FROM outbox WHERE destination_device_id = ?1 AND change_id = ?2",
                params![destination.as_str(), change_id],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()?;
        match queued {
            Some((seq, queued)) if queued == body => Ok(seq),
            Some(_) => Err(StateError::Duplicate {
                schema: ContentChange::SCHEMA.into(),
                id: join(&[destination.as_str(), change_id]),
            }),
            None => {
                self.tx.execute(
                    "INSERT INTO outbox (destination_device_id, change_id, body, enqueued_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![destination.as_str(), change_id, body, millis(self.now)?],
                )?;
                Ok(self.tx.last_insert_rowid())
            }
        }
    }

    /// Removes `change_id` from `destination`'s queue once that replica acknowledged it.
    /// Returns whether it was queued.
    pub fn acknowledge(
        &mut self,
        destination: &DeviceId,
        change_id: &ChangeId,
    ) -> Result<bool, StateError> {
        let removed = self.tx.execute(
            "DELETE FROM outbox WHERE destination_device_id = ?1 AND change_id = ?2",
            params![destination.as_str(), change_id.as_str()],
        )?;
        Ok(removed > 0)
    }

    /// Records `id` as this device's own identity. Only the first one is kept: recording the
    /// same one again changes nothing, and a different one is refused as a duplicate, because
    /// statuses and pairings name the device by it.
    pub fn set_device_id(&mut self, id: &DeviceId) -> Result<(), StateError> {
        match meta(&self.tx, DEVICE_ID)? {
            None => set_meta(&self.tx, DEVICE_ID, id.as_str()),
            Some(recorded) if recorded == id.as_str() => Ok(()),
            Some(_) => Err(StateError::Duplicate {
                schema: "store_meta".into(),
                id: DEVICE_ID.into(),
            }),
        }
    }

    pub(crate) fn tx(&self) -> &Transaction<'s> {
        &self.tx
    }
}

/// The store metadata key of this device's own identity.
const DEVICE_ID: &str = "device_id";

/// A timestamp as SQLite's integer.
pub(crate) fn millis(at: Timestamp) -> Result<i64, StateError> {
    i64::try_from(at.unix_millis())
        .map_err(|_| DomainError::new("Timestamp", ErrorKind::OutOfRange).into())
}

/// The stored form of `record`: its schema, version and fields as compact JSON with sorted
/// keys. Refused when a field is named like a secret.
pub(crate) fn encode<T: Record + Serialize>(record: &T) -> Result<String, StateError> {
    let fields = serde_json::to_value(record).map_err(|error| StateError::Integrity {
        schema: T::SCHEMA.into(),
        id: String::new(),
        reason: decode_reason(&error),
    })?;
    refuse_secrets(T::SCHEMA, &fields)?;
    let body = serde_json::json!({ "schema": T::SCHEMA, "version": T::VERSION, "record": fields });
    Ok(body.to_string())
}

/// A JSON error as its kind and position. serde_json's own message can quote the input, such as
/// a string of the wrong type, and that must not reach a log.
pub(crate) fn decode_reason(error: &serde_json::Error) -> String {
    let kind = match error.classify() {
        Category::Io => "unreadable",
        Category::Syntax => "invalid JSON",
        Category::Data => "invalid content",
        Category::Eof => "truncated",
    };
    format!("{kind} at line {}, column {}", error.line(), error.column())
}

fn refuse_secrets(schema: &str, value: &Value) -> Result<(), StateError> {
    match value {
        Value::Object(fields) => {
            for (name, field) in fields {
                let lower = name.to_ascii_lowercase();
                let secret = SECRET_FIELDS.contains(&lower.as_str())
                    || SECRET_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix));
                if secret {
                    return Err(StateError::SecretField {
                        schema: schema.into(),
                        field: name.clone(),
                    });
                }
                refuse_secrets(schema, field)?;
            }
            Ok(())
        }
        Value::Array(items) => items
            .iter()
            .try_for_each(|item| refuse_secrets(schema, item)),
        _ => Ok(()),
    }
}

pub(crate) fn store<T: Stored>(
    conn: &Connection,
    record: &T,
    now: Timestamp,
    replace: bool,
) -> Result<(), StateError> {
    let key = record.key();
    let body = encode(record)?;
    if !replace {
        let taken = conn
            .query_row(
                "SELECT 1 FROM records WHERE schema = ?1 AND id = ?2",
                params![T::SCHEMA, key],
                |_| Ok(()),
            )
            .optional()?;
        if taken.is_some() {
            return Err(StateError::Duplicate {
                schema: T::SCHEMA.into(),
                id: key,
            });
        }
    }
    conn.execute(
        "INSERT INTO records (schema, id, version, body, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT (schema, id) DO UPDATE
             SET version = excluded.version, body = excluded.body,
                 updated_at = excluded.updated_at",
        params![T::SCHEMA, key, T::VERSION, body, millis(now)?],
    )?;
    Ok(())
}

fn read<T: Stored>(conn: &Connection, key: &str) -> Result<Option<T>, StateError> {
    let row = conn
        .query_row(
            "SELECT version, body FROM records WHERE schema = ?1 AND id = ?2",
            params![T::SCHEMA, key],
            |row| Ok((row.get::<_, u32>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    row.map(|(version, body)| check::<T>(key, version, &body))
        .transpose()
}

/// The records of `T` whose rows match `condition`, an SQL expression this crate writes.
pub(crate) fn read_where<T: Stored>(
    conn: &Connection,
    condition: &str,
) -> Result<Vec<T>, StateError> {
    let mut statement = conn.prepare(&format!(
        "SELECT id, version, body FROM records WHERE schema = ?1 AND ({condition}) ORDER BY id"
    ))?;
    let rows = statement.query_map([T::SCHEMA], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, u32>(1)?,
            row.get::<_, String>(2)?,
        ))
    })?;
    let mut records = Vec::new();
    for row in rows {
        let (key, version, body) = row?;
        records.push(check::<T>(&key, version, &body)?);
    }
    Ok(records)
}

fn check<T: Stored>(key: &str, version: u32, body: &str) -> Result<T, StateError> {
    let fail = |reason: String| integrity(T::SCHEMA, key, reason);
    if version != T::VERSION {
        return Err(fail(format!("version {version}, not {}", T::VERSION)));
    }
    let record = serde_json::from_str::<Versioned<T>>(body)
        .map_err(|error| fail(decode_reason(&error)))?
        .0;
    if record.key() != key {
        return Err(fail("it is stored under another key".into()));
    }
    Ok(record)
}

fn decode_change(change_id: &str, body: &str) -> Result<ContentChange, StateError> {
    let fail = |reason: String| integrity(ContentChange::SCHEMA, change_id, reason);
    let change = serde_json::from_str::<Versioned<ContentChange>>(body)
        .map_err(|error| fail(decode_reason(&error)))?
        .0;
    if change.fields().change_id.as_str() != change_id {
        return Err(fail("it is queued under another change ID".into()));
    }
    Ok(change)
}

/// Fails unless SQLite finds the database sound and every stored row decodes, at its schema's
/// version, under its own key.
pub(crate) fn verify(conn: &Connection) -> Result<(), StateError> {
    let sqlite: String = conn.pragma_query_value(None, "integrity_check", |row| row.get(0))?;
    if sqlite != "ok" {
        return Err(integrity("sqlite", "", sqlite));
    }
    let mut statement = conn.prepare("SELECT schema, id, version, body FROM records")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, u32>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (schema, key, version, body) = row?;
        let Some(known) = RECORDS.iter().find(|known| known.schema == schema) else {
            return Err(integrity(
                &schema,
                &key,
                "a schema this build does not store".into(),
            ));
        };
        if version != known.version {
            let reason = format!("version {version}, not {}", known.version);
            return Err(integrity(&schema, &key, reason));
        }
        if (known.decode)(&body).map_err(|reason| integrity(&schema, &key, reason))? != key {
            return Err(integrity(
                &schema,
                &key,
                "it is stored under another key".into(),
            ));
        }
    }
    let mut statement = conn.prepare("SELECT change_id, body FROM outbox")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (change_id, body) = row?;
        decode_change(&change_id, &body)?;
    }
    verify_journal(conn)
}

fn integrity(schema: &str, key: &str, reason: String) -> StateError {
    StateError::Integrity {
        schema: schema.into(),
        id: key.into(),
        reason,
    }
}

fn meta(conn: &Connection, key: &str) -> Result<Option<String>, StateError> {
    Ok(conn
        .query_row(
            "SELECT value FROM store_meta WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()?)
}

fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<(), StateError> {
    conn.execute(
        "INSERT INTO store_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::migrations::migration_registry::SCHEMA_VERSION;
    use crate::migrations::migration_registry::tests::then;
    use crate::operation_journal::{EffectState, JournalEntry, JournalEntryFields};
    use nexees_domain::authority::{ApprovalFields, Direction, RemoteGrantFields, ScopeEntry};
    use nexees_domain::changes::{ChangeKind, ContentChangeFields};
    use nexees_domain::client::{PanelLayout, PanelLayoutFields, PanelState};
    use nexees_domain::device::{DeviceKind, PeerDeviceFields, ProtocolVersions, TrustState};
    use nexees_domain::errors::Outcome;
    use nexees_domain::ids::{
        ApprovalId, GrantCapability, GrantId, ImportId, OperationId, RequestId, TaskId, UserId,
        WorkspaceId,
    };
    use nexees_domain::import::{ImportState, ImportTransactionFields, PathMapping, StagedFile};
    use nexees_domain::lcl::{RevisionOrigin, SpecRevisionRecordFields, SpecRevisionState};
    use nexees_domain::requests::{PendingRequestFields, PendingState, Reconnect};
    use nexees_domain::revision::{
        ContentHash, PermissionEpoch, RevisionId, SpecRevision, WorkspaceRevision,
    };
    use nexees_domain::task::{TaskFields, TaskStatus};
    use nexees_domain::text::{Label, Note};
    use nexees_domain::workspace::{
        ContentAvailability, DeviceRoot, RelativePath, ReplicaFields, WorkspaceKind,
    };

    /// The exit code a crash child uses at its crash point, so the parent knows it got there.
    const CRASHED: i32 = 86;

    pub(crate) fn at(ms: u64) -> Timestamp {
        Timestamp::from_unix_millis(ms)
    }

    /// A scratch folder for one test, removed when the test ends.
    pub(crate) struct Scratch(PathBuf);

    impl Scratch {
        pub(crate) fn new(name: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let unique = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "nexees-state-{}-{unique}-{name}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        pub(crate) fn path(&self) -> &Path {
            &self.0
        }

        pub(crate) fn db(&self) -> PathBuf {
            self.0.join("state.db")
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    pub(crate) fn hash(c: char) -> ContentHash {
        ContentHash::new(c.to_string().repeat(64)).unwrap()
    }

    fn revision(workspace: &str, revision: &str) -> WorkspaceRevision {
        WorkspaceRevision {
            workspace_id: WorkspaceId::new(workspace).unwrap(),
            revision: RevisionId::new(revision).unwrap(),
        }
    }

    pub(crate) fn workspace(id: &str) -> WorkspaceRecord {
        WorkspaceRecord {
            workspace_id: WorkspaceId::new(id).unwrap(),
            kind: WorkspaceKind::Code,
            name: Label::new(id).unwrap(),
        }
    }

    pub(crate) fn replica(workspace: &str, device: &str, root: &Path) -> Replica {
        Replica::try_from(ReplicaFields {
            workspace_id: WorkspaceId::new(workspace).unwrap(),
            device_id: DeviceId::new(device).unwrap(),
            root: Some(DeviceRoot::new(root.to_string_lossy()).unwrap()),
            availability: ContentAvailability::Local,
        })
        .unwrap()
    }

    pub(crate) fn grant(id: &str, epoch: u64, revoked: bool) -> RemoteGrant {
        RemoteGrant::try_from(RemoteGrantFields {
            grant_id: GrantId::new(id).unwrap(),
            destination_device_id: DeviceId::new("pc").unwrap(),
            peer_device_id: DeviceId::new("phone").unwrap(),
            peer_user_id: UserId::new("u1").unwrap(),
            direction: Direction::AndroidToDesktop,
            capability: GrantCapability::new("agent_control").unwrap(),
            scope: vec![ScopeEntry::Workspace {
                workspace_id: WorkspaceId::new("w").unwrap(),
            }],
            permission_epoch: PermissionEpoch::new(epoch),
            granted_by: UserId::new("u1").unwrap(),
            granted_at: at(100),
            expires_at: None,
            revoked,
        })
        .unwrap()
    }

    /// Every grant's next epoch: the stored part of stopping remote access (RC-07).
    pub(crate) fn bumped(grant: &RemoteGrant) -> RemoteGrant {
        let mut fields = grant.clone().into_fields();
        fields.permission_epoch = fields.permission_epoch.next().unwrap();
        RemoteGrant::try_from(fields).unwrap()
    }

    pub(crate) fn approval(id: &str, epoch: u64) -> Approval {
        Approval::try_from(ApprovalFields {
            approval_id: ApprovalId::new(id).unwrap(),
            request_id: RequestId::new("q1").unwrap(),
            request_hash: hash('e'),
            destination_device_id: DeviceId::new("pc").unwrap(),
            decided_by: UserId::new("u1").unwrap(),
            decided_at: at(1_000),
            expires_at: at(61_000),
            permission_epoch: PermissionEpoch::new(epoch),
            used: false,
        })
        .unwrap()
    }

    pub(crate) fn peer(id: &str, max_version: u32, revoked: bool) -> PeerDevice {
        PeerDevice::try_from(PeerDeviceFields {
            device_id: DeviceId::new(id).unwrap(),
            user_id: UserId::new("u1").unwrap(),
            kind: DeviceKind::Android,
            display_name: Label::new("Phone").unwrap(),
            trust: if revoked {
                TrustState::Revoked
            } else {
                TrustState::Paired
            },
            paired_at: at(100),
            revoked_at: revoked.then(|| at(200)),
            protocol_versions: ProtocolVersions {
                min: 1,
                max: max_version,
            },
        })
        .unwrap()
    }

    pub(crate) fn task(id: &str, status: TaskStatus) -> Task {
        Task::try_from(TaskFields {
            task_id: TaskId::new(id).unwrap(),
            workspace_id: WorkspaceId::new("w").unwrap(),
            objective: Note::new("Make the parser reject cycles.").unwrap(),
            status,
            dependencies: vec![],
            read_order: vec![],
            verification_requirements: vec![Note::new("cargo test -p parser").unwrap()],
            decisions: vec![],
            changed_files: vec![],
            verification_results: vec![],
            blocker: None,
            checkpoint: Some(revision("w", "r7")),
            assigned_session: None,
        })
        .unwrap()
    }

    pub(crate) fn draft(id: &str) -> SpecRevisionRecord {
        SpecRevisionRecord::try_from(SpecRevisionRecordFields {
            revision: SpecRevision {
                workspace_id: WorkspaceId::new("lcl-b").unwrap(),
                revision: RevisionId::new(id).unwrap(),
            },
            origin: RevisionOrigin::Edited,
            state: SpecRevisionState::Draft,
            validation: None,
            created_at: at(300),
        })
        .unwrap()
    }

    pub(crate) fn staged_import(id: &str) -> ImportTransaction {
        let path = |p: &str| RelativePath::new(p).unwrap();
        ImportTransaction::try_from(ImportTransactionFields {
            import_id: ImportId::new(id).unwrap(),
            target_device_id: DeviceId::new("phone").unwrap(),
            workspace_id: WorkspaceId::new("lcl-b").unwrap(),
            base_revision: revision("lcl-b", "r1"),
            staged: vec![StagedFile {
                path: path("pack/master.lcl.txt"),
                content: hash('c'),
            }],
            mapping: vec![PathMapping {
                source: path("pack/master.lcl.txt"),
                destination: path("master.lcl.txt"),
            }],
            state: ImportState::Staged,
            started_at: at(400),
        })
        .unwrap()
    }

    pub(crate) fn ledger(id: &str, state: Outcome) -> LedgerEntry {
        let admitted = LedgerEntry::admit(
            RequestId::new(id).unwrap(),
            hash('a'),
            DeviceId::new("phone").unwrap(),
            OperationId::new("app_launch").unwrap(),
            at(10),
        );
        match state {
            Outcome::Accepted => admitted,
            state => admitted.record(state, at(11)).unwrap(),
        }
    }

    pub(crate) fn pending(id: &str, expires_at: u64) -> PendingRequest {
        PendingRequest::try_from(PendingRequestFields {
            request_id: RequestId::new(id).unwrap(),
            request_hash: hash('a'),
            destination_device_id: DeviceId::new("pc").unwrap(),
            operation: OperationId::new("app_launch").unwrap(),
            expires_at: at(expires_at),
            state: PendingState::Unsent,
        })
        .unwrap()
    }

    pub(crate) fn change(id: &str, content: char) -> ContentChange {
        ContentChange::try_from(ContentChangeFields {
            change_id: ChangeId::new(id).unwrap(),
            workspace_id: WorkspaceId::new("w").unwrap(),
            path: RelativePath::new("notes.md").unwrap(),
            kind: ChangeKind::Write {
                content: hash(content),
                size: 5,
            },
            base: Some(hash('a')),
            origin_device_id: DeviceId::new("phone").unwrap(),
            origin_revision: revision("w", "r2"),
            recorded_at: at(5),
        })
        .unwrap()
    }

    pub(crate) fn effect(request: &str) -> JournalEntry {
        JournalEntry::new(JournalEntryFields {
            operation: OperationId::new("app_launch").unwrap(),
            request_id: Some(RequestId::new(request).unwrap()),
            workspace_id: None,
            agent_id: None,
            generation: None,
            state: EffectState::Started,
            started_at: at(12),
            settled_at: None,
        })
        .unwrap()
    }

    /// Every stored body as `(table and schema, key, body)`, sorted: what must survive a
    /// migration or a crash byte for byte.
    pub(crate) fn bodies(db: &Path) -> Vec<(String, String, String)> {
        let conn = Connection::open(db).unwrap();
        let mut statement = conn
            .prepare(
                "SELECT 'records ' || schema, id, body FROM records
                 UNION ALL SELECT 'outbox', destination_device_id || '/' || change_id, body
                     FROM outbox
                 UNION ALL SELECT 'journal', CAST(seq AS TEXT), body FROM journal
                 ORDER BY 1, 2",
            )
            .unwrap();
        let rows = statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap();
        rows.map(Result::unwrap).collect()
    }

    /// Runs `scenario` in a child process that stops dead at its crash point, as a killed or
    /// crashed host does: no destructor runs and no transaction is rolled back by the program.
    pub(crate) fn crash_in_child(scenario: &str, db: &Path) {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "state_store::tests::crash_child_entry_point",
                "--test-threads=1",
                "--nocapture",
            ])
            .env("NEXEES_STATE_CRASH", scenario)
            .env("NEXEES_STATE_CRASH_DB", db)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(CRASHED),
            "the {scenario} child did not reach its crash point: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    pub(crate) fn crash() -> ! {
        std::process::exit(CRASHED)
    }

    /// The test-binary entry point of the crash children. Outside a crash test it does nothing.
    #[test]
    fn crash_child_entry_point() {
        let (Ok(scenario), Some(db)) = (
            std::env::var("NEXEES_STATE_CRASH"),
            std::env::var_os("NEXEES_STATE_CRASH_DB"),
        ) else {
            return;
        };
        let db = PathBuf::from(db);
        if scenario == "migration" {
            let steps = then(crate::migrations::migration_registry::tests::CRASHING_NEXT);
            let _ = StateStore::open_with(&db, at(900), &steps);
            unreachable!("the step after this build's crashes");
        }
        let (mut store, _) = StateStore::open(&db, at(900)).unwrap();
        store
            .write(at(901), |w| {
                match scenario.as_str() {
                    "uncommitted" => {
                        w.put(&workspace("in-flight"))?;
                        w.enqueue(&DeviceId::new("pc").unwrap(), &change("c-in-flight", 'f'))?;
                        crash()
                    }
                    "stop" => {
                        // Stopping remote access bumps every grant's epoch; the host dies halfway.
                        let grants = w.all::<RemoteGrant>()?;
                        w.put(&bumped(&grants[0]))?;
                        crash()
                    }
                    "effect" => {
                        // Admitted and running in this session; its effect journaled first.
                        w.put(&ledger("q7", Outcome::Running))?;
                        w.journal_start(&effect("q7")).map(|_| ())
                    }
                    "committed" => w.put(&workspace("kept")),
                    other => panic!("unknown crash scenario {other}"),
                }
            })
            .unwrap();
        // The write committed: the host dies before it can close the store.
        crash()
    }

    #[test]
    fn a_new_store_is_durable_and_at_this_builds_schema() {
        let scratch = Scratch::new("new");
        let (store, opened) = StateStore::open(&scratch.db(), at(1)).unwrap();
        assert_eq!(
            opened,
            Opened {
                schema_version: SCHEMA_VERSION,
                migrated_from: Some(0),
                previous_session: PreviousSession::None,
                interrupted: Interrupted::default(),
            }
        );
        let conn = store.connection();
        let mode: String = conn
            .pragma_query_value(None, "journal_mode", |r| r.get(0))
            .unwrap();
        let sync: i64 = conn
            .pragma_query_value(None, "synchronous", |r| r.get(0))
            .unwrap();
        assert_eq!((mode.as_str(), sync), ("wal", 2), "WAL with FULL sync");
        // Nothing is granted or enabled by default.
        assert!(store.all::<RemoteGrant>().unwrap().is_empty());
        assert!(store.all::<PeerDevice>().unwrap().is_empty());
        store.close(at(2)).unwrap();
        let (_, opened) = StateStore::open(&scratch.db(), at(3)).unwrap();
        assert_eq!(opened.previous_session, PreviousSession::Closed);
        assert_eq!(opened.migrated_from, None);
    }

    fn stored_three(scratch: &Scratch) -> StateStore {
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store
            .write(at(2), |w| {
                w.put(&workspace("w"))?;
                w.put(&replica("w", "phone", scratch.path()))?;
                w.put(&grant("g1", 3, false))
            })
            .unwrap();
        store
    }

    #[test]
    fn records_round_trip_and_are_checked_when_read() {
        let scratch = Scratch::new("records");
        let store = stored_three(&scratch);
        assert_eq!(
            store.get::<WorkspaceRecord>("w").unwrap(),
            Some(workspace("w"))
        );
        assert_eq!(
            store.get::<Replica>("w/phone").unwrap(),
            Some(replica("w", "phone", scratch.path()))
        );
        assert_eq!(
            store.all::<RemoteGrant>().unwrap(),
            vec![grant("g1", 3, false)]
        );
        assert_eq!(store.get::<Task>("w").unwrap(), None, "keys are per schema");
        assert!(store.check_integrity().is_ok());
    }

    /// A client's layout with the right sidebar as given and the other panels fixed.
    fn layout(client: &str, right_shown: bool, right_size: u32) -> ClientLayout {
        let panel = |shown, size, view: &str| PanelState {
            shown,
            size: Some(size),
            selected: Some(nexees_domain::ids::ViewId::new(view).unwrap()),
        };
        ClientLayout {
            client_id: nexees_domain::ids::ClientId::new(client).unwrap(),
            device_id: DeviceId::new("pc").unwrap(),
            panels: PanelLayout::try_from(PanelLayoutFields {
                left: panel(true, 203, "explorer-view-container"),
                right: panel(right_shown, right_size, "nexees-area-tasks"),
                bottom: panel(true, 167, "terminal-0"),
            })
            .unwrap(),
        }
    }

    #[test]
    fn a_clients_layout_is_one_record_that_its_next_layout_replaces() {
        let scratch = Scratch::new("layout");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        assert_eq!(store.get::<ClientLayout>("desktop-window").unwrap(), None);
        store
            .write(at(2), |w| w.put(&layout("desktop-window", true, 354)))
            .unwrap();
        store
            .write(at(3), |w| w.put(&layout("desktop-window", false, 420)))
            .unwrap();
        store.close(at(4)).unwrap();
        // After a restart the host finds the layout the client stored last, and only that one.
        let (store, _) = StateStore::open(&scratch.db(), at(5)).unwrap();
        assert_eq!(
            store.all::<ClientLayout>().unwrap(),
            vec![layout("desktop-window", false, 420)]
        );
    }

    #[test]
    fn a_damaged_record_is_reported_never_returned() {
        // A malformed body, a record moved to another key, a version that is not its type's.
        type Read = fn(&StateStore) -> Result<(), StateError>;
        let cases: [(&str, Read); 3] = [
            (
                "UPDATE records SET body = '{\"schema\":' WHERE id = 'w'",
                |store| store.get::<WorkspaceRecord>("w").map(|_| ()),
            ),
            ("UPDATE records SET id = 'g2' WHERE id = 'g1'", |store| {
                store.get::<RemoteGrant>("g2").map(|_| ())
            }),
            (
                "UPDATE records SET version = 2 WHERE id = 'w/phone'",
                |store| store.all::<Replica>().map(|_| ()),
            ),
        ];
        for (tamper, read) in cases {
            let scratch = Scratch::new("damaged");
            let mut store = stored_three(&scratch);
            store
                .write(at(3), |w| Ok(w.tx().execute(tamper, [])?))
                .unwrap();
            assert!(
                matches!(read(&store), Err(StateError::Integrity { .. })),
                "{tamper}"
            );
            assert!(
                matches!(store.check_integrity(), Err(StateError::Integrity { .. })),
                "{tamper}"
            );
        }
    }

    #[test]
    fn a_write_is_all_or_nothing() {
        let scratch = Scratch::new("atomic");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        let failed = store.write(at(2), |w| {
            w.put(&workspace("a"))?;
            w.insert(&task("t1", TaskStatus::InProgress))?;
            w.insert(&task("t1", TaskStatus::NotStarted))
        });
        assert!(matches!(failed, Err(StateError::Duplicate { .. })));
        assert_eq!(store.get::<WorkspaceRecord>("a").unwrap(), None);
        assert_eq!(store.get::<Task>("t1").unwrap(), None);
    }

    #[test]
    fn an_old_task_id_is_never_reused_or_completed_by_the_store() {
        let scratch = Scratch::new("tasks");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store
            .write(at(2), |w| w.insert(&task("t1", TaskStatus::InProgress)))
            .unwrap();
        let reused = store.write(at(3), |w| w.insert(&task("t1", TaskStatus::NotStarted)));
        assert!(matches!(reused, Err(StateError::Duplicate { .. })));
        drop(store);
        // After a crash the task is exactly as it was: still in progress, not completed.
        let (store, opened) = StateStore::open(&scratch.db(), at(4)).unwrap();
        assert_eq!(opened.previous_session, PreviousSession::Interrupted);
        assert_eq!(
            store.get::<Task>("t1").unwrap().map(|t| t.fields().status),
            Some(TaskStatus::InProgress)
        );
    }

    #[test]
    fn a_field_named_like_a_secret_is_never_stored() {
        #[derive(Debug, Serialize, serde::Deserialize)]
        struct Leaky {
            id: String,
            provider_api_key: String,
        }
        impl Record for Leaky {
            const SCHEMA: &'static str = "test.leaky";
            const VERSION: u32 = 1;
            const SYNC: nexees_domain::schema::SyncPolicy =
                nexees_domain::schema::SyncPolicy::Never;
        }
        impl Stored for Leaky {
            fn key(&self) -> String {
                self.id.clone()
            }
        }
        let scratch = Scratch::new("secret");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        // Assembled at run time, so the source holds no credential-shaped string.
        let value = ["sk", "test", "0000"].join("-");
        let leaky = Leaky {
            id: "l1".into(),
            provider_api_key: value,
        };
        assert_eq!(
            store.write(at(2), |w| w.put(&leaky)),
            Err(StateError::SecretField {
                schema: "test.leaky".into(),
                field: "provider_api_key".into()
            })
        );
        store.close(at(3)).unwrap();
        assert!(bodies(&scratch.db()).is_empty());
    }

    #[test]
    fn only_one_process_holds_a_store() {
        let scratch = Scratch::new("held");
        let (store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        assert_eq!(
            StateStore::open(&scratch.db(), at(2)).map(|_| ()),
            Err(StateError::InUse)
        );
        store.close(at(3)).unwrap();
        assert!(StateStore::open(&scratch.db(), at(4)).is_ok());
    }

    #[test]
    fn a_newer_store_is_refused_untouched() {
        let scratch = Scratch::new("newer");
        let (store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store.close(at(2)).unwrap();
        Connection::open(scratch.db())
            .unwrap()
            .pragma_update(None, "user_version", 7)
            .unwrap();
        let before = std::fs::read(scratch.db()).unwrap();
        assert_eq!(
            StateStore::open(&scratch.db(), at(3)).map(|_| ()),
            Err(StateError::NewerSchema {
                found: 7,
                supported: SCHEMA_VERSION
            })
        );
        assert_eq!(
            std::fs::read(scratch.db()).unwrap(),
            before,
            "not a byte written"
        );
    }

    #[test]
    fn the_outbox_keeps_each_replicas_changes_in_order_until_acknowledged() {
        let scratch = Scratch::new("outbox");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        let (pc, tablet) = (
            DeviceId::new("pc").unwrap(),
            DeviceId::new("tablet").unwrap(),
        );
        let seq = store
            .write(at(2), |w| {
                let first = w.enqueue(&pc, &change("c1", 'b'))?;
                w.enqueue(&pc, &change("c2", 'c'))?;
                w.enqueue(&tablet, &change("c1", 'b'))?;
                // Queuing the same change again is idempotent.
                assert_eq!(w.enqueue(&pc, &change("c1", 'b'))?, first);
                Ok(first)
            })
            .unwrap();
        let other = store.write(at(3), |w| w.enqueue(&pc, &change("c1", 'd')));
        assert!(
            matches!(other, Err(StateError::Duplicate { .. })),
            "another change, same ID"
        );
        let ids = |entries: Vec<OutboxEntry>| {
            entries
                .into_iter()
                .map(|e| e.change.fields().change_id.as_str().to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(store.outbox(&pc).unwrap()), ["c1", "c2"]);
        assert_eq!(store.outbox(&pc).unwrap()[0].seq, seq);
        let acknowledged = store
            .write(at(4), |w| w.acknowledge(&pc, &ChangeId::new("c1").unwrap()))
            .unwrap();
        assert!(acknowledged);
        assert_eq!(ids(store.outbox(&pc).unwrap()), ["c2"]);
        assert_eq!(
            ids(store.outbox(&tablet).unwrap()),
            ["c1"],
            "each replica acknowledges its own"
        );
    }

    #[test]
    fn a_crash_before_the_commit_keeps_the_last_committed_state() {
        let scratch = Scratch::new("crash-before");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store
            .write(at(2), |w| w.put(&workspace("committed")))
            .unwrap();
        store.close(at(3)).unwrap();
        let before = bodies(&scratch.db());
        crash_in_child("uncommitted", &scratch.db());
        let (store, opened) = StateStore::open(&scratch.db(), at(4)).unwrap();
        assert_eq!(opened.previous_session, PreviousSession::Interrupted);
        assert!(store.check_integrity().is_ok());
        store.close(at(5)).unwrap();
        assert_eq!(
            bodies(&scratch.db()),
            before,
            "nothing of the interrupted write is kept"
        );
    }

    #[test]
    fn a_crash_after_the_commit_keeps_the_write() {
        let scratch = Scratch::new("crash-after");
        let (store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store.close(at(2)).unwrap();
        crash_in_child("committed", &scratch.db());
        let (store, opened) = StateStore::open(&scratch.db(), at(3)).unwrap();
        assert_eq!(opened.previous_session, PreviousSession::Interrupted);
        assert_eq!(
            store.get::<WorkspaceRecord>("kept").unwrap(),
            Some(workspace("kept"))
        );
    }

    #[test]
    fn rc_07_stopping_remote_access_is_one_durable_change_and_local_editing_continues() {
        let scratch = Scratch::new("rc07");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store
            .write(at(2), |w| {
                w.put(&grant("g1", 3, false))?;
                w.put(&grant("g2", 3, false))?;
                w.put(&approval("a1", 3))
            })
            .unwrap();
        store.close(at(3)).unwrap();
        // The host dies after bumping one grant of two: no grant changed.
        crash_in_child("stop", &scratch.db());
        let (mut store, _) = StateStore::open(&scratch.db(), at(4)).unwrap();
        let epochs = |store: &StateStore| {
            store
                .all::<RemoteGrant>()
                .unwrap()
                .iter()
                .map(|g| g.fields().permission_epoch.get())
                .collect::<Vec<_>>()
        };
        assert_eq!(epochs(&store), [3, 3], "never a half-stopped state");
        // Completed, the stop moves every grant on, which voids the approval of epoch 3.
        store
            .write(at(5), |w| {
                for grant in w.all::<RemoteGrant>()? {
                    w.put(&bumped(&grant))?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(epochs(&store), [4, 4]);
        let approval = store.get::<Approval>("a1").unwrap().unwrap();
        let (request, pc) = (RequestId::new("q1").unwrap(), DeviceId::new("pc").unwrap());
        let usable_at = |epoch| {
            approval.check_use(
                &request,
                &hash('e'),
                &pc,
                PermissionEpoch::new(epoch),
                at(5),
            )
        };
        assert!(usable_at(3).is_ok(), "valid before the stop");
        assert_eq!(
            usable_at(4).unwrap_err().kind,
            ErrorKind::Stale,
            "void after it"
        );
        // Local editing goes on.
        store
            .write(at(6), |w| {
                w.put(&workspace("local"))?;
                w.enqueue(&DeviceId::new("pc").unwrap(), &change("c9", 'b'))
                    .map(|_| ())
            })
            .unwrap();
    }

    #[test]
    fn rc_t09_an_interrupted_effect_is_outcome_unknown_and_never_runs_again() {
        let scratch = Scratch::new("rct09");
        let (store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store.close(at(3)).unwrap();
        // The child admits and runs a request, journaled, and dies before acknowledging it or
        // settling its effect.
        crash_in_child("effect", &scratch.db());
        let (mut store, opened) = StateStore::open(&scratch.db(), at(4)).unwrap();
        assert_eq!(
            opened.interrupted,
            Interrupted {
                effects: 1,
                requests: 1
            }
        );
        let entry = store.get::<LedgerEntry>("q7").unwrap().unwrap();
        assert_eq!(entry.fields().state, Outcome::OutcomeUnknown);
        // A resend of the same request is answered with the unknown outcome, not run again.
        assert_eq!(
            entry.on_duplicate(&hash('a'), &DeviceId::new("phone").unwrap()),
            nexees_domain::requests::Duplicate::Recorded(Outcome::OutcomeUnknown)
        );
        let again = store.write(at(5), |w| w.journal_start(&effect("q7")));
        assert!(matches!(again, Err(StateError::Duplicate { .. })));
        // It stays unknown until evidence settles it.
        let (seq, _) = store.unknown_outcomes().unwrap()[0].clone();
        store
            .write(at(6), |w| w.journal_settle(seq, EffectState::Completed))
            .unwrap();
        assert!(store.unknown_outcomes().unwrap().is_empty());
    }

    #[test]
    fn rc_t10_an_expired_queued_request_is_dropped_while_content_still_syncs() {
        let scratch = Scratch::new("rct10");
        let pc = DeviceId::new("pc").unwrap();
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        store
            .write(at(2), |w| {
                w.put(&pending("q8", 61_000))?;
                w.enqueue(&pc, &change("c1", 'b')).map(|_| ())
            })
            .unwrap();
        drop(store); // offline, and the host restarts
        let (mut store, _) = StateStore::open(&scratch.db(), at(90_000)).unwrap();
        let queued = store.get::<PendingRequest>("q8").unwrap().unwrap();
        let (dropped, action) = queued.reconnect(None, at(90_000));
        assert_eq!(action, Reconnect::Drop);
        store.write(at(90_000), |w| w.put(&dropped)).unwrap();
        assert_eq!(
            store
                .get::<PendingRequest>("q8")
                .unwrap()
                .unwrap()
                .fields()
                .state,
            PendingState::Expired
        );
        // The content change is still queued for the PC, and the outbox holds nothing else.
        let outbox = store.outbox(&pc).unwrap();
        assert_eq!(outbox.len(), 1);
        assert_eq!(outbox[0].change, change("c1", 'b'));
    }

    #[test]
    fn rc_t16_update_restart_and_uninstall_in_a_disposable_profile() {
        let profile = Scratch::new("rct16-profile");
        let projects = Scratch::new("rct16-projects");
        let user_file = projects.path().join("main.rs");
        std::fs::write(&user_file, "fn main() {}\n").unwrap();
        let db = profile.db();
        let (mut store, _) = StateStore::open(&db, at(1)).unwrap();
        store
            .write(at(2), |w| {
                w.put(&grant("g1", 5, false))?;
                w.put(&grant("g2", 2, true))?;
                w.put(&peer("old-phone", 1, false))?;
                w.put(&peer("lost-phone", 1, true))?;
                w.put(&replica("w", "pc", projects.path()))
            })
            .unwrap();
        store.close(at(3)).unwrap();
        let before = bodies(&db);
        // Update: the next schema version, which adds an index and changes no record.
        let steps = then(crate::migrations::migration_registry::tests::INDEX_NEXT);
        let (store, opened) = StateStore::open_with(&db, at(4), &steps).unwrap();
        assert_eq!(opened.migrated_from, Some(SCHEMA_VERSION));
        store.close(at(5)).unwrap();
        // Restart.
        let (store, opened) = StateStore::open_with(&db, at(6), &steps).unwrap();
        assert_eq!(opened.previous_session, PreviousSession::Closed);
        // The stored versions still let the raised minimum refuse the obsolete peer.
        let old = store.get::<PeerDevice>("old-phone").unwrap().unwrap();
        let hello =
            nexees_protocol::version_negotiation::Hello::new(old.fields().protocol_versions)
                .unwrap();
        let local = nexees_protocol::version_negotiation::Hello::current();
        assert!(nexees_protocol::version_negotiation::negotiate(&local, &hello, 2).is_err());
        let owned = store.owned_files();
        store.close(at(7)).unwrap();
        assert_eq!(
            bodies(&db),
            before,
            "opt-in, grants, revocations and epochs exactly kept"
        );
        // Uninstall removes the store's own files and nothing of the user's projects.
        for file in owned.iter().filter(|file| file.exists()) {
            std::fs::remove_file(file).unwrap();
        }
        assert_eq!(std::fs::read_dir(profile.path()).unwrap().count(), 0);
        assert_eq!(
            std::fs::read_to_string(&user_file).unwrap(),
            "fn main() {}\n"
        );
    }

    #[test]
    fn the_devices_own_identity_is_recorded_once_and_kept() {
        let scratch = Scratch::new("device");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        assert_eq!(store.device_id().unwrap(), None);
        let pc = DeviceId::new("desktop-1").unwrap();
        store.write(at(2), |w| w.set_device_id(&pc)).unwrap();
        // The same identity again changes nothing; another one is refused.
        store.write(at(3), |w| w.set_device_id(&pc)).unwrap();
        let other = DeviceId::new("desktop-2").unwrap();
        assert!(matches!(
            store.write(at(4), |w| w.set_device_id(&other)),
            Err(StateError::Duplicate { .. })
        ));
        store.close(at(5)).unwrap();
        let (mut store, _) = StateStore::open(&scratch.db(), at(6)).unwrap();
        assert_eq!(store.device_id().unwrap(), Some(pc));
        // A damaged value is reported, never returned.
        store
            .write(at(7), |w| {
                let damage = "UPDATE store_meta SET value = 'not valid!' WHERE key = 'device_id'";
                Ok(w.tx().execute(damage, [])?)
            })
            .unwrap();
        assert!(matches!(
            store.device_id(),
            Err(StateError::Integrity { .. })
        ));
    }

    #[test]
    fn the_record_set_changes_only_with_a_migration() {
        // Schema version 2 stores exactly these records at these versions; it added the client
        // layout to those of version 1. Adding a record, or changing one's version, needs a new
        // migration step and a new entry here.
        let stored: Vec<_> = RECORDS.iter().map(|r| (r.schema, r.version)).collect();
        assert_eq!(
            stored,
            [
                ("nexees.workspace.workspace", 1),
                ("nexees.workspace.replica", 1),
                ("nexees.workspace.conflict", 1),
                ("nexees.device.peer", 1),
                ("nexees.device.host_capabilities", 1),
                ("nexees.client.state", 1),
                ("nexees.client.layout", 1),
                ("nexees.authority.permission_grant", 1),
                ("nexees.authority.remote_grant", 1),
                ("nexees.authority.approval", 1),
                ("nexees.requests.ledger_entry", 1),
                ("nexees.requests.pending", 1),
                ("nexees.session.agent_session", 1),
                ("nexees.session.execution_transfer", 1),
                ("nexees.handoff.handoff", 1),
                ("nexees.task.task", 1),
                ("nexees.task.evidence", 1),
                ("nexees.lcl.spec_revision", 1),
                ("nexees.lcl.adoption", 1),
                ("nexees.import.transaction", 1),
            ]
        );
        assert_eq!(SCHEMA_VERSION, 2);
        // Requests, approvals and grants are records that never travel: they have no way into
        // the outbox, whose only body is a content change.
        assert!(!<Approval as Record>::SYNC.travels());
        assert!(!<PendingRequest as Record>::SYNC.travels());
        assert!(<ContentChange as Record>::SYNC.travels());
    }
}
