use std::process::Stdio;
use tokio::process::Command;
use async_trait::async_trait;

pub mod builtin;

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

    async fn search(&self, query: &str) -> PmResult;
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
