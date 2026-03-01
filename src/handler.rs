use crate::action::Action;
use crate::app::{App, AppMode, ConfirmAction};
use crate::data::export;
use std::path::PathBuf;

/// 处理 Action，修改 App 状态
pub fn handle_action(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.should_quit = true,

        Action::MoveUp => {
            if matches!(app.mode, AppMode::ProjectFilter) {
                let len = app.projects.len() + 1; // +1 for "All"
                let i = app
                    .project_list_state
                    .selected()
                    .unwrap_or(0)
                    .checked_sub(1)
                    .unwrap_or(len - 1);
                app.project_list_state.select(Some(i));
            } else {
                if let Some(selected) = app.table_state.selected() {
                    if selected > 0 {
                        app.table_state.select(Some(selected - 1));
                        app.load_preview_for_selected();
                    }
                }
            }
        }

        Action::MoveDown => {
            if matches!(app.mode, AppMode::ProjectFilter) {
                let len = app.projects.len() + 1;
                let i = app
                    .project_list_state
                    .selected()
                    .map(|s| if s + 1 >= len { 0 } else { s + 1 })
                    .unwrap_or(0);
                app.project_list_state.select(Some(i));
            } else {
                let len = app.filtered_indices.len();
                if len > 0 {
                    let selected = app.table_state.selected().unwrap_or(0);
                    if selected + 1 < len {
                        app.table_state.select(Some(selected + 1));
                        app.load_preview_for_selected();
                    }
                }
            }
        }

        Action::JumpTop => {
            if !app.filtered_indices.is_empty() {
                app.table_state.select(Some(0));
                app.load_preview_for_selected();
            }
        }

        Action::JumpBottom => {
            let len = app.filtered_indices.len();
            if len > 0 {
                app.table_state.select(Some(len - 1));
                app.load_preview_for_selected();
            }
        }

        Action::PageUp => {
            if let Some(selected) = app.table_state.selected() {
                let new = selected.saturating_sub(10);
                app.table_state.select(Some(new));
                app.load_preview_for_selected();
            }
        }

        Action::PageDown => {
            let len = app.filtered_indices.len();
            if len > 0 {
                if let Some(selected) = app.table_state.selected() {
                    let new = (selected + 10).min(len - 1);
                    app.table_state.select(Some(new));
                    app.load_preview_for_selected();
                }
            }
        }

        Action::Select => {
            match &app.mode {
                AppMode::ProjectFilter => {
                    let selected = app.project_list_state.selected().unwrap_or(0);
                    if selected == 0 {
                        app.selected_project = None;
                    } else {
                        app.selected_project = Some(selected - 1);
                    }
                    app.mode = AppMode::Normal;
                    app.apply_filter(true);
                }
                AppMode::Normal => {
                    if let Some(session) = app.selected_session() {
                        let sid = session.session_id.clone();
                        let path = session.original_path.clone();
                        app.mode = AppMode::Confirm(ConfirmAction::ResumeSession(sid, path));
                    }
                }
                _ => {}
            }
        }

        Action::EnterSearch => {
            app.mode = AppMode::Search;
        }

        Action::ExitSearch => {
            app.mode = AppMode::Normal;
        }

        Action::SearchInput(c) => {
            app.search_text.push(c);
            app.apply_filter(false);
        }

        Action::SearchBackspace => {
            app.search_text.pop();
            app.apply_filter(false);
        }

        Action::SearchClear => {
            app.search_text.clear();
            app.apply_filter(false);
        }

        Action::CycleSortField => {
            app.sort_field = app.sort_field.next();
            app.apply_filter(true);
        }

        Action::ToggleSortOrder => {
            app.sort_order = app.sort_order.toggle();
            app.apply_filter(true);
        }

        Action::ToggleProjectFilter => {
            if matches!(app.mode, AppMode::ProjectFilter) {
                app.mode = AppMode::Normal;
            } else {
                let current = match app.selected_project {
                    Some(i) => i + 1,
                    None => 0,
                };
                app.project_list_state.select(Some(current));
                app.mode = AppMode::ProjectFilter;
            }
        }

        Action::DeleteSelected => {
            if let Some(session) = app.selected_session() {
                let sid = session.session_id.clone();
                app.mode = AppMode::Confirm(ConfirmAction::DeleteSession(sid));
            }
        }

        Action::MarkSelected => {
            if let Some(real_idx) = app.selected_real_index() {
                if app.marked_sessions.contains(&real_idx) {
                    app.marked_sessions.remove(&real_idx);
                } else {
                    app.marked_sessions.insert(real_idx);
                }
                // 自动移到下一行
                let len = app.filtered_indices.len();
                if let Some(selected) = app.table_state.selected() {
                    if selected + 1 < len {
                        app.table_state.select(Some(selected + 1));
                        app.load_preview_for_selected();
                    }
                }
            }
        }

        Action::DeleteMarked => {
            if !app.marked_sessions.is_empty() {
                let ids: Vec<String> = app
                    .marked_sessions
                    .iter()
                    .filter_map(|&idx| app.all_sessions.get(idx))
                    .map(|s| s.session_id.clone())
                    .collect();
                app.mode = AppMode::Confirm(ConfirmAction::BulkDelete(ids));
            }
        }

        Action::ExportMarkdown => {
            if matches!(app.mode, AppMode::ExportChoice) {
                // 执行导出
                do_export(app, "md");
                app.mode = AppMode::Normal;
            } else {
                app.mode = AppMode::ExportChoice;
            }
        }

        Action::ExportJson => {
            do_export(app, "json");
            app.mode = AppMode::Normal;
        }

        Action::TogglePreview => {
            app.show_preview = !app.show_preview;
        }

        Action::FocusPreview => {
            if app.show_preview {
                app.mode = AppMode::Preview;
            }
        }

        Action::UnfocusPreview => {
            app.mode = AppMode::Normal;
        }

        Action::ScrollPreviewUp => {
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }

        Action::ScrollPreviewDown => {
            app.preview_scroll = app.preview_scroll.saturating_add(1);
        }

        Action::ShowHelp => {
            app.mode = AppMode::Help;
        }

        Action::HideHelp => {
            app.mode = AppMode::Normal;
        }

        Action::ConfirmYes => {
            let mode = app.mode.clone();
            match mode {
                AppMode::Confirm(ConfirmAction::DeleteSession(sid)) => {
                    app.delete_session(&sid);
                    app.mode = AppMode::Normal;
                }
                AppMode::Confirm(ConfirmAction::BulkDelete(ids)) => {
                    let count = ids.len();
                    for sid in ids {
                        app.delete_session(&sid);
                    }
                    app.set_status(format!("Deleted {} sessions", count), false);
                    app.mode = AppMode::Normal;
                }
                AppMode::Confirm(ConfirmAction::ResumeSession(sid, path)) => {
                    app.should_resume = Some((sid, path));
                    app.mode = AppMode::Normal;
                }
                _ => {
                    app.mode = AppMode::Normal;
                }
            }
        }

        Action::ConfirmNo => {
            app.mode = AppMode::Normal;
        }

        Action::Resize(_, _) => {}
    }
}

fn do_export(app: &mut App, format: &str) {
    if let Some(session) = app.selected_session() {
        let session = session.clone();
        let filename = format!("session_{}.{}", &session.session_id[..8], format);
        let output_path = PathBuf::from(&filename);

        let result = match format {
            "md" => export::export_markdown(&session, &output_path),
            "json" => export::export_json(&session, &output_path),
            _ => return,
        };

        match result {
            Ok(_) => {
                app.set_status(format!("Exported to {}", filename), false);
            }
            Err(e) => {
                app.set_status(format!("Export failed: {}", e), true);
            }
        }
    }
}
