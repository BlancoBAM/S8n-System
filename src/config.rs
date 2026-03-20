use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub package_managers: Vec<PackageManagerConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PackageManagerConfig {
    pub name: String,
    pub command: String,
    pub search_args: Vec<String>,
    pub install_args: Vec<String>,
    pub remove_args: Vec<String>,
    pub update_args: Vec<String>,
}

pub async fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // For now return an empty/default config
    Ok(Config::default())
}

pub fn get_theme() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let path = std::path::PathBuf::from(home).join(".config").join("s8n").join("theme.toml");
    if let Ok(content) = std::fs::read_to_string(path) {
        if let Some(line) = content.lines().find(|l| l.starts_with("theme = ")) {
            let theme_name = line.replace("theme = ", "").replace("\"", "");
            return theme_name.trim().to_string();
        }
    }
    "Fire".to_string()
}

pub fn save_theme(theme_name: &str) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let dir = std::path::PathBuf::from(home).join(".config").join("s8n");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("theme.toml");
    let content = format!("theme = \"{}\"\n", theme_name);
    let _ = std::fs::write(path, content);
}
