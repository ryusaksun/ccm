# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**ccm** (Claude Code session Manager) — A Rust TUI application for browsing, searching, and managing Claude Code sessions stored in `~/.claude/projects/`. Built with ratatui + crossterm.

## Build & Run

```bash
cargo build              # Debug build
cargo build --release    # Release build (with LTO + strip)
cargo run                # Run in debug mode
```

No tests or linting configured yet.

## Architecture

The app follows a unidirectional data flow pattern: **Event → Action → Handler → State → UI**.

### Core Loop (`main.rs`)
- Terminal setup with crossterm alternate screen
- Outer loop handles session resume: after `claude -r <id>` exits, re-enters TUI
- Inner loop: render → poll events → map to action → handle action

### Data Flow

1. **`event.rs`** — Maps crossterm key events to `Action` variants based on current `AppMode`. Each mode has its own `map_*_key()` function.
2. **`action.rs`** — Enum of all possible user actions
3. **`handler.rs`** — Single `handle_action()` function: the only place that mutates `App` state
4. **`app.rs`** — Central `App` struct holding all application state, plus data operations (filtering, sorting, deletion)

### Adding a New Feature (Common Pattern)

1. Add variant to `Action` enum in `action.rs`
2. Add key mapping in the appropriate `map_*_key()` function in `event.rs` (or add a new `AppMode` variant + mapper for a new mode)
3. Handle the action in `handler.rs::handle_action()`
4. If new UI is needed, add/modify widgets in `src/ui/`

### Data Layer (`src/data/`)

- **`scanner.rs`** — Scans `~/.claude/projects/` for sessions. Two strategies:
  - From `sessions-index.json` (preferred, contains metadata)
  - Fallback: parse JSONL files directly to extract cwd, timestamps, message counts
- **`models.rs`** — All data types: `SessionRow`, `IndexEntry`, `SessionsIndex`, `PreviewLine`, `SortField`, `SortOrder`
- **`session.rs`** — Loads conversation preview from JSONL files (lazy, capped at N messages)
- **`export.rs`** — Exports sessions to Markdown or JSON format

### UI Layer (`src/ui/`)

Layout is composed top-to-bottom: search bar → session table → preview panel → status bar. Each has its own module. `popup.rs` handles all modal dialogs (confirm, help, project filter, export choice). `theme.rs` defines color constants.

### Key Design Decisions

- `App` stores `all_sessions` (full data) and `filtered_indices` (view into it) — filtering/sorting operates on indices only
- Preview is lazily loaded and cached by session ID (`preview_session_id`)
- Session resume works by setting `should_resume`, breaking the inner loop, spawning `claude -r`, then re-entering the TUI
- Deletion removes JSONL file, session directory, transcript files, and updates `sessions-index.json`
- Marked sessions track indices into `all_sessions` (not `filtered_indices`), and indices are adjusted on deletion

## Conventions

- Code comments are in Chinese (简体中文)
- No `clippy` or `rustfmt` config — use Rust defaults
