use crate::pm::{builtin::get_default_managers, PackageManager};
use clap::{Parser, Subcommand};

pub mod config;
pub mod graveyard;
pub mod pm;
pub mod tui;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "S8n — a unified system and package manager TUI for Lilith Linux",
    long_about = "S8n wraps your installed package managers (apt, flatpak, snap, cargo, am, \
                  brew, soar, pacstall, and more) into a single beautiful TUI and set of \
                  short commands. Run `s8n` with no arguments to launch the interactive interface.\n\n\
                  PACKAGE SYNTAX\n\
                  \x20 Plain name:          s8n stall firefox\n\
                  \x20 Source-prefixed:     s8n stall apt:firefox  flatpak:org.mozilla.firefox\n\
                  \x20 Multiple at once:    s8n stall vim neovim cargo:ripgrep"
)]
struct Cli {
    /// Restrict to a specific package manager (e.g. apt, flatpak, snap, cargo, brew)
    #[arg(long, short = 'm', value_name = "PM")]
    manager: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for a package across all installed package managers and their repositories.
    ///
    /// Wraps the native search commands (apt search, cargo search, flatpak search, etc.)
    /// and displays results in an interactive table inside the TUI.
    /// Optionally filter to one manager with -m / --manager.
    ///
    /// EXAMPLES
    ///   s8n srch firefox
    ///   s8n srch ripgrep
    ///   s8n search -m cargo serde
    #[command(name = "srch", alias = "search", alias = "find")]
    Search {
        /// Package name or keyword to search for
        query: Option<String>,
    },

    /// Install one or more packages via the appropriate package manager.
    ///
    /// Accepts plain names (uses the system default / highest-priority PM) or
    /// source-prefixed names to target a specific manager.
    ///
    /// EXAMPLES
    ///   s8n stall firefox
    ///   s8n stall apt:vim flatpak:org.gimp.GIMP cargo:ripgrep
    ///   s8n stall https://example.com/app.tar.gz   # URL install via soar
    #[command(name = "stall", alias = "install")]
    Stall {
        /// Package(s) to install — plain names or source:name pairs
        packages: Vec<String>,
    },

    /// Remove a package permanently (no recovery).
    ///
    /// Use `s8n brn` instead if you want to be able to recover the package later.
    ///
    /// EXAMPLES
    ///   s8n burn vim
    ///   s8n burn apt:vim flatpak:org.gimp.GIMP
    #[command(name = "burn", alias = "remove", alias = "uninstall")]
    Burn {
        /// Package(s) to remove
        packages: Vec<String>,
    },

    /// Bury (safely remove) a package into the S8n graveyard for later recovery.
    ///
    /// Uses rip2 under the hood. If rip2 is not installed, falls back to a plain
    /// uninstall. Buried packages are stored in ~/.local/share/graveyard/s8n/ and
    /// can be recovered at any time with `s8n xum`.
    ///
    /// EXAMPLES
    ///   s8n brn vim
    ///   s8n brn apt:vim cargo:ripgrep
    #[command(name = "brn", alias = "bury")]
    Brn {
        /// Package(s) to bury — plain names or source:name pairs
        packages: Vec<String>,
    },

    /// Exhume (recover) packages previously buried with `s8n brn`.
    ///
    /// With no arguments, opens an interactive graveyard browser where you can
    /// browse and restore any buried package. With package names as arguments,
    /// restores those specific packages immediately.
    ///
    /// EXAMPLES
    ///   s8n xum              # open interactive graveyard browser
    ///   s8n xum vim          # recover vim directly
    ///   s8n xum vim ripgrep  # recover multiple packages
    #[command(name = "xum", alias = "recover", alias = "restore")]
    Xum {
        /// Package name(s) to recover (leave empty for interactive browser)
        packages: Vec<String>,
    },

    /// Show all packages installed on this system across all tracked package managers.
    ///
    /// Lists packages from apt, flatpak, snap, cargo, brew, am, and any other
    /// managers detected on the system, displayed in the S8n TUI.
    ///
    /// EXAMPLES
    ///   s8n shw
    ///   s8n shw -m flatpak   # limit to flatpak packages
    #[command(name = "shw", alias = "list", alias = "installed")]
    Shw,

    /// Update and upgrade all packages from all sources via topgrade.
    ///
    /// Runs topgrade, which calls each installed package manager's update/upgrade
    /// command in sequence (apt upgrade, flatpak update, cargo install-update, etc.).
    ///
    /// EXAMPLES
    ///   s8n upd8
    #[command(name = "upd8", alias = "update", alias = "upgrade")]
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
            "apt",
            "pacstall",
            "flatpak",
            "snap",
            "brew",
            "soar",
            "npm",
            "bun",
            "pip",
            "cargo",
            "cargo-binstall",
            "am",
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
                        eprintln!(
                            "URL installs require the 'soar' backend. Re-run with `--manager soar`."
                        );
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
                    by_source
                        .entry(src.to_string())
                        .or_default()
                        .push(name.to_string());
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
                            by_source
                                .entry(src.to_string())
                                .or_default()
                                .push(name.to_string());
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
                    tui::run_burial_tui(&gyard, &available_managers, &packages, requested_manager)
                        .await?;
                }
                Err(_) => {
                    // rip2 not installed — fall back to plain remove with a notice
                    eprintln!(
                        "Note: rip2 is not installed; using plain removal (no graveyard). \
                         Install with: cargo install rm-improved"
                    );
                    let mut by_source: std::collections::HashMap<String, Vec<String>> =
                        std::collections::HashMap::new();
                    let mut default_pkgs = Vec::new();
                    for pkg in &packages {
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

        Some(Commands::Xum { packages }) => match graveyard::GraveyardConfig::new() {
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
        },

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
