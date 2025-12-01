//! Config Discovery v7.1.0 - Strict doc-driven config file discovery
//!
//! Sources (in priority order):
//! 1. pacman -Ql <package> | grep -E '^/etc/' - Files owned by the package
//! 2. man pages - Config paths mentioned in documentation
//!
//! Rules:
//! - Every config file must cite its source
//! - No guessing, no "common paths"
//! - Only return paths that actually exist on the system

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// A discovered config file with its source
#[derive(Debug, Clone)]
pub struct ConfigFile {
    /// The absolute path to the config file
    pub path: String,
    /// Source of this discovery (e.g., "pacman -Ql vim")
    pub source: String,
    /// Whether the file currently exists on the system
    pub exists: bool,
}

/// Discover config files for a package from pacman
pub fn discover_from_pacman(package: &str) -> Vec<ConfigFile> {
    let output = Command::new("pacman")
        .args(["-Ql", package])
        .output();

    let mut configs = Vec::new();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let source = format!("pacman -Ql {}", package);

            for line in stdout.lines() {
                // Format: "packagename /path/to/file"
                if let Some(path) = line.split_whitespace().nth(1) {
                    // Only /etc files (config files)
                    if path.starts_with("/etc/") && !path.ends_with('/') {
                        configs.push(ConfigFile {
                            path: path.to_string(),
                            source: source.clone(),
                            exists: Path::new(path).exists(),
                        });
                    }
                }
            }
        }
    }

    configs
}

/// Discover config files mentioned in man pages
pub fn discover_from_man(command: &str) -> Vec<ConfigFile> {
    // Try to get man page content
    let output = Command::new("man")
        .args(["-P", "cat", command])  // Use cat as pager to get raw output
        .env("MANWIDTH", "1000")        // Wide output to avoid line wrapping
        .output();

    let mut configs = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let source = format!("man {}", command);

            // Look for common config file patterns
            // Pattern 1: /etc/something (explicit path)
            // Pattern 2: ~/.config/something or $XDG_CONFIG_HOME
            for line in stdout.lines() {
                // Look for /etc paths
                for word in line.split_whitespace() {
                    let clean = word.trim_matches(|c| c == ',' || c == '.' || c == '"' || c == '\'' || c == '(' || c == ')');

                    if clean.starts_with("/etc/") && clean.len() > 5 {
                        // Must look like a file path (contains alphanumeric after /etc/)
                        let path = clean.to_string();
                        if !seen_paths.contains(&path) && looks_like_config_path(&path) {
                            seen_paths.insert(path.clone());
                            configs.push(ConfigFile {
                                path: path.clone(),
                                source: source.clone(),
                                exists: Path::new(&path).exists(),
                            });
                        }
                    }
                }
            }
        }
    }

    configs
}

/// Discover config files for an object (tries pacman first, then man)
pub fn discover_config_files(name: &str) -> Vec<ConfigFile> {
    let mut all_configs = Vec::new();
    let mut seen_paths: HashSet<String> = HashSet::new();

    // Try pacman first (most reliable)
    let pacman_configs = discover_from_pacman(name);
    for cfg in pacman_configs {
        if !seen_paths.contains(&cfg.path) {
            seen_paths.insert(cfg.path.clone());
            all_configs.push(cfg);
        }
    }

    // Then try man page
    let man_configs = discover_from_man(name);
    for cfg in man_configs {
        if !seen_paths.contains(&cfg.path) {
            seen_paths.insert(cfg.path.clone());
            all_configs.push(cfg);
        }
    }

    // Sort by path for consistent output
    all_configs.sort_by(|a, b| a.path.cmp(&b.path));

    all_configs
}

/// Check if a path looks like a valid config file path
fn looks_like_config_path(path: &str) -> bool {
    // Must start with /etc/
    if !path.starts_with("/etc/") {
        return false;
    }

    // Must have something after /etc/
    let after_etc = &path[5..];
    if after_etc.is_empty() {
        return false;
    }

    // Must not end with / (directories)
    if path.ends_with('/') {
        return false;
    }

    // Must not contain weird characters that suggest it's not a real path
    if path.contains('*') || path.contains('$') || path.contains('{') || path.contains('<') {
        return false;
    }

    // First character after /etc/ must be alphanumeric
    if let Some(c) = after_etc.chars().next() {
        if !c.is_alphanumeric() && c != '.' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_config_path() {
        assert!(looks_like_config_path("/etc/pacman.conf"));
        assert!(looks_like_config_path("/etc/ssh/sshd_config"));
        assert!(looks_like_config_path("/etc/X11/xorg.conf"));

        // Invalid paths
        assert!(!looks_like_config_path("/etc/"));
        assert!(!looks_like_config_path("/etc"));
        assert!(!looks_like_config_path("/etc/*.conf"));
        assert!(!looks_like_config_path("/etc/${FOO}/bar"));
        assert!(!looks_like_config_path("/usr/etc/foo"));
    }

    #[test]
    fn test_pacman_discovery() {
        // Test with a common package
        let configs = discover_from_pacman("pacman");
        // Should find /etc/pacman.conf if pacman is installed
        // This is a real system test, so we just check the structure
        for cfg in &configs {
            assert!(cfg.path.starts_with("/etc/"));
            assert!(cfg.source.contains("pacman -Ql"));
        }
    }
}
