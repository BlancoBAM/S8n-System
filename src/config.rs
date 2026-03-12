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
