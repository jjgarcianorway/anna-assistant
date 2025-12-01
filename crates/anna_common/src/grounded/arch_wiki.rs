//! Arch Wiki Config Discovery v7.2.0
//!
//! Extracts config hints from local arch-wiki-docs package.
//! Does NOT use network - only reads local files.
//!
//! Sources:
//! - pacman -Ql arch-wiki-docs -> lists all wiki HTML files
//! - Parse HTML for config paths mentioned in "configuration" sections
//!
//! Rules:
//! - Only extract paths that look like real file paths
//! - Classify as system (/etc, /usr) or user (~/, $HOME)
//! - Never invent or guess paths

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::Regex;

/// Arch Wiki index status
#[derive(Debug, Clone)]
pub struct ArchWikiIndex {
    /// Whether arch-wiki-docs is available
    pub enabled: bool,
    /// Root path where wiki docs are installed
    pub root: Option<PathBuf>,
    /// List of HTML files (cached from pacman -Ql)
    files: Vec<String>,
}

impl ArchWikiIndex {
    /// Detect local Arch Wiki docs package
    pub fn detect() -> Self {
        // Try arch-wiki-docs first
        if let Some(index) = Self::try_package("arch-wiki-docs") {
            return index;
        }

        // Try arch-wiki-lite as alternative
        if let Some(index) = Self::try_package("arch-wiki-lite") {
            return index;
        }

        // Not available
        Self {
            enabled: false,
            root: None,
            files: Vec::new(),
        }
    }

    fn try_package(package: &str) -> Option<Self> {
        // Check if package is installed
        let check = Command::new("pacman")
            .args(["-Qi", package])
            .output()
            .ok()?;

        if !check.status.success() {
            return None;
        }

        // Get list of files
        let output = Command::new("pacman")
            .args(["-Ql", package])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();
        let mut root: Option<PathBuf> = None;

        for line in stdout.lines() {
            // Format: "package /path/to/file"
            if let Some(path) = line.split_whitespace().nth(1) {
                // Look for HTML files
                if path.ends_with(".html") {
                    files.push(path.to_string());

                    // Detect root from first HTML file
                    if root.is_none() {
                        if let Some(parent) = Path::new(path).parent() {
                            root = Some(parent.to_path_buf());
                        }
                    }
                }
            }
        }

        if files.is_empty() {
            return None;
        }

        Some(Self {
            enabled: true,
            root,
            files,
        })
    }

    /// Find candidate wiki pages for a package/command name
    pub fn find_candidates(&self, name: &str, max: usize) -> Vec<String> {
        if !self.enabled {
            return Vec::new();
        }

        let name_lower = name.to_lowercase();
        let mut candidates = Vec::new();

        for file in &self.files {
            // Extract filename without path and extension
            let filename = Path::new(file)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Match if filename contains the name (case insensitive)
            if filename.contains(&name_lower) || name_lower.contains(&filename) {
                candidates.push(file.clone());
                if candidates.len() >= max {
                    break;
                }
            }
        }

        candidates
    }

    /// Extract config paths from a wiki HTML file
    pub fn extract_config_paths(&self, file_path: &str) -> Vec<ConfigHint> {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let mut hints = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        // Look for lines containing config-related keywords
        let config_keywords = [
            "configuration",
            "config file",
            "config directory",
            "rc file",
            ".conf",
        ];

        // Regex for system paths
        let sys_path_re = Regex::new(r#"(/(?:etc|usr|var)/[a-zA-Z0-9_./-]+)"#).unwrap();
        // Regex for user paths
        let user_path_re = Regex::new(r#"(~(?:/[a-zA-Z0-9_./-]+)+|(?:\$HOME|\$XDG_CONFIG_HOME)/[a-zA-Z0-9_./-]+)"#).unwrap();

        let _content_lower = content.to_lowercase();

        for line in content.lines() {
            let line_lower = line.to_lowercase();

            // Check if line is in a config-related context
            let is_config_context = config_keywords.iter().any(|kw| line_lower.contains(kw));

            // Also check surrounding context (simplified: just check current line)
            if !is_config_context {
                continue;
            }

            // Extract system paths
            for cap in sys_path_re.captures_iter(line) {
                let path = cap.get(1).unwrap().as_str();
                if looks_like_valid_config_path(path) && !seen.contains(path) {
                    seen.insert(path.to_string());
                    hints.push(ConfigHint {
                        path: path.to_string(),
                        path_type: PathType::System,
                    });
                }
            }

            // Extract user paths
            for cap in user_path_re.captures_iter(line) {
                let path = cap.get(1).unwrap().as_str();
                if looks_like_valid_user_path(path) && !seen.contains(path) {
                    seen.insert(path.to_string());
                    hints.push(ConfigHint {
                        path: path.to_string(),
                        path_type: PathType::User,
                    });
                }
            }
        }

        hints
    }
}

/// A config hint extracted from Arch Wiki
#[derive(Debug, Clone)]
pub struct ConfigHint {
    /// The path (may be absolute or ~-relative)
    pub path: String,
    /// Type of path (system or user)
    pub path_type: PathType,
}

/// Path type classification
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    /// System-wide config (/etc, /usr, etc)
    System,
    /// User config (~/, $HOME, $XDG_CONFIG_HOME)
    User,
}

/// Check if a path looks like a valid system config path
fn looks_like_valid_config_path(path: &str) -> bool {
    // Must start with /etc/, /usr/, or /var/
    if !path.starts_with("/etc/") && !path.starts_with("/usr/") && !path.starts_with("/var/") {
        return false;
    }

    // Must have something after the prefix
    let prefix_len = if path.starts_with("/etc/") { 5 }
        else if path.starts_with("/usr/") { 5 }
        else { 5 }; // /var/

    if path.len() <= prefix_len {
        return false;
    }

    // Must not end with /
    if path.ends_with('/') {
        return false;
    }

    // Must not contain HTML entities or weird chars
    if path.contains('&') || path.contains('<') || path.contains('>')
        || path.contains('"') || path.contains('\'') {
        return false;
    }

    // Must not be a generic placeholder
    if path.contains("...") || path.contains("*") || path.contains("{") {
        return false;
    }

    true
}

/// Check if a path looks like a valid user config path
fn looks_like_valid_user_path(path: &str) -> bool {
    // Must start with ~/ or $HOME or $XDG_CONFIG_HOME
    if !path.starts_with("~/") && !path.starts_with("$HOME/") && !path.starts_with("$XDG_CONFIG_HOME/") {
        return false;
    }

    // Must have something after the prefix
    if path.len() < 5 {
        return false;
    }

    // Must not contain weird chars
    if path.contains('&') || path.contains('<') || path.contains('>') {
        return false;
    }

    // Must not be a generic placeholder
    if path.contains("...") || path.contains("*") {
        return false;
    }

    true
}

/// Resolve a user path to absolute for existence check
pub fn resolve_user_path(path: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;

    if path.starts_with("~/") {
        Some(PathBuf::from(format!("{}{}", home, &path[1..])))
    } else if path.starts_with("$HOME/") {
        Some(PathBuf::from(format!("{}{}", home, &path[5..])))
    } else if path.starts_with("$XDG_CONFIG_HOME/") {
        let xdg = std::env::var("XDG_CONFIG_HOME")
            .unwrap_or_else(|_| format!("{}/.config", home));
        Some(PathBuf::from(format!("{}{}", xdg, &path[16..])))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_valid_config_path() {
        assert!(looks_like_valid_config_path("/etc/pacman.conf"));
        assert!(looks_like_valid_config_path("/etc/ssh/sshd_config"));
        assert!(looks_like_valid_config_path("/usr/share/vim/vimrc"));

        // Invalid
        assert!(!looks_like_valid_config_path("/etc/"));
        assert!(!looks_like_valid_config_path("/home/user/.config"));
        assert!(!looks_like_valid_config_path("/etc/..."));
        assert!(!looks_like_valid_config_path("/etc/*.conf"));
    }

    #[test]
    fn test_looks_like_valid_user_path() {
        assert!(looks_like_valid_user_path("~/.config/vim/vimrc"));
        assert!(looks_like_valid_user_path("~/.vimrc"));
        assert!(looks_like_valid_user_path("$HOME/.bashrc"));
        assert!(looks_like_valid_user_path("$XDG_CONFIG_HOME/nvim/init.lua"));

        // Invalid
        assert!(!looks_like_valid_user_path("/etc/vim"));
        assert!(!looks_like_valid_user_path("~"));
        assert!(!looks_like_valid_user_path("~/.config/*"));
    }

    #[test]
    fn test_resolve_user_path() {
        // These depend on HOME being set
        if std::env::var("HOME").is_ok() {
            let home = std::env::var("HOME").unwrap();
            assert_eq!(
                resolve_user_path("~/.vimrc"),
                Some(PathBuf::from(format!("{}/.vimrc", home)))
            );
        }
    }
}
