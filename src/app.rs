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

    /// 删除会话
    pub fn delete_session(&mut self, session_id: &str) {
        if let Some(pos) = self
            .all_sessions
            .iter()
            .position(|s| s.session_id == session_id)
        {
            let session = &self.all_sessions[pos];

            let mut errors = Vec::new();

            // 删除 JSONL 文件
            if let Err(e) = std::fs::remove_file(&session.jsonl_path) {
                errors.push(format!("JSONL: {}", e));
            }

            // 删除可能存在的同名目录
            let dir = session
                .project_dir
                .join(&session.session_id);
            if dir.exists() {
                if let Err(e) = std::fs::remove_dir_all(&dir) {
                    errors.push(format!("dir: {}", e));
                }
            }

            // 更新 sessions-index.json（如果存在）
            let index_path = session.project_dir.join("sessions-index.json");
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

            // 从内存中移除，保留其他标记
            self.marked_sessions.remove(&pos);
            self.all_sessions.remove(pos);
            // 调整剩余标记的索引（remove 导致后续索引偏移）
            self.marked_sessions = self
                .marked_sessions
                .iter()
                .map(|&idx| if idx > pos { idx - 1 } else { idx })
                .collect();
            self.apply_filter(true);

            if errors.is_empty() {
                self.status_message = Some(StatusMessage {
                    text: format!("Deleted session {}", &session_id[..8]),
                    is_error: false,
                });
            } else {
                self.status_message = Some(StatusMessage {
                    text: format!(
                        "Deleted {} (warnings: {})",
                        &session_id[..8],
                        errors.join(", ")
                    ),
                    is_error: true,
                });
            }
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
