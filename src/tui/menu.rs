use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use bubbletea_rs::gradient::gradient_filled_segment;

use super::theme;

pub enum MenuAction {
    PackMan,
    FileManager,
    ColorTheme,
    Quit,
}

pub struct MenuState {
    pub list_state: ListState,
    pub items: Vec<&'static str>,
}

impl MenuState {
    pub fn new() -> Self {
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            list_state: state,
            items: vec!["📦  Package Manager", "📁  File Manager", "🎨  Color Theme"],
        }
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<MenuAction> {
        match key {
            KeyCode::Down | KeyCode::Char('j') => self.next(),
            KeyCode::Up | KeyCode::Char('k') => self.previous(),
            KeyCode::Enter => {
                let selected = self.list_state.selected().unwrap_or(0);
                match selected {
                    0 => return Some(MenuAction::PackMan),
                    1 => return Some(MenuAction::FileManager),
                    2 => return Some(MenuAction::ColorTheme),
                    _ => {}
                }
            }
            KeyCode::Char('q') | KeyCode::Esc => return Some(MenuAction::Quit),
            _ => {}
        }
        None
    }
}

pub fn render_menu(f: &mut ratatui::Frame, state: &mut MenuState, area: Rect) {
    let w = 40.min(area.width.saturating_sub(4));
    let h = 10.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let dialog = Rect::new(x, y, w, h);

    f.render_widget(Clear, dialog);

    // Fire gradient title (approximate lipgloss 'fire' gradient colors: red to yellow)
    let fire_title = format!("  s8n · System Manager  {}", gradient_filled_segment(10, '█'));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::border())
        // Apply the title wrapped in a block
        .title(Span::styled(" s8n Menu ", theme::grid_header()))
        .style(Style::default().bg(theme::bg_color()));

    f.render_widget(block, dialog);

    // Inner area for list
    let list_area = Rect::new(dialog.x + 2, dialog.y + 2, dialog.width.saturating_sub(4), dialog.height.saturating_sub(4));

    let items: Vec<ListItem> = state.items.iter().enumerate().map(|(i, &item)| {
        let is_selected = state.list_state.selected() == Some(i);
        let prefix = if is_selected { "▶ " } else { "  " };
        let style = if is_selected {
            theme::highlight()
        } else {
            Style::default().fg(theme::text())
        };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, if is_selected { theme::search_label() } else { Style::default() }),
            Span::styled(item, style),
        ]))
    }).collect();

    let list = List::new(items);
    f.render_stateful_widget(list, list_area, &mut state.list_state);
}
