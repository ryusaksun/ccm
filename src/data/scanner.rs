use crate::data::models::{IndexEntry, SessionRow, SessionsIndex};
use chrono::DateTime;
use std::collections::HashSet;
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

        // 收集已索引的 session ID，用于补充扫描时跳过
        let mut indexed_ids = HashSet::new();

        if index_path.exists() {
            if let Some(mut indexed) = scan_from_index(&index_path, &path) {
                for row in &indexed {
                    indexed_ids.insert(row.session_id.clone());
                }
                sessions.append(&mut indexed);
            }
        }

        // 补充扫描未被索引覆盖的 JSONL 文件（传入 indexed_ids 直接跳过，避免无谓解析）
        if let Some(mut extra) = scan_from_jsonl_files(&path, &indexed_ids) {
            sessions.append(&mut extra);
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

    // 从索引获取真实项目路径（为空时回退到项目目录本身）
    let original_path = if index.original_path.is_empty() {
        project_dir.to_path_buf()
    } else {
        PathBuf::from(&index.original_path)
    };

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

/// 从 JSONL 文件直接扫描会话元数据，跳过 skip_ids 中已有的会话
fn scan_from_jsonl_files(
    project_dir: &Path,
    skip_ids: &HashSet<String>,
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

        // 跳过已被索引覆盖的会话
        if skip_ids.contains(&session_id) {
            continue;
        }

        if let Some(row) = scan_single_jsonl(&path, &session_id, project_dir) {
            rows.push(row);
        }
    }

    Some(rows)
}

/// 超长行截断阈值：超过此长度的行只解析前 HEAD_BYTES 字节提取元数据
const LONG_LINE_THRESHOLD: usize = 64 * 1024; // 64 KB
const HEAD_BYTES: usize = 4 * 1024; // 4 KB（足够提取顶层字段）

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

        // 超长行：只截取开头部分解析顶层字段，避免内存暴涨
        let parsed: serde_json::Value = if line.len() > LONG_LINE_THRESHOLD {
            // 截取前 HEAD_BYTES 做轻量解析（不一定是合法 JSON，但能提取顶层字段）
            match parse_head_fields(&line) {
                Some(v) => v,
                None => continue,
            }
        } else {
            match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            }
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
                    // 超长行无法完整提取消息文本，跳过
                    if line.len() <= LONG_LINE_THRESHOLD {
                        first_prompt = extract_message_text(&parsed);
                    }
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

/// 从超长 JSON 行的开头手动提取顶层字段（type, cwd, timestamp, gitBranch, isMeta）
fn parse_head_fields(line: &str) -> Option<serde_json::Value> {
    let head: String = line.chars().take(HEAD_BYTES).collect();
    let mut map = serde_json::Map::new();
    for key in &["type", "cwd", "timestamp", "gitBranch", "isMeta"] {
        let pattern = format!("\"{}\":", key);
        if let Some(pos) = head.find(&pattern) {
            let after = &head[pos + pattern.len()..];
            let after = after.trim_start();
            if after.starts_with('"') {
                // 字符串值：查找未转义的闭合引号
                let bytes = after.as_bytes();
                let mut i = 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2; // 跳过转义字符
                    } else if bytes[i] == b'"' {
                        let val = &after[1..i];
                        map.insert(key.to_string(), serde_json::Value::String(val.to_string()));
                        break;
                    } else {
                        i += 1;
                    }
                }
            } else if after.starts_with("true") {
                map.insert(key.to_string(), serde_json::Value::Bool(true));
            } else if after.starts_with("false") {
                map.insert(key.to_string(), serde_json::Value::Bool(false));
            }
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(map))
    }
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

/// 统计目录下直接文件的总大小（不递归子目录，避免扫描开销过大）
fn dir_size(path: &Path) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                }
            }
        }
    }
    total
}
