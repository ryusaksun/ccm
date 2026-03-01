use crate::app::App;
use crate::ui::theme;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // 左侧：统计信息
    let total_sessions = app.all_sessions.len();
    let disk_str = format_size(app.total_disk_usage);
    let marked = app.marked_sessions.len();

    let mut left_spans = vec![
        Span::styled(
            format!(" {} sessions", total_sessions),
            Style::default().fg(theme::DIM_FG),
        ),
        Span::styled(" | ", Style::default().fg(theme::BORDER_COLOR)),
        Span::styled(
            format!("Disk: {}", disk_str),
            Style::default().fg(theme::DIM_FG),
        ),
    ];

    if marked > 0 {
        left_spans.push(Span::styled(" | ", Style::default().fg(theme::BORDER_COLOR)));
        left_spans.push(Span::styled(
            format!("Marked: {}", marked),
            Style::default().fg(theme::MARKED_FG),
        ));
    }

    // 状态消息
    if let Some(ref msg) = app.status_message {
        left_spans.push(Span::styled(" | ", Style::default().fg(theme::BORDER_COLOR)));
        left_spans.push(Span::styled(
            &msg.text,
            Style::default().fg(if msg.is_error {
                theme::ERROR_FG
            } else {
                theme::SUCCESS_FG
            }),
        ));
    }

    let left = Paragraph::new(Line::from(left_spans))
        .style(Style::default().bg(theme::STATUS_BG));

    f.render_widget(left, chunks[0]);

    // 右侧：快捷键提示
    let right = Paragraph::new(Line::from(vec![
        Span::styled(
            "q:Quit /:Search r:Resume d:Delete e:Export ?:Help ",
            Style::default().fg(theme::DIM_FG),
        ),
    ]))
    .style(Style::default().bg(theme::STATUS_BG))
    .alignment(ratatui::layout::Alignment::Right);

    f.render_widget(right, chunks[1]);
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
