//! All rendering. `render` is called once per frame with the whole `App`; it
//! never mutates state (well, except ListState, which ratatui needs `&mut` for
//! to track scrolling). Keeping draw code side-effect-free makes the loop easy
//! to reason about: state changes happen in the event handler, drawing just
//! reflects state.

use crate::app::{App, Mode, Panel, View};
use crate::models::Status;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Accent colour for the focused panel's border — the lazygit "bright border on
/// the active pane" effect.
const ACCENT: Color = Color::Cyan;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Outer rows: main area on top, one-line keybinding hint at the bottom.
    let [main, hint] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).areas(frame.area());

    // Main area: narrow sidebar on the left, assignment list on the right.
    let [sidebar, list] =
        Layout::horizontal([Constraint::Length(22), Constraint::Min(0)]).areas(main);

    // Sidebar splits into Views (top) and Courses (fills the rest).
    let [views, courses] =
        Layout::vertical([Constraint::Length(6), Constraint::Min(0)]).areas(sidebar);

    render_views(frame, app, views);
    render_courses(frame, app, courses);
    render_assignments(frame, app, list);
    render_hint(frame, app, hint);

    // Popups draw on top of everything. Only stubs here — this is where your
    // add / delete-confirm forms will live.
    match app.mode {
        Mode::Adding => render_popup(frame, "Add assignment", "form goes here — Esc to cancel"),
        Mode::ConfirmDelete => {
            render_popup(frame, "Delete?", "y to confirm · n / Esc to cancel")
        }
        Mode::Normal => {}
    }
}

/// A titled block whose border brightens when its panel has focus.
fn panel_block(title: &str, focused: bool) -> Block<'_> {
    let border_style = if focused {
        Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::bordered().title(title).border_style(border_style)
}

fn render_views(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Panel::Views;
    let items: Vec<ListItem> = View::ALL
        .iter()
        .map(|v| ListItem::new(v.label()))
        .collect();
    let list = List::new(items)
        .block(panel_block("Views", focused))
        .highlight_style(highlight(focused))
        .highlight_symbol("▌");
    frame.render_stateful_widget(list, area, &mut app.view_state);
}

fn render_courses(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Panel::Courses;
    let items: Vec<ListItem> = app
        .courses
        .iter()
        .map(|c| ListItem::new(c.name.clone()))
        .collect();
    let list = List::new(items)
        .block(panel_block("Courses", focused))
        .highlight_style(highlight(focused))
        .highlight_symbol("▌");
    frame.render_stateful_widget(list, area, &mut app.course_state);
}

fn render_assignments(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Panel::Assignments;
    let visible = app.visible_assignments();

    let items: Vec<ListItem> = visible
        .iter()
        .map(|a| {
            let days = a.days_until_due();
            // Colour by urgency: overdue red, due-soon yellow, done dim, else default.
            let colour = if a.status == Status::Done {
                Color::DarkGray
            } else if days < 0 {
                Color::Red
            } else if days <= 2 {
                Color::Yellow
            } else {
                Color::White
            };

            let when = if a.status == Status::Done {
                "done".to_string()
            } else if days < 0 {
                format!("{}d overdue", -days)
            } else if days == 0 {
                "today".to_string()
            } else {
                format!("in {days}d")
            };

            let line = Line::from(vec![
                Span::raw(format!("{} ", a.status.glyph())),
                Span::styled(
                    format!("{:<28}", truncate(&a.title, 28)),
                    Style::default().fg(colour),
                ),
                Span::styled(when, Style::default().fg(colour).add_modifier(Modifier::DIM)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let title = format!("Assignments · {}", app.active_view().label());
    let list = List::new(items)
        .block(panel_block(&title, focused))
        .highlight_style(highlight(focused))
        .highlight_symbol("▌");
    frame.render_stateful_widget(list, area, &mut app.assignment_state);
}

fn render_hint(frame: &mut Frame, app: &mut App, area: Rect) {
    // Context-sensitive, like lazygit's bottom bar.
    let keys = match app.mode {
        Mode::Normal => "j/k move · Tab panel · space status · a add · d delete · q quit",
        Mode::Adding => "type · Enter save · Esc cancel",
        Mode::ConfirmDelete => "y confirm · n cancel",
    };
    let hint = Paragraph::new(keys).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(hint, area);
}

fn render_popup(frame: &mut Frame, title: &str, body: &str) {
    let area = centered_rect(50, 20, frame.area());
    frame.render_widget(Clear, area); // wipe whatever's underneath
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(ACCENT));
    let p = Paragraph::new(body).block(block);
    frame.render_widget(p, area);
}

fn highlight(focused: bool) -> Style {
    if focused {
        Style::default().bg(Color::Rgb(40, 40, 50)).bold()
    } else {
        Style::default().add_modifier(Modifier::DIM)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('…');
        t
    }
}

/// A rectangle `pct_x`% wide and `pct_y`% tall, centered in `area`.
fn centered_rect(pct_x: u16, pct_y: u16, area: Rect) -> Rect {
    let [_, mid, _] = Layout::vertical([
        Constraint::Percentage((100 - pct_y) / 2),
        Constraint::Percentage(pct_y),
        Constraint::Percentage((100 - pct_y) / 2),
    ])
    .areas(area);
    let [_, center, _] = Layout::horizontal([
        Constraint::Percentage((100 - pct_x) / 2),
        Constraint::Percentage(pct_x),
        Constraint::Percentage((100 - pct_x) / 2),
    ])
    .areas(mid);
    center
}
