//! S8n graveyard — rip2 integration for safe file burial and recovery
//!
//! Files are buried into a dedicated `s8n/` subdirectory inside rip2's graveyard.
//! This keeps s8n-deleted files separate from manually rip'd files.

use std::path::{Path, PathBuf};
use tokio::process::Command;
use which::which;

/// A file that has been buried in the graveyard
#[derive(Debug, Clone)]
pub struct BuriedFile {
    /// Original path before burial
    pub original: PathBuf,
    /// Path in the graveyard (determined by rip output)
    pub graveyard_path: PathBuf,
}

/// Graveyard configuration and operations
pub struct GraveyardConfig {
    /// Path to the `rip` or `rip2` binary
    pub rip_bin: PathBuf,
    /// The s8n-specific subdirectory in the graveyard
    pub s8n_dir: PathBuf,
}

impl GraveyardConfig {
    /// Discover the rip binary and set up the s8n graveyard directory.
    ///
    /// Graveyard dir: $XDG_DATA_HOME/graveyard/s8n or ~/.local/share/graveyard/s8n
    pub fn new() -> Result<Self, String> {
        let rip_bin = which("rip")
            .or_else(|_| which("rip2"))
            .map_err(|_| "rip2 is not installed. Install it with: cargo install rm-improved".to_string())?;

        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".local").join("share")
            });

        let s8n_dir = base.join("graveyard").join("s8n");

        Ok(Self { rip_bin, s8n_dir })
    }

    /// Bury one or more files into the s8n graveyard directory.
    /// Returns a Vec of BuriedFile with the resolved graveyard paths.
    pub async fn bury(&self, files: &[PathBuf]) -> Result<Vec<BuriedFile>, String> {
        // Ensure the s8n graveyard directory exists
        std::fs::create_dir_all(&self.s8n_dir)
            .map_err(|e| format!("Failed to create graveyard dir: {}", e))?;

        let mut buried = Vec::new();

        for file in files {
            let canonical = file
                .canonicalize()
                .unwrap_or_else(|_| file.clone());

            let status = Command::new(&self.rip_bin)
                .arg("--graveyard")
                .arg(&self.s8n_dir)
                .arg(&canonical)
                .status()
                .await
                .map_err(|e| format!("Failed to run rip: {}", e))?;

            if !status.success() {
                return Err(format!("rip failed to bury: {}", canonical.display()));
            }

            // rip places files in graveyard/<original_path_without_leading_slash>
            // e.g. burying /home/user/test.txt -> s8n_dir/home/user/test.txt
            let relative = if canonical.is_absolute() {
                canonical.strip_prefix("/").unwrap_or(&canonical).to_path_buf()
            } else {
                canonical.clone()
            };
            let graveyard_path = self.s8n_dir.join(&relative);

            buried.push(BuriedFile {
                original: canonical,
                graveyard_path,
            });
        }

        Ok(buried)
    }

    /// Exhume (recover) buried files from the s8n graveyard.
    /// If `files` is None, runs rip in interactive unbury mode.
    /// If `files` is Some, tries to unbury each by path match.
    pub async fn exhume(&self, files: Option<&[String]>) -> Result<(), String> {
        match files {
            None => {
                // Interactive mode: rip --graveyard <dir> -u
                let status = Command::new(&self.rip_bin)
                    .arg("--graveyard")
                    .arg(&self.s8n_dir)
                    .arg("-u")
                    .status()
                    .await
                    .map_err(|e| format!("Failed to run rip: {}", e))?;

                if !status.success() {
                    return Err("rip unbury failed".to_string());
                }
            }
            Some(targets) => {
                for target in targets {
                    // Try to find the buried file by name pattern
                    let buried_path = self.find_buried(target);
                    let status = if let Some(path) = buried_path {
                        Command::new(&self.rip_bin)
                            .arg("--graveyard")
                            .arg(&self.s8n_dir)
                            .arg("-u")
                            .arg(&path)
                            .status()
                            .await
                            .map_err(|e| format!("Failed to run rip: {}", e))?
                    } else {
                        // Fall back to letting rip figure it out
                        Command::new(&self.rip_bin)
                            .arg("--graveyard")
                            .arg(&self.s8n_dir)
                            .arg("-u")
                            .arg(target)
                            .status()
                            .await
                            .map_err(|e| format!("Failed to run rip: {}", e))?
                    };

                    if !status.success() {
                        return Err(format!("Failed to recover: {}", target));
                    }
                }
            }
        }
        Ok(())
    }

    /// List all buried files in the s8n graveyard directory recursively.
    pub fn list_buried(&self) -> Vec<BuriedEntry> {
        let mut entries = Vec::new();
        if !self.s8n_dir.exists() {
            return entries;
        }
        Self::walk_dir(&self.s8n_dir, &self.s8n_dir, &mut entries);
        // Sort by modification time (newest first)
        entries.sort_by_key(|e| std::cmp::Reverse(e.modified));
        entries
    }

    fn walk_dir(base: &Path, dir: &Path, out: &mut Vec<BuriedEntry>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::walk_dir(base, &path, out);
            } else {
                let meta = std::fs::metadata(&path);
                let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = meta
                    .as_ref()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                // Reconstruct the original path: strip s8n_dir prefix, re-add leading /
                let original = path
                    .strip_prefix(base)
                    .ok()
                    .map(|rel| Path::new("/").join(rel))
                    .unwrap_or_else(|| path.clone());
                out.push(BuriedEntry {
                    name: path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                    graveyard_path: path,
                    original_path: original,
                    size_bytes,
                    modified,
                });
            }
        }
    }

    fn find_buried(&self, name: &str) -> Option<PathBuf> {
        let entries = self.list_buried();
        entries
            .into_iter()
            .find(|e| e.name == name || e.original_path.to_string_lossy().contains(name))
            .map(|e| e.graveyard_path)
    }
}

/// A file entry found in the graveyard
#[derive(Debug, Clone)]
pub struct BuriedEntry {
    pub name: String,
    pub graveyard_path: PathBuf,
    pub original_path: PathBuf,
    pub size_bytes: u64,
    pub modified: u64, // Unix timestamp
}

impl BuriedEntry {
    /// Format the modification time as a human-readable string
    pub fn modified_str(&self) -> String {
        if self.modified == 0 {
            return "unknown".to_string();
        }
        let secs = self.modified;
        // Simple date formatting without extra deps
        // Just show days/hours ago
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let diff = now.saturating_sub(secs);
        if diff < 60 {
            format!("{}s ago", diff)
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else {
            format!("{}d ago", diff / 86400)
        }
    }

    /// Format file size as human-readable string
    pub fn size_str(&self) -> String {
        let s = self.size_bytes;
        if s < 1024 {
            format!("{}B", s)
        } else if s < 1024 * 1024 {
            format!("{:.1}K", s as f64 / 1024.0)
        } else if s < 1024 * 1024 * 1024 {
            format!("{:.1}M", s as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1}G", s as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
