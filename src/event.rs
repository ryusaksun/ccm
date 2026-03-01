use crate::action::Action;
use crate::app::AppMode;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// 将终端事件映射为 Action
pub fn map_event(mode: &AppMode, event: Event) -> Option<Action> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => map_key(mode, key),
        Event::Resize(w, h) => Some(Action::Resize(w, h)),
        _ => None,
    }
}

fn map_key(mode: &AppMode, key: KeyEvent) -> Option<Action> {
    match mode {
        AppMode::Normal => map_normal_key(key),
        AppMode::Search => map_search_key(key),
        AppMode::Preview => map_preview_key(key),
        AppMode::Confirm(_) => map_confirm_key(key),
        AppMode::Help => map_help_key(key),
        AppMode::ProjectFilter => map_project_filter_key(key),
        AppMode::ExportChoice => map_export_key(key),
    }
}

fn map_normal_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Char('g') | KeyCode::Home => Some(Action::JumpTop),
        KeyCode::Char('G') | KeyCode::End => Some(Action::JumpBottom),
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::PageUp)
        }
        KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::PageDown)
        }
        KeyCode::PageUp => Some(Action::PageUp),
        KeyCode::PageDown => Some(Action::PageDown),
        KeyCode::Char('/') => Some(Action::EnterSearch),
        KeyCode::Enter | KeyCode::Char('r') => Some(Action::Select),
        KeyCode::Char('d') => Some(Action::DeleteSelected),
        KeyCode::Char('m') | KeyCode::Char(' ') => Some(Action::MarkSelected),
        KeyCode::Char('D') => Some(Action::DeleteMarked),
        KeyCode::Char('e') => Some(Action::ExportMarkdown),
        KeyCode::Char('s') => Some(Action::CycleSortField),
        KeyCode::Char('S') => Some(Action::ToggleSortOrder),
        KeyCode::Char('f') => Some(Action::ToggleProjectFilter),
        KeyCode::Char('p') => Some(Action::TogglePreview),
        KeyCode::Tab => Some(Action::FocusPreview),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        _ => None,
    }
}

fn map_search_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Enter => Some(Action::ExitSearch),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(Action::SearchClear)
        }
        KeyCode::Char(c) => Some(Action::SearchInput(c)),
        _ => None,
    }
}

fn map_preview_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Tab | KeyCode::Esc => Some(Action::UnfocusPreview),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::ScrollPreviewDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::ScrollPreviewUp),
        _ => None,
    }
}

fn map_confirm_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Some(Action::ConfirmYes),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Some(Action::ConfirmNo),
        _ => None,
    }
}

fn map_help_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => Some(Action::HideHelp),
        _ => None,
    }
}

fn map_project_filter_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Esc => Some(Action::ToggleProjectFilter),
        KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Enter => Some(Action::Select),
        _ => None,
    }
}

fn map_export_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('m') | KeyCode::Char('1') => Some(Action::ExportMarkdown),
        KeyCode::Char('j') | KeyCode::Char('2') => Some(Action::ExportJson),
        KeyCode::Esc => Some(Action::ConfirmNo),
        _ => None,
    }
}
