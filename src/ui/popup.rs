use crate::app::{App, AppMode, ConfirmAction};
use crate::ui::theme;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

/// 渲染确认对话框
pub fn render_confirm(f: &mut Frame, app: &App) {
    if let AppMode::Confirm(ref action) = app.mode {
        let message = match action {
            ConfirmAction::DeleteSession(sid) => {
                format!("Delete session {}...?\n\n(y)es / (n)o", &sid[..8.min(sid.len())])
            }
            ConfirmAction::BulkDelete(ids) => {
                format!(
                    "Delete {} sessions?\n\n(y)es / (n)o",
                    ids.len()
                )
            }
            ConfirmAction::ResumeSession(sid, ref path) => {
                format!(
                    "Resume session {}...?\nProject: {}\n\n(y)es / (n)o",
                    &sid[..8.min(sid.len())],
                    path.display()
                )
            }
        };

        let area = centered_rect(50, 30, f.area());
        f.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::SEARCH_FG))
            .title(Span::styled(
                " Confirm ",
                Style::default().fg(theme::SEARCH_FG),
            ));

        let paragraph = Paragraph::new(message)
            .block(block)
            .style(Style::default().fg(theme::NORMAL_FG))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}

/// 渲染帮助覆盖层
pub fn render_help(f: &mut Frame) {
    let area = centered_rect(60, 70, f.area());
    f.render_widget(Clear, area);

    let items = vec![
        ("j/k, ↑/↓", "Move up/down"),
        ("g/G", "Jump to top/bottom"),
        ("Ctrl-b/f", "Page up/down"),
        ("/", "Search"),
        ("f", "Filter by project"),
        ("s/S", "Cycle sort field / Toggle order"),
        ("Enter, r", "Resume session"),
        ("d", "Delete session"),
        ("m, Space", "Mark/unmark session"),
        ("D", "Delete marked sessions"),
        ("e", "Export session"),
        ("p", "Toggle preview panel"),
        ("Tab", "Focus/unfocus preview"),
        ("?", "Toggle help"),
        ("q, Esc", "Quit"),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(key, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:>14}", key),
                    Style::default().fg(theme::SEARCH_FG),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(*desc, Style::default().fg(theme::NORMAL_FG)),
            ]))
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme::SEARCH_FG))
            .title(Span::styled(
                " Help (press ? or Esc to close) ",
                Style::default().fg(theme::SEARCH_FG),
            )),
    );

    f.render_widget(list, area);
}

/// 渲染项目过滤器下拉列表
pub fn render_project_filter(f: &mut Frame, app: &mut App) {
    let mut items: Vec<ListItem> = vec![ListItem::new(Line::from(Span::styled(
        "All",
        Style::default().fg(theme::NORMAL_FG),
    )))];

    for project in &app.projects {
        items.push(ListItem::new(Line::from(Span::styled(
            project.as_str(),
            Style::default().fg(theme::NORMAL_FG),
        ))));
    }

    let area = Rect {
        x: f.area().width / 3,
        y: 3,
        width: (f.area().width / 3).min(40),
        height: (items.len() as u16 + 2).min(f.area().height / 2),
    };

    f.render_widget(Clear, area);

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::SEARCH_FG))
                .title(" Select Project "),
        )
        .highlight_style(theme::selected_style())
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut app.project_list_state);
}

/// 渲染导出选择对话框
pub fn render_export_choice(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);

    let paragraph = Paragraph::new("Export format:\n\n(m) Markdown\n(j) JSON\n\n(Esc) Cancel")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme::SEARCH_FG))
                .title(" Export "),
        )
        .style(Style::default().fg(theme::NORMAL_FG));

    f.render_widget(paragraph, area);
}

/// 居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)])
        .flex(Flex::Center)
        .split(area);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)])
        .flex(Flex::Center)
        .split(vertical[0]);
    horizontal[0]
}
