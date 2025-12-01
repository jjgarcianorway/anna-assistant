//! Arch Wiki Config Discovery v7.6.1
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
//! - v7.6.1: Strip HTML before extraction, filter paths by identity

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
        self.extract_config_paths_for_identity(file_path, None)
    }

    /// Extract config paths from a wiki HTML file, filtered by identity
    /// v7.6.1: Identity-focused filtering to avoid unrelated paths
    pub fn extract_config_paths_for_identity(&self, file_path: &str, identity: Option<&str>) -> Vec<ConfigHint> {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        // v7.6.1: Strip HTML before processing
        let clean_content = strip_html(&content);

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

        for line in clean_content.lines() {
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
                    // v7.6.1: Filter by identity if provided
                    if let Some(id) = identity {
                        if !path_belongs_to_identity(path, id) {
                            continue;
                        }
                    }
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
                    // v7.6.1: Filter by identity if provided
                    if let Some(id) = identity {
                        if !path_belongs_to_identity(path, id) {
                            continue;
                        }
                    }
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

/// Strip HTML tags from content - v7.6.1
/// Removes all HTML tags, keeping text content
pub fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut chars = html.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
            // Add space after certain tags to separate words
            result.push(' ');
        } else if !in_tag {
            result.push(c);
        }
    }

    // Clean up multiple spaces and decode common HTML entities
    result = result
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");

    // Collapse multiple whitespace
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&result, " ").trim().to_string()
}

/// Check if a path belongs to a specific identity - v7.6.1
/// Uses word boundary matching to avoid "nvim" matching "vim"
fn path_belongs_to_identity(path: &str, identity: &str) -> bool {
    let path_lower = path.to_lowercase();
    let id_lower = identity.to_lowercase();

    // v7.6.1: Exclude paths that belong to other well-known tools
    if is_path_for_other_tool(&path_lower, &id_lower) {
        return false;
    }

    // v7.6.1: Exclude non-config paths (scripts, wallpapers, icons, etc.)
    if is_non_config_path(&path_lower) {
        return false;
    }

    // v7.6.1: Exclude paths that primarily belong to a conflicting identity
    // E.g., /nvim/ paths should not show up for vim (even if they end in .vim)
    if id_lower == "vim" && path_contains_identity_segment(&path_lower, "nvim") {
        return false;
    }
    // nvim should not match pure /vim/ paths
    if id_lower == "nvim" && path_contains_identity_segment(&path_lower, "vim")
        && !path_contains_identity_segment(&path_lower, "nvim") {
        return false;
    }

    // Check for exact segment match (bounded by / or . or end)
    if path_contains_identity_segment(&path_lower, &id_lower) {
        return true;
    }

    // Known aliases and short names
    let aliases = get_identity_aliases(&id_lower);
    for alias in aliases {
        if path_contains_identity_segment(&path_lower, &alias) {
            return true;
        }
    }

    false
}

/// Check if path belongs to a different well-known tool - v7.6.1
fn is_path_for_other_tool(path: &str, current_identity: &str) -> bool {
    let other_tools = [
        "uwsm", "mako", "waybar", "dunst", "rofi", "wofi", "swaylock",
        "swayidle", "wlogout", "eww", "ags", "nwg", "wlr", "sway",
        "kitty", "alacritty", "foot", "wezterm",
    ];

    for tool in &other_tools {
        if *tool == current_identity {
            continue;
        }
        if path.contains(&format!("/{}/", tool)) || path.contains(&format!("/.{}", tool)) {
            return true;
        }
    }

    false
}

/// Check if path is clearly not a config file - v7.6.1
fn is_non_config_path(path: &str) -> bool {
    let non_config_patterns = [
        "/scripts/", "/wallpapers/", "/wallpapers", "/icons/",
        "/themes/", "/backgrounds/", "/screenshots/", "/cache/", "/logs/",
    ];

    for pattern in &non_config_patterns {
        if path.contains(pattern) {
            return true;
        }
    }

    if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") ||
       path.ends_with(".svg") || path.ends_with(".gif") || path.ends_with(".ico") ||
       path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".zsh") {
        return true;
    }

    false
}

/// Check if path contains identity as a distinct segment
/// Matches: /vim/, /vim., /vim at end, but NOT /nvim/
fn path_contains_identity_segment(path: &str, identity: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = path[start..].find(identity) {
        let abs_pos = start + pos;
        let end_pos = abs_pos + identity.len();

        let before_ok = abs_pos == 0 ||
            path.chars().nth(abs_pos - 1).map(|c| c == '/' || c == '.' || c == '-' || c == '_').unwrap_or(false);

        let after_ok = end_pos >= path.len() ||
            path.chars().nth(end_pos).map(|c| c == '/' || c == '.' || c == '-' || c == '_').unwrap_or(false);

        if before_ok && after_ok {
            return true;
        }

        start = abs_pos + 1;
        if start >= path.len() {
            break;
        }
    }

    false
}

/// Get known aliases for an identity - v7.6.1
/// These are small inline special cases, not a huge hardcoded table
fn get_identity_aliases(identity: &str) -> Vec<String> {
    match identity {
        "hyprland" => vec!["hypr".to_string()],
        "neovim" => vec!["nvim".to_string()],
        "networkmanager" => vec!["nm".to_string(), "network-manager".to_string()],
        "pulseaudio" => vec!["pulse".to_string()],
        "pipewire" => vec!["pw".to_string()],
        "bluetooth" | "bluez" => vec!["bt".to_string(), "bluetooth".to_string(), "bluez".to_string()],
        _ => vec![],
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

    // Must have something after the prefix (all prefixes are 5 chars: /etc/, /usr/, /var/)
    const PREFIX_LEN: usize = 5;
    if path.len() <= PREFIX_LEN {
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
