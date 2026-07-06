//! Application state and the logic that mutates it.
//!
//! Two enums drive the whole UI:
//!   * `Panel` — which of the three panes currently has focus.
//!   * `Mode`  — whether we're navigating normally or inside a popup.
//! Everything the event loop does is a `match` on one of these. This is the
//! pattern that replaces the pile of boolean flags you'd reach for in C.

use crate::db::Db;
use crate::models::{Assignment, Course};
use ratatui::widgets::ListState;

/// Which pane has focus. Tab cycles through them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Views,
    Courses,
    Assignments,
}

/// The saved "views" in the top-left panel. Each is just a predicate over the
/// assignment list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Today,
    Week,
    Overdue,
    All,
}

impl View {
    pub const ALL: [View; 4] = [View::Today, View::Week, View::Overdue, View::All];

    pub fn label(self) -> &'static str {
        match self {
            View::Today => "Today",
            View::Week => "This week",
            View::Overdue => "Overdue",
            View::All => "All",
        }
    }

    /// Does this assignment belong in this view?
    fn matches(self, a: &Assignment) -> bool {
        let days = a.days_until_due();
        match self {
            View::Today => days == 0,
            View::Week => (0..=7).contains(&days),
            View::Overdue => a.is_overdue(),
            View::All => true,
        }
    }
}

/// Top-level UI mode. Only `Normal` is fully wired in this skeleton; the popup
/// variants are here to show the shape you'll grow into (see the `match` in
/// `ui::render` and `main`'s event loop).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Adding,
    ConfirmDelete,
}

pub struct App {
    pub db: Db,
    pub courses: Vec<Course>,
    pub assignments: Vec<Assignment>,

    pub focus: Panel,
    pub mode: Mode,

    // ListState owns the "selected index" for each list. Note we store *state*,
    // never a reference to the selected item — we look the item up by index
    // when we need it. That's the borrow-checker-friendly pattern.
    pub view_state: ListState,
    pub course_state: ListState,
    pub assignment_state: ListState,

    pub should_quit: bool,
}

impl App {
    pub fn new(db: Db) -> rusqlite::Result<App> {
        let courses = db.courses()?;
        let assignments = db.assignments()?;

        let mut view_state = ListState::default();
        view_state.select(Some(0));
        let mut course_state = ListState::default();
        course_state.select(if courses.is_empty() { None } else { Some(0) });
        let assignment_state = ListState::default();

        let mut app = App {
            db,
            courses,
            assignments,
            focus: Panel::Assignments,
            mode: Mode::Normal,
            view_state,
            course_state,
            assignment_state,
            should_quit: false,
        };
        // Select the first visible assignment, if any.
        app.assignment_state
            .select(if app.visible_assignments().is_empty() {
                None
            } else {
                Some(0)
            });
        Ok(app)
    }

    pub fn active_view(&self) -> View {
        View::ALL[self.view_state.selected().unwrap_or(0)]
    }

    /// The assignments the right-hand pane should show: filtered by the active
    /// view, and (when the Courses pane is focused or a course is selected)
    /// optionally by course. Returns owned clones for simplicity — the dataset
    /// is tiny, so cloning costs nothing and keeps the borrows trivial.
    pub fn visible_assignments(&self) -> Vec<Assignment> {
        let view = self.active_view();
        self.assignments
            .iter()
            .filter(|a| view.matches(a))
            .cloned()
            .collect()
    }

    /// Reload from the database after a mutation.
    pub fn reload(&mut self) -> rusqlite::Result<()> {
        self.courses = self.db.courses()?;
        self.assignments = self.db.assignments()?;
        self.clamp_selection();
        Ok(())
    }

    /// Keep the selected index in range after the list length changes.
    fn clamp_selection(&mut self) {
        let len = self.visible_assignments().len();
        let sel = match len {
            0 => None,
            n => Some(self.assignment_state.selected().unwrap_or(0).min(n - 1)),
        };
        self.assignment_state.select(sel);
    }

    // --- navigation -------------------------------------------------------

    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            Panel::Views => Panel::Courses,
            Panel::Courses => Panel::Assignments,
            Panel::Assignments => Panel::Views,
        };
    }

    /// Focus a specific panel directly (bound to the number keys).
    pub fn focus_panel(&mut self, panel: Panel) {
        self.focus = panel;
    }

    /// Move the selection in whichever pane has focus. `delta` is +1 or -1.
    pub fn move_selection(&mut self, delta: isize) {
        let (state, len) = match self.focus {
            Panel::Views => (&mut self.view_state, View::ALL.len()),
            Panel::Courses => (&mut self.course_state, self.courses.len()),
            Panel::Assignments => (
                &mut self.assignment_state,
                // Can't call self.visible_assignments() here — self is already
                // borrowed mutably. Recompute the length inline instead.
                self.assignments
                    .iter()
                    .filter(|a| {
                        View::ALL[self.view_state.selected().unwrap_or(0)].matches(a)
                    })
                    .count(),
            ),
        };
        if len == 0 {
            state.select(None);
            return;
        }
        let current = state.selected().unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        state.select(Some(next));
    }

    /// The assignment currently highlighted in the right pane, if any.
    pub fn selected_assignment(&self) -> Option<Assignment> {
        let idx = self.assignment_state.selected()?;
        self.visible_assignments().into_iter().nth(idx)
    }

    // --- mutations --------------------------------------------------------

    /// Cycle the highlighted assignment's status and persist it.
    pub fn cycle_status(&mut self) -> rusqlite::Result<()> {
        if let Some(a) = self.selected_assignment() {
            self.db.set_status(a.id, a.status.next())?;
            self.reload()?;
        }
        Ok(())
    }

    /// Delete the highlighted assignment and persist it.
    pub fn delete_selected(&mut self) -> rusqlite::Result<()> {
        if let Some(a) = self.selected_assignment() {
            self.db.delete_assignment(a.id)?;
            self.reload()?;
        }
        self.mode = Mode::Normal;
        Ok(())
    }
}