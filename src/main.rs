mod action;
mod app;
mod data;
mod event;
mod handler;
mod ui;

use app::App;
use crossterm::{
    cursor::Show,
    event as crossterm_event,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

/// 默认的 last-dir 文件路径
const DEFAULT_LAST_DIR_FILE: &str = "/tmp/ccm_last_dir";

/// 终端恢复守卫：无论正常退出还是报错，都会尝试恢复终端状态
struct TerminalRestoreGuard;

impl Drop for TerminalRestoreGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, Show);
    }
}

fn run_tui(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let _restore = TerminalRestoreGuard;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui::layout::render(f, app))?;

        if crossterm_event::poll(Duration::from_millis(100))? {
            let event = crossterm_event::read()?;
            if let Some(action) = event::map_event(&app.mode, event) {
                handler::handle_action(app, action);
            }
        }

        if app.should_quit || app.should_resume.is_some() {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // 解析参数
    let mut last_dir_file: Option<PathBuf> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!("ccm - Claude Code session Manager");
                println!();
                println!("A TUI tool for browsing, searching, and managing Claude Code sessions.");
                println!();
                println!("Usage: ccm [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --last-dir <FILE>  Write last session directory to FILE on exit");
                println!("  -h, --help         Show help");
                println!("  -V, --version      Show version");
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
                println!();
                println!("Shell integration (add to ~/.zshrc or ~/.bashrc):");
                println!("  ccm() {{");
                println!("    command ccm --last-dir /tmp/ccm_last_dir \"$@\"");
                println!("    if [ -f /tmp/ccm_last_dir ]; then");
                println!("      local info=(\"${{(@f)$(cat /tmp/ccm_last_dir)}}\")");
                println!("      rm -f /tmp/ccm_last_dir");
                println!("      if [ ${{#info[@]}} -eq 2 ]; then");
                println!("        builtin cd -- \"${{info[2]}}\" && claude -r \"${{info[1]}}\"");
                println!("      elif [ -d \"${{info[1]}}\" ]; then");
                println!("        builtin cd -- \"${{info[1]}}\"");
                println!("      fi");
                println!("    fi");
                println!("  }}");
                return Ok(());
            }
            "--version" | "-V" => {
                println!("ccm {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--last-dir" => {
                if i + 1 < args.len() {
                    last_dir_file = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                    continue;
                } else {
                    eprintln!("--last-dir requires a file path argument");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                eprintln!("Use --help for usage information.");
                std::process::exit(1);
            }
        }
    }

    // 创建应用
    let mut app = App::new();
    run_tui(&mut app)?;

    // 如果要恢复会话：写入恢复信息，由 shell wrapper 负责 cd + 启动 claude
    if let Some((session_id, project_path)) = app.should_resume.take() {
        let file = last_dir_file
            .as_deref()
            .unwrap_or_else(|| std::path::Path::new(DEFAULT_LAST_DIR_FILE));
        let content = format!("{}\n{}", session_id, project_path.display());
        if let Err(e) = std::fs::write(file, &content) {
            eprintln!("[ccm] failed to write {}: {}", file.display(), e);
        }
    }

    Ok(())
}
