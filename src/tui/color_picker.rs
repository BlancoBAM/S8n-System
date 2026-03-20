use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use bubbletea_rs::gradient::gradient_filled_segment;
use std::fs;
use std::path::PathBuf;

use super::theme;

#[derive(Clone)]
pub struct ColorTheme {
    pub name: &'static str,
    // (r,g,b) color stops could be added here for pure rust gradient gen.
    // For now we just use a preset name to write to config.
}

pub struct ColorPickerState {
    pub list_state: ListState,
    pub themes: Vec<ColorTheme>,
}

impl ColorPickerState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        let themes = vec![
            ColorTheme { name: "Fire" },
            ColorTheme { name: "Sunset" },
            ColorTheme { name: "Ocean" },
            ColorTheme { name: "Forest" },
            ColorTheme { name: "Purple Dream" },
        ];
        Self {
            list_state: state,
            themes,
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i >= self.themes.len() - 1 { 0 } else { i + 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
        // Live preview
        theme::set_theme(self.themes[i].name);
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => if i == 0 { self.themes.len() - 1 } else { i - 1 },
            None => 0,
        };
        self.list_state.select(Some(i));
        // Live preview
        theme::set_theme(self.themes[i].name);
    }

    pub fn apply_selected(&self) {
        if let Some(i) = self.list_state.selected() {
            let theme = &self.themes[i];
            crate::config::save_theme(&theme.name);
            theme::reload(); // Reload theme from config
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Enter => {
                self.apply_selected();
                // Don't quit, let user see applied theme visually
            }
            KeyCode::Char('q') | KeyCode::Esc => return true,
            _ => {}
        }
        false
    }
}

pub fn render_color_picker(f: &mut ratatui::Frame, state: &mut ColorPickerState, area: Rect) {
    let w = 50.min(area.width.saturating_sub(4));
    let h = 12.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let dialog = Rect::new(x, y, w, h);

    f.render_widget(Clear, dialog);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        .title(Span::styled(" 🎨 Color Theme Selector ", theme::grid_header()))
        .style(Style::default().bg(theme::bg_color()));

    f.render_widget(block, dialog);

    let list_area = Rect::new(dialog.x + 2, dialog.y + 2, dialog.width.saturating_sub(4), dialog.height.saturating_sub(4));

    let current_theme = crate::config::get_theme();

    let items: Vec<ListItem> = state.themes.iter().enumerate().map(|(i, theme)| {
        let is_selected = state.list_state.selected() == Some(i);
        let prefix = if is_selected { "▶ " } else { "  " };
        let active_mark = if theme.name == current_theme { " (saved)" } else { "" };
        
        let style = if is_selected {
            theme::highlight()
        } else {
            Style::default().fg(theme::THEME.read().unwrap().text)
        };
        
        ListItem::new(Line::from(vec![
            Span::styled(prefix, if is_selected { theme::search_label() } else { Style::default() }),
            Span::styled(format!("{:<20}{}", theme.name, active_mark), style),
        ]))
    }).collect();

    let list = List::new(items);
    f.render_stateful_widget(list, list_area, &mut state.list_state);
    
    // Help line
    let help_area = Rect::new(dialog.x + 2, dialog.y + dialog.height - 2, dialog.width.saturating_sub(4), 1);
    f.render_widget(Paragraph::new(Span::styled("[↑↓] preview   [Enter] save   [q] back", theme::dim())), help_area);
}
