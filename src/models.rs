//! Core domain types: courses, assignments, and their status.
//!
//! Note the deliberate absence of references between these structs. A `Course`
//! does not hold a `Vec<&Assignment>`; instead an `Assignment` stores the
//! `course_id` it belongs to, and we join them at query time. This is the
//! idiomatic Rust move — ownership stays simple and the borrow checker stays
//! quiet.

use chrono::{Local, NaiveDate};

/// Where an assignment is in its lifecycle.
///
/// Modeling this as an enum (rather than, say, a string or an integer flag)
/// means the compiler forces every `match` to handle every case. Add a variant
/// later and the compiler shows you every place that needs updating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Todo,
    Doing,
    Done,
}

impl Status {
    /// The glyph shown in the list, lazygit-style.
    pub fn glyph(self) -> &'static str {
        match self {
            Status::Todo => "○",
            Status::Doing => "◐",
            Status::Done => "●",
        }
    }

    /// Round-trips through the database as a short lowercase string.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::Doing => "doing",
            Status::Done => "done",
        }
    }

    /// Parse back from the stored string. Unknown values fall back to `Todo`
    /// rather than panicking — a corrupt row shouldn't crash the app.
    pub fn from_str(s: &str) -> Status {
        match s {
            "doing" => Status::Doing,
            "done" => Status::Done,
            _ => Status::Todo,
        }
    }

    /// Advance to the next status, cycling around. Bound to a keypress so you
    /// can tap through todo → doing → done → todo.
    pub fn next(self) -> Status {
        match self {
            Status::Todo => Status::Doing,
            Status::Doing => Status::Done,
            Status::Done => Status::Todo,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Course {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub id: i64,
    pub course_id: i64,
    pub title: String,
    pub due: NaiveDate,
    pub status: Status,
}

impl Assignment {
    /// Days until due. Negative means overdue. Computed in Rust rather than in
    /// SQL because SQLite has no real date type — we store an ISO-8601 string
    /// and do the arithmetic here with chrono.
    pub fn days_until_due(&self) -> i64 {
        (self.due - Local::now().date_naive()).num_days()
    }

    pub fn is_overdue(&self) -> bool {
        self.status != Status::Done && self.days_until_due() < 0
    }
}
