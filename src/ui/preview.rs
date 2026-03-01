use crate::app::{App, AppMode};
use crate::data::models::PreviewLine;
use crate::ui::theme;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use unicode_width::UnicodeWidthChar;

pub fn render(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
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
        // 预览为空时不要保留历史滚动位置，避免出现“可持续滚动但内容空白”
        app.preview_scroll = 0;
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
                session.session_id[..8.min(session.session_id.len())].to_string(),
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

    let content_width = area.width.saturating_sub(2) as usize; // 减去左右边框

    for preview_line in &app.preview_lines {
        match preview_line {
            PreviewLine::User(text) => {
                let prefix = "User: ";
                let max_w = content_width.saturating_sub(prefix.len());
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::USER_FG)),
                    Span::styled(
                        truncate_to_width(text, max_w),
                        Style::default().fg(theme::USER_FG),
                    ),
                ]));
            }
            PreviewLine::Assistant(text) => {
                let prefix = "Assistant: ";
                let max_w = content_width.saturating_sub(prefix.len());
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::ASSISTANT_FG)),
                    Span::styled(
                        truncate_to_width(text, max_w),
                        Style::default().fg(theme::ASSISTANT_FG),
                    ),
                ]));
            }
            PreviewLine::System(text) => {
                let prefix = "System: ";
                let max_w = content_width.saturating_sub(prefix.len());
                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme::SYSTEM_FG)),
                    Span::styled(
                        truncate_to_width(text, max_w),
                        Style::default().fg(theme::SYSTEM_FG),
                    ),
                ]));
            }
            PreviewLine::Truncated => {
                lines.push(Line::from(Span::styled(
                    "[...truncated]",
                    Style::default().fg(theme::DIM_FG),
                )));
            }
        }
        // 对话之间空一行
        lines.push(Line::from(""));
    }

    // 手动分页，彻底避免 Paragraph::scroll 的越界问题
    let visible_height = area.height.saturating_sub(2) as usize; // 减去上下边框
    let max_scroll = lines.len().saturating_sub(visible_height);
    let scroll = (app.preview_scroll as usize).min(max_scroll);
    app.preview_scroll = scroll as u16;

    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).take(visible_height).collect();

    let paragraph = Paragraph::new(visible_lines).block(block);

    f.render_widget(paragraph, area);
}

/// 清理文本并按显示宽度截断，确保不超过一行
fn truncate_to_width(text: &str, max_width: usize) -> String {
    let mut width = 0;
    let mut result = String::new();
    for c in text.chars() {
        if c == '\n' {
            result.push(' ');
            width += 1;
        } else {
            let cw = c.width().unwrap_or(0);
            if width + cw > max_width.saturating_sub(3) {
                result.push_str("...");
                return result;
            }
            result.push(c);
            width += cw;
        }
    }
    result
}
