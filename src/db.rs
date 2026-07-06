//! SQLite persistence via rusqlite.
//!
//! One `Db` wraps one `Connection`. The schema is created on open if it does
//! not exist, so first run and subsequent runs take the same path. Dates are
//! stored as ISO-8601 TEXT (`YYYY-MM-DD`) because SQLite has no native date
//! type — see `models::Assignment::days_until_due` for where we turn that back
//! into arithmetic.

use crate::models::{Assignment, Course, Status};
use chrono::NaiveDate;
use rusqlite::{Connection, Result};

pub struct Db {
    conn: Connection,
}

impl Db {
    /// Open (or create) the database at `path` and ensure the schema exists.
    pub fn open(path: &str) -> Result<Db> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS course (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS assignment (
                id        INTEGER PRIMARY KEY,
                course_id INTEGER NOT NULL REFERENCES course(id) ON DELETE CASCADE,
                title     TEXT NOT NULL,
                due       TEXT NOT NULL,   -- ISO-8601 YYYY-MM-DD
                status    TEXT NOT NULL DEFAULT 'todo'
            );
            ",
        )?;
        // Foreign keys are off by default in SQLite; turn them on so the
        // ON DELETE CASCADE above actually fires.
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        Ok(Db { conn })
    }

    pub fn add_course(&self, name: &str) -> Result<i64> {
        self.conn
            .execute("INSERT INTO course (name) VALUES (?1)", [name])?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn add_assignment(
        &self,
        course_id: i64,
        title: &str,
        due: NaiveDate,
        status: Status,
    ) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO assignment (course_id, title, due, status)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![course_id, title, due.to_string(), status.as_str()],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn set_status(&self, assignment_id: i64, status: Status) -> Result<()> {
        self.conn.execute(
            "UPDATE assignment SET status = ?1 WHERE id = ?2",
            rusqlite::params![status.as_str(), assignment_id],
        )?;
        Ok(())
    }

    pub fn delete_assignment(&self, assignment_id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM assignment WHERE id = ?1", [assignment_id])?;
        Ok(())
    }

    pub fn courses(&self) -> Result<Vec<Course>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM course ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Course {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    /// All assignments, soonest-due first. The UI filters/groups these in
    /// memory; for a dataset this small there's no reason to push that into SQL.
    pub fn assignments(&self) -> Result<Vec<Assignment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, course_id, title, due, status
             FROM assignment ORDER BY due",
        )?;
        let rows = stmt.query_map([], |row| {
            let due_text: String = row.get(3)?;
            let status_text: String = row.get(4)?;
            Ok(Assignment {
                id: row.get(0)?,
                course_id: row.get(1)?,
                title: row.get(2)?,
                // A bad date in the DB shouldn't crash us; fall back to today.
                due: due_text
                    .parse::<NaiveDate>()
                    .unwrap_or_else(|_| chrono::Local::now().date_naive()),
                status: Status::from_str(&status_text),
            })
        })?;
        rows.collect()
    }

    /// True when there's nothing stored yet — used to decide whether to seed
    /// sample data on first run.
    pub fn is_empty(&self) -> Result<bool> {
        let n: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM assignment", [], |r| r.get(0))?;
        Ok(n == 0)
    }
}
