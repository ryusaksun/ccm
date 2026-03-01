use crate::data::models::PreviewLine;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// 从 JSONL 文件加载会话预览（懒加载，最多 max_lines 条对话）
pub fn load_preview(path: &Path, max_lines: usize) -> Vec<PreviewLine> {
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return Vec::new(),
    };

    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line in reader.lines() {
        if lines.len() >= max_lines {
            break;
        }

        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = match parsed.get("type").and_then(|t| t.as_str()) {
            Some(t) => t.to_string(),
            None => continue,
        };

        // 跳过 isMeta 消息
        if parsed
            .get("isMeta")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            continue;
        }

        match msg_type.as_str() {
            "user" => {
                let text = extract_content(&parsed);
                if !text.is_empty() {
                    lines.push(PreviewLine::User(truncate_text(&text, 500)));
                }
            }
            "assistant" => {
                let text = extract_content(&parsed);
                if !text.is_empty() {
                    lines.push(PreviewLine::Assistant(truncate_text(&text, 500)));
                }
            }
            "system" => {
                let text = extract_content(&parsed);
                if !text.is_empty() {
                    lines.push(PreviewLine::System(truncate_text(&text, 200)));
                }
            }
            _ => {}
        }
    }

    lines
}

/// 提取消息的文本内容
fn extract_content(parsed: &serde_json::Value) -> String {
    if let Some(msg) = parsed.get("message") {
        if let Some(content) = msg.get("content") {
            if let Some(text) = content.as_str() {
                return text.to_string();
            }
            if let Some(arr) = content.as_array() {
                let mut parts = Vec::new();
                for item in arr {
                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            parts.push(text.to_string());
                        }
                    }
                }
                return parts.join("\n");
            }
        }
    }
    String::new()
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() > max_chars {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{}...", truncated)
    } else {
        text.to_string()
    }
}
