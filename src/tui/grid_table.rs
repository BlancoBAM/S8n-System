use super::theme;
/// GridTable — a ratatui widget that renders a table with real Unicode grid lines
/// drawn directly into the terminal buffer.
///
/// Columns are separated by │ (U+2502)
/// Rows are separated by ─ (U+2500) lines after the header
/// The active row is highlighted with a vivid purple background
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::Style,
    text::Span,
    widgets::Widget,
};

/// A single column definition
pub struct Column {
    pub header: &'static str,
    pub width: Constraint,
}

/// A single cell value
pub struct GridCell {
    pub text: String,
    pub style: Style,
}

/// A single data row
pub struct GridRow {
    pub cells: Vec<GridCell>,
}

pub struct GridTable<'a> {
    pub columns: &'a [Column],
    pub rows: &'a [GridRow],
    pub selected: Option<usize>, // 0-indexed selected row (within current page)
    pub header_style: Style,
    pub separator_style: Style,
    pub selected_style: Style,
}

impl<'a> Widget for GridTable<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height < 3 || area.width < 4 {
            return;
        }

        // Compute actual column pixel widths
        let widths = compute_widths(self.columns, area.width);

        // ── Draw outer border ──
        draw_box(buf, area, theme::grid_header());

        // ── Draw header row (row y = area.y + 1) ──
        let header_y = area.y + 1;
        let mut x = area.x + 1;

        for (col_idx, col) in self.columns.iter().enumerate() {
            let w = widths[col_idx];
            // Draw header text, truncated and padded
            let text = truncate_pad(col.header, w.saturating_sub(1) as usize);
            let span = Span::styled(format!(" {}", text), self.header_style);
            buf.set_span(x, header_y, &span, w);

            // Draw column separator │ after every column except the last
            if col_idx < self.columns.len() - 1 {
                let sep_x = x + w;
                if sep_x < area.x + area.width - 1 {
                    buf.get_mut(sep_x, header_y)
                        .set_symbol("│")
                        .set_style(self.separator_style);
                }
            }
            x += w + 1; // +1 for separator
        }

        // ── Draw header separator line ── (─┼─ between header and data)
        let sep_y = area.y + 2;
        if sep_y < area.y + area.height - 1 {
            x = area.x + 1;
            for col_idx in 0..self.columns.len() {
                let w = widths[col_idx];
                for fill_x in x..(x + w).min(area.x + area.width - 1) {
                    buf.get_mut(fill_x, sep_y)
                        .set_symbol("─")
                        .set_style(self.separator_style);
                }
                if col_idx < self.columns.len() - 1 {
                    let sep_x = x + w;
                    if sep_x < area.x + area.width - 1 {
                        buf.get_mut(sep_x, sep_y)
                            .set_symbol("┼")
                            .set_style(self.separator_style);
                    }
                }
                x += w + 1;
            }
            // Left and right T-junctions
            buf.get_mut(area.x, sep_y)
                .set_symbol("├")
                .set_style(self.separator_style);
            if area.x + area.width > 0 {
                buf.get_mut(area.x + area.width - 1, sep_y)
                    .set_symbol("┤")
                    .set_style(self.separator_style);
            }
        }

        // ── Draw data rows ──
        let data_start_y = area.y + 3; // After border-top, header, and separator
        let max_rows = (area.height as usize).saturating_sub(4); // border top+bottom + header + sep

        for (row_idx, row) in self.rows.iter().take(max_rows).enumerate() {
            let row_y = data_start_y + row_idx as u16;
            if row_y >= area.y + area.height - 1 {
                break;
            }

            let is_selected = self.selected == Some(row_idx);
            let row_bg = if is_selected {
                self.selected_style
            } else {
                Style::default().bg(theme::bg_color())
            };

            // Fill row background
            for fill_x in (area.x + 1)..(area.x + area.width - 1) {
                buf.get_mut(fill_x, row_y).set_style(row_bg);
            }

            // Draw selection indicator
            if is_selected {
                buf.get_mut(area.x, row_y)
                    .set_symbol("▌")
                    .set_style(Style::default().fg(theme::hot_pink()));
            }

            x = area.x + 1;
            for (col_idx, _col) in self.columns.iter().enumerate() {
                let w = widths[col_idx];
                if let Some(cell) = row.cells.get(col_idx) {
                    let text = truncate_pad(&cell.text, w.saturating_sub(1) as usize);
                    let cell_style = if is_selected {
                        // Keep fg from cell style but override bg
                        cell.style.bg(theme::overlay_color())
                    } else {
                        cell.style
                    };
                    let span = Span::styled(format!(" {}", text), cell_style);
                    buf.set_span(x, row_y, &span, w);
                }

                // Column separator │
                if col_idx < self.columns.len() - 1 {
                    let sep_x = x + w;
                    if sep_x < area.x + area.width - 1 {
                        buf.get_mut(sep_x, row_y)
                            .set_symbol("│")
                            .set_style(self.separator_style);
                    }
                }
                x += w + 1;
            }
        }
    }
}

/// Truncate text to max bytes, padding with spaces to fill width
fn truncate_pad(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    // Measure display width
    let mut result = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        if width + ch_w > max {
            // Add ellipsis if we're truncating
            if max >= 1 {
                result.push('…');
            }
            break;
        }
        result.push(ch);
        width += ch_w;
    }
    // Pad to max width
    while width < max {
        result.push(' ');
        width += 1;
    }
    result
}

/// Draw a single-line box border around an area
fn draw_box(buf: &mut Buffer, area: Rect, style: Style) {
    // Corners
    buf.get_mut(area.x, area.y).set_symbol("┌").set_style(style);
    buf.get_mut(area.x + area.width - 1, area.y)
        .set_symbol("┐")
        .set_style(style);
    buf.get_mut(area.x, area.y + area.height - 1)
        .set_symbol("└")
        .set_style(style);
    buf.get_mut(area.x + area.width - 1, area.y + area.height - 1)
        .set_symbol("┘")
        .set_style(style);
    // Top and bottom edges
    for x in (area.x + 1)..(area.x + area.width - 1) {
        buf.get_mut(x, area.y).set_symbol("─").set_style(style);
        buf.get_mut(x, area.y + area.height - 1)
            .set_symbol("─")
            .set_style(style);
    }
    // Left and right edges
    for y in (area.y + 1)..(area.y + area.height - 1) {
        buf.get_mut(area.x, y).set_symbol("│").set_style(style);
        buf.get_mut(area.x + area.width - 1, y)
            .set_symbol("│")
            .set_style(style);
    }
}

/// Compute pixel widths for columns given available total width
fn compute_widths(columns: &[Column], total_width: u16) -> Vec<u16> {
    // Account for outer borders (2) and separators between columns (n-1)
    let separators = columns.len().saturating_sub(1) as u16;
    let available = total_width.saturating_sub(2 + separators);

    let mut widths = Vec::with_capacity(columns.len());
    let mut used = 0u16;

    // First pass: allocate fixed widths
    for col in columns.iter() {
        match col.width {
            Constraint::Length(n) => {
                widths.push(n);
                used += n;
            }
            _ => {
                widths.push(0); // placeholder
            }
        }
    }

    // If fixed columns already exceed available space, scale them down proportionally
    if used > available && available > 0 {
        let scale = available as f64 / used as f64;
        let mut scaled_used = 0u16;
        for (i, col) in columns.iter().enumerate() {
            if matches!(col.width, Constraint::Length(_)) {
                widths[i] = (widths[i] as f64 * scale).max(1.0) as u16;
                scaled_used += widths[i];
            }
        }
        used = scaled_used;
    }

    // Second pass: distribute remaining to Percentage and Min
    let remaining = available.saturating_sub(used);
    let flexible_count = columns
        .iter()
        .filter(|c| !matches!(c.width, Constraint::Length(_)))
        .count();
    let per_flex = if flexible_count > 0 {
        remaining / flexible_count as u16
    } else {
        0
    };

    for (i, col) in columns.iter().enumerate() {
        match col.width {
            Constraint::Percentage(p) => {
                widths[i] = (available * p / 100).min(remaining);
            }
            Constraint::Min(n) => {
                widths[i] = per_flex.max(n);
            }
            _ => {}
        }
    }

    widths
}
