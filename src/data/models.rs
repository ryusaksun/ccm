use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::PathBuf;

/// sessions-index.json 文件结构
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsIndex {
    pub entries: Vec<IndexEntry>,
    #[serde(default)]
    #[allow(dead_code)]
    pub original_path: String,
}

/// sessions-index.json 中的单个条目
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub session_id: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub full_path: String,
    #[serde(default)]
    pub first_prompt: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub message_count: u32,
    #[serde(default)]
    pub created: String,
    #[serde(default)]
    pub modified: String,
    #[serde(default)]
    pub git_branch: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub project_path: String,
    #[serde(default)]
    pub is_sidechain: bool,
}

/// 在 UI 中展示的会话行
#[derive(Debug, Clone)]
pub struct SessionRow {
    pub session_id: String,
    pub project_name: String,
    pub project_dir: PathBuf,
    pub original_path: PathBuf,
    pub first_prompt: String,
    pub summary: String,
    pub message_count: u32,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
    pub git_branch: String,
    #[allow(dead_code)]
    pub file_size: u64,
    pub jsonl_path: PathBuf,
    #[allow(dead_code)]
    pub is_sidechain: bool,
}

/// 预览面板中的对话行
#[derive(Debug, Clone)]
pub enum PreviewLine {
    User(String),
    Assistant(String),
    System(String),
    Truncated,
}

/// 排序字段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortField {
    Modified,
    Created,
    Messages,
    Project,
}

impl SortField {
    pub fn label(&self) -> &str {
        match self {
            SortField::Modified => "Modified",
            SortField::Created => "Created",
            SortField::Messages => "Messages",
            SortField::Project => "Project",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SortField::Modified => SortField::Created,
            SortField::Created => SortField::Messages,
            SortField::Messages => SortField::Project,
            SortField::Project => SortField::Modified,
        }
    }
}

/// 排序方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&self) -> Self {
        match self {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SortOrder::Ascending => "↑",
            SortOrder::Descending => "↓",
        }
    }
}
