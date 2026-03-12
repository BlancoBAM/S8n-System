use clap::{Parser, Subcommand};
use s8n::{
    pm::{builtin::get_default_managers, PackageManager, PmResult},
    ui::{run_tui, UIOperation},
};

pub mod config;
pub mod pm;
pub mod ui;

/// s8n - Universal Package Manager Wrapper
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for packages across connected repositories
    Search {
        /// The package name to search for
        query: String,
    },
    /// Install packages
    Stall {
        /// Packages or URLs to install
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Delete or remove packages
    Burn {
        /// Packages to remove
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Update all installed packages (including topgrade)
    Upd8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Load config and available package managers
    // let _cfg = config::load_config().await?;
    let managers = get_default_managers();
    let mut available_managers: Vec<_> = managers.into_iter().filter(|m| m.is_available()).collect();

    // The primary system manager (apt or pacstall or flatpak etc.)
    // We try to find first non-specific one like apt or pacstall
    let primary_pm = available_managers.iter().find(|m| {
        let name = m.name();
        name == "apt" || name == "pacstall" || name == "brew"
    }).unwrap_or_else(|| available_managers.first().expect("No package managers found"));

    match cli.command {
        Commands::Search { query } => {
            println!("Searching for '{}' across connected repositories...", query);
            for pm in &available_managers {
                // Not all have search (e.g. topgrade, bun)
                if pm.name() == "topgrade" || pm.name() == "bun" { continue; }
                println!("--- Results from {} ---", pm.name());
                let _ = pm.search(&query).await;
                println!();
            }
        }
        Commands::Stall { packages } => {
            // Check if packages look like URLs (e.g. github links) -> send to soar
            // For now, route all to primary_pm if no direct URL, else soar
            let (urls, pkgs): (Vec<String>, Vec<String>) = packages.into_iter().partition(|p| p.starts_with("http://") || p.starts_with("https://"));

            if !pkgs.is_empty() {
                run_tui(primary_pm, pkgs, UIOperation::Install).await?;
            }
            if !urls.is_empty() {
                if let Some(soar) = available_managers.iter().find(|m| m.name() == "soar") {
                    run_tui(soar, urls, UIOperation::Install).await?;
                } else {
                    eprintln!("Warning: URL packages provided but 'soar' is not available.");
                }
            }
        }
        Commands::Burn { packages } => {
            run_tui(primary_pm, packages, UIOperation::Remove).await?;
        }
        Commands::Upd8 => {
            // If topgrade is available, run it. Otherwise run primary update
            if let Some(topgrade) = available_managers.iter().find(|m| m.name() == "topgrade") {
                run_tui(topgrade, vec![], UIOperation::Update).await?;
            } else {
                run_tui(primary_pm, vec![], UIOperation::Update).await?;
            }
        }
    }

    Ok(())
}
