//! The operation journal: the write-ahead record of every effect, so an interrupted effect is
//! reconciled and never replayed blindly (ST-JOURNAL, RC-17, SI-13, TH-46, TH-22).
//!
//! An effect is journaled before it starts and settled once its outcome is known. An effect that
//! had started and was not settled when the host stopped may or may not have happened. Opening
//! the store turns it into outcome-unknown, as it does the remote requests the destination had
//! admitted or was running, and only evidence of what happened settles it.
//!
//! An effect that belongs to a remote request is journaled at most once per request ID, so a
//! request whose effect is outcome-unknown cannot start it again (SI-13). Each entry names the
//! agent session and generation it ran under, so the effect of a reply from an older generation
//! can be told apart (TH-22). Checkpoints are TASK-026's, and resuming work is TASK-038's.

use nexees_domain::errors::{DomainError, ErrorKind};
use nexees_domain::ids::{AgentSessionId, OperationId, RequestId, WorkspaceId};
use nexees_domain::requests::LedgerEntry;
use nexees_domain::revision::Generation;
use nexees_domain::schema::{Record, SyncPolicy, Validate, Versioned};
use nexees_domain::time::Timestamp;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::{Deserialize, Serialize};

use crate::state_store::{StateError, StateStore, Write, decode_reason, encode, read_where, store};

/// Where a journaled effect stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectState {
    /// Started; its outcome is not known yet.
    Started,
    /// It happened.
    Completed,
    /// It failed without effect.
    Failed,
    /// It may or may not have happened; only evidence settles it.
    OutcomeUnknown,
}

impl EffectState {
    /// The name the journal's `state` column holds.
    fn name(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::OutcomeUnknown => "outcome_unknown",
        }
    }
}

/// The fields of a [`JournalEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntryFields {
    /// What the effect does, by its operation's registry key.
    pub operation: OperationId,
    /// The remote request the effect carries out, if it is one.
    pub request_id: Option<RequestId>,
    /// The workspace it acts on, if any.
    pub workspace_id: Option<WorkspaceId>,
    /// The agent session that started it, if one did.
    pub agent_id: Option<AgentSessionId>,
    /// That session's generation when the effect started; present exactly with `agent_id`.
    pub generation: Option<Generation>,
    /// Where it stands.
    pub state: EffectState,
    /// When it started.
    pub started_at: Timestamp,
    /// When it settled or became outcome-unknown; absent exactly while it is started.
    pub settled_at: Option<Timestamp>,
}

impl Validate for JournalEntryFields {
    fn validate(&self) -> Result<(), DomainError> {
        if self.agent_id.is_some() != self.generation.is_some() {
            return Err(DomainError::new("generation", ErrorKind::Mismatch));
        }
        match (self.state, self.settled_at) {
            (EffectState::Started, None) => Ok(()),
            (EffectState::Started, Some(_)) => {
                Err(DomainError::new("settled_at", ErrorKind::Unexpected))
            }
            (_, None) => Err(DomainError::new("settled_at", ErrorKind::Missing)),
            (_, Some(settled)) if settled < self.started_at => {
                Err(DomainError::new("settled_at", ErrorKind::OutOfRange))
            }
            _ => Ok(()),
        }
    }
}

/// One journaled effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry(JournalEntryFields);

impl JournalEntry {
    /// A validated entry.
    pub fn new(fields: JournalEntryFields) -> Result<Self, DomainError> {
        fields.validate()?;
        Ok(Self(fields))
    }

    /// The validated fields.
    pub fn fields(&self) -> &JournalEntryFields {
        &self.0
    }

    /// The entry with a new state, settled at `now`. A started effect completes, fails or
    /// becomes outcome-unknown; an unknown outcome is settled only as completed or failed, from
    /// evidence. Completed and failed are final. A wall clock that moved backwards since the
    /// effect started cannot make it settle before it started, nor stop the store from opening.
    pub fn settle(&self, state: EffectState, now: Timestamp) -> Result<Self, DomainError> {
        use EffectState::*;
        if !matches!(
            (self.0.state, state),
            (Started, Completed | Failed | OutcomeUnknown) | (OutcomeUnknown, Completed | Failed)
        ) {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition));
        }
        Self::new(JournalEntryFields {
            state,
            settled_at: Some(now.max(self.0.started_at)),
            ..self.0.clone()
        })
    }
}

impl Serialize for JournalEntry {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for JournalEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(JournalEntryFields::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Record for JournalEntry {
    const SCHEMA: &'static str = "nexees.state.journal_entry";
    const VERSION: u32 = 1;
    const SYNC: SyncPolicy = SyncPolicy::Never;
}

/// The position of an entry in the journal, which never changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JournalSeq(i64);

impl JournalSeq {
    /// The position as a number.
    pub fn get(self) -> i64 {
        self.0
    }
}

/// What the restart rules turned outcome-unknown when the store opened.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Interrupted {
    /// Journaled effects that had started and not settled.
    pub effects: usize,
    /// Remote requests the destination had admitted or was running.
    pub requests: usize,
}

impl Write<'_> {
    /// Journals `entry`, which must be started, before its effect starts. Fails when its remote
    /// request already has an effect in the journal, whatever that effect's state.
    pub fn journal_start(&mut self, entry: &JournalEntry) -> Result<JournalSeq, StateError> {
        if entry.0.state != EffectState::Started {
            return Err(DomainError::new("state", ErrorKind::InvalidTransition).into());
        }
        if let Some(request) = &entry.0.request_id {
            let taken = self
                .tx()
                .query_row(
                    "SELECT 1 FROM journal WHERE request_id = ?1",
                    [request.as_str()],
                    |_| Ok(()),
                )
                .optional()?;
            if taken.is_some() {
                return Err(StateError::Duplicate {
                    schema: JournalEntry::SCHEMA.into(),
                    id: request.as_str().into(),
                });
            }
        }
        self.tx().execute(
            "INSERT INTO journal (request_id, state, body) VALUES (?1, ?2, ?3)",
            params![
                entry.0.request_id.as_ref().map(|r| r.as_str()),
                entry.0.state.name(),
                encode(entry)?
            ],
        )?;
        Ok(JournalSeq(self.tx().last_insert_rowid()))
    }

    /// Settles the effect at `seq` as `state`, at this transaction's time.
    pub fn journal_settle(
        &mut self,
        seq: JournalSeq,
        state: EffectState,
    ) -> Result<JournalEntry, StateError> {
        let entry = read_entry(self.tx(), seq)?.ok_or_else(|| StateError::Missing {
            schema: JournalEntry::SCHEMA.into(),
            id: seq.0.to_string(),
        })?;
        let settled = entry.settle(state, self.now())?;
        rewrite(self.tx(), seq, &settled)?;
        Ok(settled)
    }
}

impl StateStore {
    /// The journal entry at `seq`, if there is one.
    pub fn journal_entry(&self, seq: JournalSeq) -> Result<Option<JournalEntry>, StateError> {
        read_entry(self.connection(), seq)
    }

    /// Every effect whose outcome is unknown, oldest first. Each blocks its replay until
    /// evidence settles it (SI-13).
    pub fn unknown_outcomes(&self) -> Result<Vec<(JournalSeq, JournalEntry)>, StateError> {
        read_state(self.connection(), EffectState::OutcomeUnknown)
    }
}

/// Applies the restart rules: every started effect and every admitted or running remote
/// request becomes outcome-unknown at `now`.
pub(crate) fn mark_interrupted(
    tx: &Transaction<'_>,
    now: Timestamp,
) -> Result<Interrupted, StateError> {
    let started = read_state(tx, EffectState::Started)?;
    for (seq, entry) in &started {
        rewrite(tx, *seq, &entry.settle(EffectState::OutcomeUnknown, now)?)?;
    }
    // Only entries a restart changes; the ledger's states are kebab-case on the wire.
    let in_flight = read_where::<LedgerEntry>(
        tx,
        "json_extract(body, '$.record.state') IN ('accepted', 'running')",
    )?;
    for entry in &in_flight {
        store(tx, &entry.after_restart(now), now, true)?;
    }
    Ok(Interrupted {
        effects: started.len(),
        requests: in_flight.len(),
    })
}

/// Fails unless every journal row decodes and agrees with its own columns.
pub(crate) fn verify_journal(conn: &Connection) -> Result<(), StateError> {
    let mut statement = conn.prepare("SELECT seq, request_id, state, body FROM journal")?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, Option<String>>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;
    for row in rows {
        let (seq, request, state, body) = row?;
        let entry = decode(seq, &body)?;
        let consistent = entry.0.request_id.as_ref().map(|r| r.as_str()) == request.as_deref()
            && entry.0.state.name() == state;
        if !consistent {
            return Err(integrity(seq, "its columns disagree with its body".into()));
        }
    }
    Ok(())
}

fn read_entry(conn: &Connection, seq: JournalSeq) -> Result<Option<JournalEntry>, StateError> {
    let body = conn
        .query_row("SELECT body FROM journal WHERE seq = ?1", [seq.0], |row| {
            row.get::<_, String>(0)
        })
        .optional()?;
    body.map(|body| decode(seq.0, &body)).transpose()
}

fn read_state(
    conn: &Connection,
    state: EffectState,
) -> Result<Vec<(JournalSeq, JournalEntry)>, StateError> {
    let mut statement =
        conn.prepare("SELECT seq, body FROM journal WHERE state = ?1 ORDER BY seq")?;
    let rows = statement.query_map([state.name()], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut entries = Vec::new();
    for row in rows {
        let (seq, body) = row?;
        entries.push((JournalSeq(seq), decode(seq, &body)?));
    }
    Ok(entries)
}

fn rewrite(conn: &Connection, seq: JournalSeq, entry: &JournalEntry) -> Result<(), StateError> {
    conn.execute(
        "UPDATE journal SET state = ?1, body = ?2 WHERE seq = ?3",
        params![entry.0.state.name(), encode(entry)?, seq.0],
    )?;
    Ok(())
}

fn decode(seq: i64, body: &str) -> Result<JournalEntry, StateError> {
    serde_json::from_str::<Versioned<JournalEntry>>(body)
        .map(|versioned| versioned.0)
        .map_err(|error| integrity(seq, decode_reason(&error)))
}

fn integrity(seq: i64, reason: String) -> StateError {
    StateError::Integrity {
        schema: JournalEntry::SCHEMA.into(),
        id: seq.to_string(),
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::tests::{Scratch, at, ledger};
    use nexees_domain::errors::Outcome;

    fn effect(request: Option<&str>, agent: Option<(&str, u64)>) -> JournalEntry {
        JournalEntry::new(JournalEntryFields {
            operation: OperationId::new("file_write").unwrap(),
            request_id: request.map(|r| RequestId::new(r).unwrap()),
            workspace_id: Some(WorkspaceId::new("w").unwrap()),
            agent_id: agent.map(|(a, _)| AgentSessionId::new(a).unwrap()),
            generation: agent.map(|(_, g)| Generation::try_from(g).unwrap()),
            state: EffectState::Started,
            started_at: at(10),
            settled_at: None,
        })
        .unwrap()
    }

    #[test]
    fn effects_settle_only_along_the_allowed_paths() {
        let started = effect(None, None);
        let unknown = started.settle(EffectState::OutcomeUnknown, at(11)).unwrap();
        assert_eq!(
            unknown
                .settle(EffectState::Started, at(12))
                .unwrap_err()
                .kind,
            ErrorKind::InvalidTransition
        );
        let done = unknown.settle(EffectState::Completed, at(12)).unwrap();
        assert!(done.settle(EffectState::Failed, at(13)).is_err());
        // The clock moved backwards: the effect settles when it started, never before.
        let failed = started.settle(EffectState::Failed, at(9)).unwrap();
        assert_eq!(failed.fields().settled_at, Some(at(10)));
    }

    #[test]
    fn entries_name_the_generation_exactly_with_the_session() {
        let mut fields = effect(None, Some(("s1", 2))).fields().clone();
        fields.generation = None;
        assert_eq!(
            JournalEntry::new(fields).unwrap_err(),
            DomainError::new("generation", ErrorKind::Mismatch)
        );
        let mut fields = effect(None, None).fields().clone();
        fields.settled_at = Some(at(11));
        assert!(
            JournalEntry::new(fields).is_err(),
            "a started effect has not settled"
        );
    }

    #[test]
    fn a_remote_request_has_at_most_one_journaled_effect() {
        let scratch = Scratch::new("journal-once");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        let seq = store
            .write(at(20), |w| {
                w.journal_start(&effect(Some("q1"), Some(("s1", 1))))
            })
            .unwrap();
        store
            .write(at(30), |w| w.journal_settle(seq, EffectState::Completed))
            .unwrap();
        // Even a completed effect is never started again for the same request.
        let again = store.write(at(40), |w| w.journal_start(&effect(Some("q1"), None)));
        assert!(matches!(again, Err(StateError::Duplicate { .. })));
        // Effects outside remote requests are not deduplicated.
        store
            .write(at(50), |w| w.journal_start(&effect(None, None)))
            .unwrap();
        store
            .write(at(50), |w| w.journal_start(&effect(None, None)))
            .unwrap();
        assert!(store.check_integrity().is_ok());
    }

    #[test]
    fn a_restart_turns_started_effects_and_running_requests_outcome_unknown() {
        let scratch = Scratch::new("journal-restart");
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        let seq = store
            .write(at(2), |w| {
                w.put(&ledger("q1", Outcome::Running))?;
                w.put(&ledger("q2", Outcome::Completed))?;
                w.journal_start(&effect(Some("q1"), None))
            })
            .unwrap();
        drop(store); // the host stops without closing the store
        let (store, opened) = StateStore::open(&scratch.db(), at(50)).unwrap();
        assert_eq!(
            opened.interrupted,
            Interrupted {
                effects: 1,
                requests: 1
            }
        );
        let entry = store.journal_entry(seq).unwrap().unwrap();
        assert_eq!(entry.fields().state, EffectState::OutcomeUnknown);
        assert_eq!(entry.fields().settled_at, Some(at(50)));
        assert_eq!(store.unknown_outcomes().unwrap().len(), 1);
        let q1 = store.get::<LedgerEntry>("q1").unwrap().unwrap();
        assert_eq!(q1.fields().state, Outcome::OutcomeUnknown);
        let q2 = store.get::<LedgerEntry>("q2").unwrap().unwrap();
        assert_eq!(
            q2.fields().state,
            Outcome::Completed,
            "a settled request stays settled"
        );
    }
}
