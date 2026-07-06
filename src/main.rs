//! plansh — a lazygit-style TUI planner for college assignments.
//!
//! Structure:
//!   models  — the data (Course, Assignment, Status)
//!   db      — SQLite persistence
//!   app     — application state + the logic that mutates it
//!   ui      — rendering (pure-ish; reads state, draws a frame)
//!   main    — terminal setup/teardown + the event loop
//!
//! The event loop is the heart of any TUI: draw the current state, block for
//! an input event, mutate state in response, repeat.

mod app;
mod db;
mod models;
mod ui;

use app::{App, Mode, Panel};
use chrono::{Duration, Local};
use db::Db;
use models::Status;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;

fn main() -> io::Result<()> {
    let db = Db::open("plansh.db").expect("open database");
    seed_if_empty(&db).expect("seed database");

    let mut app = App::new(db).expect("build app state");

    // ratatui::init() enters raw mode + the alternate screen and hands back a
    // terminal. ratatui::restore() undoes it. Using them as a pair means even
    // if we return early, the terminal is left clean (they also install a panic
    // hook that restores on panic — no more garbled shell after a crash).
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, app))?;

        // Block until the next event. `read` returns key presses, resizes, etc.
        let Event::Key(key) = event::read()? else {
            continue;
        };
        // On Windows a key event fires on both press and release; guard so each
        // keystroke acts once. Harmless on macOS/Linux.
        if key.kind != KeyEventKind::Press {
            continue;
        }

        // Dispatch depends on the mode — the enum-driven pattern in action.
        match app.mode {
            Mode::Normal => handle_normal(app, key.code),
            Mode::ConfirmDelete => handle_confirm_delete(app, key.code),
            Mode::Adding => handle_adding(app, key.code),
        }
    }
    Ok(())
}

fn handle_normal(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        // j/k swapped from the vim default per preference: k moves down, j up.
        // Arrow keys stay conventional.
        KeyCode::Char('k') | KeyCode::Down => app.move_selection(1),
        KeyCode::Char('j') | KeyCode::Up => app.move_selection(-1),
        // Jump straight to a panel by number, lazygit-style. Tab still cycles.
        KeyCode::Tab => app.focus_next(),
        KeyCode::Char('1') => app.focus_panel(Panel::Views),
        KeyCode::Char('2') => app.focus_panel(Panel::Courses),
        KeyCode::Char('3') => app.focus_panel(Panel::Assignments),
        KeyCode::Char(' ') => {
            // Cycle status of the highlighted assignment.
            let _ = app.cycle_status();
        }
        KeyCode::Char('d') => {
            if app.selected_assignment().is_some() {
                app.mode = Mode::ConfirmDelete;
            }
        }
        KeyCode::Char('a') => app.mode = Mode::Adding,
        _ => {}
    }
}

fn handle_confirm_delete(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('y') => {
            let _ = app.delete_selected();
        }
        KeyCode::Char('n') | KeyCode::Esc => app.mode = Mode::Normal,
        _ => {}
    }
}

fn handle_adding(app: &mut App, code: KeyCode) {
    // Stub: the real form (text input for title, date, course pick) is your
    // next build step. For now Esc just backs out so the mode is reachable and
    // exitable.
    if code == KeyCode::Esc {
        app.mode = Mode::Normal;
    }
}

/// Populate a few rows on first run so there's something to look at.
fn seed_if_empty(db: &Db) -> rusqlite::Result<()> {
    if !db.is_empty()? {
        return Ok(());
    }
    let today = Local::now().date_naive();
    let parallel = db.add_course("Parallel Programming")?;
    let os = db.add_course("Operating Systems")?;
    let db_course = db.add_course("Databases")?;

    db.add_assignment(parallel, "MPI Parallel I/O benchmark", today - Duration::days(2), Status::Doing)?;
    db.add_assignment(parallel, "Game of Life writeup", today + Duration::days(1), Status::Todo)?;
    db.add_assignment(os, "CPU scheduler simulator", today + Duration::days(5), Status::Todo)?;
    db.add_assignment(os, "Threads reading", today - Duration::days(1), Status::Todo)?;
    db.add_assignment(db_course, "Query optimizer PR", today + Duration::days(10), Status::Todo)?;
    db.add_assignment(db_course, "Schema design", today, Status::Done)?;
    Ok(())
}