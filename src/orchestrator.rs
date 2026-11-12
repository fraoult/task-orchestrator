//! Task-orchestrator additions to Kanzo's board.
//!
//! Our schema changes live in [`ORCH_MIGRATIONS`], versioned under their own `meta` key
//! (`orch_schema_version`), so Kanzo's `MIGRATIONS` and `schema_version` stay exactly as
//! upstream has them and upstream migrations keep merging cleanly. They run right after
//! Kanzo's, from [`crate::store::Store::migrate`].

use rusqlite::{Connection, OptionalExtension};

use crate::store::Result;

/// The `meta` key that holds how many of [`ORCH_MIGRATIONS`] have been applied.
const VERSION_KEY: &str = "orch_schema_version";

/// Only ever append: each entry runs once, in order, on every board.
const ORCH_MIGRATIONS: &[&str] = &[
    // Who holds a task and whether they are still alive. All empty on an unclaimed task.
    // `events` has no foreign key on purpose: the history of a task outlives the task, and
    // a reference would either block deleting it or cascade the history away.
    "ALTER TABLE tasks ADD COLUMN claimed_by TEXT;
     ALTER TABLE tasks ADD COLUMN claimed_at TEXT;
     ALTER TABLE tasks ADD COLUMN heartbeat_at TEXT;
     ALTER TABLE tasks ADD COLUMN worktree TEXT;
     CREATE TABLE events (
         id      INTEGER PRIMARY KEY AUTOINCREMENT,
         task_id INTEGER NOT NULL,
         kind    TEXT NOT NULL,
         actor   TEXT NOT NULL DEFAULT '',
         detail  TEXT NOT NULL DEFAULT '',
         at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
     );
     CREATE INDEX events_by_task ON events(task_id, id);
     CREATE TRIGGER events_no_update BEFORE UPDATE ON events BEGIN SELECT RAISE(ABORT, 'events are append-only'); END;
     CREATE TRIGGER events_no_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT, 'events are append-only'); END;",
];

fn version(c: &Connection) -> rusqlite::Result<usize> {
    Ok(c.query_row("SELECT value FROM meta WHERE key = ?1", [VERSION_KEY], |r| r.get::<_, String>(0))
        .optional()?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

/// Applies the migrations this board hasn't had yet. Expects Kanzo's migrations to have
/// run, so `meta` and `tasks` exist. Like Kanzo's, it writes nothing when up to date, and
/// leaves alone a board already at a newer version than this build knows.
pub fn migrate(c: &Connection) -> Result<()> {
    let latest = ORCH_MIGRATIONS.len();
    if version(c)? >= latest {
        return Ok(());
    }
    c.execute_batch("BEGIN IMMEDIATE")?;
    let applied = (|| -> Result<()> {
        // Read again inside the transaction: another process may have migrated meanwhile.
        let current = version(c)?;
        if current >= latest {
            return Ok(());
        }
        for sql in &ORCH_MIGRATIONS[current..] {
            c.execute_batch(sql)?;
        }
        c.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            [VERSION_KEY, &latest.to_string()],
        )?;
        Ok(())
    })();
    match applied {
        Ok(()) => Ok(c.execute_batch("COMMIT")?),
        Err(e) => {
            let _ = c.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn columns(c: &Connection, table: &str) -> Vec<String> {
        let mut stmt = c.prepare(&format!("SELECT name FROM pragma_table_info('{table}')")).unwrap();
        stmt.query_map([], |r| r.get(0)).unwrap().collect::<rusqlite::Result<_>>().unwrap()
    }

    fn schema(c: &Connection) -> Vec<(String, String)> {
        let mut stmt = c.prepare("SELECT name, coalesce(sql, '') FROM sqlite_master ORDER BY name").unwrap();
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?))).unwrap().collect::<rusqlite::Result<_>>().unwrap()
    }

    fn assert_migrated(c: &Connection) {
        let cols = columns(c, "tasks");
        for col in ["claimed_by", "claimed_at", "heartbeat_at", "worktree"] {
            assert!(cols.iter().any(|c| c == col), "tasks has no {col}: {cols:?}");
        }
        assert_eq!(columns(c, "events"), ["id", "task_id", "kind", "actor", "detail", "at"]);
        assert_eq!(version(c).unwrap(), ORCH_MIGRATIONS.len());
    }

    #[test]
    fn applies_to_a_fresh_board() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanzo.db");
        Store::open(&path, None).unwrap().migrate().unwrap();
        assert_migrated(&Connection::open(&path).unwrap());
    }

    #[test]
    fn applies_to_an_existing_kanzo_board_and_keeps_its_tasks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanzo.db");
        let task = {
            let s = Store::open(&path, None).unwrap();
            s.migrate().unwrap();
            let p = s.create_project("api", "API", None).unwrap();
            let t = s.create_task(p.id, "Fix login", "").unwrap();
            drop(s);
            // Turn it back into a board as plain Kanzo leaves it.
            let c = Connection::open(&path).unwrap();
            c.execute_batch(
                "DROP TRIGGER events_no_update; DROP TRIGGER events_no_delete; DROP TABLE events;
                 ALTER TABLE tasks DROP COLUMN claimed_by; ALTER TABLE tasks DROP COLUMN claimed_at;
                 ALTER TABLE tasks DROP COLUMN heartbeat_at; ALTER TABLE tasks DROP COLUMN worktree;
                 DELETE FROM meta WHERE key = 'orch_schema_version';",
            )
            .unwrap();
            assert!(!columns(&c, "tasks").iter().any(|c| c == "claimed_by"));
            t
        };
        let s = Store::open(&path, None).unwrap();
        s.migrate().unwrap();
        assert_migrated(&Connection::open(&path).unwrap());
        assert_eq!(s.find_task(&task.key()).unwrap().title, "Fix login");
    }

    #[test]
    fn running_twice_changes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanzo.db");
        let s = Store::open(&path, None).unwrap();
        s.migrate().unwrap();
        let c = Connection::open(&path).unwrap();
        let (before, changes) = (schema(&c), s.changes().unwrap());
        s.migrate().unwrap();
        migrate(&c).unwrap();
        assert_eq!(schema(&c), before);
        assert_eq!(s.changes().unwrap(), changes);
        assert_migrated(&c);
    }

    #[test]
    fn leaves_a_newer_board_alone() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(
            "CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
             INSERT INTO meta VALUES ('orch_schema_version', '999');",
        )
        .unwrap();
        migrate(&c).unwrap();
        assert_eq!(version(&c).unwrap(), 999);
    }

    #[test]
    fn events_are_append_only() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("kanzo.db");
        Store::open(&path, None).unwrap().migrate().unwrap();
        let c = Connection::open(&path).unwrap();
        c.execute("INSERT INTO events (task_id, kind, actor) VALUES (1, 'claimed', 'agent')", []).unwrap();
        assert!(c.execute("UPDATE events SET actor = 'x'", []).is_err());
        assert!(c.execute("DELETE FROM events", []).is_err());
    }
}
