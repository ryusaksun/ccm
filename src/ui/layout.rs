use crate::app::{App, AppMode};
use crate::ui::{popup, preview, search_bar, session_list, status_bar};
use ratatui::layout::{Constraint, Layout};
use ratatui::Frame;

/// 渲染整个 UI
pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = if app.show_preview {
        Layout::vertical([
            Constraint::Length(3),     // 搜索栏
            Constraint::Percentage(55), // 会话列表
            Constraint::Percentage(35), // 预览面板
            Constraint::Length(1),     // 状态栏
        ])
        .split(f.area())
    } else {
        Layout::vertical([
            Constraint::Length(3),  // 搜索栏
            Constraint::Min(5),    // 会话列表
            Constraint::Length(1), // 状态栏
        ])
        .split(f.area())
    };

    // 搜索栏
    search_bar::render(f, app, chunks[0]);

    // 会话列表
    session_list::render(f, app, chunks[1]);

    // 预览面板（如果显示）
    if app.show_preview && chunks.len() > 3 {
        preview::render(f, app, chunks[2]);
        status_bar::render(f, app, chunks[3]);
    } else {
        status_bar::render(f, app, chunks[chunks.len() - 1]);
    }

    // 弹出层
    match &app.mode {
        AppMode::Confirm(_) => popup::render_confirm(f, app),
        AppMode::Help => popup::render_help(f),
        AppMode::ProjectFilter => popup::render_project_filter(f, app),
        AppMode::ExportChoice => popup::render_export_choice(f),
        _ => {}
    }
}
