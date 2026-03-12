use super::{PackageManager, PmResult, run_command_interactive};
use async_trait::async_trait;
use tokio::process::Command;
use which::which;

/// Generic structural wrapper for package managers where we just pass subcommands
pub struct GenericWrapper {
    /// The name of the package manager, e.g. "apt"
    pub name: String,
    /// The binary executable, e.g. "apt-get"
    pub binary: String,
    
    pub search_cmd: Vec<String>,
    pub install_cmd: Vec<String>,
    pub remove_cmd: Vec<String>,
    pub update_cmd: Vec<String>,
}

#[async_trait]
impl PackageManager for GenericWrapper {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        which(&self.binary).is_ok()
    }

    async fn search(&self, query: &str) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        for arg in &self.search_cmd {
            cmd.arg(arg);
        }
        cmd.arg(query);
        run_command_interactive(&mut cmd).await
    }

    async fn install(&self, packages: &[String]) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        for arg in &self.install_cmd {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_interactive(&mut cmd).await
    }

    async fn remove(&self, packages: &[String]) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        for arg in &self.remove_cmd {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_interactive(&mut cmd).await
    }

    async fn update(&self) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        for arg in &self.update_cmd {
            cmd.arg(arg);
        }
        run_command_interactive(&mut cmd).await
    }
}

pub fn get_default_managers() -> Vec<Box<dyn PackageManager>> {
    vec![
        Box::new(GenericWrapper {
            name: "apt".into(),
            binary: "apt".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into(), "-y".into()],
            remove_cmd: vec!["remove".into(), "-y".into()],
            update_cmd: vec!["upgrade".into(), "-y".into()],
        }),
        Box::new(GenericWrapper {
            name: "pacstall".into(),
            binary: "pacstall".into(),
            search_cmd: vec!["-S".into()], // Search in pacstall
            install_cmd: vec!["-I".into()],
            remove_cmd: vec!["-R".into()],
            update_cmd: vec!["-U".into(), "all".into()], // Update all
        }),
        Box::new(GenericWrapper {
            name: "flatpak".into(),
            binary: "flatpak".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into(), "-y".into()],
            remove_cmd: vec!["uninstall".into(), "-y".into()],
            update_cmd: vec!["update".into(), "-y".into()],
        }),
        Box::new(GenericWrapper {
            name: "snap".into(),
            binary: "snap".into(),
            search_cmd: vec!["find".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["remove".into()],
            update_cmd: vec!["refresh".into()],
        }),
        Box::new(GenericWrapper {
            name: "brew".into(),
            binary: "brew".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["uninstall".into()],
            update_cmd: vec!["upgrade".into()], // Update packages
        }),
        Box::new(GenericWrapper {
            name: "soar".into(), // pkgforge soar
            binary: "soar".into(),
            search_cmd: vec!["list".into()], // Equivalent search command?
            install_cmd: vec!["add".into()], // Supports URLs/appimages usually
            remove_cmd: vec!["remove".into()],
            update_cmd: vec!["update".into()],
        }),
        Box::new(GenericWrapper {
            name: "npm".into(),
            binary: "npm".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into(), "-g".into()], // Usually want global installs for a system wrapper
            remove_cmd: vec!["uninstall".into(), "-g".into()],
            update_cmd: vec!["update".into(), "-g".into()],
        }),
        Box::new(GenericWrapper {
            name: "bun".into(),
            binary: "bun".into(),
            search_cmd: vec![], // bun doesn't natively have a search command
            install_cmd: vec!["add".into(), "-g".into()],
            remove_cmd: vec!["remove".into(), "-g".into()],
            update_cmd: vec!["update".into(), "-g".into()],
        }),
        Box::new(GenericWrapper {
            name: "pip".into(), // Handles pypi
            binary: "pip".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["uninstall".into(), "-y".into()],
            update_cmd: vec!["install".into(), "--upgrade".into()], // pip doesn't have a simple "update all" that's standard without listing
        }),
        Box::new(GenericWrapper {
            name: "topgrade".into(),
            binary: "topgrade".into(),
            search_cmd: vec![], // Topgrade doesn't search
            install_cmd: vec![], // Topgrade doesn't install
            remove_cmd: vec![], // Topgrade doesn't remove
            update_cmd: vec!["--yes".into()], // Run all upgrades quietly
        }),
    ]
}
