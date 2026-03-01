use crate::app::{App, AppMode};
use crate::data::models::PreviewLine;
use crate::ui::theme;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = matches!(app.mode, AppMode::Preview);

    let border_style = if is_focused {
        Style::default().fg(theme::SEARCH_FG)
    } else {
        theme::border_style()
    };

    let title = if is_focused {
        " Preview (Tab: unfocus, j/k: scroll) "
    } else {
        " Preview (Tab: focus) "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, Style::default().fg(theme::HEADER_FG)));

    if app.preview_lines.is_empty() {
        let empty = Paragraph::new("No preview available")
            .style(theme::dim_style())
            .block(block);
        f.render_widget(empty, area);
        return;
    }

    let mut lines: Vec<Line> = Vec::new();

    // 显示完整项目路径和会话信息
    if let Some(session) = app.selected_session() {
        lines.push(Line::from(vec![
            Span::styled(" Project: ", Style::default().fg(theme::DIM_FG)),
            Span::styled(
                session.project_name.clone(),
                Style::default().fg(theme::HEADER_FG),
            ),
        ]));
        let date_str = session
            .modified
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_default();
        let mut meta_spans = vec![
            Span::styled(" Session: ", Style::default().fg(theme::DIM_FG)),
            Span::styled(
                &session.session_id[..8.min(session.session_id.len())],
                Style::default().fg(theme::DIM_FG),
            ),
        ];
        if !date_str.is_empty() {
            meta_spans.push(Span::styled("  |  ", Style::default().fg(theme::BORDER_COLOR)));
            meta_spans.push(Span::styled(date_str, Style::default().fg(theme::DIM_FG)));
        }
        if !session.git_branch.is_empty() {
            meta_spans.push(Span::styled("  |  ", Style::default().fg(theme::BORDER_COLOR)));
            meta_spans.push(Span::styled(
                session.git_branch.clone(),
                Style::default().fg(theme::DIM_FG),
            ));
        }
        lines.push(Line::from(meta_spans));
        lines.push(Line::from(Span::styled(
            "─".repeat(60),
            Style::default().fg(theme::BORDER_COLOR),
        )));
    }

    for preview_line in &app.preview_lines {
        match preview_line {
            PreviewLine::User(text) => {
                lines.push(Line::from(vec![
                    Span::styled("User: ", Style::default().fg(theme::USER_FG)),
                    Span::styled(
                        clean_text(text),
                        Style::default().fg(theme::USER_FG),
                    ),
                ]));
            }
            PreviewLine::Assistant(text) => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "Assistant: ",
                        Style::default().fg(theme::ASSISTANT_FG),
                    ),
                    Span::styled(
                        clean_text(text),
                        Style::default().fg(theme::ASSISTANT_FG),
                    ),
                ]));
            }
            PreviewLine::System(text) => {
                lines.push(Line::from(vec![
                    Span::styled(
                        "System: ",
                        Style::default().fg(theme::SYSTEM_FG),
                    ),
                    Span::styled(
                        clean_text(text),
                        Style::default().fg(theme::SYSTEM_FG),
                    ),
                ]));
            }
        }
        // 对话之间空一行
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.preview_scroll, 0));

    f.render_widget(paragraph, area);
}

/// 清理文本中的换行符
fn clean_text(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    // 截断到合理长度
    if cleaned.len() > 300 {
        let truncated: String = cleaned.chars().take(300).collect();
        format!("{}...", truncated)
    } else {
        cleaned
    }
}
