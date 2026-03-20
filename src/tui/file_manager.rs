use crate::tui::theme;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct FileEntry {
    name: String,
    is_dir: bool,
    size: u64,
}

pub struct FileManagerState {
    pub current_dir: PathBuf,
    pub parent_entries: Vec<FileEntry>,
    pub current_entries: Vec<FileEntry>,
    pub preview_text: Vec<String>,
    pub list_state: ListState,

    // For move operation
    pub marked_for_move: Option<PathBuf>,
}

impl Default for FileManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl FileManagerState {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        let mut state = Self {
            current_dir: PathBuf::from(home),
            parent_entries: Vec::new(),
            current_entries: Vec::new(),
            preview_text: Vec::new(),
            list_state: ListState::default(),
            marked_for_move: None,
        };
        state.refresh();
        state
    }

    pub fn refresh(&mut self) {
        self.current_entries = read_dir(&self.current_dir);
        if let Some(parent) = self.current_dir.parent() {
            self.parent_entries = read_dir(parent);
        } else {
            self.parent_entries.clear();
        }

        // Ensure valid selection
        if self.current_entries.is_empty() {
            self.list_state.select(None);
            self.preview_text.clear();
        } else {
            let i = self
                .list_state
                .selected()
                .unwrap_or(0)
                .min(self.current_entries.len().saturating_sub(1));
            self.list_state.select(Some(i));
            self.update_preview();
        }
    }

    fn update_preview(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(entry) = self.current_entries.get(i) {
                let path = self.current_dir.join(&entry.name);
                self.preview_text = generate_preview(&path, entry.is_dir);
            }
        } else {
            self.preview_text.clear();
        }
    }

    pub fn next(&mut self) {
        if self.current_entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.current_entries.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_preview();
    }

    pub fn previous(&mut self) {
        if self.current_entries.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.current_entries.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_preview();
    }

    pub fn drill_down(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(entry) = self.current_entries.get(i) {
                if entry.is_dir {
                    let next_dir = self.current_dir.join(&entry.name);
                    // attempt to read it
                    if fs::read_dir(&next_dir).is_ok() {
                        self.current_dir = next_dir;
                        self.list_state.select(Some(0));
                        self.refresh();
                    }
                }
            }
        }
    }

    pub fn drill_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            let old_name = self
                .current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.to_string());
            self.current_dir = parent.to_path_buf();
            self.refresh();

            // Try to select the directory we just left
            if let Some(name) = old_name {
                if let Some(i) = self.current_entries.iter().position(|e| e.name == name) {
                    self.list_state.select(Some(i));
                    self.update_preview();
                }
            }
        }
    }

    // Returns true to quit this mode
    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => self.drill_down(),
            KeyCode::Left | KeyCode::Char('h') => self.drill_up(),

            // File ops
            KeyCode::Char('o') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(entry) = self.current_entries.get(i) {
                        if !entry.is_dir {
                            let path = self.current_dir.join(&entry.name);
                            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                        }
                    }
                }
            }
            KeyCode::Char('e') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(entry) = self.current_entries.get(i) {
                        let path = self.current_dir.join(&entry.name);
                        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                        let _ = std::process::Command::new(editor).arg(path).spawn();
                    }
                }
            }
            KeyCode::Char('d') => {
                // Basic deletion - realistically would ask for confirm
                if let Some(i) = self.list_state.selected() {
                    if let Some(entry) = self.current_entries.get(i) {
                        let path = self.current_dir.join(&entry.name);
                        if entry.is_dir {
                            let _ = fs::remove_dir_all(path);
                        } else {
                            let _ = fs::remove_file(path);
                        }
                        self.refresh();
                    }
                }
            }
            KeyCode::Char('m') => {
                if let Some(i) = self.list_state.selected() {
                    if let Some(entry) = self.current_entries.get(i) {
                        self.marked_for_move = Some(self.current_dir.join(&entry.name));
                    }
                }
            }
            KeyCode::Char('p') => {
                if let Some(src) = &self.marked_for_move {
                    if let Some(file_name) = src.file_name() {
                        let dest = self.current_dir.join(file_name);
                        let _ = fs::rename(src, dest);
                        self.marked_for_move = None;
                        self.refresh();
                    }
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => return true,
            _ => {}
        }
        false
    }
}

pub fn render_file_manager(f: &mut ratatui::Frame, state: &mut FileManagerState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let main_area = chunks[0];
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // parent
            Constraint::Percentage(40), // current
            Constraint::Percentage(35), // preview
        ])
        .split(main_area);

    // ── Parent pane
    let p_items: Vec<ListItem> = state
        .parent_entries
        .iter()
        .map(|e| {
            let prefix = if e.is_dir { "▸ " } else { "  " };
            let name = &e.name;
            ListItem::new(Line::from(Span::styled(
                format!("{}{}", prefix, name),
                theme::dim(),
            )))
        })
        .collect();
    let parent_name = state
        .current_dir
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("/");
    let p_list = List::new(p_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border())
            .title(Span::styled(parent_name, theme::grid_header())),
    );
    f.render_widget(p_list, columns[0]);

    // ── Current pane
    let c_items: Vec<ListItem> = state
        .current_entries
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let is_selected = state.list_state.selected() == Some(i);
            let prefix = if e.is_dir {
                if is_selected {
                    "▸ "
                } else {
                    "▸ "
                }
            } else {
                "  "
            };

            let mut style = if is_selected {
                theme::highlight()
            } else {
                Style::default().fg(theme::text())
            };

            // Render name
            let name = &e.name;

            // "Fire gradient" selected colorization simulation
            // The lipgloss-rs example uses a CIELAB gradient mapping. Since we can't emit ANSI raw safely to ratatui ListItem easily anymore,
            // we map it manually to a style using ratatui's RGB if selected.
            if is_selected {
                // Hot orange-red to yellow fire gradient mapped to foreground text
                // For true gradient we'd need ansi-to-tui to span each character.
                // Instead we just use the vibrant overlay background + hot text
                style = Style::default()
                    .bg(theme::overlay_color())
                    .fg(theme::hot_pink())
                    .add_modifier(Modifier::BOLD);
            }

            ListItem::new(Line::from(vec![
                Span::styled(
                    prefix,
                    if is_selected {
                        theme::search_label()
                    } else {
                        Style::default()
                    },
                ),
                Span::styled(name.clone(), style),
            ]))
        })
        .collect();
    let current_name = state
        .current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("/");
    let c_list = List::new(c_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border())
            .title(Span::styled(current_name, theme::grid_header())),
    );
    f.render_stateful_widget(c_list, columns[1], &mut state.list_state);

    // ── Preview pane
    let preview_lines: Vec<Line> = state
        .preview_text
        .iter()
        .map(|s| Line::from(s.as_str()))
        .collect();
    let preview = Paragraph::new(preview_lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme::border())
                .title(Span::styled(" Preview ", theme::grid_header())),
        );
    f.render_widget(preview, columns[2]);

    // ── Bottom help
    let help_text =
        "[↑↓] nav   [←→] in/out   [d] del   [o] open   [e] edit   [m] move   [p] paste   [q] back";
    let help = Paragraph::new(Span::styled(help_text, theme::dim())).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border()),
    );
    f.render_widget(help, chunks[1]);
}

// --- Helpers ---

fn read_dir(path: &Path) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    if let Ok(rd) = fs::read_dir(path) {
        for entry in rd.flatten() {
            if let Ok(metadata) = entry.metadata() {
                entries.push(FileEntry {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    is_dir: metadata.is_dir(),
                    size: metadata.len(),
                });
            }
        }
    }
    // Sort dirs first, then alphabetical
    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

fn generate_preview(path: &Path, is_dir: bool) -> Vec<String> {
    let mut lines = Vec::new();
    if is_dir {
        let entries = read_dir(path);
        lines.push("📁 Directory".to_string());
        lines.push(format!("Contains {} items", entries.len()));
        lines.push("".to_string());
        for e in entries.iter().take(20) {
            lines.push(format!("{} {}", if e.is_dir { "▸" } else { " " }, e.name));
        }
        if entries.len() > 20 {
            lines.push("...".to_string());
        }
    } else {
        if let Ok(metadata) = fs::metadata(path) {
            lines.push(format!("📄 File: {} bytes", metadata.len()));
            lines.push("".to_string());
        }
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines().take(30) {
                // simple truncate to 100 char to prevent wrap mess
                let truncated: String = line.chars().take(80).collect();
                lines.push(truncated);
            }
        } else {
            lines.push("Binary or unreadable file".to_string());
        }
    }
    lines
}
