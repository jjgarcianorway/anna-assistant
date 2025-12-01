//! Config Discovery v7.2.0 - Multi-source config file discovery
//!
//! Sources (in priority order):
//! 1. pacman -Ql <package> | grep -E '^/etc/' - Files owned by the package
//! 2. man pages - Config paths mentioned in documentation
//! 3. Arch Wiki (local) - Config paths mentioned in wiki docs
//!
//! Rules:
//! - Every config file must cite its source
//! - No guessing, no "common paths"
//! - Paths from multiple sources are merged with combined attribution

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::arch_wiki::{ArchWikiIndex, PathType, resolve_user_path};

/// A discovered config file with its source(s)
#[derive(Debug, Clone)]
pub struct ConfigFile {
    /// The path (absolute or ~-relative for user paths)
    pub path: String,
    /// Sources of this discovery (e.g., "pacman -Ql vim, Arch Wiki")
    pub source: String,
    /// Whether the file currently exists on the system
    pub exists: bool,
    /// Whether this is a user config (~/) or system config (/etc)
    pub is_user_config: bool,
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
                            is_user_config: false,
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
    let mut seen_paths: HashMap<String, bool> = HashMap::new();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let source = format!("man {}", command);

            for line in stdout.lines() {
                // Look for /etc paths
                for word in line.split_whitespace() {
                    let clean = word.trim_matches(|c| c == ',' || c == '.' || c == '"' || c == '\'' || c == '(' || c == ')');

                    // System paths
                    if clean.starts_with("/etc/") && clean.len() > 5 {
                        let path = clean.to_string();
                        if !seen_paths.contains_key(&path) && looks_like_config_path(&path) {
                            seen_paths.insert(path.clone(), false);
                            configs.push(ConfigFile {
                                path: path.clone(),
                                source: source.clone(),
                                exists: Path::new(&path).exists(),
                                is_user_config: false,
                            });
                        }
                    }

                    // User paths (~/)
                    if clean.starts_with("~/") && clean.len() > 2 {
                        let path = clean.to_string();
                        if !seen_paths.contains_key(&path) && looks_like_user_config_path(&path) {
                            let exists = resolve_user_path(&path)
                                .map(|p| p.exists())
                                .unwrap_or(false);
                            seen_paths.insert(path.clone(), true);
                            configs.push(ConfigFile {
                                path,
                                source: source.clone(),
                                exists,
                                is_user_config: true,
                            });
                        }
                    }
                }
            }
        }
    }

    configs
}

/// Discover config files from Arch Wiki (if available)
pub fn discover_from_arch_wiki(name: &str) -> Vec<ConfigFile> {
    let wiki = ArchWikiIndex::detect();
    if !wiki.enabled {
        return Vec::new();
    }

    let candidates = wiki.find_candidates(name, 3);
    let mut configs = Vec::new();
    let mut seen_paths: HashMap<String, bool> = HashMap::new();

    for file_path in candidates {
        let hints = wiki.extract_config_paths(&file_path);
        for hint in hints {
            if seen_paths.contains_key(&hint.path) {
                continue;
            }

            let (exists, is_user) = match hint.path_type {
                PathType::System => (Path::new(&hint.path).exists(), false),
                PathType::User => {
                    let exists = resolve_user_path(&hint.path)
                        .map(|p| p.exists())
                        .unwrap_or(false);
                    (exists, true)
                }
            };

            seen_paths.insert(hint.path.clone(), is_user);
            configs.push(ConfigFile {
                path: hint.path,
                source: "Arch Wiki".to_string(),
                exists,
                is_user_config: is_user,
            });
        }
    }

    configs
}

/// Discover config files for an object from all sources
/// Merges duplicate paths with combined source attribution
pub fn discover_config_files(name: &str) -> Vec<ConfigFile> {
    // Track paths and their sources
    let mut path_map: HashMap<String, (Vec<String>, bool, bool)> = HashMap::new(); // path -> (sources, exists, is_user)

    // Try pacman first (most reliable)
    for cfg in discover_from_pacman(name) {
        let entry = path_map.entry(cfg.path.clone()).or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config));
        entry.0.push(cfg.source);
        entry.1 = entry.1 || cfg.exists; // exists if any source says it exists
    }

    // Then try man page
    for cfg in discover_from_man(name) {
        let entry = path_map.entry(cfg.path.clone()).or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config));
        if !entry.0.iter().any(|s| s.starts_with("man ")) {
            entry.0.push(cfg.source);
        }
        entry.1 = entry.1 || cfg.exists;
    }

    // Then try Arch Wiki
    for cfg in discover_from_arch_wiki(name) {
        let entry = path_map.entry(cfg.path.clone()).or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config));
        if !entry.0.contains(&"Arch Wiki".to_string()) {
            entry.0.push(cfg.source);
        }
        entry.1 = entry.1 || cfg.exists;
    }

    // Convert to final list
    let mut all_configs: Vec<ConfigFile> = path_map
        .into_iter()
        .map(|(path, (sources, exists, is_user))| ConfigFile {
            path,
            source: sources.join(", "),
            exists,
            is_user_config: is_user,
        })
        .collect();

    // Sort: system configs first, then user configs, alphabetically within each
    all_configs.sort_by(|a, b| {
        match (a.is_user_config, b.is_user_config) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        }
    });

    all_configs
}

/// Check if a path looks like a valid system config file path
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

/// Check if a user path looks valid
fn looks_like_user_config_path(path: &str) -> bool {
    if !path.starts_with("~/") {
        return false;
    }

    if path.len() < 3 {
        return false;
    }

    // Must not contain weird chars
    if path.contains('*') || path.contains('{') || path.contains('<') {
        return false;
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
    fn test_looks_like_user_config_path() {
        assert!(looks_like_user_config_path("~/.vimrc"));
        assert!(looks_like_user_config_path("~/.config/nvim/init.lua"));

        // Invalid
        assert!(!looks_like_user_config_path("~/"));
        assert!(!looks_like_user_config_path("~"));
        assert!(!looks_like_user_config_path("~/.config/*"));
    }

    #[test]
    fn test_pacman_discovery() {
        // Test with a common package
        let configs = discover_from_pacman("pacman");
        // Should find /etc/pacman.conf if pacman is installed
        for cfg in &configs {
            assert!(cfg.path.starts_with("/etc/"));
            assert!(cfg.source.contains("pacman -Ql"));
            assert!(!cfg.is_user_config);
        }
    }
}
