mod action;
mod app;
mod config;
mod data;
mod error;
mod event;
mod handler;
mod ui;

use app::App;
use crossterm::{
    event as crossterm_event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 处理 --help 和 --version
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                println!("ccm - Claude Code session Manager");
                println!();
                println!("A TUI tool for browsing, searching, and managing Claude Code sessions.");
                println!();
                println!("Usage: ccm");
                println!();
                println!("Keybindings:");
                println!("  j/k, ↑/↓     Move up/down");
                println!("  g/G          Jump to top/bottom");
                println!("  /            Search");
                println!("  f            Filter by project");
                println!("  s/S          Cycle sort / Toggle order");
                println!("  Enter, r     Resume session");
                println!("  d            Delete session");
                println!("  m, Space     Mark/unmark session");
                println!("  D            Delete marked sessions");
                println!("  e            Export session");
                println!("  p            Toggle preview");
                println!("  Tab          Focus preview");
                println!("  ?            Help");
                println!("  q, Esc       Quit");
                return Ok(());
            }
            "--version" | "-V" => {
                println!("ccm {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }

    loop {
        // 初始化终端
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 创建应用
        let mut app = App::new();

        // 主事件循环
        loop {
            terminal.draw(|f| ui::layout::render(f, &mut app))?;

            if crossterm_event::poll(Duration::from_millis(250))? {
                let event = crossterm_event::read()?;
                if let Some(action) = event::map_event(&app.mode, event) {
                    handler::handle_action(&mut app, action);
                }
            }

            if app.should_quit || app.should_resume.is_some() {
                break;
            }
        }

        // 恢复终端
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        // 如果要恢复会话
        if let Some((session_id, project_path)) = app.should_resume.take() {
            println!("Resuming session {} in {}...", &session_id[..8], project_path.display());
            let status = std::process::Command::new("claude")
                .arg("-r")
                .arg(&session_id)
                .current_dir(&project_path)
                .status();

            match status {
                Ok(_) => {
                    // claude 退出后，重新进入 TUI
                    continue;
                }
                Err(e) => {
                    eprintln!("Failed to launch claude: {}", e);
                    eprintln!("Make sure 'claude' is in your PATH.");
                    std::process::exit(1);
                }
            }
        }

        // 正常退出
        break;
    }

    Ok(())
}
