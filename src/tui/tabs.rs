use super::theme;
/// Dynamic tab bar widget — shows page numbers normally,
/// switches to source names when a multi-source package is highlighted
use ratatui::{buffer::Buffer, layout::Rect, text::Span, widgets::Widget};

pub enum TabMode<'a> {
    /// Normal mode: show source tabs with result counts ("All", "apt (12)", etc.)
    Sources { titles: &'a [String], active: usize },
    /// Page mode: show page numbers when scrolling past list boundaries
    Pages { current: usize, total: usize },
    /// Source selection mode: when a multi-source package is highlighted
    PackageSources {
        sources: &'a [String],
        selected: usize,
        pkg_name: &'a str,
    },
}

pub struct TabBar<'a> {
    pub mode: TabMode<'a>,
}

impl<'a> Widget for TabBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        // Clear entire row with background color
        for x in area.x..area.x.saturating_add(area.width).min(buf.area.width) {
            buf[(x, area.y)]
                .set_char(' ')
                .set_bg(theme::bg_color())
                .set_fg(theme::bg_color());
        }

        match self.mode {
            TabMode::Sources { titles, active } => {
                render_tabs(area, buf, titles, active);
            }
            TabMode::Pages { current, total } => {
                let titles: Vec<String> = (1..=total)
                    .map(|p| {
                        if p == current + 1 {
                            format!("Page {}", p)
                        } else {
                            format!("{}", p)
                        }
                    })
                    .collect();
                render_tabs(area, buf, &titles, current);
            }
            TabMode::PackageSources {
                sources,
                selected,
                pkg_name,
            } => {
                let label = format!(" {} ▸ ", pkg_name);
                let label_span = Span::styled(label.clone(), theme::dim());
                let mut x = area.x + 1;
                buf.set_span(x, area.y, &label_span, label.len() as u16);
                x += label.len() as u16;

                for (i, src) in sources.iter().enumerate() {
                    if x + src.len() as u16 + 5 >= area.x + area.width {
                        break;
                    }

                    let (style, border_l, border_r) = if i == selected {
                        (theme::active_tab(), "┃ ", " ┃")
                    } else {
                        (theme::tab(), "│ ", " │")
                    };

                    let tab_text = format!("{}{}{}", border_l, src, border_r);
                    let span = Span::styled(tab_text.clone(), style);
                    buf.set_span(x, area.y, &span, tab_text.len() as u16);
                    x += tab_text.len() as u16 + 1;
                }
            }
        }
    }
}

fn render_tabs(area: Rect, buf: &mut Buffer, titles: &[String], active: usize) {
    let mut x = area.x + 1;
    for (i, title) in titles.iter().enumerate() {
        if x + title.len() as u16 + 5 >= area.x + area.width {
            break;
        }

        let (style, border_l, border_r) = if i == active {
            (theme::active_tab(), "┃ ", " ┃")
        } else {
            (theme::tab(), "│ ", " │")
        };

        let tab_text = format!("{}{}{}", border_l, title, border_r);
        let span = Span::styled(tab_text.clone(), style);
        buf.set_span(x, area.y, &span, tab_text.len() as u16);
        x += tab_text.len() as u16 + 1;
    }
}
