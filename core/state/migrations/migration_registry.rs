//! The ordered schema migrations of the state store (SS-STATE, FD-STORAGE, TH-40, SI-26).
//!
//! The store's schema version is SQLite's `user_version`. It covers both the tables and the set
//! of record schemas and versions the store keeps. A change to either is a new step, so an older
//! build refuses the store instead of misreading it.
//!
//! Opening a store runs every pending step in one transaction and checks the result before it
//! commits:
//! - every record of a schema the step does not declare in [`Migration::rewrites`] is
//!   byte-identical to before. A defective step therefore cannot reset a grant, a revocation, an
//!   epoch, an unsynced change or a journal entry (TH-40, SI-26);
//! - every stored body still decodes strictly as its type. No domain record has a default, so a
//!   step cannot leave a field to be filled in, least of all with an allow;
//! - SQLite's own integrity check passes.
//!
//! A failure, or a crash at any point, leaves the store at its previous version with every record
//! unchanged and readable by the build that wrote it. That is the recoverable path SS-UPDATES
//! relies on. A store newer than the build is refused untouched; nothing migrates downwards.
//!
//! The comparison reads the tables as version 1 lays them out. A step that changes the layout of
//! `records`, `outbox` or `journal` changes the query of `snapshot` in the same change.

use std::cmp::Ordering;

use nexees_domain::changes::ContentChange;
use nexees_domain::schema::Record;
use nexees_domain::time::Timestamp;
use rusqlite::{Transaction, params};

use crate::operation_journal::JournalEntry;
use crate::state_store::{StateError, millis, verify};

/// The store schema version this build reads and writes.
pub const SCHEMA_VERSION: u32 = 1;

/// One migration step, from `version - 1` to `version`.
#[derive(Debug, Clone, Copy)]
pub struct Migration {
    /// The version the step produces.
    pub version: u32,
    /// What the step changes, as the migration history records it.
    pub description: &'static str,
    /// The record schemas whose stored bodies the step rewrites. Every other record must come
    /// out byte-identical; the outbox and the journal count under the schemas of their bodies.
    pub rewrites: &'static [&'static str],
    /// The step itself.
    pub apply: fn(&Transaction<'_>) -> Result<(), StateError>,
}

/// The migrations of this build, in order. The last one's version is [`SCHEMA_VERSION`].
pub const MIGRATIONS: &[Migration] = &[Migration {
    version: 1,
    description: "The initial store: metadata, migration history, versioned records, the outbox \
                  and the operation journal",
    rewrites: &[],
    apply: create_version_1,
}];

/// What a build with a list of steps does with a store at some schema version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    /// The store is at the build's version.
    Current,
    /// The store is older: migrate it.
    Upgrade {
        /// The store's version.
        from: u32,
        /// The build's version.
        to: u32,
    },
    /// The store is newer than the build: refuse it and leave it untouched.
    Newer {
        /// The store's version.
        found: u32,
        /// The build's version.
        supported: u32,
    },
}

/// The plan for a store at version `found`, for a build with `steps`.
pub fn plan(found: u32, steps: &[Migration]) -> Plan {
    let supported = steps.last().map_or(0, |step| step.version);
    match found.cmp(&supported) {
        Ordering::Equal => Plan::Current,
        Ordering::Less => Plan::Upgrade {
            from: found,
            to: supported,
        },
        Ordering::Greater => Plan::Newer { found, supported },
    }
}

/// Fails unless the steps are numbered 1, 2, 3 and so on, without a gap.
pub fn check_order(steps: &[Migration]) -> Result<(), StateError> {
    for (expected, step) in (1..).zip(steps) {
        if step.version != expected {
            return Err(StateError::Migration {
                version: step.version,
                reason: format!("step {expected} expected: steps are numbered without a gap"),
            });
        }
    }
    Ok(())
}

/// Applies the steps after version `from` inside `tx` and verifies the result; the caller
/// commits. On any error the caller's transaction rolls back, so the store keeps `from`.
pub(crate) fn upgrade(
    tx: &Transaction<'_>,
    steps: &[Migration],
    from: u32,
    now: Timestamp,
) -> Result<(), StateError> {
    check_order(steps)?;
    for step in steps.iter().filter(|step| step.version > from) {
        let refuse = |reason: String| StateError::Migration {
            version: step.version,
            reason,
        };
        // Before version 1 there are no tables, so there is nothing to compare.
        let compare = step.version > 1;
        if compare {
            snapshot(tx, "migration_before")?;
        }
        (step.apply)(tx).map_err(|error| refuse(error.to_string()))?;
        if compare {
            snapshot(tx, "migration_after")?;
            let changed = undeclared_changes(tx, step.rewrites)?;
            if let Some(first) = changed.first() {
                return Err(refuse(format!(
                    "it changed {} record(s) of schemas it does not declare, the first {first}",
                    changed.len()
                )));
            }
        }
        tx.execute(
            "INSERT INTO migration_history (version, description, applied_at) VALUES (?1, ?2, ?3)",
            params![step.version, step.description, millis(now)?],
        )
        .map_err(|error| refuse(error.to_string()))?;
        tx.pragma_update(None, "user_version", step.version)
            .map_err(|error| refuse(error.to_string()))?;
    }
    tx.execute_batch(
        "DROP TABLE IF EXISTS temp.migration_before; DROP TABLE IF EXISTS temp.migration_after;",
    )?;
    verify(tx).map_err(|error| StateError::Migration {
        version: steps.last().map_or(0, |step| step.version),
        reason: error.to_string(),
    })
}

/// Copies every stored body, keyed by schema and key, into the temporary table `name`. It reads
/// the tables as version 1 lays them out.
fn snapshot(tx: &Transaction<'_>, name: &str) -> Result<(), StateError> {
    tx.execute_batch(&format!(
        "DROP TABLE IF EXISTS temp.{name};
         CREATE TEMP TABLE {name} AS
             SELECT schema, id, body FROM records
             UNION ALL SELECT '{change}', destination_device_id || '/' || change_id, body
                 FROM outbox
             UNION ALL SELECT '{journal}', CAST(seq AS TEXT), body FROM journal;",
        change = ContentChange::SCHEMA,
        journal = JournalEntry::SCHEMA,
    ))?;
    Ok(())
}

/// The records, as `schema/key`, that differ between the two snapshots or exist in only one, of
/// the schemas `rewrites` does not declare.
fn undeclared_changes(tx: &Transaction<'_>, rewrites: &[&str]) -> Result<Vec<String>, StateError> {
    let mut statement = tx.prepare(
        "SELECT before.schema, before.id FROM temp.migration_before AS before
             LEFT JOIN temp.migration_after AS after
                 ON after.schema = before.schema AND after.id = before.id
             WHERE after.body IS NOT before.body
         UNION
         SELECT after.schema, after.id FROM temp.migration_after AS after
             LEFT JOIN temp.migration_before AS before
                 ON before.schema = after.schema AND before.id = after.id
             WHERE before.id IS NULL
         ORDER BY 1, 2",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut changed = Vec::new();
    for row in rows {
        let (schema, key) = row?;
        if !rewrites.contains(&schema.as_str()) {
            changed.push(format!("{schema}/{key}"));
        }
    }
    Ok(changed)
}

/// Version 1, the initial store:
/// - `store_meta`: the recovery metadata of the sessions;
/// - `migration_history`: each version applied, and when;
/// - `records`: one versioned record per schema and key, as strict JSON;
/// - `outbox`: content changes waiting for a peer replica, in order;
/// - `journal`: the operation journal, with at most one effect per remote request.
fn create_version_1(tx: &Transaction<'_>) -> Result<(), StateError> {
    tx.execute_batch(
        "CREATE TABLE store_meta (
             key TEXT PRIMARY KEY,
             value TEXT NOT NULL
         ) STRICT;
         CREATE TABLE migration_history (
             version INTEGER PRIMARY KEY,
             description TEXT NOT NULL,
             applied_at INTEGER NOT NULL
         ) STRICT;
         CREATE TABLE records (
             schema TEXT NOT NULL,
             id TEXT NOT NULL,
             version INTEGER NOT NULL,
             body TEXT NOT NULL,
             updated_at INTEGER NOT NULL,
             PRIMARY KEY (schema, id)
         ) STRICT, WITHOUT ROWID;
         CREATE TABLE outbox (
             seq INTEGER PRIMARY KEY AUTOINCREMENT,
             destination_device_id TEXT NOT NULL,
             change_id TEXT NOT NULL,
             body TEXT NOT NULL,
             enqueued_at INTEGER NOT NULL,
             UNIQUE (destination_device_id, change_id)
         ) STRICT;
         CREATE TABLE journal (
             seq INTEGER PRIMARY KEY AUTOINCREMENT,
             request_id TEXT UNIQUE,
             state TEXT NOT NULL,
             body TEXT NOT NULL
         ) STRICT;
         CREATE INDEX journal_by_state ON journal (state);",
    )?;
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::state_store::StateStore;
    use crate::state_store::tests::{
        Scratch, approval, at, bodies, change, crash, crash_in_child, draft, grant, ledger, peer,
        staged_import, task,
    };
    use nexees_domain::errors::Outcome;
    use nexees_domain::ids::DeviceId;
    use nexees_domain::task::TaskStatus;

    fn step(version: u32) -> Migration {
        Migration {
            version,
            description: "test",
            rewrites: &[],
            apply: |_| Ok(()),
        }
    }

    fn add_index(tx: &Transaction<'_>) -> Result<(), StateError> {
        tx.execute_batch("CREATE INDEX records_by_update ON records (updated_at);")?;
        Ok(())
    }

    fn crash_halfway(tx: &Transaction<'_>) -> Result<(), StateError> {
        tx.execute_batch("CREATE TABLE half_done (x INTEGER);")?;
        crash()
    }

    fn fail(tx: &Transaction<'_>) -> Result<(), StateError> {
        tx.execute_batch("CREATE TABLE records (x INTEGER);")?;
        Ok(())
    }

    fn reset_revocations(tx: &Transaction<'_>) -> Result<(), StateError> {
        tx.execute(
            "UPDATE records SET body = json_set(body, '$.record.revoked', json('false'))
             WHERE schema = ?1",
            [nexees_domain::authority::RemoteGrant::SCHEMA],
        )?;
        Ok(())
    }

    fn drop_revocations(tx: &Transaction<'_>) -> Result<(), StateError> {
        tx.execute(
            "UPDATE records SET body = json_remove(body, '$.record.revoked') WHERE schema = ?1",
            [nexees_domain::authority::RemoteGrant::SCHEMA],
        )?;
        Ok(())
    }

    /// A later schema version that adds an index and changes no record.
    pub(crate) const INDEX_V2: Migration = Migration {
        version: 2,
        description: "test: an index",
        rewrites: &[],
        apply: add_index,
    };

    /// A later schema version during which the host dies.
    pub(crate) const CRASHING_V2: Migration = Migration {
        version: 2,
        description: "test: the host dies",
        rewrites: &[],
        apply: crash_halfway,
    };

    /// A store at version 1 holding unsynced changes, a draft, a staged import, tasks and
    /// security state, closed in order.
    fn populated(scratch: &Scratch) {
        let (mut store, _) = StateStore::open(&scratch.db(), at(1)).unwrap();
        let pc = DeviceId::new("pc").unwrap();
        store
            .write(at(2), |w| {
                w.enqueue(&pc, &change("c1", 'b'))?;
                w.enqueue(&pc, &change("c2", 'c'))?;
                w.put(&draft("r5"))?;
                w.put(&staged_import("i1"))?;
                w.put(&task("t1", TaskStatus::InProgress))?;
                w.put(&task("t2", TaskStatus::NotStarted))?;
                w.put(&grant("g1", 5, false))?;
                w.put(&grant("g2", 2, true))?;
                w.put(&approval("a1", 5))?;
                w.put(&peer("phone", 1, false))?;
                w.put(&peer("lost", 1, true))?;
                w.put(&ledger("q1", Outcome::Completed))
            })
            .unwrap();
        store.close(at(3)).unwrap();
    }

    fn version(scratch: &Scratch) -> u32 {
        rusqlite::Connection::open(scratch.db())
            .unwrap()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn the_registry_is_numbered_without_gaps() {
        assert!(check_order(MIGRATIONS).is_ok());
        assert_eq!(
            MIGRATIONS.last().map(|step| step.version),
            Some(SCHEMA_VERSION)
        );
        assert!(check_order(&[step(1), step(3)]).is_err());
        assert!(check_order(&[step(2)]).is_err());
    }

    #[test]
    fn a_store_is_current_older_or_newer_than_the_build() {
        assert_eq!(plan(1, MIGRATIONS), Plan::Current);
        assert_eq!(plan(0, MIGRATIONS), Plan::Upgrade { from: 0, to: 1 });
        assert_eq!(
            plan(4, MIGRATIONS),
            Plan::Newer {
                found: 4,
                supported: 1
            }
        );
    }

    #[test]
    fn a_migration_keeps_unsynced_changes_drafts_tasks_and_security_state() {
        let scratch = Scratch::new("migrate");
        populated(&scratch);
        let before = bodies(&scratch.db());
        let (store, opened) =
            StateStore::open_with(&scratch.db(), at(4), &[MIGRATIONS[0], INDEX_V2]).unwrap();
        assert_eq!(opened.migrated_from, Some(1));
        assert_eq!(opened.schema_version, 2);
        // Old task IDs keep their status: nothing becomes completed by migrating.
        let statuses: Vec<_> = store
            .all::<nexees_domain::task::Task>()
            .unwrap()
            .iter()
            .map(|t| t.fields().status)
            .collect();
        assert_eq!(statuses, [TaskStatus::InProgress, TaskStatus::NotStarted]);
        store.close(at(5)).unwrap();
        assert_eq!(
            bodies(&scratch.db()),
            before,
            "every record kept byte for byte"
        );
        assert_eq!(version(&scratch), 2);
    }

    #[test]
    fn a_migration_that_changes_records_it_does_not_declare_is_rolled_back() {
        let scratch = Scratch::new("undeclared");
        populated(&scratch);
        let before = bodies(&scratch.db());
        let resetting = Migration {
            version: 2,
            description: "test: a defective step that resets revocations",
            rewrites: &[],
            apply: reset_revocations,
        };
        let refused = StateStore::open_with(&scratch.db(), at(4), &[MIGRATIONS[0], resetting]);
        let Err(StateError::Migration { version: 2, reason }) = refused.map(|_| ()) else {
            panic!("the defective step must be refused");
        };
        assert!(
            reason.contains("nexees.authority.remote_grant/g2"),
            "{reason}"
        );
        assert_eq!(version(&scratch), 1);
        assert_eq!(bodies(&scratch.db()), before);
    }

    #[test]
    fn a_migration_that_leaves_a_record_undecodable_is_rolled_back() {
        let scratch = Scratch::new("undecodable");
        populated(&scratch);
        let before = bodies(&scratch.db());
        // Declared, so the comparison allows it; but no field may go missing, to be defaulted.
        let dropping = Migration {
            version: 2,
            description: "test: a step that drops a field",
            rewrites: &[nexees_domain::authority::RemoteGrant::SCHEMA],
            apply: drop_revocations,
        };
        let refused = StateStore::open_with(&scratch.db(), at(4), &[MIGRATIONS[0], dropping]);
        assert!(matches!(
            refused.map(|_| ()),
            Err(StateError::Migration { version: 2, .. })
        ));
        assert_eq!(version(&scratch), 1);
        assert_eq!(bodies(&scratch.db()), before);
    }

    #[test]
    fn a_failing_step_keeps_the_previous_version() {
        let scratch = Scratch::new("failing");
        populated(&scratch);
        let before = bodies(&scratch.db());
        let failing = Migration {
            version: 2,
            description: "test: a failing step",
            rewrites: &[],
            apply: fail,
        };
        let refused = StateStore::open_with(&scratch.db(), at(4), &[MIGRATIONS[0], failing]);
        assert!(matches!(
            refused.map(|_| ()),
            Err(StateError::Migration { version: 2, .. })
        ));
        assert_eq!(version(&scratch), 1);
        assert_eq!(bodies(&scratch.db()), before);
        // The build that wrote it still opens it.
        assert!(StateStore::open(&scratch.db(), at(5)).is_ok());
    }

    #[test]
    fn a_crash_during_a_migration_keeps_the_previous_version() {
        let scratch = Scratch::new("crash-migration");
        populated(&scratch);
        let before = bodies(&scratch.db());
        crash_in_child("migration", &scratch.db());
        assert_eq!(version(&scratch), 1);
        assert_eq!(bodies(&scratch.db()), before);
        let half_done: i64 = rusqlite::Connection::open(scratch.db())
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_schema WHERE name = 'half_done'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(half_done, 0, "nothing of the interrupted step is left");
        // A later open migrates normally, and the migration is checked as a whole.
        let (store, opened) =
            StateStore::open_with(&scratch.db(), at(5), &[MIGRATIONS[0], INDEX_V2]).unwrap();
        assert_eq!(opened.migrated_from, Some(1));
        assert!(store.check_integrity().is_ok());
    }
}
