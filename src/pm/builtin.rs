use super::{
    run_command_captured, run_command_interactive, PackageInfo, PackageManager,
    PmResult,
};
use async_trait::async_trait;
use tokio::process::Command;
use which::which;

/// Generic structural wrapper for package managers where we just pass subcommands
pub struct GenericWrapper {
    pub name: String,
    pub binary: String,
    pub search_cmd: Vec<String>,
    pub install_cmd: Vec<String>,
    pub remove_cmd: Vec<String>,
    pub update_cmd: Vec<String>,
    pub list_cmd: Vec<String>,
    /// If true, install/remove/update commands are wrapped with `sudo`
    pub needs_sudo: bool,
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

    async fn search_captured(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        if self.search_cmd.is_empty() {
            return Ok(vec![]);
        }
        let mut cmd = Command::new(&self.binary);
        for arg in &self.search_cmd {
            cmd.arg(arg);
        }
        cmd.arg(query);
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_search_output(&self.name, &output))
    }

    async fn install(&self, packages: &[String]) -> PmResult {
        let mut cmd = if self.needs_sudo {
            let mut c = Command::new("sudo");
            c.arg(&self.binary);
            c
        } else {
            Command::new(&self.binary)
        };
        for arg in &self.install_cmd {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_interactive(&mut cmd).await
    }

    async fn remove(&self, packages: &[String]) -> PmResult {
        let mut cmd = if self.needs_sudo {
            let mut c = Command::new("sudo");
            c.arg(&self.binary);
            c
        } else {
            Command::new(&self.binary)
        };
        for arg in &self.remove_cmd {
            cmd.arg(arg);
        }
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_interactive(&mut cmd).await
    }

    async fn update(&self) -> PmResult {
        let mut cmd = if self.needs_sudo {
            let mut c = Command::new("sudo");
            c.arg(&self.binary);
            c
        } else {
            Command::new(&self.binary)
        };
        for arg in &self.update_cmd {
            cmd.arg(arg);
        }
        run_command_interactive(&mut cmd).await
    }

    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String> {
        if self.list_cmd.is_empty() {
            return Ok(vec![]);
        }
        let mut cmd = Command::new(&self.binary);
        for arg in &self.list_cmd {
            cmd.arg(arg);
        }
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_list_output(&self.name, &output))
    }
}

/// Parse search output from various package managers into structured PackageInfo
fn parse_search_output(source: &str, output: &str) -> Vec<PackageInfo> {
    let mut results = match source {
        "apt" => parse_apt_output(output, source),
        "flatpak" => parse_flatpak_output(output, source),
        "snap" => parse_snap_output(output, source),
        "brew" => parse_brew_output(output, source),
        "npm" => parse_npm_output(output, source),
        "pip" => parse_pip_output(output, source),
        "pacstall" => parse_pacstall_output(output, source),
        "soar" => parse_soar_output(output, source),
        "cargo" => parse_cargo_search_output(output),
        "am" | "appman" => parse_am_output(output, source),
        _ => parse_generic_output(output, source),
    };
    // Global filter: remove blank-name entries produced by any parser
    results.retain(|p| !p.name.trim().is_empty() && p.name.chars().any(|c| c.is_alphanumeric()));
    results
}

/// Parse list-installed output from package managers into structured PackageInfo
fn parse_list_output(source: &str, output: &str) -> Vec<PackageInfo> {
    let mut results = match source {
        "apt" => parse_apt_list_output(output, source),
        "flatpak" => parse_flatpak_list_output(output, source),
        "snap" => parse_snap_list_output(output, source),
        "brew" => parse_brew_list_output(output, source),
        "npm" => parse_npm_list_output(output, source),
        "pip" => parse_pip_list_output(output, source),
        "pacstall" => parse_pacstall_list_output(output, source),
        "soar" => parse_soar_list_output(output, source),
        "cargo" => parse_cargo_list_output(output),
        "am" | "appman" => parse_am_list_output(output, source),
        _ => parse_generic_output(output, source),
    };
    results.retain(|p| !p.name.trim().is_empty() && p.name.chars().any(|c| c.is_alphanumeric()));
    results
}

/// soar list output: filter out metadata/box lines (total/installed/available)
fn parse_soar_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| {
            let t = l.trim();
            // Skip empty lines, box-drawing characters, metadata headers
            !t.is_empty()
                && !t.starts_with('┌')
                && !t.starts_with('│')
                && !t.starts_with('└')
                && !t.starts_with('├')
                && !t.starts_with('─')
                && !t.starts_with('+')
                && !t.starts_with('-')
                && !t.contains("Total")
                && !t.contains("Installed")
                && !t.contains("Available")
                && !t.contains("total")
                && !t.contains("installed")
                && !t.contains("available")
        })
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, ' ').collect();
            PackageInfo {
                name: parts[0].to_string(),
                version: parts.get(1).unwrap_or(&"").to_string(),
                description: String::new(),
                source: source.to_string(),
                installed: false,
            }
        })
        .collect()
}

/// apt search output: lines like "package/suite version arch\n  Description"
fn parse_apt_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line.starts_with("Sorting") || line.starts_with("Full Text") {
            i += 1;
            continue;
        }
        // apt search format: "name/suite version arch [installed,automatic]"
        if !line.starts_with(' ') && line.contains('/') {
            let parts: Vec<&str> = line.splitn(2, '/').collect();
            let name = parts[0].to_string();
            let rest = parts.get(1).unwrap_or(&"");
            let tokens: Vec<&str> = rest.split_whitespace().collect();
            let version = tokens.first().unwrap_or(&"").to_string();
            let installed = line.contains("[installed");

            let description = if i + 1 < lines.len() && lines[i + 1].starts_with("  ") {
                lines[i + 1].trim().to_string()
            } else {
                String::new()
            };

            results.push(PackageInfo {
                name,
                version,
                description,
                source: source.to_string(),
                installed,
            });
            i += 2; // skip description line
        } else {
            i += 1;
        }
    }
    results
}

/// flatpak search output: tab-separated "Name\tDescription\tApplication ID\tVersion\tBranch\tRemotes"
fn parse_flatpak_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for line in output.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let display_name = parts[0].trim();
            let description = parts.get(1).unwrap_or(&"").trim().to_string();
            let app_id = parts.get(2).unwrap_or(&"").trim().to_string();
            let version = parts.get(3).unwrap_or(&"").trim().to_string();

            results.push(PackageInfo {
                name: if app_id.is_empty() {
                    display_name.to_string()
                } else {
                    app_id
                },
                version,
                description,
                source: source.to_string(),
                installed: false,
            });
        }
    }
    results
}

/// snap find output: "Name  Version  Publisher  Notes  Summary"
fn parse_snap_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for (i, line) in output.lines().enumerate() {
        if i == 0 {
            continue;
        } // skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[0].to_string();
            let version = parts[1].to_string();
            // summary is everything after publisher + notes columns
            let description = if parts.len() > 4 {
                parts[4..].join(" ")
            } else {
                String::new()
            };
            results.push(PackageInfo {
                name,
                version,
                description,
                source: source.to_string(),
                installed: false,
            });
        }
    }
    results
}

/// brew search output: just names, one per line
fn parse_brew_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("==>"))
        .map(|l| PackageInfo {
            name: l.trim().to_string(),
            version: String::new(),
            description: "Homebrew formula/cask".to_string(),
            source: source.to_string(),
            installed: false,
        })
        .collect()
}

/// npm search output (table format)
fn parse_npm_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for (i, line) in output.lines().enumerate() {
        if i == 0 {
            continue;
        } // skip header ("NAME | ...")
        let line = line.trim();
        if line.is_empty() || line.starts_with('|') {
            continue;
        }
        // npm search in table: "name  |  description  |  author  |  date  |  version  |  keywords"
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let name = parts[0].trim().to_string();
            let description = parts.get(1).unwrap_or(&"").trim().to_string();
            let version = parts.get(4).unwrap_or(&"").trim().to_string();
            if !name.is_empty() {
                results.push(PackageInfo {
                    name,
                    version,
                    description,
                    source: source.to_string(),
                    installed: false,
                });
            }
        }
    }
    results
}

/// pip search (deprecated, but parse if available) or pip index versions
fn parse_pip_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // pip search format: "name (version) - description"
        if let Some(paren_pos) = line.find('(') {
            if let Some(close) = line.find(')') {
                let name = line[..paren_pos].trim().to_string();
                let version = line[paren_pos + 1..close].to_string();
                let description = if line.len() > close + 3 {
                    line[close + 3..].to_string()
                } else {
                    String::new()
                };
                results.push(PackageInfo {
                    name,
                    version,
                    description,
                    source: source.to_string(),
                    installed: false,
                });
                continue;
            }
        }
        // Fallback: just a name
        results.push(PackageInfo {
            name: line.to_string(),
            version: String::new(),
            description: String::new(),
            source: source.to_string(),
            installed: false,
        });
    }
    results
}

/// pacstall -S output: lines like "pkgname" or "pkgname @ version" (@ marks installed)
fn parse_pacstall_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('['))
        .map(|l| {
            let trimmed = l.trim();
            // Format: "pkgname @ version" — @ is the installed marker separator in pacstall
            let (name, version, installed) = if let Some(at_pos) = trimmed.find(" @ ") {
                let name = trimmed[..at_pos].trim().to_string();
                let ver = trimmed[at_pos + 3..].trim().to_string();
                (name, ver, true)
            } else {
                // Just a package name with no version info
                let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
                let name = parts[0].to_string();
                let ver = parts
                    .get(1)
                    .unwrap_or(&"")
                    .trim()
                    .trim_start_matches('@')
                    .trim()
                    .to_string();
                (name, ver, false)
            };
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed,
            }
        })
        .collect()
}

/// Fallback: treat each line as a package name
fn parse_generic_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| PackageInfo {
            name: l.trim().to_string(),
            version: String::new(),
            description: String::new(),
            source: source.to_string(),
            installed: true,
        })
        .collect()
}

// ── List-installed parsers ───────────────────────────────────────────────────

fn parse_apt_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("Listing"))
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let version = parts.get(1).unwrap_or(&"").to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_flatpak_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("Name"))
        .map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            let name = parts.first().unwrap_or(&"").trim().to_string();
            let version = parts.get(3).unwrap_or(&"").trim().to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_snap_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .enumerate()
        .filter(|(i, l)| *i > 0 && !l.trim().is_empty())
        .map(|(_, l)| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let version = parts.get(1).unwrap_or(&"").to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_brew_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.contains("==>"))
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let version = parts.get(1).unwrap_or(&"").to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_npm_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with('/')
            || line.contains("@") && line.starts_with("├")
            || line.starts_with("└")
        {
            continue;
        }
        // npm ls output: package@version
        if let Some(at_pos) = line.find('@') {
            let name = line[..at_pos]
                .trim()
                .trim_start_matches("├── ")
                .trim_start_matches("└── ");
            let version = line[at_pos + 1..].trim();
            if !name.is_empty() && !name.contains('/') {
                results.push(PackageInfo {
                    name: name.to_string(),
                    version: version.to_string(),
                    description: String::new(),
                    source: source.to_string(),
                    installed: true,
                });
            }
        }
    }
    results
}

fn parse_pip_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("Package") && !l.starts_with("---"))
        .map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let version = parts.get(1).unwrap_or(&"").to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_pacstall_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('['))
        .map(|l| {
            let trimmed = l.trim();
            let parts: Vec<&str> = trimmed.splitn(2, char::is_whitespace).collect();
            let name = parts.first().unwrap_or(&"").to_string();
            let version = parts.get(1).unwrap_or(&"").trim().to_string();
            PackageInfo {
                name,
                version,
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

fn parse_soar_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('┌')
                && !t.starts_with('│')
                && !t.starts_with('└')
                && !t.starts_with('├')
                && !t.starts_with('─')
                && !t.starts_with('+')
                && !t.starts_with('-')
                && !t.contains("Total")
                && !t.contains("Installed")
                && !t.contains("Available")
        })
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, ' ').collect();
            PackageInfo {
                name: parts.first().unwrap_or(&"").to_string(),
                version: parts.get(1).unwrap_or(&"").to_string(),
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
}

// ── Cargo wrapper ───────────────────────────────────────────────────

/// Cargo wrapper — wraps `cargo install`, `cargo uninstall`, `cargo search`
pub struct CargoWrapper;

#[async_trait]
impl PackageManager for CargoWrapper {
    fn name(&self) -> &str {
        "cargo"
    }

    fn is_available(&self) -> bool {
        which("cargo").is_ok()
    }

    async fn search(&self, query: &str) -> PmResult {
        let mut cmd = Command::new("cargo");
        cmd.arg("search").arg(query);
        run_command_interactive(&mut cmd).await
    }

    async fn search_captured(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("search").arg(query).arg("--limit").arg("10");
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_cargo_search_output(&output))
    }

    async fn install(&self, packages: &[String]) -> PmResult {
        // Prefer cargo-binstall for binary installs if available
        if which("cargo-binstall").is_ok() {
            let mut cmd = Command::new("cargo-binstall");
            cmd.arg("--no-confirm");
            for pkg in packages {
                cmd.arg(pkg);
            }
            return run_command_quiet(&mut cmd).await;
        }
        let mut cmd = Command::new("cargo");
        cmd.arg("install");
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_quiet(&mut cmd).await
    }

    async fn remove(&self, packages: &[String]) -> PmResult {
        let mut cmd = Command::new("cargo");
        cmd.arg("uninstall");
        for pkg in packages {
            cmd.arg(pkg);
        }
        run_command_quiet(&mut cmd).await
    }

    async fn update(&self) -> PmResult {
        PmResult::Error("Use 'cargo install --force <pkg>' to update crates.".into())
    }

    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String> {
        let mut cmd = Command::new("cargo");
        cmd.arg("install").arg("--list");
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_cargo_list_output(&output))
    }
}

/// Parse `cargo search` output: lines like `name = "version" # description`
fn parse_cargo_search_output(output: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for line in output.lines() {
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') || !t.contains(" = \"") {
            continue;
        }
        // Format: `crate-name = "version"    # description`
        let Some(eq_pos) = t.find(" = \"") else { continue };
        let name = t[..eq_pos].trim();
        let rest = &t[eq_pos + 4..];
        let (version, description) = if let Some(hash_pos) = rest.find("\" # ") {
            (&rest[..hash_pos], &rest[hash_pos + 4..])
        } else if let Some(quote_pos) = rest.find('"') {
            (&rest[..quote_pos], "")
        } else {
            (rest, "")
        };
        if name.is_empty() { continue; }
        results.push(PackageInfo {
            name: name.to_string(),
            version: version.trim().to_string(),
            description: description.trim().to_string(),
            source: "cargo".to_string(),
            installed: false,
        });
    }
    results
}

/// Parse `cargo install --list` output:
/// ```
/// crate-name v0.1.0 (/path):
///     binary-name
/// ```
fn parse_cargo_list_output(output: &str) -> Vec<PackageInfo> {
    let mut results = Vec::new();
    for line in output.lines() {
        let t = line.trim();
        // Package header lines: end with ':' and contain " v"
        if t.ends_with(':') && t.contains(" v") {
            let rest = &t[..t.len() - 1];
            // Strip optional " (/path)" suffix before version
            let rest = if let Some(paren) = rest.find(" (") {
                &rest[..paren]
            } else {
                rest
            };
            if let Some(v_pos) = rest.rfind(" v") {
                let name = rest[..v_pos].trim().to_string();
                let version = rest[v_pos + 1..].trim().to_string();
                if !name.is_empty() {
                    results.push(PackageInfo {
                        name,
                        version,
                        description: String::new(),
                        source: "cargo".to_string(),
                        installed: true,
                    });
                }
            }
        }
    }
    results
}

// ── AM package manager wrapper ───────────────────────────────────────

/// AM (AppImage Manager) — supports `am` or `appman` binary
pub struct AmWrapper {
    binary: String,
}

impl AmWrapper {
    pub fn new() -> Self {
        let binary = if which("am").is_ok() {
            "am".to_string()
        } else {
            "appman".to_string()
        };
        Self { binary }
    }
}

#[async_trait]
impl PackageManager for AmWrapper {
    fn name(&self) -> &str {
        "am"
    }
    fn is_available(&self) -> bool {
        which("am").is_ok() || which("appman").is_ok()
    }
    async fn search(&self, query: &str) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-q").arg(query);
        run_command_interactive(&mut cmd).await
    }
    async fn search_captured(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-q").arg(query);
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_am_output(&output, "am"))
    }
    async fn install(&self, packages: &[String]) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-i");
        for pkg in packages { cmd.arg(pkg); }
        run_command_quiet(&mut cmd).await
    }
    async fn remove(&self, packages: &[String]) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-R");
        for pkg in packages { cmd.arg(pkg); }
        run_command_quiet(&mut cmd).await
    }
    async fn update(&self) -> PmResult {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-u").arg("all");
        run_command_quiet(&mut cmd).await
    }
    async fn list_installed(&self) -> Result<Vec<PackageInfo>, String> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("-l");
        let output = run_command_captured(&mut cmd).await?;
        Ok(parse_am_list_output(&output, "am"))
    }
}

fn parse_am_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with('-') && !t.starts_with('=')
        })
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, char::is_whitespace).collect();
            PackageInfo {
                name: parts[0].to_string(),
                version: String::new(),
                description: parts.get(1).unwrap_or(&"").trim().to_string(),
                source: source.to_string(),
                installed: false,
            }
        })
        .collect()
}

fn parse_am_list_output(output: &str, source: &str) -> Vec<PackageInfo> {
    output
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with('-') && !t.starts_with('=')
        })
        .map(|l| {
            let parts: Vec<&str> = l.trim().splitn(2, char::is_whitespace).collect();
            PackageInfo {
                name: parts[0].to_string(),
                version: parts.get(1).unwrap_or(&"").trim().to_string(),
                description: String::new(),
                source: source.to_string(),
                installed: true,
            }
        })
        .collect()
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
            list_cmd: vec!["list".into(), "--installed".into()],
            needs_sudo: true,
        }),
        Box::new(GenericWrapper {
            name: "pacstall".into(),
            binary: "pacstall".into(),
            search_cmd: vec!["-S".into()],
            install_cmd: vec!["-I".into()],
            remove_cmd: vec!["-R".into()],
            update_cmd: vec!["-U".into(), "all".into()],
            list_cmd: vec!["-L".into()],
            needs_sudo: false, // pacstall handles its own privilege escalation
        }),
        Box::new(GenericWrapper {
            name: "flatpak".into(),
            binary: "flatpak".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into(), "-y".into()],
            remove_cmd: vec!["uninstall".into(), "-y".into()],
            update_cmd: vec!["update".into(), "-y".into()],
            list_cmd: vec!["list".into()],
            needs_sudo: false, // flatpak user installs don't need sudo
        }),
        Box::new(GenericWrapper {
            name: "snap".into(),
            binary: "snap".into(),
            search_cmd: vec!["find".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["remove".into()],
            update_cmd: vec!["refresh".into()],
            list_cmd: vec!["list".into()],
            needs_sudo: true,
        }),
        Box::new(GenericWrapper {
            name: "brew".into(),
            binary: "brew".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["uninstall".into()],
            update_cmd: vec!["upgrade".into()],
            list_cmd: vec!["list".into()],
            needs_sudo: false,
        }),
        Box::new(GenericWrapper {
            name: "soar".into(),
            binary: "soar".into(),
            search_cmd: vec!["list".into()],
            install_cmd: vec!["add".into()],
            remove_cmd: vec!["remove".into()],
            update_cmd: vec!["update".into()],
            list_cmd: vec!["list".into(), "--installed".into()],
            needs_sudo: false,
        }),
        Box::new(GenericWrapper {
            name: "npm".into(),
            binary: "npm".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into(), "-g".into()],
            remove_cmd: vec!["uninstall".into(), "-g".into()],
            update_cmd: vec!["update".into(), "-g".into()],
            list_cmd: vec!["list".into(), "-g".into(), "--depth=0".into()],
            needs_sudo: false,
        }),
        Box::new(GenericWrapper {
            name: "bun".into(),
            binary: "bun".into(),
            search_cmd: vec![],
            install_cmd: vec!["add".into(), "-g".into()],
            remove_cmd: vec!["remove".into(), "-g".into()],
            update_cmd: vec!["update".into(), "-g".into()],
            list_cmd: vec!["pm".into(), "ls".into(), "-g".into()],
            needs_sudo: false,
        }),
        Box::new(GenericWrapper {
            name: "pip".into(),
            binary: "pip".into(),
            search_cmd: vec!["search".into()],
            install_cmd: vec!["install".into()],
            remove_cmd: vec!["uninstall".into(), "-y".into()],
            update_cmd: vec!["install".into(), "--upgrade".into()],
            list_cmd: vec!["list".into()],
            needs_sudo: false,
        }),
        Box::new(GenericWrapper {
            name: "topgrade".into(),
            binary: "topgrade".into(),
            search_cmd: vec![],
            install_cmd: vec![],
            remove_cmd: vec![],
            update_cmd: vec!["--yes".into()],
            list_cmd: vec![],
            needs_sudo: false,
        }),
        Box::new(CargoWrapper),
        Box::new(AmWrapper::new()),
        Box::new(GenericWrapper {
            name: "cargo-binstall".into(),
            binary: "cargo-binstall".into(),
            search_cmd: vec![],  // no native search; installs are via cargo-binstall
            install_cmd: vec!["--no-confirm".into()],
            remove_cmd: vec![], // removal is handled by `cargo uninstall`
            update_cmd: vec!["--no-confirm".into()],
            list_cmd: vec![],
        }),
    ]
}
