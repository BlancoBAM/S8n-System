use std::process::Stdio;
use tokio::process::Command;
use async_trait::async_trait;

pub mod builtin;

/// Structured information about a package from search results
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: String,
    pub installed: bool,
}

/// Result from a package manager operation
#[derive(Debug)]
pub enum PmResult {
    Success,
    CommandFailed(i32, String),
    Error(String),
}

/// Represents an abstract Package Manager that can be invoked via CLI
#[async_trait]
pub trait PackageManager: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;

    /// Original interactive search (prints to stdout)
    async fn search(&self, query: &str) -> PmResult;

    /// Captured search returning structured results
    async fn search_captured(&self, query: &str) -> Result<Vec<PackageInfo>, String> {
        // Default: no structured results
        let _ = query;
        Ok(vec![])
    }

    async fn install(&self, packages: &[String]) -> PmResult;
    async fn remove(&self, packages: &[String]) -> PmResult;
    async fn update(&self) -> PmResult;
}

/// Helper function to run a command and wait for standard success/failure without capturing output
pub async fn run_command_interactive(cmd: &mut Command) -> PmResult {
    match cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit())
        .status()
        .await
    {
        Ok(status) => {
            if status.success() {
                PmResult::Success
            } else {
                PmResult::CommandFailed(status.code().unwrap_or(-1), format!("Command failed: {:?}", cmd))
            }
        }
        Err(e) => PmResult::Error(e.to_string()),
    }
}

/// Helper function to run a command and capture its stdout
pub async fn run_command_captured(cmd: &mut Command) -> Result<String, String> {
    match cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Err(format!("Exit {}: {}", output.status.code().unwrap_or(-1), stderr))
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

/// Run a command silently (all output piped, not inherited).
/// Use this inside the TUI alternate screen so external command output
/// doesn't corrupt the viewport.
pub async fn run_command_quiet(cmd: &mut Command) -> PmResult {
    match cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                PmResult::Success
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                PmResult::CommandFailed(output.status.code().unwrap_or(-1), stderr)
            }
        }
        Err(e) => PmResult::Error(e.to_string()),
    }
}
