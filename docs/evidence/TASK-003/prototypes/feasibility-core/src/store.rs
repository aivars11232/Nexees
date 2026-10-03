//! Durable device-local state in SQLite (bundled, so Desktop and Android run
//! the same SQLite version).
//!
//! The schema is deliberately tiny: saved revisions of workspace files,
//! directional grants, and received remote commands. It exists to prove WAL
//! durability, transactional rollback, crash recovery and an in-place
//! migration, not to define the TASK-008 schema.

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

/// The schema version [`Store::open`] migrates to.
pub const SCHEMA_VERSION: i64 = 2;

/// Each step moves the schema from version `index` to `index + 1`.
const MIGRATIONS: [&str; 2] = [
    "CREATE TABLE revisions (
         workspace TEXT NOT NULL,
         path TEXT NOT NULL,
         revision INTEGER NOT NULL,
         sha256 TEXT NOT NULL,
         content BLOB NOT NULL,
         PRIMARY KEY (workspace, path, revision)
     );",
    "CREATE TABLE grants (
         direction TEXT PRIMARY KEY,
         enabled INTEGER NOT NULL CHECK (enabled IN (0, 1)),
         changed_at INTEGER NOT NULL
     );
     CREATE TABLE commands (
         id TEXT PRIMARY KEY,
         received_at INTEGER NOT NULL,
         expires_at INTEGER NOT NULL,
         action TEXT NOT NULL,
         status TEXT NOT NULL,
         detail TEXT NOT NULL
     );",
];

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Saved {
    pub revision: i64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: String,
    pub expires_at: u64,
    pub action: String,
    pub status: String,
    pub detail: String,
}

impl Store {
    /// Opens or creates the store and migrates it to [`SCHEMA_VERSION`].
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        Store::open_at(path, SCHEMA_VERSION)
    }

    /// Opens the store migrated only as far as `version`. The probe uses it to
    /// create an old database and prove the upgrade keeps its data.
    pub fn open_at(path: &Path, version: i64) -> rusqlite::Result<Store> {
        let conn = Connection::open(path)?;
        // WAL survives process death without losing committed transactions;
        // FULL also survives power loss after a commit returns.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let mut store = Store { conn };
        store.migrate(version)?;
        Ok(store)
    }

    fn migrate(&mut self, target: i64) -> rusqlite::Result<()> {
        let mut current: i64 = self.conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        while current < target {
            // One transaction per step: a crash leaves the previous version
            // intact, never a half-applied schema.
            let tx = self.conn.transaction()?;
            tx.execute_batch(MIGRATIONS[current as usize])?;
            tx.pragma_update(None, "user_version", current + 1)?;
            tx.commit()?;
            current += 1;
        }
        Ok(())
    }

    pub fn version(&self) -> rusqlite::Result<i64> {
        self.conn.pragma_query_value(None, "user_version", |r| r.get(0))
    }

    pub fn journal_mode(&self) -> rusqlite::Result<String> {
        self.conn.pragma_query_value(None, "journal_mode", |r| r.get(0))
    }

    /// `ok` when SQLite finds no corruption.
    pub fn integrity(&self) -> rusqlite::Result<String> {
        self.conn.pragma_query_value(None, "integrity_check", |r| r.get(0))
    }

    /// For the crash probe, which needs a transaction it never commits.
    pub fn connection(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Records the next revision of `path` in one transaction.
    pub fn save_revision(
        &mut self,
        workspace: &str,
        path: &str,
        content: &[u8],
    ) -> rusqlite::Result<Saved> {
        let sha256 = crate::sha256_hex(content);
        let tx = self.conn.transaction()?;
        let revision: i64 = tx.query_row(
            "SELECT COALESCE(MAX(revision), 0) + 1 FROM revisions WHERE workspace = ?1 AND path = ?2",
            params![workspace, path],
            |r| r.get(0),
        )?;
        tx.execute(
            "INSERT INTO revisions (workspace, path, revision, sha256, content) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![workspace, path, revision, sha256, content],
        )?;
        tx.commit()?;
        Ok(Saved { revision, sha256 })
    }

    pub fn latest(&self, workspace: &str, path: &str) -> rusqlite::Result<Option<(Saved, Vec<u8>)>> {
        self.conn
            .query_row(
                "SELECT revision, sha256, content FROM revisions
                 WHERE workspace = ?1 AND path = ?2 ORDER BY revision DESC LIMIT 1",
                params![workspace, path],
                |r| Ok((Saved { revision: r.get(0)?, sha256: r.get(1)? }, r.get(2)?)),
            )
            .optional()
    }

    pub fn set_grant(&mut self, direction: &str, enabled: bool) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO grants (direction, enabled, changed_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(direction) DO UPDATE SET enabled = excluded.enabled, changed_at = excluded.changed_at",
            params![direction, enabled, crate::unix_now() as i64],
        )?;
        Ok(())
    }

    /// A grant that was never recorded is off.
    pub fn grant(&self, direction: &str) -> rusqlite::Result<bool> {
        Ok(self
            .conn
            .query_row(
                "SELECT enabled FROM grants WHERE direction = ?1",
                params![direction],
                |r| r.get::<_, bool>(0),
            )
            .optional()?
            .unwrap_or(false))
    }

    /// Records a command ID once. `false` means the ID was already seen, so
    /// the request is a replay and must not run again.
    pub fn claim_command(&mut self, id: &str, expires_at: u64, action: &str) -> rusqlite::Result<bool> {
        let inserted = self.conn.execute(
            "INSERT OR IGNORE INTO commands (id, received_at, expires_at, action, status, detail)
             VALUES (?1, ?2, ?3, ?4, 'accepted', '')",
            params![id, crate::unix_now() as i64, expires_at as i64, action],
        )?;
        Ok(inserted == 1)
    }

    pub fn set_command_status(&mut self, id: &str, status: &str, detail: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE commands SET status = ?2, detail = ?3 WHERE id = ?1",
            params![id, status, detail],
        )?;
        Ok(())
    }

    pub fn command(&self, id: &str) -> rusqlite::Result<Option<Command>> {
        self.conn
            .query_row(
                "SELECT id, expires_at, action, status, detail FROM commands WHERE id = ?1",
                params![id],
                row_command,
            )
            .optional()
    }

    pub fn commands_with_status(&self, status: &str) -> rusqlite::Result<Vec<Command>> {
        let mut statement = self.conn.prepare(
            "SELECT id, expires_at, action, status, detail FROM commands WHERE status = ?1 ORDER BY received_at",
        )?;
        let rows = statement.query_map(params![status], row_command)?;
        rows.collect()
    }
}

fn row_command(r: &rusqlite::Row<'_>) -> rusqlite::Result<Command> {
    Ok(Command {
        id: r.get(0)?,
        expires_at: r.get::<_, i64>(1)? as u64,
        action: r.get(2)?,
        status: r.get(3)?,
        detail: r.get(4)?,
    })
}
