//! Lilith Linux Package Manager
//!
//! Reads the `packages.toml` manifest from
//! `https://blancobam.github.io/lilith-packages/packages.toml`
//! and provides search / install / remove for all listed packages.
//!
//! The manifest is cached at `$XDG_CONFIG_HOME/s8n/lilith-cache.toml`
//! (or `~/.config/s8n/lilith-cache.toml`) and refreshed every hour.
//!
//! Install methods:
//!   cargo    → cargo install <crate_name>
//!   deb      → download URL, sudo dpkg -i
//!   github   → GitHub API latest-release → download matching asset
//!   tarball  → download tarball, extract binary, install to ~/.local/bin
//!   binary   → download binary, chmod +x, install to ~/.local/bin
//!   appimage → download AppImage, chmod +x, symlink from ~/.local/bin
//!   script   → curl | sh
//!   git      → git clone --depth 1, run install_cmd
//!   npm      → npm install -g <npm_name>
//!   pip      → pip3 install <pip_name>
//!   apt      → sudo apt install -y <apt_name>
//!   flatpak  → flatpak install -y flathub <flatpak_id>

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::Deserialize;
use tokio::process::Command;
use which::which;

use crate::pm::{run_command_captured, run_command_quiet, PackageInfo, PackageManager, PmResult};

// ─── Constants ────────────────────────────────────────────────────────────────

const MANIFEST_URL: &str =
    "https://blancobam.github.io/lilith-packages/packages.toml";
/// Cache lifetime: 1 hour.
const CACHE_TTL_SECS: u64 = 3_600;

// ─── Manifest types ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Clone)]
pub struct LilithManifest {
    #[serde(default)]
    pub packages: Vec<LilithPackage>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct LilithPackage {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "ver_latest")]
    pub version: String,
    #[serde(default = "cat_xtra")]
    pub category: String,
    /// Install method string.
    #[serde(default)]
    pub install: String,

    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub github: String,
    #[serde(default)]
    pub asset: String,
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub crate_name: String,
    #[serde(default)]
    pub npm_name: String,
    #[serde(default)]
    pub pip_name: String,
    #[serde(default)]
    pub apt_name: String,
    #[serde(default)]
    pub flatpak_id: String,
    #[serde(default)]
    pub install_cmd: String,
}

fn ver_latest() -> String { "latest".into() }
fn cat_xtra() -> String { "xtra".into() }

// ─── Path helpers (no `dirs` dep) ────────────────────────────────────────────

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            PathBuf::from(home).join(".config")
        })
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
}

// ─── LilithPm ─────────────────────────────────────────────────────────────────

/// Stateless package manager that fetches its package list from GitHub Pages.
pub struct LilithPm;

impl Default for LilithPm {
    fn default() -> Self { Self::new() }
}

impl LilithPm {
    pub fn new() -> Self { Self }

    // ── Cache management ──────────────────────────────────────────────────────

    fn cache_path() -> PathBuf {
        config_dir().join("s8n").join("lilith-cache.toml")
    }

    fn cache_is_fresh() -> bool {
        let path = Self::cache_path();
        if !path.exists() { return false; }
        let mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now.saturating_sub(mtime) < CACHE_TTL_SECS
    }

    /// Fetch or read the cached manifest. Returns None on failure.
    async fn get_manifest() -> Option<LilithManifest> {
        let cache_path = Self::cache_path();

        // Serve from file cache if fresh.
        if Self::cache_is_fresh() {
            if let Ok(content) = std::fs::read_to_string(&cache_path) {
                if let Ok(m) = toml::from_str::<LilithManifest>(&content) {
                    return Some(m);
                }
            }
        }

        // Fetch fresh from GitHub Pages.
        let mut cmd = Command::new("curl");
        cmd.args([
            "-fsSL",
            "--max-time", "20",
            "--user-agent", "s8n-package-manager/0.4",
            MANIFEST_URL,
        ]);
        if let Ok(content) = run_command_captured(&mut cmd).await {
            if let Ok(manifest) = toml::from_str::<LilithManifest>(&content) {
                // Persist to cache.
                if let Some(parent) = cache_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&cache_path, &content);
                return Some(manifest);
            }
        }

        // Fallback: stale cache is better than nothing.
        if let Ok(content) = std::fs::read_to_string(&cache_path) {
            if let Ok(m) = toml::from_str::<LilithManifest>(&content) {
                return Some(m);
            }
        }

        None
    }

    // ── Conversion ────────────────────────────────────────────────────────────

    fn to_info(pkg: &LilithPackage) -> PackageInfo {
        let bin = if !pkg.binary.is_empty() { &pkg.binary } else { &pkg.name };
        PackageInfo {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            description: pkg.description.clone(),
            source: "lilith".into(),
            installed: which(bin).is_ok(),
        }
    }

    // ── GitHub release resolver ───────────────────────────────────────────────

    async fn resolve_github_url(pkg: &LilithPackage) -> Option<String> {
        if pkg.github.is_empty() || pkg.asset.is_empty() {
            return None;
        }
        let api_url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            pkg.github
        );
        let mut cmd = Command::new("curl");
        cmd.args([
            "-fsSL", "--max-time", "15",
            "-H", "Accept: application/vnd.github+json",
            "-H", "X-GitHub-Api-Version: 2022-11-28",
            &api_url,
        ]);
        let json = run_command_captured(&mut cmd).await.ok()?;
        let pattern = pkg.asset.to_lowercase();

        // Lightweight scan for matching browser_download_url lines.
        for line in json.lines() {
            let lower = line.to_lowercase();
            if lower.contains("browser_download_url") && lower.contains(&pattern) {
                if let Some(start) = line.find("https://") {
                    if let Some(rel_end) = line[start..].find('"') {
                        return Some(line[start..start + rel_end].to_string());
                    }
                }
            }
        }
        None
    }

    // ── Install dispatcher ────────────────────────────────────────────────────

    async fn install_one(pkg: &LilithPackage) -> PmResult {
        match pkg.install.as_str() {
            "cargo" => {
                let krate = if !pkg.crate_name.is_empty() { &pkg.crate_name } else { &pkg.name };
                let mut cmd = Command::new("cargo");
                cmd.args(["install", krate]);
                run_command_quiet(&mut cmd).await
            }
            "apt" => {
                let apt = if !pkg.apt_name.is_empty() { &pkg.apt_name } else { &pkg.name };
                let mut cmd = Command::new("sudo");
                cmd.args(["apt", "install", "-y", apt]);
                run_command_quiet(&mut cmd).await
            }
            "flatpak" => {
                let id = if !pkg.flatpak_id.is_empty() { &pkg.flatpak_id } else { &pkg.name };
                let mut cmd = Command::new("flatpak");
                cmd.args(["install", "-y", "flathub", id]);
                run_command_quiet(&mut cmd).await
            }
            "npm" => {
                let npm = if !pkg.npm_name.is_empty() { &pkg.npm_name } else { &pkg.name };
                let mut cmd = Command::new("npm");
                cmd.args(["install", "-g", npm]);
                run_command_quiet(&mut cmd).await
            }
            "pip" => {
                let pip = if !pkg.pip_name.is_empty() { &pkg.pip_name } else { &pkg.name };
                let mut cmd = Command::new("pip3");
                cmd.args(["install", "--user", pip]);
                run_command_quiet(&mut cmd).await
            }
            "deb" => {
                let url = if !pkg.url.is_empty() {
                    pkg.url.clone()
                } else {
                    match Self::resolve_github_url(pkg).await {
                        Some(u) => u,
                        None => return PmResult::CommandFailed(
                            -1, format!("No .deb URL for '{}'", pkg.name)),
                    }
                };
                let script = format!(
                    "TMP=$(mktemp --suffix=.deb) && \
                     curl -fsSL -o \"$TMP\" '{url}' && \
                     sudo dpkg -i \"$TMP\"; \
                     rm -f \"$TMP\""
                );
                let mut cmd = Command::new("bash");
                cmd.args(["-c", &script]);
                run_command_quiet(&mut cmd).await
            }
            "script" => {
                if pkg.url.is_empty() {
                    return PmResult::CommandFailed(
                        -1, format!("No installer URL for '{}'", pkg.name));
                }
                let script = format!("curl -fsSL '{}' | sh", pkg.url);
                let mut cmd = Command::new("bash");
                cmd.args(["-c", &script]);
                run_command_quiet(&mut cmd).await
            }
            "git" => {
                let clone_url = if !pkg.url.is_empty() {
                    pkg.url.clone()
                } else if !pkg.github.is_empty() {
                    format!("https://github.com/{}.git", pkg.github)
                } else {
                    return PmResult::CommandFailed(
                        -1, format!("No git URL for '{}'", pkg.name));
                };
                let safe = pkg.name.replace(['/', ' '], "-");
                let tmp = format!("/tmp/lilith-install-{safe}");
                let run_cmd = if pkg.install_cmd.is_empty() {
                    "bash install.sh".into()
                } else {
                    pkg.install_cmd.clone()
                };
                let script = format!(
                    "rm -rf '{tmp}' && \
                     git clone --depth 1 '{clone_url}' '{tmp}' && \
                     cd '{tmp}' && {run_cmd}; \
                     rm -rf '{tmp}'"
                );
                let mut cmd = Command::new("bash");
                cmd.args(["-c", &script]);
                run_command_quiet(&mut cmd).await
            }
            method @ ("github" | "binary" | "tarball" | "appimage") => {
                let url = if !pkg.url.is_empty() {
                    pkg.url.clone()
                } else {
                    match Self::resolve_github_url(pkg).await {
                        Some(u) => u,
                        None => return PmResult::CommandFailed(
                            -1, format!("No download URL for '{}'", pkg.name)),
                    }
                };
                Self::install_from_url(pkg, &url, method).await
            }
            other => PmResult::CommandFailed(
                -1, format!("Unknown install method '{}' for '{}'", other, pkg.name)),
        }
    }

    async fn install_from_url(pkg: &LilithPackage, url: &str, method: &str) -> PmResult {
        let bin_dir = home_dir().join(".local").join("bin");
        let _ = std::fs::create_dir_all(&bin_dir);
        let bin_dir_s = bin_dir.to_string_lossy();

        let bin_name = if !pkg.binary.is_empty() {
            pkg.binary.clone()
        } else {
            pkg.name.clone()
        };

        let script = match method {
            "binary" => format!(
                "curl -fsSL -o '{bin_dir_s}/{bin_name}' '{url}' && \
                 chmod +x '{bin_dir_s}/{bin_name}'"
            ),
            "tarball" => {
                let decomp = if url.ends_with(".tar.xz") || url.ends_with(".txz") {
                    "tar -xJf"
                } else if url.ends_with(".tar.bz2") {
                    "tar -xjf"
                } else {
                    "tar -xzf"
                };
                format!(
                    "TMP=$(mktemp -d) && \
                     curl -fsSL -o \"$TMP/archive\" '{url}' && \
                     {decomp} \"$TMP/archive\" -C \"$TMP\" 2>/dev/null; \
                     FOUND=$(find \"$TMP\" -type f -name '{bin_name}' | head -1); \
                     if [ -z \"$FOUND\" ]; then \
                       FOUND=$(find \"$TMP\" -type f -perm /111 ! -name '*.so' ! -name '*.dylib' | head -1); \
                     fi; \
                     [ -n \"$FOUND\" ] && install -m755 \"$FOUND\" '{bin_dir_s}/{bin_name}'; \
                     rm -rf \"$TMP\""
                )
            }
            "appimage" => {
                let opt = format!("/opt/lilith/{bin_name}.AppImage");
                format!(
                    "sudo mkdir -p /opt/lilith && \
                     sudo curl -fsSL -o '{opt}' '{url}' && \
                     sudo chmod +x '{opt}' && \
                     ln -sf '{opt}' '{bin_dir_s}/{bin_name}'"
                )
            }
            // github — smart: try as binary first, then tarball
            _ => format!(
                "TMP=$(mktemp -d) && \
                 curl -fsSL -o \"$TMP/asset\" '{url}'; \
                 if file \"$TMP/asset\" 2>/dev/null | grep -q 'ELF'; then \
                   install -m755 \"$TMP/asset\" '{bin_dir_s}/{bin_name}'; \
                 else \
                   tar -xf \"$TMP/asset\" -C \"$TMP\" 2>/dev/null; \
                   FOUND=$(find \"$TMP\" -type f -name '{bin_name}' | head -1); \
                   [ -z \"$FOUND\" ] && FOUND=$(find \"$TMP\" -type f -perm /111 ! -name '*.so' ! -name '*.dylib' | head -1); \
                   [ -n \"$FOUND\" ] && install -m755 \"$FOUND\" '{bin_dir_s}/{bin_name}'; \
                 fi; \
                 rm -rf \"$TMP\""
            ),
        };

        let mut cmd = Command::new("bash");
        cmd.args(["-c", &script]);
        run_command_quiet(&mut cmd).await
    }

    // ── Remove dispatcher ─────────────────────────────────────────────────────

    async fn remove_one(pkg: &LilithPackage) -> PmResult {
        match pkg.install.as_str() {
            "cargo" => {
                let krate = if !pkg.crate_name.is_empty() { &pkg.crate_name } else { &pkg.name };
                let mut cmd = Command::new("cargo");
                cmd.args(["uninstall", krate]);
                run_command_quiet(&mut cmd).await
            }
            "apt" => {
                let apt = if !pkg.apt_name.is_empty() { &pkg.apt_name } else { &pkg.name };
                let mut cmd = Command::new("sudo");
                cmd.args(["apt", "remove", "-y", apt]);
                run_command_quiet(&mut cmd).await
            }
            "flatpak" => {
                let id = if !pkg.flatpak_id.is_empty() { &pkg.flatpak_id } else { &pkg.name };
                let mut cmd = Command::new("flatpak");
                cmd.args(["remove", "-y", id]);
                run_command_quiet(&mut cmd).await
            }
            "npm" => {
                let npm = if !pkg.npm_name.is_empty() { &pkg.npm_name } else { &pkg.name };
                let mut cmd = Command::new("npm");
                cmd.args(["uninstall", "-g", npm]);
                run_command_quiet(&mut cmd).await
            }
            "pip" => {
                let pip = if !pkg.pip_name.is_empty() { &pkg.pip_name } else { &pkg.name };
                let mut cmd = Command::new("pip3");
                cmd.args(["uninstall", "-y", pip]);
                run_command_quiet(&mut cmd).await
            }
            "deb" => {
                let pkg_name = if !pkg.apt_name.is_empty() {
                    pkg.apt_name.clone()
                } else {
                    pkg.name.clone()
                };
                let mut cmd = Command::new("sudo");
                cmd.args(["dpkg", "--remove", &pkg_name]);
                run_command_quiet(&mut cmd).await
            }
            _ => {
                // Remove the binary from PATH and any AppImage in /opt/lilith
                let bin = if !pkg.binary.is_empty() { &pkg.binary } else { &pkg.name };
                if let Ok(path) = which(bin) {
                    let path_s = path.to_string_lossy().to_string();
                    let appimage = format!("/opt/lilith/{bin}.AppImage");
                    let script =
                        format!("rm -f '{path_s}' '{appimage}' 2>/dev/null; true");
                    let mut cmd = Command::new("bash");
                    cmd.args(["-c", &script]);
                    run_command_quiet(&mut cmd).await
                } else {
                    PmResult::CommandFailed(
                        -1,
                        format!("Binary '{}' not found in PATH", bin),
                    )
                }
            }
        }
    }
}

// ─── PackageManager impl ──────────────────────────────────────────────────────

#[async_trait]
impl PackageManager for LilithPm {
    fn name(&self) -> &str { "lilith" }

    /// Always available — we use `curl` (universally available) to fetch the manifest.
    fn is_available(&self) -> bool { true }

    /// Interactive search: fetch manifest, print matching packages to stdout.
    async fn search(&self, query: &str) -> PmResult {
        match Self::get_manifest().await {
            None => {
                eprintln!("[lilith] Failed to fetch package manifest from {MANIFEST_URL}");
                PmResult::CommandFailed(-1, "Manifest fetch failed".into())
            }
            Some(manifest) => {
                let q = query.to_lowercase();
                let hits: Vec<_> = manifest
                    .packages
                    .iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&q)
                            || p.description.to_lowercase().contains(&q)
                    })
                    .collect();

                if hits.is_empty() {
                    println!("[lilith] No packages matching '{query}'");
                } else {
                    println!("[lilith] Packages matching '{query}':");
                    for pkg in &hits {
                        let status = if which(if !pkg.binary.is_empty() {
                            &pkg.binary
                        } else {
                            &pkg.name
                        })
                        .is_ok()
                        {
                            "installed"
                        } else {
                            "available"
                        };
                        println!(
                            "  {:30} {:10} {:8} {}",
                            pkg.name, pkg.version, status, pkg.description
                        );
                    }
                }
                PmResult::Success
            }
        }
    }

    async fn search_captured(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        let manifest = Self::get_manifest()
            .await
            .ok_or_else(|| "Failed to fetch Lilith package manifest".to_string())?;

        let q = query.to_lowercase();
        let results = manifest
            .packages
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.description.to_lowercase().contains(&q)
                    || p.category.to_lowercase().contains(&q)
            })
            .map(Self::to_info)
            .collect();

        Ok(results)
    }

    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String> {
        let manifest = Self::get_manifest()
            .await
            .ok_or_else(|| "Failed to fetch Lilith package manifest".to_string())?;

        let installed = manifest
            .packages
            .iter()
            .map(Self::to_info)
            .filter(|p| p.installed)
            .collect();

        Ok(installed)
    }

    /// Install each named package from the Lilith manifest.
    async fn install(&self, packages: &[String]) -> PmResult {
        let manifest = match Self::get_manifest().await {
            Some(m) => m,
            None => return PmResult::CommandFailed(
                -1, "Failed to fetch Lilith package manifest".into()),
        };

        for name in packages {
            let pkg = match manifest.packages.iter().find(|p| p.name == name.as_str()) {
                Some(p) => p.clone(),
                None => {
                    eprintln!("[lilith] Package '{name}' not found in manifest");
                    return PmResult::CommandFailed(
                        -1,
                        format!("Package '{name}' not found in Lilith repository"),
                    );
                }
            };
            match Self::install_one(&pkg).await {
                PmResult::Success => {}
                err => return err,
            }
        }
        PmResult::Success
    }

    /// Remove each named package.
    async fn remove(&self, packages: &[String]) -> PmResult {
        let manifest = match Self::get_manifest().await {
            Some(m) => m,
            None => return PmResult::CommandFailed(
                -1, "Failed to fetch Lilith package manifest".into()),
        };

        for name in packages {
            let pkg = match manifest.packages.iter().find(|p| p.name == name.as_str()) {
                Some(p) => p.clone(),
                None => {
                    // Best-effort: remove binary by name even if not in manifest.
                    if let Ok(path) = which(name.as_str()) {
                        let _ = std::fs::remove_file(&path);
                        continue;
                    }
                    return PmResult::CommandFailed(
                        -1,
                        format!("Package '{name}' not found in Lilith repository"),
                    );
                }
            };
            match Self::remove_one(&pkg).await {
                PmResult::Success => {}
                err => return err,
            }
        }
        PmResult::Success
    }

    /// Force-refresh the manifest cache on next access.
    async fn update(&self) -> PmResult {
        let cache = Self::cache_path();
        if cache.exists() {
            let _ = std::fs::remove_file(&cache);
        }
        PmResult::Success
    }
}
