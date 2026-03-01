use crate::app::App;
use crate::ui::theme;
use ratatui::layout::Constraint;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Row, Table};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

pub fn render(f: &mut Frame, app: &mut App, area: ratatui::layout::Rect) {
    let header_cells = ["Date", "Project", "Summary", "Msgs", "Branch"]
        .iter()
        .map(|h| Cell::from(*h).style(theme::header_style()));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(_, &real_idx)| {
            let session = &app.all_sessions[real_idx];

            let date_str = session
                .modified
                .map(|dt| dt.format("%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "N/A".to_string());

            // 截断项目名以适应列宽
            let project = truncate_str(&session.project_name, 18);

            // 显示 summary，如果为空则显示 firstPrompt 的开头
            let summary_text = if !session.summary.is_empty() && session.summary != session.first_prompt {
                &session.summary
            } else {
                &session.first_prompt
            };
            // 清理换行符
            let summary_clean: String = summary_text
                .chars()
                .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
                .collect();

            let msgs = format!("{}", session.message_count);

            let branch = truncate_str(&session.git_branch, 12);

            let is_marked = app.marked_sessions.contains(&real_idx);

            let style = if is_marked {
                Style::default()
                    .fg(theme::MARKED_FG)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme::normal_style()
            };

            let prefix = if is_marked { "* " } else { "  " };

            Row::new(vec![
                Cell::from(format!("{}{}", prefix, date_str)),
                Cell::from(project),
                Cell::from(summary_clean),
                Cell::from(msgs),
                Cell::from(branch),
            ])
            .style(style)
            .height(1)
        })
        .collect();

    let widths = [
        Constraint::Length(14),
        Constraint::Length(20),
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(14),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border_style())
        .title(Line::from(vec![
            Span::styled(" Sessions ", theme::header_style()),
            Span::styled(
                format!("({}) ", app.filtered_indices.len()),
                Style::default().fg(theme::DIM_FG),
            ),
        ]));

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme::selected_style())
        .highlight_symbol("▶ ");

    f.render_stateful_widget(table, area, &mut app.table_state);
}

fn truncate_str(s: &str, max_width: usize) -> String {
    let width = UnicodeWidthStr::width(s);
    if width <= max_width {
        s.to_string()
    } else {
        let mut result = String::new();
        let mut current_width = 0;
        for c in s.chars() {
            let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
            if current_width + char_width + 2 > max_width {
                result.push_str("..");
                break;
            }
            result.push(c);
            current_width += char_width;
        }
        result
    }
}
