use super::theme;
/// Paginator widget with animated dot indicators
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Paginator {
    pub current_page: usize,
    pub total_pages: usize,
    pub tick: u64, // animation tick for subtle glow effect
}

impl Widget for Paginator {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.total_pages <= 1 || area.height == 0 {
            return;
        }

        let mut spans = Vec::new();

        // Left arrow
        if self.current_page > 0 {
            spans.push(Span::styled("◀ ", theme::source_tag()));
        } else {
            spans.push(Span::styled("  ", theme::dim()));
        }

        // Page dots with glow animation on active dot
        let max_dots = 9.min(self.total_pages);
        let start = if self.total_pages <= max_dots || self.current_page < max_dots / 2 {
            0
        } else if self.current_page > self.total_pages - max_dots / 2 - 1 {
            self.total_pages - max_dots
        } else {
            self.current_page - max_dots / 2
        };

        for i in start..(start + max_dots).min(self.total_pages) {
            if i == self.current_page {
                // Active dot with animation: alternate between ● and ◉
                let dot = if self.tick % 4 < 2 { "● " } else { "◉ " };
                spans.push(Span::styled(dot, theme::active_tab()));
            } else {
                spans.push(Span::styled("○ ", theme::dim()));
            }
        }

        // Right arrow
        if self.current_page < self.total_pages.saturating_sub(1) {
            spans.push(Span::styled(" ▶", theme::source_tag()));
        }

        // Page counter
        spans.push(Span::styled(
            format!("  Page {}/{}", self.current_page + 1, self.total_pages),
            theme::status_text(),
        ));

        let line = Line::from(spans);
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .render(area, buf);
    }
}
