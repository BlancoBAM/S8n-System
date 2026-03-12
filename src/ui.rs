use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    terminal::{Terminal, TerminalOptions, Viewport},
    widgets::{Gauge, Paragraph},
};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{self, stdout},
    time::Duration,
};
use tokio::time;

use crate::pm::{PackageManager, PmResult};

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub enum UIOperation {
    Install,
    Remove,
    Update,
}

pub async fn run_tui(
    pm: &Box<dyn PackageManager>,
    packages: Vec<String>,
    op: UIOperation,
) -> io::Result<()> {
    if packages.is_empty() && !matches!(op, UIOperation::Update) {
        return Ok(());
    }

    let items = if matches!(op, UIOperation::Update) {
        vec!["System Packages".to_string()]
    } else {
        packages.clone()
    };
    
    let total = items.len();
    if total == 0 {
        return Ok(());
    }

    // Set up inline terminal
    let mut stdout = stdout();
    execute!(stdout, Hide)?;
    
    // We don't necessarily need raw mode if we just print inline and exit on Ctrl-C
    // but raw mode helps intercept keys if we wanted to. We'll skip raw mode to let
    // standard output behave nicely, but ratatui inline mostly works fine.
    // Wait, ratatui inline actually needs raw mode to avoid weird line wrappings and echoes.
    // enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Inline(1),
        },
    )?;

    let mut spinner_idx = 0;
    
    for (i, pkg) in items.iter().enumerate() {
        let pkg_slice = vec![pkg.clone()];
        let op_future = match op {
            UIOperation::Install => pm.install(&pkg_slice),
            UIOperation::Remove => pm.remove(&pkg_slice),
            UIOperation::Update => pm.update(),
        };

        tokio::pin!(op_future);

        let mut done = false;
        let mut success = true;

        while !done {
            terminal.draw(|f| {
                let size = f.size();
                let spinner = SPINNER_FRAMES[spinner_idx % SPINNER_FRAMES.len()];
                
                let action_text = match op {
                    UIOperation::Install => "Installing",
                    UIOperation::Remove => "Removing",
                    UIOperation::Update => "Updating",
                };

                // The text
                let info_str = format!("{} {} {} ", spinner, action_text, pkg);
                let info = Span::styled(info_str, Style::default().fg(Color::Magenta));
                
                // Progress
                let ratio = if total > 0 { i as f64 / total as f64 } else { 0.0 };
                let prog_str = format!(" {}/{}", i, total);

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(info.width() as u16 + 2),
                        Constraint::Min(10), // Gauge
                        Constraint::Length(prog_str.len() as u16 + 2),
                    ])
                    .split(size);

                f.render_widget(Paragraph::new(info), chunks[0]);
                
                let gauge = Gauge::default()
                    .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
                    .ratio(ratio)
                    .label("");
                f.render_widget(gauge, chunks[1]);
                
                f.render_widget(Paragraph::new(prog_str), chunks[2]);
            })?;

            // Tick wait and check future
            let timeout = time::sleep(Duration::from_millis(100));
            tokio::select! {
                res = &mut op_future => {
                    done = true;
                    if let PmResult::Success = res {
                        success = true;
                    } else {
                        success = false;
                    }
                }
                _ = timeout => {
                    spinner_idx += 1;
                }
            }
        }

        // Clean inline output to write success checkmark
        terminal.clear()?;
        println!("{} {}", if success { "✓" } else { "✗" }, pkg);
    }

    // Final Done message
    println!("Done! Processed {} packages.", total);

    // disable_raw_mode()?;
    execute!(terminal.backend_mut(), Show)?;

    Ok(())
}
