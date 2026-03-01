use crate::app::{App, AppMode};
use crate::ui::theme;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
    ])
    .split(area);

    // 搜索框
    let is_searching = matches!(app.mode, AppMode::Search);
    let search_style = if is_searching {
        Style::default().fg(theme::SEARCH_FG)
    } else {
        Style::default().fg(theme::DIM_FG)
    };

    let search_text = if app.search_text.is_empty() && !is_searching {
        " / to search".to_string()
    } else {
        format!(" {}{}", app.search_text, if is_searching { "▎" } else { "" })
    };

    let search = Paragraph::new(Line::from(vec![
        Span::styled(" Search:", search_style),
        Span::styled(&search_text, search_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if is_searching {
                Style::default().fg(theme::SEARCH_FG)
            } else {
                theme::border_style()
            }),
    );

    f.render_widget(search, chunks[0]);

    // 项目过滤器
    let project_text = match app.selected_project {
        Some(i) => app.projects.get(i).map(|s| s.as_str()).unwrap_or("All"),
        None => "All",
    };
    let is_filtering = matches!(app.mode, AppMode::ProjectFilter);
    let filter_style = if is_filtering {
        Style::default().fg(theme::SEARCH_FG)
    } else {
        Style::default().fg(theme::DIM_FG)
    };

    let filter = Paragraph::new(Line::from(vec![
        Span::styled(" Project: ", filter_style),
        Span::styled(project_text, filter_style),
        Span::styled(if is_filtering { " ▼" } else { " ▸" }, filter_style),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(if is_filtering {
                Style::default().fg(theme::SEARCH_FG)
            } else {
                theme::border_style()
            }),
    );

    f.render_widget(filter, chunks[1]);

    // 排序指示器
    let sort = Paragraph::new(Line::from(vec![
        Span::styled(" Sort: ", Style::default().fg(theme::DIM_FG)),
        Span::styled(
            format!("{} {}", app.sort_field.label(), app.sort_order.label()),
            Style::default().fg(theme::DIM_FG),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style()),
    );

    f.render_widget(sort, chunks[2]);
}
