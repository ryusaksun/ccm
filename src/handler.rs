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
            app.apply_filter(true);
        }

        Action::SearchBackspace => {
            app.search_text.pop();
            app.apply_filter(true);
        }

        Action::SearchClear => {
            app.search_text.clear();
            app.apply_filter(true);
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
            if !app.preview_lines.is_empty() {
                app.preview_scroll = app.preview_scroll.saturating_sub(1);
            }
        }

        Action::ScrollPreviewDown => {
            if !app.preview_lines.is_empty() {
                app.preview_scroll = app.preview_scroll.saturating_add(1);
            }
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
                    app.delete_sessions_bulk(&ids);
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
        let sid_short: String = session.session_id.chars().take(8).collect();
        let filename = format!("session_{}.{}", sid_short, format);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{SessionRow, SortField, SortOrder};
    use chrono::Utc;
    use ratatui::widgets::{ListState, TableState};
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        dir.push(format!("ccm_{}_{}_{}", prefix, std::process::id(), ts));
        fs::create_dir_all(&dir).expect("should create temp dir");
        dir
    }

    fn build_session(
        session_id: &str,
        project_name: &str,
        project_dir: &Path,
        jsonl_path: &Path,
        first_prompt: &str,
        message_count: u32,
    ) -> SessionRow {
        SessionRow {
            session_id: session_id.to_string(),
            project_name: project_name.to_string(),
            project_dir: project_dir.to_path_buf(),
            original_path: project_dir.to_path_buf(),
            first_prompt: first_prompt.to_string(),
            summary: first_prompt.to_string(),
            message_count,
            created: Some(Utc::now()),
            modified: Some(Utc::now()),
            git_branch: "main".to_string(),
            file_size: 0,
            jsonl_path: jsonl_path.to_path_buf(),
            is_sidechain: false,
        }
    }

    fn build_app(sessions: Vec<SessionRow>) -> App {
        let mut projects: Vec<String> = sessions
            .iter()
            .map(|s| s.project_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        projects.sort();

        let mut table_state = TableState::default();
        if !sessions.is_empty() {
            table_state.select(Some(0));
        }

        App {
            mode: AppMode::Normal,
            should_quit: false,
            should_resume: None,
            all_sessions: sessions.clone(),
            filtered_indices: (0..sessions.len()).collect(),
            table_state,
            search_text: String::new(),
            sort_field: SortField::Modified,
            sort_order: SortOrder::Descending,
            projects,
            selected_project: None,
            project_list_state: ListState::default(),
            preview_lines: Vec::new(),
            preview_scroll: 0,
            preview_session_id: None,
            show_preview: true,
            marked_sessions: HashSet::new(),
            total_disk_usage: 0,
            status_message: None,
        }
    }

    #[test]
    fn search_input_preserves_current_sort_order() {
        let root = temp_dir("search_sort");
        let project = root.join("proj");
        fs::create_dir_all(&project).expect("should create project dir");

        let sid_a = "session-a";
        let sid_b = "session-b";
        let jsonl_a = project.join(format!("{}.jsonl", sid_a));
        let jsonl_b = project.join(format!("{}.jsonl", sid_b));
        fs::write(&jsonl_a, "{\"type\":\"user\",\"message\":{\"content\":\"a\"}}\n")
            .expect("should write jsonl a");
        fs::write(&jsonl_b, "{\"type\":\"user\",\"message\":{\"content\":\"b\"}}\n")
            .expect("should write jsonl b");

        let mut app = build_app(vec![
            build_session(sid_a, "proj", &project, &jsonl_a, "findme", 1),
            build_session(sid_b, "proj", &project, &jsonl_b, "findme", 10),
        ]);

        app.sort_field = SortField::Messages;
        app.sort_order = SortOrder::Descending;
        app.apply_filter(true);
        assert_eq!(app.filtered_indices, vec![1, 0]);

        handle_action(&mut app, Action::SearchInput('f'));
        assert_eq!(app.search_text, "f");
        assert_eq!(app.filtered_indices, vec![1, 0]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn export_json_with_short_session_id_does_not_panic() {
        let root = temp_dir("export_short_id");
        let project = root.join("proj");
        fs::create_dir_all(&project).expect("should create project dir");

        let sid = "x1";
        let jsonl = project.join(format!("{}.jsonl", sid));
        fs::write(&jsonl, "{\"type\":\"user\",\"message\":{\"content\":\"hello\"}}\n")
            .expect("should write jsonl");

        let mut app = build_app(vec![build_session(
            sid,
            "proj",
            &project,
            &jsonl,
            "hello",
            1,
        )]);

        let output = PathBuf::from("session_x1.json");
        let _ = fs::remove_file(&output);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            do_export(&mut app, "json");
        }));
        assert!(result.is_ok(), "export should not panic on short session id");
        assert!(output.exists(), "expected exported json file to exist");
        assert!(
            app.status_message
                .as_ref()
                .is_some_and(|m| !m.is_error && m.text.contains("Exported to session_x1.json"))
        );

        let _ = fs::remove_file(output);
        let _ = fs::remove_dir_all(root);
    }
}
