use ratatui::style::{Color, Modifier, Style};
use std::sync::RwLock;

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub surface: Color,
    pub overlay: Color,
    pub overlay_alt: Color,
    pub hot_pink: Color,
    pub electric_cyan: Color,
    pub neon_green: Color,
    pub vivid_purple: Color,
    pub light_purple: Color,
    pub bright_yellow: Color,
    pub neon_orange: Color,
    pub bright_red: Color,
    pub gradient_pink: Color,
    pub teal: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub grid: Color,
}

// ── Base Palette: Fire (Default) ─────────────────────────────────────────────
pub const FIRE_PALETTE: Palette = Palette {
    bg: Color::Rgb(25, 15, 15),
    surface: Color::Rgb(45, 25, 20),
    overlay: Color::Rgb(140, 40, 20),
    overlay_alt: Color::Rgb(180, 60, 30),
    hot_pink: Color::Rgb(255, 100, 0),        // Vibrant Orange
    electric_cyan: Color::Rgb(255, 220, 0),   // Bright Yellow
    neon_green: Color::Rgb(255, 60, 0),       // Red-Orange
    vivid_purple: Color::Rgb(200, 50, 0),     // Deep Orange
    light_purple: Color::Rgb(255, 180, 100),  // Light Orange 
    bright_yellow: Color::Rgb(255, 255, 100), // Pure Yellow
    neon_orange: Color::Rgb(255, 140, 0),     // Neon Orange
    bright_red: Color::Rgb(255, 30, 30),      // Pure Red
    gradient_pink: Color::Rgb(255, 80, 0),    // Orange-Red
    teal: Color::Rgb(255, 160, 0),            // Yellow-Orange
    text: Color::Rgb(250, 240, 230),          // Warm White
    text_dim: Color::Rgb(160, 120, 110),      // Warm Dim
    border: Color::Rgb(120, 50, 30),          // Orange-Brown Border
    grid: Color::Rgb(80, 30, 20),             // Darkest Border
};

// ── Purple Dream Palette (Original Lipgloss vibe) ────────────────────────────
pub const PURPLE_DREAM_PALETTE: Palette = Palette {
    bg: Color::Rgb(22, 17, 35),
    surface: Color::Rgb(40, 30, 65),
    overlay: Color::Rgb(95, 60, 180),
    overlay_alt: Color::Rgb(120, 75, 210),
    hot_pink: Color::Rgb(255, 0, 170),
    electric_cyan: Color::Rgb(0, 255, 255),
    neon_green: Color::Rgb(57, 255, 20),
    vivid_purple: Color::Rgb(138, 43, 226),
    light_purple: Color::Rgb(180, 130, 255),
    bright_yellow: Color::Rgb(255, 255, 0),
    neon_orange: Color::Rgb(255, 165, 0),
    bright_red: Color::Rgb(255, 50, 50),
    gradient_pink: Color::Rgb(200, 50, 180),
    teal: Color::Rgb(0, 200, 180),
    text: Color::Rgb(240, 240, 250),
    text_dim: Color::Rgb(140, 130, 170),
    border: Color::Rgb(100, 80, 160),
    grid: Color::Rgb(70, 55, 120),
};

// ── Sunset Palette ───────────────────────────────────────────────────────────
pub const SUNSET_PALETTE: Palette = Palette {
    bg: Color::Rgb(30, 15, 20),
    surface: Color::Rgb(50, 25, 30),
    overlay: Color::Rgb(180, 60, 60),
    overlay_alt: Color::Rgb(210, 80, 80),
    hot_pink: Color::Rgb(250, 100, 50),
    electric_cyan: Color::Rgb(255, 200, 0),
    neon_green: Color::Rgb(100, 255, 50),
    vivid_purple: Color::Rgb(200, 50, 100),
    light_purple: Color::Rgb(255, 150, 180),
    bright_yellow: Color::Rgb(255, 250, 100),
    neon_orange: Color::Rgb(255, 120, 0),
    bright_red: Color::Rgb(255, 40, 40),
    gradient_pink: Color::Rgb(220, 80, 120),
    teal: Color::Rgb(50, 200, 150),
    text: Color::Rgb(250, 240, 240),
    text_dim: Color::Rgb(170, 130, 130),
    border: Color::Rgb(150, 60, 80),
    grid: Color::Rgb(90, 40, 50),
};

// ── Ocean Palette ────────────────────────────────────────────────────────────
pub const OCEAN_PALETTE: Palette = Palette {
    bg: Color::Rgb(10, 20, 35),
    surface: Color::Rgb(20, 40, 65),
    overlay: Color::Rgb(40, 100, 180),
    overlay_alt: Color::Rgb(60, 120, 210),
    hot_pink: Color::Rgb(0, 150, 255),
    electric_cyan: Color::Rgb(0, 255, 200),
    neon_green: Color::Rgb(20, 255, 150),
    vivid_purple: Color::Rgb(0, 100, 255),
    light_purple: Color::Rgb(100, 180, 255),
    bright_yellow: Color::Rgb(150, 255, 255),
    neon_orange: Color::Rgb(0, 200, 255),
    bright_red: Color::Rgb(255, 80, 100),
    gradient_pink: Color::Rgb(0, 120, 220),
    teal: Color::Rgb(0, 255, 255),
    text: Color::Rgb(230, 240, 255),
    text_dim: Color::Rgb(120, 150, 180),
    border: Color::Rgb(50, 100, 180),
    grid: Color::Rgb(30, 60, 100),
};

// ── Forest Palette ───────────────────────────────────────────────────────────
pub const FOREST_PALETTE: Palette = Palette {
    bg: Color::Rgb(15, 25, 15),
    surface: Color::Rgb(25, 45, 25),
    overlay: Color::Rgb(50, 120, 60),
    overlay_alt: Color::Rgb(70, 150, 80),
    hot_pink: Color::Rgb(100, 255, 50),
    electric_cyan: Color::Rgb(150, 255, 0),
    neon_green: Color::Rgb(0, 255, 100),
    vivid_purple: Color::Rgb(40, 180, 80),
    light_purple: Color::Rgb(120, 220, 140),
    bright_yellow: Color::Rgb(200, 255, 100),
    neon_orange: Color::Rgb(150, 200, 50),
    bright_red: Color::Rgb(255, 100, 50),
    gradient_pink: Color::Rgb(80, 200, 100),
    teal: Color::Rgb(50, 255, 150),
    text: Color::Rgb(240, 250, 240),
    text_dim: Color::Rgb(130, 160, 130),
    border: Color::Rgb(60, 140, 70),
    grid: Color::Rgb(40, 80, 50),
};

// ── Global Mutable Theme ─────────────────────────────────────────────────────
pub static THEME: RwLock<Palette> = RwLock::new(FIRE_PALETTE);

pub fn set_theme(name: &str) {
    let p = match name {
        "Sunset" => SUNSET_PALETTE,
        "Ocean" => OCEAN_PALETTE,
        "Forest" => FOREST_PALETTE,
        "Purple Dream" => PURPLE_DREAM_PALETTE,
        // Any other or "Fire" falls back to Fire
        _ => FIRE_PALETTE, 
    };
    if let Ok(mut theme) = THEME.write() {
        *theme = p;
    }
}

pub fn reload() {
    let name = crate::config::get_theme();
    set_theme(&name);
}

// ── Exported Style Accessors ─────────────────────────────────────────────────
macro_rules! style_getter {
    ($name:ident, $field:ident) => {
        pub fn $name() -> Style {
            let color = THEME.read().unwrap().$field;
            Style::default().fg(color)
        }
    };
}

style_getter!(dim, text_dim);

pub fn title() -> Style      { Style::default().fg(THEME.read().unwrap().text).add_modifier(Modifier::BOLD) }
pub fn highlight() -> Style  { let t = THEME.read().unwrap(); Style::default().fg(t.text).bg(t.overlay).add_modifier(Modifier::BOLD) }
pub fn active_tab() -> Style { let t = THEME.read().unwrap(); Style::default().fg(Color::White).bg(t.hot_pink).add_modifier(Modifier::BOLD) }
pub fn tab() -> Style        { let t = THEME.read().unwrap(); Style::default().fg(t.light_purple).bg(t.surface) }

pub fn number() -> Style     { Style::default().fg(THEME.read().unwrap().electric_cyan).add_modifier(Modifier::BOLD) }
pub fn pkg_name() -> Style   { Style::default().fg(THEME.read().unwrap().text).add_modifier(Modifier::BOLD) }
pub fn version() -> Style    { Style::default().fg(THEME.read().unwrap().neon_orange) }
pub fn desc() -> Style       { Style::default().fg(THEME.read().unwrap().text_dim) }
pub fn source_tag() -> Style { Style::default().fg(THEME.read().unwrap().hot_pink) }
pub fn success() -> Style    { Style::default().fg(THEME.read().unwrap().neon_green).add_modifier(Modifier::BOLD) }
pub fn error() -> Style      { Style::default().fg(THEME.read().unwrap().bright_red).add_modifier(Modifier::BOLD) }

pub fn border() -> Style     { Style::default().fg(THEME.read().unwrap().border) }
pub fn grid() -> Style       { Style::default().fg(THEME.read().unwrap().grid) }
pub fn grid_header() -> Style { Style::default().fg(THEME.read().unwrap().vivid_purple).add_modifier(Modifier::BOLD) }
pub fn grid_separator() -> Style { Style::default().fg(THEME.read().unwrap().grid) }

pub fn progress() -> Style   { let t = THEME.read().unwrap(); Style::default().fg(t.hot_pink).bg(t.overlay) }
pub fn progress_bar() -> Style { let t = THEME.read().unwrap(); Style::default().fg(t.teal).bg(t.surface) }
pub fn input() -> Style      { let t = THEME.read().unwrap(); Style::default().fg(t.text).bg(t.surface) }
pub fn search_label() -> Style { Style::default().fg(THEME.read().unwrap().hot_pink).add_modifier(Modifier::BOLD) }

pub fn status_bar() -> Style { let t = THEME.read().unwrap(); Style::default().fg(Color::Black).bg(t.hot_pink).add_modifier(Modifier::BOLD) }
pub fn status_text() -> Style { Style::default().fg(THEME.read().unwrap().light_purple) }

pub fn btn_yes() -> Style    { let t = THEME.read().unwrap(); Style::default().fg(Color::Black).bg(t.neon_green).add_modifier(Modifier::BOLD) }
pub fn btn_no() -> Style     { let t = THEME.read().unwrap(); Style::default().fg(Color::Black).bg(t.bright_red).add_modifier(Modifier::BOLD) }
pub fn btn_dim() -> Style    { let t = THEME.read().unwrap(); Style::default().fg(t.text_dim).bg(t.surface) }

// Export colors heavily used as backgrounds in UI frames directly
pub fn bg_color() -> Color { THEME.read().unwrap().bg }
pub fn overlay_color() -> Color { THEME.read().unwrap().overlay }
pub fn hot_pink() -> Color { THEME.read().unwrap().hot_pink }
pub fn neon_green() -> Color { THEME.read().unwrap().neon_green }
pub fn vivid_purple() -> Color { THEME.read().unwrap().vivid_purple }
pub fn text() -> Color { THEME.read().unwrap().text }

