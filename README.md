# plansh

A terminal planner, made mostly for me to learn Rust.

## Build & run

```
cargo run
```

First run seeds a few sample assignments and creates `plansh.db` (SQLite) in the
working directory.

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
│▌Today      ││▌○ Operating System HW 2     today  │
│ This week  ││ ◐ Data Structures HW 1  2d overdue │
│ Overdue    ││ ● Economics HW 3              done │
│ All        ││                                    │
├─Courses────┤│                                    │
│ Econ       ││                                    │
│ OS         ││                                    │
│ DS         ││                                    │
└────────────┘└────────────────────────────────────┘
 j/k move · Tab panel · space status · a add · d delete · q quit
```
