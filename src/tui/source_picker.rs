//! source_picker — interactive table for choosing which package source to use
//! when the same package name is found in multiple backends.

use super::{grid_table, theme};
use crate::pm::PackageInfo;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{self, stdout};
use std::time::Duration;

/// Display an interactive source-picker TUI.
/// Returns the index into `candidates` the user selected, or None if cancelled.
pub async fn run_source_picker(candidates: &[PackageInfo]) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(0);
    }

    terminal::enable_raw_mode().ok()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide).ok()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).ok()?;

    let result = picker_loop(&mut terminal, candidates).await;

    terminal::disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show).ok();

    result
}

async fn picker_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    candidates: &[PackageInfo],
) -> Option<usize> {
    let mut selected = 0usize;
    let pkg_name = &candidates[0].name;

    loop {
        terminal
            .draw(|f| {
                let area = f.area();
                f.render_widget(
                    Block::default().style(Style::default().bg(theme::bg_color())),
                    area,
                );

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(4),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // Header
                f.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled("  📦 ", Style::default().fg(theme::hot_pink())),
                        Span::styled(
                            format!("Multiple sources found for \"{}\"", pkg_name),
                            Style::default().fg(theme::vivid_purple()),
                        ),
                        Span::styled(
                            " — choose one:",
                            theme::dim(),
                        ),
                    ]))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(theme::border())
                            .style(Style::default().bg(theme::bg_color())),
                    ),
                    chunks[0],
                );

                // Table
                let desc_max = (chunks[1].width as usize).saturating_sub(4 + 16 + 14 + 6).max(10);
                let rows: Vec<grid_table::GridRow> = candidates
                    .iter()
                    .enumerate()
                    .map(|(i, pkg)| {
                        let ver: String = if pkg.version.is_empty() {
                            "—".to_string()
                        } else {
                            pkg.version.chars().take(12).collect()
                        };
                        let desc: String = pkg.description.chars().take(desc_max).collect();
                        grid_table::GridRow {
                            cells: vec![
                                grid_table::GridCell { text: format!("{}", i + 1), style: theme::number() },
                                grid_table::GridCell { text: pkg.source.clone(), style: theme::source_tag() },
                                grid_table::GridCell { text: ver, style: theme::version() },
                                grid_table::GridCell { text: desc, style: theme::desc() },
                            ],
                        }
                    })
                    .collect();

                let columns = [
                    grid_table::Column { header: "#",           width: Constraint::Length(4) },
                    grid_table::Column { header: "Source",      width: Constraint::Length(16) },
                    grid_table::Column { header: "Version",     width: Constraint::Length(14) },
                    grid_table::Column { header: "Description", width: Constraint::Min(10) },
                ];

                f.render_widget(
                    grid_table::GridTable {
                        columns: &columns,
                        rows: &rows,
                        selected: Some(selected),
                        header_style: theme::grid_header(),
                        separator_style: theme::grid_separator(),
                        selected_style: theme::highlight(),
                    },
                    chunks[1],
                );

                // Help bar
                f.render_widget(
                    Paragraph::new(Span::styled(
                        " ↑↓ navigate  •  Enter select  •  q / Esc cancel",
                        theme::status_text(),
                    )),
                    chunks[2],
                );
            })
            .ok();

        if event::poll(Duration::from_millis(100)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                match key.code {
                    KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected + 1 < candidates.len() { selected += 1; }
                    }
                    KeyCode::Enter => return Some(selected),
                    KeyCode::Char('q') | KeyCode::Esc => return None,
                    _ => {}
                }
            }
        }
    }
}
