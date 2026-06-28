use crate::pm::{builtin::get_default_managers, PackageManager};
use clap::{Parser, Subcommand};

pub mod config;
pub mod graveyard;
pub mod pm;
pub mod tui;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Specify a package manager to use (e.g., apt, flatpak, snap, brew)
    #[arg(long, short = 'm')]
    manager: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for packages across all available sources and display results in an
    /// interactive table (wraps apt search, cargo search, flatpak search, etc.)
    #[command(alias = "find", alias = "srch")]
    Search { query: Option<String> },

    /// Install packages (supports source:package syntax, e.g. apt:firefox)
    #[command(alias = "install")]
    Stall { packages: Vec<String> },

    /// Remove packages tracked by S8n (uninstalls via the appropriate package manager)
    #[command(alias = "remove", alias = "uninstall")]
    Burn { packages: Vec<String> },

    /// Bury (remove) S8n-tracked packages via rip2 graveyard so they can be recovered later
    /// Supports source:package syntax, e.g. "s8n brn apt:vim" or "s8n brn firefox"
    #[command(alias = "bury")]
    Brn { packages: Vec<String> },

    /// Exhume (recover) packages previously buried with `s8n brn`
    /// Run without arguments to list the graveyard interactively
    #[command(alias = "recover", alias = "restore")]
    Xum { packages: Vec<String> },

    /// List all installed packages system-wide from all tracked package managers
    #[command(alias = "list", alias = "installed")]
    Shw,

    /// Update all system packages
    #[command(alias = "update", alias = "upgrade")]
    Upd8,
}

fn choose_primary_manager<'a>(
    managers: &'a [Box<dyn PackageManager>],
    requested: Option<&str>,
) -> Result<&'a dyn PackageManager, String> {
    if let Some(requested) = requested {
        return managers
            .iter()
            .find(|manager| manager.name() == requested)
            .map(|manager| manager.as_ref())
            .ok_or_else(|| {
                let names: Vec<_> = managers.iter().map(|m| m.name()).collect();
                format!(
                    "Package manager '{}' not available. Available: {}",
                    requested,
                    names.join(", ")
                )
            });
    }

    managers
        .iter()
        .find(|manager| matches!(manager.name(), "apt" | "pacstall" | "brew"))
        .or_else(|| managers.first())
        .map(|manager| manager.as_ref())
        .ok_or_else(|| "No package managers found on this system".to_string())
}

/// Parse source:package syntax (e.g., "apt:firefox" → ("apt", "firefox"))
fn parse_source_prefix(input: &str) -> (Option<&str>, &str) {
    if let Some(colon) = input.find(':') {
        let source = &input[..colon];
        let pkg = &input[colon + 1..];
        // Only treat as source prefix if source is a known PM name
        let known = [
            "apt", "pacstall", "flatpak", "snap", "brew", "soar", "npm", "bun", "pip",
            "cargo", "cargo-binstall", "am",
        ];
        if known.contains(&source) && !pkg.is_empty() {
            return (Some(source), pkg);
        }
    }
    (None, input)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let managers = get_default_managers();
    let available_managers: Vec<Box<dyn PackageManager>> = managers
        .into_iter()
        .filter(|manager| manager.is_available())
        .collect();

    if available_managers.is_empty() {
        return Err("No supported package managers were found on this system".into());
    }

    let requested_manager = cli.manager.as_deref();

    match cli.command {
        None => {
            // Launch the unified main TUI
            tui::run_main_tui(available_managers, requested_manager).await?;
        }
        Some(Commands::Search { query }) => {
            // Launch full-screen search TUI directly
            let search_managers: Vec<Box<dyn PackageManager>> =
                if let Some(requested) = requested_manager {
                    available_managers
                        .into_iter()
                        .filter(|m| m.name() == requested)
                        .collect()
                } else {
                    available_managers
                };

            tui::run_search_tui(&search_managers, query.as_deref()).await?;
        }

        Some(Commands::Stall { packages }) => {
            if packages.is_empty() {
                return Err("Provide at least one package or URL to install".into());
            }

            let (urls, pkgs): (Vec<String>, Vec<String>) = packages
                .into_iter()
                .partition(|p| p.starts_with("http://") || p.starts_with("https://"));

            // Handle regular packages (with source prefix support)
            if !pkgs.is_empty() {
                // Group packages by source
                let mut by_source: std::collections::HashMap<String, Vec<String>> =
                    std::collections::HashMap::new();
                let mut default_pkgs = Vec::new();

                for pkg in &pkgs {
                    let (source, name) = parse_source_prefix(pkg);
                    if let Some(src) = source {
                        by_source
                            .entry(src.to_string())
                            .or_default()
                            .push(name.to_string());
                    } else {
                        default_pkgs.push(name.to_string());
                    }
                }

                // Install source-specific packages
                for (source, source_pkgs) in &by_source {
                    if let Ok(pm) = choose_primary_manager(&available_managers, Some(source)) {
                        tui::run_progress_tui(pm, source_pkgs.clone(), "install").await?;
                    } else {
                        eprintln!("Package manager '{}' not available", source);
                    }
                }

                // Install default packages using primary manager
                if !default_pkgs.is_empty() {
                    let pm = choose_primary_manager(&available_managers, requested_manager)?;
                    tui::run_progress_tui(pm, default_pkgs, "install").await?;
                }
            }

            // Handle URL installs via soar
            if !urls.is_empty() {
                if let Some(requested) = requested_manager {
                    if requested == "soar" {
                        let pm = choose_primary_manager(&available_managers, Some("soar"))?;
                        tui::run_progress_tui(pm, urls, "install").await?;
                    } else {
                        eprintln!("URL installs require the 'soar' backend. Re-run with `--manager soar`.");
                    }
                } else if let Some(soar) = available_managers.iter().find(|m| m.name() == "soar") {
                    tui::run_progress_tui(soar.as_ref(), urls, "install").await?;
                } else {
                    eprintln!("Warning: URL packages provided but 'soar' is not available.");
                }
            }
        }

        Some(Commands::Burn { packages }) => {
            if packages.is_empty() {
                return Err("Provide at least one package to remove".into());
            }
            // Group by source prefix
            let mut by_source: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            let mut default_pkgs = Vec::new();
            for pkg in &packages {
                let (source, name) = parse_source_prefix(pkg);
                if let Some(src) = source {
                    by_source.entry(src.to_string()).or_default().push(name.to_string());
                } else {
                    default_pkgs.push(name.to_string());
                }
            }
            for (source, src_pkgs) in &by_source {
                if let Ok(pm) = choose_primary_manager(&available_managers, Some(source)) {
                    tui::run_progress_tui(pm, src_pkgs.clone(), "remove").await?;
                }
            }
            if !default_pkgs.is_empty() {
                let pm = choose_primary_manager(&available_managers, requested_manager)?;
                tui::run_progress_tui(pm, default_pkgs, "remove").await?;
            }
        }

        Some(Commands::Brn { packages }) => {
            if packages.is_empty() {
                return Err("Provide at least one package name to bury".into());
            }
            // Build graveyard config — if rip2 is not available, fall back to plain remove
            match graveyard::GraveyardConfig::new() {
                Ok(gyard) => {
                    // Group by source
                    let mut by_source: std::collections::HashMap<String, Vec<String>> =
                        std::collections::HashMap::new();
                    let mut default_pkgs = Vec::new();
                    for pkg in &packages {
                        let (source, name) = parse_source_prefix(pkg);
                        if let Some(src) = source {
                            by_source.entry(src.to_string()).or_default().push(name.to_string());
                        } else {
                            default_pkgs.push(name.to_string());
                        }
                    }
                    // Resolve managers for source-specific packages
                    let mut all_pkgs_with_pm: Vec<(String, &dyn PackageManager)> = Vec::new();
                    for (source, src_pkgs) in &by_source {
                        if let Ok(pm) = choose_primary_manager(&available_managers, Some(source)) {
                            for pkg in src_pkgs {
                                all_pkgs_with_pm.push((pkg.clone(), pm));
                            }
                        }
                    }
                    if !default_pkgs.is_empty() {
                        let pm = choose_primary_manager(&available_managers, requested_manager)?;
                        for pkg in &default_pkgs {
                            all_pkgs_with_pm.push((pkg.clone(), pm));
                        }
                    }
                    tui::run_burial_tui(&gyard, &available_managers, &packages, requested_manager).await?;
                }
                Err(_) => {
                    // rip2 not installed — fall back to plain remove with a notice
                    eprintln!("Note: rip2 is not installed; using plain removal (no graveyard). Install with: cargo install rm-improved");
                    let mut by_source: std::collections::HashMap<String, Vec<String>> =
                        std::collections::HashMap::new();
                    let mut default_pkgs = Vec::new();
                    for pkg in &packages {
                        let (source, name) = parse_source_prefix(pkg);
                        if let Some(src) = source {
                            by_source.entry(src.to_string()).or_default().push(name.to_string());
                        } else {
                            default_pkgs.push(name.to_string());
                        }
                    }
                    for (source, src_pkgs) in &by_source {
                        if let Ok(pm) = choose_primary_manager(&available_managers, Some(source)) {
                            tui::run_progress_tui(pm, src_pkgs.clone(), "remove").await?;
                        }
                    }
                    if !default_pkgs.is_empty() {
                        let pm = choose_primary_manager(&available_managers, requested_manager)?;
                        tui::run_progress_tui(pm, default_pkgs, "remove").await?;
                    }
                }
            }
        }

        Some(Commands::Xum { packages }) => {
            match graveyard::GraveyardConfig::new() {
                Ok(gyard) => {
                    if packages.is_empty() {
                        tui::run_graveyard_tui(&gyard, &available_managers).await?;
                    } else {
                        tui::run_exhume_tui(&gyard, &available_managers, &packages).await?;
                    }
                }
                Err(e) => {
                    return Err(format!("Cannot open graveyard: {}", e).into());
                }
            }
        }

        Some(Commands::Shw) => {
            tui::run_installed_view_tui(available_managers).await?;
        }

        Some(Commands::Upd8) => {
            let pm = if let Some(requested) = requested_manager {
                choose_primary_manager(&available_managers, Some(requested))?
            } else if let Some(topgrade) =
                available_managers.iter().find(|m| m.name() == "topgrade")
            {
                topgrade.as_ref()
            } else {
                choose_primary_manager(&available_managers, None)?
            };
            tui::run_progress_tui(pm, vec![], "update").await?;
        }
    }

    Ok(())
}
