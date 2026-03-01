use ratatui::style::{Color, Modifier, Style};

pub const HEADER_BG: Color = Color::Rgb(30, 40, 60);
pub const HEADER_FG: Color = Color::Rgb(180, 200, 230);
pub const SELECTED_BG: Color = Color::Rgb(40, 60, 90);
pub const SELECTED_FG: Color = Color::Rgb(220, 230, 255);
pub const NORMAL_FG: Color = Color::Rgb(180, 180, 190);
pub const DIM_FG: Color = Color::Rgb(100, 105, 115);
pub const BORDER_COLOR: Color = Color::Rgb(60, 70, 90);
pub const SEARCH_FG: Color = Color::Rgb(255, 200, 80);
pub const USER_FG: Color = Color::Rgb(100, 200, 255);
pub const ASSISTANT_FG: Color = Color::Rgb(120, 230, 150);
pub const SYSTEM_FG: Color = Color::Rgb(180, 140, 200);
pub const MARKED_FG: Color = Color::Rgb(255, 180, 80);
pub const ERROR_FG: Color = Color::Rgb(255, 100, 100);
pub const SUCCESS_FG: Color = Color::Rgb(100, 230, 140);
pub const STATUS_BG: Color = Color::Rgb(25, 30, 45);

pub fn header_style() -> Style {
    Style::default().fg(HEADER_FG).bg(HEADER_BG).add_modifier(Modifier::BOLD)
}

pub fn selected_style() -> Style {
    Style::default().fg(SELECTED_FG).bg(SELECTED_BG)
}

pub fn normal_style() -> Style {
    Style::default().fg(NORMAL_FG)
}

pub fn dim_style() -> Style {
    Style::default().fg(DIM_FG)
}

pub fn border_style() -> Style {
    Style::default().fg(BORDER_COLOR)
}
