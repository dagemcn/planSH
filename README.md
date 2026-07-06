# plansh

A lazygit-style terminal planner for college assignments. A Rust learning project.

## Build & run

```
cargo run
```

First run seeds a few sample assignments and creates `plansh.db` (SQLite) in the
working directory. Delete that file to start fresh.

> Don't commit a `Cargo.lock` copied from anywhere else — run `cargo run` once
> and let your toolchain generate a fresh one against current crate versions.

## Keys

| Key        | Action                                  |
|------------|-----------------------------------------|
| `j` / `k`  | Move selection in the focused panel     |
| `Tab`      | Cycle focus: Views → Courses → Assignments |
| `space`    | Cycle status (todo → doing → done)      |
| `a`        | Add assignment (popup stub — your build)|
| `d`        | Delete selected (asks to confirm)       |
| `q`        | Quit                                    |

## Layout

```
┌─Views──────┐┌─Assignments · This week────────────┐
│▌Today      ││▌○ Game of Life writeup      today  │
│ This week  ││ ◐ MPI I/O benchmark      2d overdue │
│ Overdue    ││ ● Schema design               done │
│ All        ││                                    │
├─Courses────┤│                                    │
│ Parallel   ││                                    │
│ OS         ││                                    │
│ Databases  ││                                    │
└────────────┘└────────────────────────────────────┘
 j/k move · Tab panel · space status · a add · d delete · q quit
```

## Module map

- `models.rs` — `Course`, `Assignment`, `Status`. No inter-struct references;
  relationships are by id. Status is an enum (exhaustive `match`).
- `db.rs` — one `Db` wrapping a rusqlite `Connection`. Schema created on open.
  Dates stored as ISO-8601 TEXT (SQLite has no date type).
- `app.rs` — `App` state plus `Panel` / `View` / `Mode` enums. Selection lives
  in ratatui `ListState` (an index, never a borrowed item).
- `ui.rs` — pure-ish rendering. Focused panel gets a bright border.
- `main.rs` — terminal setup/teardown + the draw/read/mutate event loop.

## Where to go next (rough order)

1. **Build the add form.** `Mode::Adding` is reachable but stubbed. Add a text
   input (capture chars in `handle_adding`, store a draft `String` on `App`),
   then a due-date field and a course picker. Call `db.add_assignment` on Enter.
2. **Filter assignments by selected course** when the Courses panel drives it.
3. **Edit mode** (`e`) — reuse the add form pre-filled from the selection.
4. **Persist window state** or add sort options.
5. Swap the seed data out once your own assignments are in.
