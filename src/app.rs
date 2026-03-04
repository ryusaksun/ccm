use crate::data::models::{PreviewLine, SessionRow, SortField, SortOrder};
use crate::data::scanner;
use crate::data::session;
use ratatui::widgets::{ListState, TableState};
use std::collections::HashSet;
use std::path::PathBuf;

/// 应用模式
#[derive(Debug, Clone)]
pub enum AppMode {
    Normal,
    Search,
    Preview,
    Confirm(ConfirmAction),
    Help,
    ProjectFilter,
    ExportChoice,
}

/// 需要确认的操作
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    DeleteSession(String),
    BulkDelete(Vec<String>),
    ResumeSession(String, std::path::PathBuf),
}

/// 状态栏消息
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

/// 应用状态
pub struct App {
    pub mode: AppMode,
    pub should_quit: bool,
    pub should_resume: Option<(String, PathBuf)>, // (session_id, project_path)

    // 数据
    pub all_sessions: Vec<SessionRow>,
    pub filtered_indices: Vec<usize>,

    // 表格状态
    pub table_state: TableState,

    // 搜索
    pub search_text: String,

    // 排序
    pub sort_field: SortField,
    pub sort_order: SortOrder,

    // 项目过滤
    pub projects: Vec<String>,
    pub selected_project: Option<usize>, // None = 所有项目
    pub project_list_state: ListState,

    // 预览
    pub preview_lines: Vec<PreviewLine>,
    pub preview_scroll: u16,
    pub preview_session_id: Option<String>,
    pub show_preview: bool,

    // 标记
    pub marked_sessions: HashSet<usize>,

    // 统计
    pub total_disk_usage: u64,

    // 状态消息
    pub status_message: Option<StatusMessage>,
}

enum DeleteOutcome {
    Deleted { has_warning: bool },
    Failed,
}

impl App {
    pub fn new() -> Self {
        let (all_sessions, total_disk_usage) = scanner::scan_all_sessions();

        // 收集所有项目名
        let mut projects: Vec<String> = all_sessions
            .iter()
            .map(|s| s.project_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        projects.sort();

        let filtered_indices: Vec<usize> = (0..all_sessions.len()).collect();

        let mut app = App {
            mode: AppMode::Normal,
            should_quit: false,
            should_resume: None,
            all_sessions,
            filtered_indices,
            table_state: TableState::default(),
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
            total_disk_usage,
            status_message: None,
        };

        app.apply_filter(true);
        if !app.filtered_indices.is_empty() {
            app.table_state.select(Some(0));
            app.load_preview_for_selected();
        }

        app
    }

    /// 获取当前选中的会话
    pub fn selected_session(&self) -> Option<&SessionRow> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i))
            .and_then(|&idx| self.all_sessions.get(idx))
    }

    /// 获取当前选中的真实索引
    pub fn selected_real_index(&self) -> Option<usize> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_indices.get(i).copied())
    }

    /// 应用搜索和过滤
    pub fn apply_filter(&mut self, needs_sort: bool) {
        let search_lower = self.search_text.to_lowercase();
        let project_filter = self
            .selected_project
            .and_then(|i| self.projects.get(i))
            .cloned();

        let mut indices: Vec<usize> = self
            .all_sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                // 项目过滤
                if let Some(ref proj) = project_filter {
                    if &s.project_name != proj {
                        return false;
                    }
                }

                // 搜索过滤
                if !search_lower.is_empty() {
                    let matches = s.first_prompt.to_lowercase().contains(&search_lower)
                        || s.summary.to_lowercase().contains(&search_lower)
                        || s.session_id.to_lowercase().contains(&search_lower)
                        || s.project_name.to_lowercase().contains(&search_lower)
                        || s.git_branch.to_lowercase().contains(&search_lower);
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .map(|(i, _)| i)
            .collect();

        // 仅在排序条件变化时重新排序
        if needs_sort {
            self.sort_indices(&mut indices);
        }

        self.filtered_indices = indices;

        // 调整选择
        if self.filtered_indices.is_empty() {
            self.table_state.select(None);
        } else {
            let selected = self
                .table_state
                .selected()
                .unwrap_or(0)
                .min(self.filtered_indices.len() - 1);
            self.table_state.select(Some(selected));
        }

        self.load_preview_for_selected();
    }

    /// 根据当前会话重建项目列表，并尽量保持当前选中项目
    fn refresh_projects(&mut self) {
        let selected_project_name = self
            .selected_project
            .and_then(|i| self.projects.get(i))
            .cloned();

        let mut projects: Vec<String> = self
            .all_sessions
            .iter()
            .map(|s| s.project_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        projects.sort();
        self.projects = projects;

        self.selected_project = selected_project_name
            .and_then(|name| self.projects.iter().position(|p| p == &name));

        let popup_selected = self.selected_project.map(|i| i + 1).unwrap_or(0);
        self.project_list_state.select(Some(popup_selected));
    }

    fn sort_indices(&self, indices: &mut [usize]) {
        let sessions = &self.all_sessions;
        let field = self.sort_field;
        let order = self.sort_order;

        indices.sort_by(|&a, &b| {
            let sa = &sessions[a];
            let sb = &sessions[b];
            let cmp = match field {
                SortField::Modified => sa.modified.cmp(&sb.modified),
                SortField::Created => sa.created.cmp(&sb.created),
                SortField::Messages => sa.message_count.cmp(&sb.message_count),
                SortField::Project => sa.project_name.cmp(&sb.project_name),
            };
            match order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    /// 加载当前选中会话的预览
    pub fn load_preview_for_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            let sid = session.session_id.clone();
            if self.preview_session_id.as_ref() != Some(&sid) {
                self.preview_lines = session::load_preview(&session.jsonl_path, 100);
                self.preview_session_id = Some(sid);
                self.preview_scroll = 0;
            }
        } else {
            self.preview_lines.clear();
            self.preview_session_id = None;
            self.preview_scroll = 0;
        }
    }

    /// 删除会话（删除文件 + 从内存移除，但不调用 apply_filter）
    /// 返回删除结果
    fn delete_session_inner(&mut self, session_id: &str) -> Option<DeleteOutcome> {
        let pos = self
            .all_sessions
            .iter()
            .position(|s| s.session_id == session_id)?;

        let session = &self.all_sessions[pos];
        let jsonl_path = session.jsonl_path.clone();
        let project_dir = session.project_dir.clone();
        let session_id_owned = session.session_id.clone();
        let mut has_error = false;
        let mut can_remove_from_memory = true;

        // 删除 JSONL 文件
        match std::fs::remove_file(&jsonl_path) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => {
                has_error = true;
                can_remove_from_memory = false;
            }
        }
        if !can_remove_from_memory {
            return Some(DeleteOutcome::Failed);
        }

        // 删除可能存在的同名目录
        let dir = project_dir.join(&session_id_owned);
        if dir.exists() {
            if let Err(_) = std::fs::remove_dir_all(&dir) {
                has_error = true;
            }
        }

        // 更新 sessions-index.json（如果存在）
        let index_path = project_dir.join("sessions-index.json");
        if index_path.exists() {
            self.remove_from_index(&index_path, session_id);
        }

        // 删除对应的 transcript（如果存在）
        if let Some(home) = dirs::home_dir() {
            let transcripts_dir = home.join(".claude").join("transcripts");
            if let Ok(entries) = std::fs::read_dir(&transcripts_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.contains(session_id) {
                        let _ = std::fs::remove_file(entry.path());
                    }
                }
            }
        }

        // 从内存中移除
        self.marked_sessions.remove(&pos);
        self.all_sessions.remove(pos);
        // 调整剩余标记的索引（remove 导致后续索引偏移）
        self.marked_sessions = self
            .marked_sessions
            .iter()
            .map(|&idx| if idx > pos { idx - 1 } else { idx })
            .collect();

        Some(DeleteOutcome::Deleted {
            has_warning: has_error,
        })
    }

    /// 删除单个会话
    pub fn delete_session(&mut self, session_id: &str) {
        let sid_short: String = session_id.chars().take(8).collect();
        match self.delete_session_inner(session_id) {
            Some(DeleteOutcome::Deleted { has_warning: false }) => {
                self.refresh_projects();
                self.apply_filter(true);
                self.set_status(format!("Deleted session {}", sid_short), false);
            }
            Some(DeleteOutcome::Deleted { has_warning: true }) => {
                self.refresh_projects();
                self.apply_filter(true);
                self.set_status(format!("Deleted {} (with warnings)", sid_short), true);
            }
            Some(DeleteOutcome::Failed) => {
                self.set_status(format!("Failed to delete {}", sid_short), true);
            }
            None => {}
        }
    }

    /// 批量删除会话
    pub fn delete_sessions_bulk(&mut self, session_ids: &[String]) {
        let mut deleted = 0;
        let mut warnings = 0;
        let mut failed = 0;
        for sid in session_ids {
            if let Some(result) = self.delete_session_inner(sid) {
                match result {
                    DeleteOutcome::Deleted { has_warning } => {
                        deleted += 1;
                        if has_warning {
                            warnings += 1;
                        }
                    }
                    DeleteOutcome::Failed => {
                        failed += 1;
                    }
                }
            }
        }

        if deleted > 0 {
            self.refresh_projects();
            self.apply_filter(true);
        }

        if failed > 0 {
            self.set_status(
                format!("Deleted {} sessions, {} failed", deleted, failed),
                true,
            );
        } else if warnings > 0 {
            self.set_status(
                format!("Deleted {} sessions ({} with warnings)", deleted, warnings),
                true,
            );
        } else {
            self.set_status(format!("Deleted {} sessions", deleted), false);
        }
    }

    fn remove_from_index(&self, index_path: &std::path::Path, session_id: &str) {
        if let Ok(content) = std::fs::read_to_string(index_path) {
            if let Ok(mut index) =
                serde_json::from_str::<serde_json::Value>(&content)
            {
                if let Some(entries) = index.get_mut("entries").and_then(|e| e.as_array_mut()) {
                    entries.retain(|e| {
                        e.get("sessionId")
                            .and_then(|s| s.as_str())
                            .map_or(true, |s| s != session_id)
                    });
                    if let Ok(json) = serde_json::to_string_pretty(&index) {
                        let _ = std::fs::write(index_path, json);
                    }
                }
            }
        }
    }

    pub fn set_status(&mut self, text: String, is_error: bool) {
        self.status_message = Some(StatusMessage { text, is_error });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::fs;
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
        project_dir: &std::path::Path,
        jsonl_path: &std::path::Path,
        message_count: u32,
    ) -> SessionRow {
        SessionRow {
            session_id: session_id.to_string(),
            project_name: project_name.to_string(),
            project_dir: project_dir.to_path_buf(),
            original_path: project_dir.to_path_buf(),
            first_prompt: "hello".to_string(),
            summary: "hello".to_string(),
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
    fn delete_failure_keeps_session_in_memory() {
        let root = temp_dir("delete_fail");
        let project_dir = root.join("proj");
        fs::create_dir_all(&project_dir).expect("should create project dir");

        let sid = "missing-session-001";
        // 让 jsonl_path 指向目录，remove_file 会失败（非 NotFound）
        let bad_jsonl = project_dir.join(format!("{}.jsonl", sid));
        fs::create_dir_all(&bad_jsonl).expect("should create bad jsonl dir");
        let mut app = build_app(vec![build_session(
            sid,
            "proj-a",
            &project_dir,
            &bad_jsonl,
            1,
        )]);

        app.delete_session(sid);

        assert_eq!(app.all_sessions.len(), 1);
        assert_eq!(app.all_sessions[0].session_id, sid);
        assert!(
            app.status_message
                .as_ref()
                .is_some_and(|m| m.is_error && m.text.contains("Failed to delete")),
            "delete failure should set an error status message"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn delete_success_refreshes_project_list() {
        let root = temp_dir("delete_success");
        let project_a = root.join("proj_a");
        let project_b = root.join("proj_b");
        fs::create_dir_all(&project_a).expect("should create project a dir");
        fs::create_dir_all(&project_b).expect("should create project b dir");

        let sid_a = "aaaaaaaa";
        let sid_b = "bbbbbbbb";
        let jsonl_a = project_a.join(format!("{}.jsonl", sid_a));
        let jsonl_b = project_b.join(format!("{}.jsonl", sid_b));
        fs::write(&jsonl_a, "{\"type\":\"user\",\"message\":{\"content\":\"a\"}}\n")
            .expect("should write jsonl a");
        fs::write(&jsonl_b, "{\"type\":\"user\",\"message\":{\"content\":\"b\"}}\n")
            .expect("should write jsonl b");

        let mut app = build_app(vec![
            build_session(sid_a, "proj-a", &project_a, &jsonl_a, 1),
            build_session(sid_b, "proj-b", &project_b, &jsonl_b, 2),
        ]);

        app.delete_session(sid_a);

        assert_eq!(app.all_sessions.len(), 1);
        assert_eq!(app.all_sessions[0].session_id, sid_b);
        assert_eq!(app.projects, vec!["proj-b".to_string()]);
        assert_eq!(app.filtered_indices, vec![0]);

        let _ = fs::remove_dir_all(root);
    }
}
