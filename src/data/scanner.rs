use crate::data::models::{IndexEntry, SessionRow, SessionsIndex};
use chrono::DateTime;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// 从绝对路径生成短名（去掉 home 前缀）
fn shorten_path(abs_path: &Path) -> String {
    let home = dirs::home_dir().unwrap_or_default();
    let home_str = home.to_string_lossy();

    let path_str = abs_path.to_string_lossy();
    let short = path_str
        .strip_prefix(home_str.as_ref())
        .unwrap_or(&path_str)
        .trim_start_matches('/');

    if short.is_empty() {
        "~".to_string()
    } else {
        short.to_string()
    }
}

/// 扫描所有项目目录，返回所有会话和总磁盘占用
pub fn scan_all_sessions() -> (Vec<SessionRow>, u64) {
    let claude_dir = match dirs::home_dir() {
        Some(h) => h.join(".claude").join("projects"),
        None => return (Vec::new(), 0),
    };

    if !claude_dir.exists() {
        return (Vec::new(), 0);
    }

    let mut sessions = Vec::new();
    let mut total_size = 0u64;

    let entries = match fs::read_dir(&claude_dir) {
        Ok(e) => e,
        Err(_) => return (Vec::new(), 0),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // 累计目录大小
        total_size += dir_size(&path);

        let index_path = path.join("sessions-index.json");

        if index_path.exists() {
            // 有索引文件：从 originalPath 获取真实路径
            if let Some(mut indexed) = scan_from_index(&index_path, &path) {
                sessions.append(&mut indexed);
            }
        } else {
            // 没有索引文件：扫描 JSONL，从 cwd 字段获取真实路径
            if let Some(mut scanned) = scan_from_jsonl_files(&path) {
                sessions.append(&mut scanned);
            }
        }
    }

    // 按 modified 降序排序
    sessions.sort_by(|a, b| b.modified.cmp(&a.modified));
    (sessions, total_size)
}

/// 从 sessions-index.json 读取会话列表
fn scan_from_index(
    index_path: &Path,
    project_dir: &Path,
) -> Option<Vec<SessionRow>> {
    let content = fs::read_to_string(index_path).ok()?;
    let index: SessionsIndex = serde_json::from_str(&content).ok()?;

    // 从索引获取真实项目路径
    let original_path = PathBuf::from(&index.original_path);

    let rows: Vec<SessionRow> = index
        .entries
        .into_iter()
        .filter_map(|entry| {
            // 优先使用 entry 自身的 projectPath，否则用索引的 originalPath
            let entry_path = if !entry.project_path.is_empty() {
                PathBuf::from(&entry.project_path)
            } else {
                original_path.clone()
            };
            let entry_name = shorten_path(&entry_path);
            index_entry_to_row(entry, &entry_name, &entry_path, project_dir)
        })
        .collect();

    Some(rows)
}

/// 将索引条目转为显示行
fn index_entry_to_row(
    entry: IndexEntry,
    project_name: &str,
    original_path: &Path,
    project_dir: &Path,
) -> Option<SessionRow> {
    let jsonl_path = project_dir.join(format!("{}.jsonl", &entry.session_id));
    let file_size = fs::metadata(&jsonl_path).map(|m| m.len()).unwrap_or(0);

    let created = DateTime::parse_from_rfc3339(&entry.created)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let modified = DateTime::parse_from_rfc3339(&entry.modified)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc));

    Some(SessionRow {
        session_id: entry.session_id,
        project_name: project_name.to_string(),
        project_dir: project_dir.to_path_buf(),
        original_path: original_path.to_path_buf(),
        first_prompt: entry.first_prompt,
        summary: entry.summary,
        message_count: entry.message_count,
        created,
        modified,
        git_branch: entry.git_branch,
        file_size,
        jsonl_path,
        is_sidechain: entry.is_sidechain,
    })
}

/// 从 JSONL 文件直接扫描会话元数据
fn scan_from_jsonl_files(
    project_dir: &Path,
) -> Option<Vec<SessionRow>> {
    let entries = fs::read_dir(project_dir).ok()?;
    let mut rows = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !name.ends_with(".jsonl") {
            continue;
        }

        let session_id = name.trim_end_matches(".jsonl").to_string();
        // 验证是 UUID 格式
        if session_id.len() < 32 {
            continue;
        }

        if let Some(row) = scan_single_jsonl(&path, &session_id, project_dir) {
            rows.push(row);
        }
    }

    Some(rows)
}

/// 从单个 JSONL 文件提取元数据
fn scan_single_jsonl(
    path: &Path,
    session_id: &str,
    project_dir: &Path,
) -> Option<SessionRow> {
    let file = fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    let reader = BufReader::new(file);

    let mut first_prompt = String::new();
    let mut message_count: u32 = 0;
    let mut first_timestamp: Option<String> = None;
    let mut last_timestamp: Option<String> = None;
    let mut git_branch = String::new();
    let mut cwd: Option<String> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 提取 cwd（只取第一个）
        if cwd.is_none() {
            if let Some(c) = parsed.get("cwd").and_then(|c| c.as_str()) {
                if !c.is_empty() {
                    cwd = Some(c.to_string());
                }
            }
        }

        let msg_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "user" | "assistant" => {
                message_count += 1;

                if let Some(ts) = parsed.get("timestamp").and_then(|t| t.as_str()) {
                    if first_timestamp.is_none() {
                        first_timestamp = Some(ts.to_string());
                    }
                    last_timestamp = Some(ts.to_string());
                }

                if msg_type == "user" && first_prompt.is_empty() {
                    if parsed
                        .get("isMeta")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        continue;
                    }
                    first_prompt = extract_message_text(&parsed);
                }

                if git_branch.is_empty() {
                    if let Some(branch) =
                        parsed.get("gitBranch").and_then(|b| b.as_str())
                    {
                        git_branch = branch.to_string();
                    }
                }
            }
            _ => {}
        }
    }

    // 截断 firstPrompt
    if first_prompt.chars().count() > 200 {
        first_prompt = format!("{}...", first_prompt.chars().take(200).collect::<String>());
    }

    let created = first_timestamp
        .as_ref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));
    let modified = last_timestamp
        .as_ref()
        .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    // 从 cwd 获取真实项目路径
    let original_path = cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| project_dir.to_path_buf());
    let project_name = shorten_path(&original_path);

    Some(SessionRow {
        session_id: session_id.to_string(),
        project_name,
        project_dir: project_dir.to_path_buf(),
        original_path,
        first_prompt: first_prompt.clone(),
        summary: first_prompt,
        message_count,
        created,
        modified,
        git_branch,
        file_size,
        jsonl_path: path.to_path_buf(),
        is_sidechain: false,
    })
}

/// 从消息 JSON 中提取文本内容
fn extract_message_text(parsed: &serde_json::Value) -> String {
    if let Some(msg) = parsed.get("message") {
        if let Some(content) = msg.get("content") {
            if let Some(text) = content.as_str() {
                return text.to_string();
            }
            if let Some(arr) = content.as_array() {
                let mut parts = Vec::new();
                for item in arr {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        parts.push(text.to_string());
                    }
                }
                return parts.join(" ");
            }
        }
    }
    String::new()
}

fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else {
                total += fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}
