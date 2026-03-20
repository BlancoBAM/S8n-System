//! S8n TUI — Full-screen Charm-style terminal interface
//!
//! Rendering strategy:
//! - GridTable (grid_table.rs): custom Buffer-writing widget with real Unicode box-drawing grid lines
//! - bubbletea_rs::gradient::gradient_filled_segment for animated gradient progress bars
//! - braille spinner frames + LGStyle coloring matching bubbletea-rs package-manager example
//! - ratatui for layout, input widgets, overlays

pub mod color_picker;
pub mod file_manager;
pub mod grid_table;
pub mod menu;
pub mod paginator;
pub mod tabs;
pub mod theme;

use crate::pm::{PackageInfo, PackageManager, PmResult};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, TableState, Wrap},
    Terminal,
};
use std::io::{self, stdout};
use std::time::Duration;

// bubbletea-rs ecosystem for gradient progress bar and lipgloss-style spinner colors
use bubbletea_rs::gradient::gradient_filled_segment;
use lipgloss_extras::lipgloss::{Color as LGColor, Style as LGStyle};

// ── App state ───────────────────────────────────────────────────────────────

/// Braille spinner — from the bubbletea-rs package-manager example
const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

#[derive(Clone, PartialEq)]
pub enum AppMode {
    Menu,
    FileManager,
    ColorTheme,
    PackManSearch,
    PackManProgress,
}

#[derive(Clone, PartialEq)]
enum Mode {
    Input,    // typing search query
    Browse,   // navigating results with ↑↓
    Confirm,  // install/remove confirmation
    Progress, // operation running
    Done,     // finished, press q to exit
}

struct App {
    mode: Mode,
    search_input: String,
    cursor_pos: usize,
    tick: u64, // animation tick

    // Results grouped by source
    results_by_source: Vec<(String, Vec<PackageInfo>)>,
    all_results: Vec<PackageInfo>,

    // Tab navigation
    tab_titles: Vec<String>,
    active_tab: usize,

    // List navigation
    list_state: TableState,
    page: usize,
    page_size: usize,

    // Source selector for highlighted package
    source_options: Vec<String>,
    source_selected: usize,

    // Confirm / progress
    confirm_action: String, // "install" or "remove"
    confirm_packages: Vec<String>,
    pub confirm_selected_yes: bool, // Tracks which button is highlighted
    progress_items: Vec<ProgressItem>,
    status_message: String,

    should_quit: bool,
}

#[derive(Clone)]
struct ProgressItem {
    name: String,
    done: bool,
    success: bool,
}

impl App {
    fn new() -> Self {
        Self {
            mode: Mode::Input,
            search_input: String::new(),
            cursor_pos: 0,
            tick: 0,
            results_by_source: Vec::new(),
            all_results: Vec::new(),
            tab_titles: vec!["All".into()],
            active_tab: 0,
            list_state: TableState::default(),
            page: 0,
            page_size: 20,
            source_options: Vec::new(),
            source_selected: 0,
            confirm_action: String::new(),
            confirm_packages: Vec::new(),
            confirm_selected_yes: true,
            progress_items: Vec::new(),
            status_message: String::new(),
            should_quit: false,
        }
    }

    fn current_results(&self) -> &[PackageInfo] {
        if self.active_tab == 0 {
            &self.all_results
        } else if let Some((_src, results)) = self.results_by_source.get(self.active_tab - 1) {
            results
        } else {
            &[]
        }
    }

    fn page_items(&self) -> Vec<(usize, &PackageInfo)> {
        let results = self.current_results();
        let start = self.page * self.page_size;
        results
            .iter()
            .enumerate()
            .skip(start)
            .take(self.page_size)
            .collect()
    }

    fn total_pages(&self) -> usize {
        let total = self.current_results().len();
        if total == 0 {
            1
        } else {
            total.div_ceil(self.page_size)
        }
    }

    fn selected_absolute_index(&self) -> Option<usize> {
        self.list_state
            .selected()
            .map(|rel| self.page * self.page_size + rel)
    }

    fn selected_package(&self) -> Option<&PackageInfo> {
        self.selected_absolute_index()
            .and_then(|idx| self.current_results().get(idx))
    }

    fn update_source_options(&mut self) {
        if let Some(pkg) = self.selected_package() {
            let name = &pkg.name;
            let sources: Vec<String> = self
                .all_results
                .iter()
                .filter(|p| p.name == *name)
                .map(|p| p.source.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            self.source_options = sources;
            self.source_selected = 0;
        } else {
            self.source_options.clear();
        }
    }

    fn set_results(&mut self, results_by_source: Vec<(String, Vec<PackageInfo>)>) {
        self.tab_titles = vec!["All".into()];
        self.all_results.clear();
        self.results_by_source.clear();

        for (source, pkgs) in results_by_source {
            self.tab_titles.push(format!("{} ({})", source, pkgs.len()));
            self.all_results.extend(pkgs.clone());
            self.results_by_source.push((source, pkgs));
        }

        self.active_tab = 0;
        self.page = 0;
        self.list_state.select(if self.all_results.is_empty() {
            None
        } else {
            Some(0)
        });
        self.update_source_options();
    }
}

// ── Rendering ───────────────────────────────────────────────────────────────

fn render(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        // Dark background
        f.render_widget(
            Block::default().style(Style::default().bg(theme::bg_color())),
            size,
        );

        match app.mode {
            Mode::Input | Mode::Browse => render_search_view(f, app, size),
            Mode::Confirm => {
                render_search_view(f, app, size);
                render_confirm_overlay(f, app, size);
            }
            Mode::Progress => render_progress_view(f, app, size),
            Mode::Done => render_done_view(f, app, size),
        }
    })?;
    Ok(())
}

fn render_search_view(f: &mut ratatui::Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // tabs
            Constraint::Length(3), // search input
            Constraint::Min(5),    // results table
            Constraint::Length(1), // paginator
            Constraint::Length(1), // status bar
        ])
        .split(area);

    // ── Dynamic Tabs ──
    // If a multi-source package is highlighted, show source selection tabs
    // Otherwise show normal source filter tabs (or page tabs if navigating pages)
    let tab_mode = if app.mode == Mode::Browse && app.source_options.len() > 1 {
        let pkg_name = app
            .selected_package()
            .map(|p| p.name.as_str())
            .unwrap_or("");
        tabs::TabMode::PackageSources {
            sources: &app.source_options,
            selected: app.source_selected,
            pkg_name,
        }
    } else if app.total_pages() > 1 && app.mode == Mode::Browse {
        tabs::TabMode::Pages {
            current: app.page,
            total: app.total_pages(),
        }
    } else {
        tabs::TabMode::Sources {
            titles: &app.tab_titles,
            active: app.active_tab,
        }
    };
    f.render_widget(tabs::TabBar { mode: tab_mode }, chunks[0]);

    // ── Search input ──
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.mode == Mode::Input {
            Style::default().fg(theme::hot_pink())
        } else {
            theme::border()
        })
        .title(Span::styled(" 🔍 Search ", theme::search_label()))
        .style(Style::default().bg(theme::bg_color()));

    let input_text = if app.search_input.is_empty() && app.mode != Mode::Input {
        Paragraph::new(Span::styled("Type to search packages...", theme::dim()))
    } else {
        let mut display = app.search_input.clone();
        if app.mode == Mode::Input {
            // Animated cursor: alternate between │ and ▏
            let cursor_char = if app.tick % 6 < 3 { '│' } else { '▏' };
            display.insert(app.cursor_pos, cursor_char);
        }
        Paragraph::new(Span::styled(display, theme::input()))
    };
    f.render_widget(input_text.block(input_block), chunks[1]);

    // ── Searching spinner ──
    if !app.status_message.is_empty() {
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner_frames[(app.tick as usize) % spinner_frames.len()];
        let spinner_area = Rect::new(chunks[1].x + chunks[1].width - 20, chunks[1].y + 1, 18, 1);
        f.render_widget(
            Paragraph::new(Span::styled(
                format!("{} {}", frame, app.status_message),
                Style::default().fg(theme::hot_pink()),
            )),
            spinner_area,
        );
    }

    // ── Package table via GridTable (writes Unicode grid lines directly to Buffer) ──
    app.page_size = (chunks[2].height as usize).saturating_sub(4).max(1);

    let items = app.page_items();
    let selected_rel = app.list_state.selected();

    let desc_col_max = (chunks[2].width as usize)
        .saturating_sub(5 + 26 + 14 + 12 + 6)
        .max(10);

    let grid_rows: Vec<grid_table::GridRow> = items
        .iter()
        .map(|(global_idx, pkg)| {
            let installed_marker = if pkg.installed { " ✓" } else { "" };
            let name_raw = format!("{}{}", pkg.name, installed_marker);
            let ver = if pkg.version.is_empty() {
                "—".to_string()
            } else {
                pkg.version.chars().take(12).collect()
            };
            let src: String = pkg.source.chars().take(10).collect();
            let desc: String = pkg.description.chars().take(desc_col_max).collect();

            grid_table::GridRow {
                cells: vec![
                    grid_table::GridCell {
                        text: format!("{}", global_idx + 1),
                        style: theme::number(),
                    },
                    grid_table::GridCell {
                        text: name_raw,
                        style: if pkg.installed {
                            theme::success()
                        } else {
                            theme::pkg_name()
                        },
                    },
                    grid_table::GridCell {
                        text: ver,
                        style: theme::version(),
                    },
                    grid_table::GridCell {
                        text: src,
                        style: theme::source_tag(),
                    },
                    grid_table::GridCell {
                        text: desc,
                        style: theme::desc(),
                    },
                ],
            }
        })
        .collect();

    let columns = [
        grid_table::Column {
            header: "#",
            width: Constraint::Length(5),
        },
        grid_table::Column {
            header: "Name",
            width: Constraint::Percentage(25),
        },
        grid_table::Column {
            header: "Version",
            width: Constraint::Length(14),
        },
        grid_table::Column {
            header: "Source",
            width: Constraint::Length(12),
        },
        grid_table::Column {
            header: "Description",
            width: Constraint::Min(10),
        },
    ];

    let result_count = app.current_results().len();
    // Render block title separately before the grid table
    f.render_widget(
        Block::default()
            .borders(Borders::NONE)
            .title(Span::styled(
                format!(" ☰ {} packages ", result_count),
                theme::title(),
            ))
            .style(Style::default().bg(theme::bg_color())),
        chunks[2],
    );

    f.render_widget(
        grid_table::GridTable {
            columns: &columns,
            rows: &grid_rows,
            selected: selected_rel,
            header_style: theme::grid_header(),
            separator_style: theme::grid_separator(),
            selected_style: theme::highlight(),
        },
        chunks[2],
    );

    // ── Paginator ──
    f.render_widget(
        paginator::Paginator {
            current_page: app.page,
            total_pages: app.total_pages(),
            tick: app.tick,
        },
        chunks[3],
    );

    // ── Status / help bar ──
    let help = match app.mode {
        Mode::Input => " ↩ search • tab results • esc quit",
        Mode::Browse => {
            " ↑↓ navigate • ←→ source/page • tab filter • i install • d remove • / search • q quit"
        }
        _ => " q quit",
    };
    let status_line = Line::from(vec![
        Span::styled(" S8N ", theme::status_bar()),
        Span::styled(" ", Style::default()),
        Span::styled(help, theme::status_text()),
    ]);
    f.render_widget(Paragraph::new(status_line), chunks[4]);
}

fn render_confirm_overlay(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let w = 55.min(area.width - 4);
    let h = 8.min(area.height - 2);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;
    let dialog = Rect::new(x, y, w, h);

    f.render_widget(Clear, dialog);

    let action = if app.confirm_action == "install" {
        "Install"
    } else {
        "Remove"
    };
    let pkgs = app.confirm_packages.join(", ");
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::vivid_purple()))
        .title(Span::styled(
            format!(" {} Confirmation ", action),
            Style::default()
                .fg(theme::hot_pink())
                .add_modifier(ratatui::style::Modifier::BOLD),
        ))
        .style(Style::default().bg(theme::overlay_color()));

    let yes_style = if app.confirm_selected_yes {
        theme::btn_yes()
    } else {
        theme::btn_dim()
    };

    let no_style = if !app.confirm_selected_yes {
        theme::btn_no()
    } else {
        theme::btn_dim()
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Are you sure you want to {} {}?",
                action.to_lowercase(),
                pkgs
            ),
            theme::pkg_name(),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("         ", Style::default()),
            Span::styled("  Yes  ", yes_style),
            Span::styled("   ", Style::default()),
            Span::styled(" Cancel ", no_style),
        ]),
    ];

    f.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        dialog,
    );
}

fn render_progress_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title
            Constraint::Min(3),    // items
            Constraint::Length(4), // gradient progress bar
        ])
        .split(area);

    // Title with braille spinner (color #63 matches bubbletea-rs package-manager example)
    let action_text = if app.confirm_action == "install" {
        "Installing"
    } else {
        "Removing"
    };
    let frame = SPINNER_FRAMES[(app.tick as usize) % SPINNER_FRAMES.len()];
    let frame_style = LGStyle::new().foreground(LGColor::from("63"));
    let spinner_str = frame_style.render(frame);
    let title = Paragraph::new(Text::raw(format!(
        "  {} {} packages...",
        spinner_str, action_text
    )))
    .block(Block::default().style(Style::default().bg(theme::bg_color())));
    f.render_widget(title, chunks[0]);

    // Items list with per-item braille spinners (offset from main spinner)
    let items: Vec<ListItem> = app
        .progress_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let (icon, icon_style) = if item.done {
                if item.success {
                    ("  ✓ ", Style::default().fg(Color::Rgb(0, 255, 0))) // 42 green
                } else {
                    ("  ✗ ", Style::default().fg(Color::Rgb(255, 0, 0))) // 196 red
                }
            } else {
                let spin = SPINNER_FRAMES[(app.tick as usize + i * 3) % SPINNER_FRAMES.len()];
                (spin, Style::default().fg(theme::vivid_purple())) // 63 purple
            };

            let name_style = if item.done {
                Style::default().fg(Color::Rgb(100, 100, 100)) // 240 gray
            } else {
                Style::default()
                    .fg(Color::Rgb(255, 100, 200))
                    .add_modifier(ratatui::style::Modifier::BOLD) // 212 pink
            };

            let line = Line::from(vec![
                Span::styled(
                    if !item.done {
                        format!("  {} ", icon)
                    } else {
                        icon.to_string()
                    },
                    icon_style,
                ),
                Span::styled(item.name.clone(), name_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border())
            .style(Style::default().bg(theme::bg_color())),
    );
    f.render_widget(list, chunks[1]);

    // Animated gradient progress bar from bubbletea-rs
    let done_count = app.progress_items.iter().filter(|i| i.done).count();
    let total = app.progress_items.len().max(1);
    let ratio = done_count as f64 / total as f64;
    let bar_width = (chunks[2].width as usize).saturating_sub(4).min(80);
    let filled = (bar_width as f64 * ratio).round() as usize;
    let empty = bar_width.saturating_sub(filled);
    let gradient_bar = format!(
        "{}{}",
        gradient_filled_segment(filled, '█'),
        "░".repeat(empty)
    );
    let progress_text = format!("  {}  {}/{}", gradient_bar, done_count, total);
    f.render_widget(
        Paragraph::new(Text::raw(progress_text)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border())
                .title(Span::styled(" Progress ", theme::search_label()))
                .style(Style::default().bg(theme::bg_color())),
        ),
        chunks[2],
    );
}

fn render_done_view(f: &mut ratatui::Frame, app: &App, area: Rect) {
    render_progress_view(f, app, area);
    // Overlay done message in a lipgloss-style card
    let msg_w = 50.min(area.width - 4);
    let x = (area.width - msg_w) / 2;
    let y = area.height / 2;
    let msg_area = Rect::new(x, y, msg_w, 5);
    f.render_widget(Clear, msg_area);

    let success_count = app.progress_items.iter().filter(|i| i.success).count();
    let fail_count = app
        .progress_items
        .iter()
        .filter(|i| i.done && !i.success)
        .count();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::neon_green()))
        .title(Span::styled(" Complete ", theme::success()))
        .style(Style::default().bg(theme::overlay_color()));
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  ✓ {} succeeded", success_count), theme::success()),
            Span::styled("  ", Style::default()),
            if fail_count > 0 {
                Span::styled(format!("✗ {} failed", fail_count), theme::error())
            } else {
                Span::styled("", Style::default())
            },
        ]),
        Line::from(Span::styled("  Press q to exit", theme::dim())),
    ];
    f.render_widget(Paragraph::new(text).block(block), msg_area);
}

// ── Event handling ──────────────────────────────────────────────────────────

fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
    match app.mode {
        Mode::Input => handle_input_key(app, key, modifiers),
        Mode::Browse => handle_browse_key(app, key, modifiers),
        Mode::Confirm => handle_confirm_key(app, key),
        Mode::Done => {
            if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
                app.should_quit = true;
            }
            None
        }
        Mode::Progress => None,
    }
}

enum Action {
    Search(String),
    Install(Vec<String>, String), // packages, source
    Remove(Vec<String>),
    FuzzySearch,
}

fn handle_input_key(app: &mut App, key: KeyCode, _modifiers: KeyModifiers) -> Option<Action> {
    match key {
        KeyCode::Char(c) => {
            app.search_input.insert(app.cursor_pos, c);
            app.cursor_pos += 1;
        }
        KeyCode::Backspace => {
            if app.cursor_pos > 0 {
                app.cursor_pos -= 1;
                app.search_input.remove(app.cursor_pos);
            }
        }
        KeyCode::Delete => {
            if app.cursor_pos < app.search_input.len() {
                app.search_input.remove(app.cursor_pos);
            }
        }
        KeyCode::Left => {
            app.cursor_pos = app.cursor_pos.saturating_sub(1);
        }
        KeyCode::Right => {
            app.cursor_pos = (app.cursor_pos + 1).min(app.search_input.len());
        }
        KeyCode::Home => app.cursor_pos = 0,
        KeyCode::End => app.cursor_pos = app.search_input.len(),
        KeyCode::Enter => {
            if !app.search_input.is_empty() {
                let query = app.search_input.clone();
                app.status_message = format!("Searching for '{}'...", query);
                return Some(Action::Search(query));
            }
        }
        KeyCode::Tab => {
            // Switch to browse mode if we have results
            if !app.all_results.is_empty() {
                app.mode = Mode::Browse;
                if app.list_state.selected().is_none() {
                    app.list_state.select(Some(0));
                    app.update_source_options();
                }
            }
        }
        KeyCode::Esc => app.should_quit = true,
        _ => {}
    }
    None
}

fn handle_browse_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
    if modifiers.contains(KeyModifiers::CONTROL) && key == KeyCode::Char('f') {
        return Some(Action::FuzzySearch);
    }
    let results_len = app.current_results().len();
    let page_start = app.page * app.page_size;
    let page_end = (page_start + app.page_size).min(results_len);
    let page_items = page_end - page_start;

    match key {
        KeyCode::Up => {
            if let Some(sel) = app.list_state.selected() {
                if sel > 0 {
                    app.list_state.select(Some(sel - 1));
                } else if app.page > 0 {
                    app.page -= 1;
                    let new_page_items = app.page_size.min(results_len - app.page * app.page_size);
                    app.list_state
                        .select(Some(new_page_items.saturating_sub(1)));
                }
            }
            app.update_source_options();
        }
        KeyCode::Down => {
            if let Some(sel) = app.list_state.selected() {
                if sel + 1 < page_items {
                    app.list_state.select(Some(sel + 1));
                } else if app.page + 1 < app.total_pages() {
                    app.page += 1;
                    app.list_state.select(Some(0));
                }
            }
            app.update_source_options();
        }
        KeyCode::Left => {
            if app.source_options.len() > 1 {
                app.source_selected = app.source_selected.saturating_sub(1);
            } else if app.page > 0 {
                app.page -= 1;
                app.list_state.select(Some(0));
                app.update_source_options();
            }
        }
        KeyCode::Right => {
            if app.source_options.len() > 1 {
                app.source_selected = (app.source_selected + 1).min(app.source_options.len() - 1);
            } else if app.page + 1 < app.total_pages() {
                app.page += 1;
                app.list_state.select(Some(0));
                app.update_source_options();
            }
        }
        KeyCode::Tab => {
            app.active_tab = (app.active_tab + 1) % app.tab_titles.len();
            app.page = 0;
            app.list_state.select(if app.current_results().is_empty() {
                None
            } else {
                Some(0)
            });
            app.update_source_options();
        }
        KeyCode::BackTab => {
            app.active_tab = if app.active_tab == 0 {
                app.tab_titles.len() - 1
            } else {
                app.active_tab - 1
            };
            app.page = 0;
            app.list_state.select(if app.current_results().is_empty() {
                None
            } else {
                Some(0)
            });
            app.update_source_options();
        }
        KeyCode::Char('i') | KeyCode::Enter => {
            if let Some(pkg) = app.selected_package() {
                let _source = if app.source_options.len() > 1 {
                    app.source_options
                        .get(app.source_selected)
                        .cloned()
                        .unwrap_or_default()
                } else {
                    pkg.source.clone()
                };
                let pkg_name = pkg.name.clone();
                app.confirm_action = "install".to_string();
                app.confirm_packages = vec![pkg_name];
                app.confirm_selected_yes = true;
                app.mode = Mode::Confirm;
                return None;
            }
        }
        KeyCode::Char('d') | KeyCode::Char('r') => {
            if let Some(pkg) = app.selected_package() {
                let pkg_name = pkg.name.clone();
                app.confirm_action = "remove".to_string();
                app.confirm_packages = vec![pkg_name];
                app.confirm_selected_yes = true;
                app.mode = Mode::Confirm;
            }
        }
        KeyCode::Char('/') => {
            app.mode = Mode::Input;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
    None
}

fn handle_confirm_key(app: &mut App, key: KeyCode) -> Option<Action> {
    let confirm_yes = || {
        let pkgs = app.confirm_packages.clone();
        let action = app.confirm_action.clone();

        let new_app_mode = Mode::Progress;
        let progress_items = pkgs
            .iter()
            .map(|p| ProgressItem {
                name: p.clone(),
                done: false,
                success: false,
            })
            .collect();

        if action == "install" {
            let source = if app.source_options.len() > 1 {
                app.source_options
                    .get(app.source_selected)
                    .cloned()
                    .unwrap_or_default()
            } else if let Some(pkg) = app.all_results.iter().find(|p| p.name == pkgs[0]) {
                pkg.source.clone()
            } else {
                String::new()
            };
            Some((new_app_mode, progress_items, Action::Install(pkgs, source)))
        } else {
            Some((new_app_mode, progress_items, Action::Remove(pkgs)))
        }
    };

    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
            app.confirm_selected_yes = !app.confirm_selected_yes;
            None
        }
        KeyCode::Enter => {
            if app.confirm_selected_yes {
                if let Some((m, items, act)) = confirm_yes() {
                    app.mode = m;
                    app.progress_items = items;
                    return Some(act);
                }
                None
            } else {
                app.mode = Mode::Browse;
                None
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some((m, items, act)) = confirm_yes() {
                app.mode = m;
                app.progress_items = items;
                return Some(act);
            }
            None
        }
        KeyCode::Char('n')
        | KeyCode::Char('N')
        | KeyCode::Char('c')
        | KeyCode::Char('C')
        | KeyCode::Char('b')
        | KeyCode::Char('B')
        | KeyCode::Esc => {
            app.mode = Mode::Browse;
            None
        }
        _ => None,
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Launch the full-screen search TUI (used by `s8n search` direct CLI)
pub async fn run_search_tui(
    managers: &[Box<dyn PackageManager>],
    initial_query: Option<&str>,
) -> io::Result<()> {
    // Load config theme
    theme::reload();

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_search_tui_inner(&mut terminal, managers, initial_query).await?;

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

/// The inner loop of the search TUI (callable without taking over terminal setup)
pub async fn run_search_tui_inner(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    managers: &[Box<dyn PackageManager>],
    initial_query: Option<&str>,
) -> io::Result<()> {
    let mut app = App::new();

    // Pre-fill query if provided
    if let Some(q) = initial_query {
        app.search_input = q.to_string();
        app.cursor_pos = q.len();
        // Immediately search
        app.status_message = format!("Searching for '{}'...", q);
        let results = search_all(managers, q).await;
        app.set_results(results);
        app.mode = Mode::Browse;
        if app.list_state.selected().is_none() && !app.all_results.is_empty() {
            app.list_state.select(Some(0));
            app.update_source_options();
        }
        app.status_message.clear();
    }

    // Main loop
    loop {
        app.tick += 1;
        render(terminal, &mut app)?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if let Some(action) = handle_key(&mut app, key_event.code, key_event.modifiers) {
                    match action {
                        Action::Search(query) => {
                            app.status_message = format!("Searching for '{}'...", query);
                            render(terminal, &mut app)?;
                            let results = search_all(managers, &query).await;
                            app.set_results(results);
                            app.mode = Mode::Browse;
                            if app.list_state.selected().is_none() && !app.all_results.is_empty() {
                                app.list_state.select(Some(0));
                                app.update_source_options();
                            }
                            app.status_message.clear();
                        }
                        Action::Install(pkgs, source) => {
                            render(terminal, &mut app)?;
                            // Find the right manager
                            let pm = managers
                                .iter()
                                .find(|m| m.name() == source)
                                .or_else(|| managers.first());
                            if let Some(pm) = pm {
                                for (i, pkg) in pkgs.iter().enumerate() {
                                    let result = pm.install(&[pkg.clone()]).await;
                                    if let Some(item) = app.progress_items.get_mut(i) {
                                        item.done = true;
                                        item.success = matches!(result, PmResult::Success);
                                    }
                                    render(terminal, &mut app)?;
                                }
                            }
                            app.mode = Mode::Done;
                        }
                        Action::Remove(pkgs) => {
                            render(terminal, &mut app)?;
                            if let Some(pm) = managers.first() {
                                for (i, pkg) in pkgs.iter().enumerate() {
                                    let result = pm.remove(&[pkg.clone()]).await;
                                    if let Some(item) = app.progress_items.get_mut(i) {
                                        item.done = true;
                                        item.success = matches!(result, PmResult::Success);
                                    }
                                    render(terminal, &mut app)?;
                                }
                            }
                            app.mode = Mode::Done;
                        }
                        Action::FuzzySearch => {
                            if app.all_results.is_empty() {
                                continue;
                            }
                            // Suspend TUI
                            terminal::disable_raw_mode()?;
                            execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;

                            // Prepare items for Skim
                            use std::io::Write;
                            use std::process::{Command, Stdio};

                            let child_res = Command::new("sk")
                                .arg("--ansi")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn();

                            match child_res {
                                Ok(mut child) => {
                                    let mut items_text = String::new();
                                    for pkg in &app.all_results {
                                        items_text
                                            .push_str(&format!("{} [{}]\n", pkg.name, pkg.source));
                                    }

                                    if let Some(mut stdin) = child.stdin.take() {
                                        let _ = stdin.write_all(items_text.as_bytes());
                                    }

                                    let output = child.wait_with_output().unwrap_or_else(|_| {
                                        std::process::Output {
                                            status: std::os::unix::process::ExitStatusExt::from_raw(
                                                1,
                                            ),
                                            stdout: Vec::new(),
                                            stderr: Vec::new(),
                                        }
                                    });

                                    // Restore TUI
                                    terminal::enable_raw_mode()?;
                                    execute!(
                                        terminal.backend_mut(),
                                        EnterAlternateScreen,
                                        cursor::Hide
                                    )?;
                                    terminal.clear()?;

                                    if output.status.success() {
                                        let selected = String::from_utf8_lossy(&output.stdout);
                                        let selected_line = selected.trim();
                                        if !selected_line.is_empty() {
                                            if let Some(idx) = selected_line.rfind(" [") {
                                                let name = &selected_line[..idx];
                                                let filtered: Vec<PackageInfo> = app
                                                    .all_results
                                                    .clone()
                                                    .into_iter()
                                                    .filter(|p| p.name == name)
                                                    .collect();
                                                if !filtered.is_empty() {
                                                    let mut map = std::collections::HashMap::new();
                                                    for p in filtered {
                                                        map.entry(p.source.clone())
                                                            .or_insert_with(Vec::new)
                                                            .push(p);
                                                    }
                                                    let mut new_results: Vec<(
                                                        String,
                                                        Vec<PackageInfo>,
                                                    )> = map.into_iter().collect();
                                                    new_results.sort_by(|a, b| a.0.cmp(&b.0));
                                                    app.set_results(new_results);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Restore TUI and show error
                                    terminal::enable_raw_mode()?;
                                    execute!(
                                        terminal.backend_mut(),
                                        EnterAlternateScreen,
                                        cursor::Hide
                                    )?;
                                    terminal.clear()?;
                                    app.status_message =
                                        "Error: Skim (sk) fuzzy finder is not installed."
                                            .to_string();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// The unified application loop for `s8n` CLI command without subcommands
pub async fn run_main_tui(
    managers: Vec<Box<dyn PackageManager>>,
    requested_manager: Option<&str>,
) -> io::Result<()> {
    // Load config theme
    theme::reload();

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut current_mode = AppMode::Menu;
    let mut menu_state = menu::MenuState::new();
    let mut fm_state = file_manager::FileManagerState::new();
    let mut cp_state = color_picker::ColorPickerState::new();

    let search_managers: Vec<Box<dyn PackageManager>> = if let Some(requested) = requested_manager {
        managers
            .into_iter()
            .filter(|m| m.name() == requested)
            .collect()
    } else {
        managers
    };

    loop {
        match current_mode {
            AppMode::Menu => {
                terminal.draw(|f| {
                    let size = f.area();
                    f.render_widget(
                        ratatui::widgets::Block::default()
                            .style(ratatui::style::Style::default().bg(theme::bg_color())),
                        size,
                    );
                    menu::render_menu(f, &mut menu_state, size);
                })?;

                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if let Some(action) = menu_state.handle_key(key.code) {
                            match action {
                                menu::MenuAction::PackMan => current_mode = AppMode::PackManSearch,
                                menu::MenuAction::FileManager => {
                                    current_mode = AppMode::FileManager
                                }
                                menu::MenuAction::ColorTheme => current_mode = AppMode::ColorTheme,
                                menu::MenuAction::Quit => break,
                            }
                        }
                    }
                }
            }
            AppMode::FileManager => {
                terminal.draw(|f| {
                    let size = f.area();
                    f.render_widget(
                        ratatui::widgets::Block::default()
                            .style(ratatui::style::Style::default().bg(theme::bg_color())),
                        size,
                    );
                    file_manager::render_file_manager(f, &mut fm_state, size);
                })?;

                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if fm_state.handle_key(key.code) {
                            current_mode = AppMode::Menu;
                        }
                    }
                }
            }
            AppMode::ColorTheme => {
                terminal.draw(|f| {
                    let size = f.area();
                    f.render_widget(
                        ratatui::widgets::Block::default()
                            .style(ratatui::style::Style::default().bg(theme::bg_color())),
                        size,
                    );
                    color_picker::render_color_picker(f, &mut cp_state, size);
                })?;

                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if cp_state.handle_key(key.code) {
                            current_mode = AppMode::Menu;
                        }
                    }
                }
            }
            AppMode::PackManSearch => {
                // Hand off terminal control entirely to the search sub-app
                run_search_tui_inner(&mut terminal, &search_managers, None).await?;
                // Once it returns (user pressed q or Esc), go back to main menu
                current_mode = AppMode::Menu;
            }
            _ => {
                // Return to menu as fallback
                current_mode = AppMode::Menu;
            }
        }
    }

    // Restore terminal
    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

/// Launch inline progress TUI for install/remove/update (non-search operations)
pub async fn run_progress_tui(
    pm: &dyn PackageManager,
    packages: Vec<String>,
    action: &str, // "install", "remove", "update"
) -> io::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items: Vec<String> = if action == "update" && packages.is_empty() {
        vec!["System Packages".to_string()]
    } else {
        packages.clone()
    };

    let mut app = App::new();
    app.confirm_action = action.to_string();
    app.mode = Mode::Progress;
    app.progress_items = items
        .iter()
        .map(|p| ProgressItem {
            name: p.clone(),
            done: false,
            success: false,
        })
        .collect();

    render(&mut terminal, &mut app)?;

    for (i, pkg) in items.iter().enumerate() {
        let result = match action {
            "install" => pm.install(&[pkg.clone()]).await,
            "remove" => pm.remove(&[pkg.clone()]).await,
            "update" => pm.update().await,
            _ => PmResult::Error("Unknown action".into()),
        };
        if let Some(item) = app.progress_items.get_mut(i) {
            item.done = true;
            item.success = matches!(result, PmResult::Success);
        }
        render(&mut terminal, &mut app)?;
    }

    app.mode = Mode::Done;
    render(&mut terminal, &mut app)?;

    // Wait for quit
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if matches!(
                    key_event.code,
                    KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter
                ) {
                    break;
                }
            }
        }
    }

    terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, cursor::Show)?;
    Ok(())
}

/// Search all available managers concurrently and collect results
/// Search all available managers concurrently and collect results
async fn search_all(
    managers: &[Box<dyn PackageManager>],
    query: &str,
) -> Vec<(String, Vec<PackageInfo>)> {
    let mut results: std::collections::HashMap<String, Vec<PackageInfo>> =
        std::collections::HashMap::new();
    let terms: Vec<&str> = query
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // Search sequentially to avoid overwhelming the terminal
    for pm in managers {
        if !pm.is_available() || matches!(pm.name(), "topgrade" | "bun") {
            continue;
        }
        for term in &terms {
            if let Ok(pkgs) = pm.search_captured(term).await {
                if !pkgs.is_empty() {
                    results
                        .entry(pm.name().to_string())
                        .or_default()
                        .extend(pkgs);
                }
            }
        }
    }

    // Convert hashmap back to vec and sort
    let mut final_results: Vec<(String, Vec<PackageInfo>)> = results.into_iter().collect();
    final_results.sort_by(|a, b| a.0.cmp(&b.0));
    final_results
}
