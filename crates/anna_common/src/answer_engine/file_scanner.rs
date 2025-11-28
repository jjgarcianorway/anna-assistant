//! Safe File Scanner for v0.23.0
//!
//! Provides controlled file scanning within allowed paths:
//! - System scope: /etc, /usr/share, standard config paths
//! - User scope: $HOME/.config/**, $HOME/.local/share/**
//! - Resource limits on files and bytes scanned

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::protocol_v23::KnowledgeScope;

// ============================================================================
// ALLOWED PATHS
// ============================================================================

/// System-scope allowed base paths for scanning
pub const SYSTEM_ALLOWED_PATHS: &[&str] = &[
    "/etc",
    "/usr/share",
    "/usr/local/share",
    "/var/lib",
];

/// User-scope allowed relative paths (relative to $HOME)
pub const USER_ALLOWED_RELATIVE_PATHS: &[&str] = &[
    ".config",
    ".local/share",
    ".local/state",
];

/// Maximum file size to read (bytes)
pub const MAX_FILE_SIZE: usize = 64 * 1024; // 64 KB

/// Maximum files per scan operation
pub const MAX_FILES_PER_SCAN: usize = 50;

/// Maximum total bytes per scan operation
pub const MAX_BYTES_PER_SCAN: usize = 512 * 1024; // 512 KB

// ============================================================================
// SCAN CONFIG
// ============================================================================

/// Configuration for a file scan operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    /// Target scope
    pub scope: KnowledgeScope,
    /// File patterns to match (glob-style)
    pub patterns: Vec<String>,
    /// Maximum files to scan
    pub max_files: usize,
    /// Maximum bytes to read
    pub max_bytes: usize,
    /// Maximum file size per file
    pub max_file_size: usize,
    /// Home directory (for user scope)
    pub home_dir: Option<PathBuf>,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            scope: KnowledgeScope::User,
            patterns: Vec::new(),
            max_files: MAX_FILES_PER_SCAN,
            max_bytes: MAX_BYTES_PER_SCAN,
            max_file_size: MAX_FILE_SIZE,
            home_dir: None,
        }
    }
}

impl ScanConfig {
    /// Create config for user scope scanning
    pub fn user_scope(home_dir: PathBuf) -> Self {
        Self {
            scope: KnowledgeScope::User,
            home_dir: Some(home_dir),
            ..Default::default()
        }
    }

    /// Create config for system scope scanning
    pub fn system_scope() -> Self {
        Self {
            scope: KnowledgeScope::System,
            home_dir: None,
            ..Default::default()
        }
    }

    /// Set patterns to search for
    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns = patterns;
        self
    }
}

// ============================================================================
// PATH VALIDATION
// ============================================================================

/// Result of path validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathValidation {
    /// Path is allowed
    Allowed,
    /// Path is outside allowed directories
    OutsideAllowed,
    /// Path contains dangerous components (e.g., ..)
    Dangerous,
    /// Path is invalid
    Invalid,
}

/// Validate if a path is allowed for scanning
pub fn validate_path(path: &Path, scope: KnowledgeScope, home_dir: Option<&Path>) -> PathValidation {
    // Check for dangerous components
    let path_str = path.to_string_lossy();
    if path_str.contains("..") {
        return PathValidation::Dangerous;
    }

    // Canonicalize would resolve symlinks, but we work with what we have
    let path = match path.to_str() {
        Some(s) => s,
        None => return PathValidation::Invalid,
    };

    match scope {
        KnowledgeScope::System => {
            // Check against system allowed paths
            for allowed in SYSTEM_ALLOWED_PATHS {
                if path.starts_with(allowed) {
                    return PathValidation::Allowed;
                }
            }
            PathValidation::OutsideAllowed
        }
        KnowledgeScope::User => {
            // Need home directory for user scope
            let home = match home_dir {
                Some(h) => h,
                None => return PathValidation::Invalid,
            };

            let home_str = match home.to_str() {
                Some(s) => s,
                None => return PathValidation::Invalid,
            };

            // Check against user allowed paths
            for relative in USER_ALLOWED_RELATIVE_PATHS {
                let allowed = format!("{}/{}", home_str, relative);
                if path.starts_with(&allowed) {
                    return PathValidation::Allowed;
                }
            }
            PathValidation::OutsideAllowed
        }
    }
}

/// Get base directories for a scope
pub fn get_allowed_base_dirs(scope: KnowledgeScope, home_dir: Option<&Path>) -> Vec<PathBuf> {
    match scope {
        KnowledgeScope::System => {
            SYSTEM_ALLOWED_PATHS.iter()
                .map(|p| PathBuf::from(p))
                .collect()
        }
        KnowledgeScope::User => {
            match home_dir {
                Some(home) => {
                    USER_ALLOWED_RELATIVE_PATHS.iter()
                        .map(|rel| home.join(rel))
                        .collect()
                }
                None => Vec::new(),
            }
        }
    }
}

// ============================================================================
// SCAN RESULT
// ============================================================================

/// A single file found during scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    /// Full path to the file
    pub path: PathBuf,
    /// File size in bytes
    pub size: usize,
    /// Whether the file was read
    pub was_read: bool,
    /// Relevant content extracted (if any)
    pub content_preview: Option<String>,
    /// Magic strings found (for relevance detection)
    pub magic_matches: Vec<String>,
}

/// Result of a scan operation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    /// Files found and processed
    pub files: Vec<ScannedFile>,
    /// Total bytes scanned
    pub bytes_scanned: usize,
    /// Whether scan hit file limit
    pub hit_file_limit: bool,
    /// Whether scan hit byte limit
    pub hit_byte_limit: bool,
    /// Errors encountered (path -> error)
    pub errors: Vec<(PathBuf, String)>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ScanResult {
    /// Check if any files were found
    pub fn has_files(&self) -> bool {
        !self.files.is_empty()
    }

    /// Get count of files found
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Get files with specific magic string matches
    pub fn files_with_magic(&self, magic: &str) -> Vec<&ScannedFile> {
        self.files.iter()
            .filter(|f| f.magic_matches.iter().any(|m| m.contains(magic)))
            .collect()
    }
}

// ============================================================================
// KNOWN FILE PATTERNS
// ============================================================================

/// Well-known config file patterns for various applications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownPattern {
    /// Application name
    pub app: String,
    /// Fact key prefix this pattern fills
    pub fact_prefix: String,
    /// File patterns (glob-style)
    pub patterns: Vec<String>,
    /// Magic strings to look for in content
    pub magic_strings: Vec<String>,
    /// Target scope
    pub scope: KnowledgeScope,
}

/// Get known patterns for common applications
pub fn get_known_patterns() -> Vec<KnownPattern> {
    vec![
        // Neovim
        KnownPattern {
            app: "nvim".to_string(),
            fact_prefix: "user.editor.nvim".to_string(),
            patterns: vec![
                ".config/nvim/init.lua".to_string(),
                ".config/nvim/init.vim".to_string(),
            ],
            magic_strings: vec!["vim.opt".to_string(), "require(".to_string()],
            scope: KnowledgeScope::User,
        },
        // Vim
        KnownPattern {
            app: "vim".to_string(),
            fact_prefix: "user.editor.vim".to_string(),
            patterns: vec![
                ".vimrc".to_string(),
                ".config/vim/vimrc".to_string(),
            ],
            magic_strings: vec!["set ".to_string(), "syntax ".to_string()],
            scope: KnowledgeScope::User,
        },
        // Hyprland
        KnownPattern {
            app: "hyprland".to_string(),
            fact_prefix: "user.dewm.hyprland".to_string(),
            patterns: vec![
                ".config/hypr/hyprland.conf".to_string(),
            ],
            magic_strings: vec!["monitor=".to_string(), "bind=".to_string()],
            scope: KnowledgeScope::User,
        },
        // Git
        KnownPattern {
            app: "git".to_string(),
            fact_prefix: "user.vcs.git".to_string(),
            patterns: vec![
                ".gitconfig".to_string(),
                ".config/git/config".to_string(),
            ],
            magic_strings: vec!["[user]".to_string(), "[core]".to_string()],
            scope: KnowledgeScope::User,
        },
        // Bash
        KnownPattern {
            app: "bash".to_string(),
            fact_prefix: "user.shell.bash".to_string(),
            patterns: vec![
                ".bashrc".to_string(),
                ".bash_profile".to_string(),
            ],
            magic_strings: vec!["export ".to_string(), "alias ".to_string()],
            scope: KnowledgeScope::User,
        },
        // Zsh
        KnownPattern {
            app: "zsh".to_string(),
            fact_prefix: "user.shell.zsh".to_string(),
            patterns: vec![
                ".zshrc".to_string(),
                ".config/zsh/.zshrc".to_string(),
            ],
            magic_strings: vec!["export ".to_string(), "source ".to_string()],
            scope: KnowledgeScope::User,
        },
        // Fish
        KnownPattern {
            app: "fish".to_string(),
            fact_prefix: "user.shell.fish".to_string(),
            patterns: vec![
                ".config/fish/config.fish".to_string(),
            ],
            magic_strings: vec!["set ".to_string(), "function ".to_string()],
            scope: KnowledgeScope::User,
        },
        // Foot terminal
        KnownPattern {
            app: "foot".to_string(),
            fact_prefix: "user.terminal.foot".to_string(),
            patterns: vec![
                ".config/foot/foot.ini".to_string(),
            ],
            magic_strings: vec!["[main]".to_string(), "font=".to_string()],
            scope: KnowledgeScope::User,
        },
        // Alacritty terminal
        KnownPattern {
            app: "alacritty".to_string(),
            fact_prefix: "user.terminal.alacritty".to_string(),
            patterns: vec![
                ".config/alacritty/alacritty.toml".to_string(),
                ".config/alacritty/alacritty.yml".to_string(),
            ],
            magic_strings: vec!["[font]".to_string(), "font:".to_string()],
            scope: KnowledgeScope::User,
        },
    ]
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_system_path() {
        assert_eq!(
            validate_path(Path::new("/etc/pacman.conf"), KnowledgeScope::System, None),
            PathValidation::Allowed
        );
        assert_eq!(
            validate_path(Path::new("/root/.bashrc"), KnowledgeScope::System, None),
            PathValidation::OutsideAllowed
        );
    }

    #[test]
    fn test_validate_user_path() {
        let home = PathBuf::from("/home/user");
        assert_eq!(
            validate_path(
                Path::new("/home/user/.config/nvim/init.lua"),
                KnowledgeScope::User,
                Some(&home)
            ),
            PathValidation::Allowed
        );
        assert_eq!(
            validate_path(
                Path::new("/home/user/Documents/secret.txt"),
                KnowledgeScope::User,
                Some(&home)
            ),
            PathValidation::OutsideAllowed
        );
    }

    #[test]
    fn test_dangerous_path() {
        assert_eq!(
            validate_path(
                Path::new("/etc/../root/.bashrc"),
                KnowledgeScope::System,
                None
            ),
            PathValidation::Dangerous
        );
    }

    #[test]
    fn test_get_allowed_base_dirs() {
        let system_dirs = get_allowed_base_dirs(KnowledgeScope::System, None);
        assert!(system_dirs.contains(&PathBuf::from("/etc")));

        let home = PathBuf::from("/home/user");
        let user_dirs = get_allowed_base_dirs(KnowledgeScope::User, Some(&home));
        assert!(user_dirs.contains(&PathBuf::from("/home/user/.config")));
    }

    #[test]
    fn test_known_patterns() {
        let patterns = get_known_patterns();
        assert!(!patterns.is_empty());

        let nvim = patterns.iter().find(|p| p.app == "nvim");
        assert!(nvim.is_some());
        assert_eq!(nvim.unwrap().scope, KnowledgeScope::User);
    }

    #[test]
    fn test_scan_config_defaults() {
        let config = ScanConfig::default();
        assert_eq!(config.max_files, MAX_FILES_PER_SCAN);
        assert_eq!(config.max_bytes, MAX_BYTES_PER_SCAN);
    }
}
