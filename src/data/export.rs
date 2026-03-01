use crate::data::models::SessionRow;
use crate::data::session::load_preview;
use crate::data::models::PreviewLine;
use std::fs;
use std::path::Path;

/// 导出会话为 Markdown
pub fn export_markdown(session: &SessionRow, output_path: &Path) -> std::io::Result<()> {
    let preview = load_preview(&session.jsonl_path, 500);

    let mut content = String::new();
    content.push_str(&format!("# Session: {}\n\n", session.summary));
    content.push_str(&format!("- **Session ID**: `{}`\n", session.session_id));
    content.push_str(&format!("- **Project**: {}\n", session.project_name));
    if let Some(created) = session.created {
        content.push_str(&format!(
            "- **Created**: {}\n",
            created.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    if let Some(modified) = session.modified {
        content.push_str(&format!(
            "- **Modified**: {}\n",
            modified.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    content.push_str(&format!("- **Messages**: {}\n", session.message_count));
    if !session.git_branch.is_empty() {
        content.push_str(&format!("- **Branch**: {}\n", session.git_branch));
    }
    content.push_str("\n---\n\n");

    for line in &preview {
        match line {
            PreviewLine::User(text) => {
                content.push_str(&format!("## User\n\n{}\n\n", text));
            }
            PreviewLine::Assistant(text) => {
                content.push_str(&format!("## Assistant\n\n{}\n\n", text));
            }
            PreviewLine::System(text) => {
                content.push_str(&format!("## System\n\n{}\n\n", text));
            }
        }
    }

    fs::write(output_path, content)
}

/// 导出会话为 JSON（将 JSONL 转为 JSON 数组）
pub fn export_json(session: &SessionRow, output_path: &Path) -> std::io::Result<()> {
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(&session.jsonl_path)?;
    let reader = BufReader::new(file);
    let mut items: Vec<serde_json::Value> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            items.push(value);
        }
    }

    let json = serde_json::to_string_pretty(&items)?;
    fs::write(output_path, json)
}
