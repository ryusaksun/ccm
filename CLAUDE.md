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

1. **`event.rs`** — Maps crossterm key events to `Action` variants based on current `AppMode` (Normal, Search, Preview, Confirm, Help, ProjectFilter, ExportChoice)
2. **`action.rs`** — Enum of all possible user actions
3. **`handler.rs`** — Applies actions to `App` state (the single mutation point)
4. **`app.rs`** — Central `App` struct holding all application state

### Data Layer (`src/data/`)

- **`scanner.rs`** — Scans `~/.claude/projects/` for sessions. Two strategies:
  - From `sessions-index.json` (preferred, contains metadata)
  - Fallback: parse JSONL files directly to extract cwd, timestamps, message counts
- **`models.rs`** — Data types: `SessionRow`, `IndexEntry`, `SessionsIndex`, `PreviewLine`, `SortField`, `SortOrder`
- **`session.rs`** — Loads conversation preview from JSONL files (lazy, capped at N messages)
- **`export.rs`** — Exports sessions to Markdown or JSON format

### UI Layer (`src/ui/`)

- **`layout.rs`** — Top-level layout: search bar → session table → preview panel → status bar
- **`session_list.rs`** — Main session table widget
- **`preview.rs`** — Conversation preview panel (User/Assistant/System messages)
- **`search_bar.rs`** — Search input with sort indicator
- **`status_bar.rs`** — Bottom bar showing session count, disk usage, status messages
- **`popup.rs`** — Modal dialogs (confirm, help, project filter, export choice)
- **`theme.rs`** — Color scheme constants

### Key Design Decisions

- `App` stores `all_sessions` (full data) and `filtered_indices` (view into it) — filtering/sorting operates on indices only
- Preview is lazily loaded and cached by session ID (`preview_session_id`)
- Session resume works by setting `should_resume`, breaking the inner loop, spawning `claude -r`, then re-entering the TUI
- Deletion removes JSONL file, session directory, transcript files, and updates `sessions-index.json`
