//! Config Discovery v7.8.0 - Config Hygiene and Precise Sources
//!
//! Sources (priority order):
//! 1. filesystem - Direct filesystem checks for conventional locations
//! 2. pacman -Ql <package> - Files under /etc/ and config templates
//! 3. man pages - FILES and CONFIGURATION sections
//! 4. Arch Wiki local mirror - Config paths from documentation (if installed)
//! 5. systemd unit files - For services: unit paths, drop-ins, EnvironmentFile
//!
//! Rules:
//! - Every config file must cite its source
//! - Filesystem is always checked and trusted first
//! - Paths from multiple sources are merged with combined attribution
//! - Precedence rules only stated when documented
//! - Status indicators: [present], [missing], [recommended]
//! - v7.5.0: Improved path filtering, better deduplication
//! - v7.6.0: Honest source reporting when Arch Wiki not available
//! - v7.6.1: Identity-focused filtering, ranked paths, lean output
//! - v7.8.0: Filesystem source, precise status indicators, source attribution per path

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use regex::Regex;

use super::arch_wiki::{ArchWikiIndex, PathType, resolve_user_path};

/// Status indicator for config files - v7.8.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigStatus {
    /// File exists on the filesystem
    Present,
    /// File is documented but does not exist
    Missing,
    /// File is recommended by docs but does not exist
    Recommended,
}

impl ConfigStatus {
    /// Format for display
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigStatus::Present => "present",
            ConfigStatus::Missing => "missing",
            ConfigStatus::Recommended => "recommended",
        }
    }
}

/// Config file category - v7.8.0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigCategory {
    /// System-wide config (/etc, /usr/share)
    System,
    /// User config (~/, $XDG_CONFIG_HOME)
    User,
    /// Other common locations (templates, drop-ins, etc.)
    Other,
}

/// A discovered config file with its source(s)
#[derive(Debug, Clone)]
pub struct ConfigFile {
    /// The path (absolute or ~-relative for user paths)
    pub path: String,
    /// Sources of this discovery (e.g., "pacman -Ql vim, man vim")
    pub source: String,
    /// Whether the file currently exists on the system
    pub exists: bool,
    /// Whether this is a user config (~/) or system config (/etc)
    pub is_user_config: bool,
    /// Whether this is a directory (for drop-in dirs)
    pub is_directory: bool,
    /// Status indicator - v7.8.0
    pub status: ConfigStatus,
    /// Category (System/User/Other) - v7.8.0
    pub category: ConfigCategory,
    /// Whether this is a recommended/default path from docs - v7.8.0
    pub is_recommended: bool,
}

/// Documented precedence rule
#[derive(Debug, Clone)]
pub struct PrecedenceRule {
    /// Description of the precedence (e.g., "user config overrides system config")
    pub description: String,
    /// Source of this rule (e.g., "man vim", "Arch Wiki")
    pub source: String,
    /// Whether this is a conventional assumption or explicit documentation
    pub is_conventional: bool,
}

/// Complete config info for an object
#[derive(Debug, Clone, Default)]
pub struct ConfigInfo {
    /// System-level config files (/etc/, /usr/share/, etc)
    pub system_configs: Vec<ConfigFile>,
    /// User-level config files (~/, $XDG_CONFIG_HOME, etc)
    pub user_configs: Vec<ConfigFile>,
    /// Other common locations (templates, drop-ins, portals, etc.) - v7.8.0
    pub other_configs: Vec<ConfigFile>,
    /// Documented precedence rules
    pub precedence_rules: Vec<PrecedenceRule>,
    /// Whether any config was discovered
    pub has_configs: bool,
    /// What sources were used for discovery (v7.6.0)
    pub source_description: String,
}

/// Check if local Arch Wiki docs are available
pub fn is_arch_wiki_available() -> bool {
    ArchWikiIndex::detect().enabled
}

/// Get source description based on what's available - v7.8.0
pub fn get_source_description() -> String {
    let wiki_available = is_arch_wiki_available();
    if wiki_available {
        "filesystem, pacman -Ql, man pages, local Arch Wiki".to_string()
    } else {
        "filesystem, pacman -Ql, man pages".to_string()
    }
}

// ============================================================================
// Source 0: Filesystem discovery - v7.8.0
// ============================================================================

/// Discover config files by checking conventional filesystem locations
/// This is the most trusted source - if a file exists, it exists.
pub fn discover_from_filesystem(name: &str) -> Vec<ConfigFile> {
    let mut configs = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg_config = std::env::var("XDG_CONFIG_HOME")
        .unwrap_or_else(|_| format!("{}/.config", home));

    // Get identity aliases for broader matching
    let aliases = get_identity_aliases(name);
    let mut names_to_check: Vec<&str> = vec![name];
    for alias in &aliases {
        names_to_check.push(alias);
    }

    for check_name in &names_to_check {
        // System locations to check
        let system_paths = [
            format!("/etc/{}", check_name),
            format!("/etc/{}.conf", check_name),
            format!("/etc/{}/{}.conf", check_name, check_name),
            format!("/usr/share/{}", check_name),
            format!("/usr/share/{}/{}.conf", check_name, check_name),
        ];

        for path in &system_paths {
            if Path::new(path).exists() {
                configs.push(ConfigFile {
                    path: path.clone(),
                    source: "filesystem".to_string(),
                    exists: true,
                    is_user_config: false,
                    is_directory: Path::new(path).is_dir(),
                    status: ConfigStatus::Present,
                    category: ConfigCategory::System,
                    is_recommended: false,
                });
            }
        }

        // User locations to check
        let user_paths = [
            format!("{}/{}", xdg_config, check_name),
            format!("{}/{}/{}.conf", xdg_config, check_name, check_name),
            format!("{}/.{}", home, check_name),
            format!("{}/.{}rc", home, check_name),
        ];

        for path in &user_paths {
            if Path::new(path).exists() {
                // Convert to tilde form for display
                let display_path = if path.starts_with(&home) {
                    path.replacen(&home, "~", 1)
                } else {
                    path.clone()
                };
                configs.push(ConfigFile {
                    path: display_path,
                    source: "filesystem".to_string(),
                    exists: true,
                    is_user_config: true,
                    is_directory: Path::new(path).is_dir(),
                    status: ConfigStatus::Present,
                    category: ConfigCategory::User,
                    is_recommended: false,
                });
            }
        }
    }

    configs
}

/// Service-specific config info
#[derive(Debug, Clone, Default)]
pub struct ServiceConfigInfo {
    /// Main unit file path
    pub unit_file: Option<ConfigFile>,
    /// Override unit file in /etc/systemd/system/
    pub override_unit: Option<ConfigFile>,
    /// Drop-in directory
    pub drop_in_dir: Option<ConfigFile>,
    /// Drop-in files within the directory
    pub drop_in_files: Vec<ConfigFile>,
    /// Related config files (from EnvironmentFile=, Config=, etc)
    pub related_configs: Vec<ConfigFile>,
    /// Standard package/command configs
    pub package_configs: ConfigInfo,
}

// ============================================================================
// Source 1: pacman -Ql discovery
// ============================================================================

/// Discover config files from pacman -Ql
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
                    // Include: /etc files (configs) or /usr/{share,lib} templates
                    let is_etc_config = path.starts_with("/etc/") && !path.ends_with('/');
                    let is_usr_template = (path.starts_with("/usr/share/") || path.starts_with("/usr/lib/"))
                        && is_config_template(path);

                    if is_etc_config || is_usr_template {
                        let exists = Path::new(path).exists();
                        // v7.8.0: Determine category
                        let category = if path.starts_with("/etc/") {
                            ConfigCategory::System
                        } else {
                            ConfigCategory::Other // Templates go to Other
                        };
                        configs.push(ConfigFile {
                            path: path.to_string(),
                            source: source.clone(),
                            exists,
                            is_user_config: false,
                            is_directory: false,
                            status: if exists { ConfigStatus::Present } else { ConfigStatus::Missing },
                            category,
                            is_recommended: false,
                        });
                    }
                }
            }
        }
    }

    configs
}

/// Check if a path looks like a config template (stricter for pacman -Ql filtering)
fn is_config_template(path: &str) -> bool {
    let lower = path.to_lowercase();
    // Must be an actual config file, not just any file with /config/ in path
    // Avoid Cargo.toml, test files, etc.
    let is_config_ext = lower.ends_with(".conf")
        || lower.ends_with(".ini")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".cfg")
        || lower.ends_with("rc");

    // For templates specifically, check for defaults dir
    let is_defaults_template = lower.contains("/defaults/") || lower.ends_with("/defaults");

    // Exclude test files, cargo files, and other non-config items
    let is_excluded = lower.contains("/test")
        || lower.contains("cargo.toml")
        || lower.contains(".cocci")
        || lower.contains("/coccinelle/")
        || lower.contains("/credential/")
        || lower.contains("netrc")
        || lower.contains("/libgit");

    (is_config_ext || is_defaults_template) && !is_excluded
}

// ============================================================================
// Source 2: man page discovery
// ============================================================================

/// Discover config files from man pages (FILES and CONFIGURATION sections)
pub fn discover_from_man(command: &str) -> (Vec<ConfigFile>, Vec<PrecedenceRule>) {
    let output = Command::new("man")
        .args(["-P", "cat", command])
        .env("MANWIDTH", "1000")
        .output();

    let mut configs = Vec::new();
    let mut precedence = Vec::new();
    let mut seen_paths: HashMap<String, bool> = HashMap::new();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let source = format!("man {}", command);
            let content = stdout.to_string();

            // Extract from FILES section
            if let Some(files_section) = extract_man_section(&content, "FILES") {
                extract_paths_from_text(&files_section, &source, &mut configs, &mut seen_paths);

                // Look for precedence hints in FILES section
                extract_precedence_from_text(&files_section, &source, &mut precedence);
            }

            // Extract from CONFIGURATION section
            if let Some(config_section) = extract_man_section(&content, "CONFIGURATION") {
                extract_paths_from_text(&config_section, &source, &mut configs, &mut seen_paths);
                extract_precedence_from_text(&config_section, &source, &mut precedence);
            }

            // Also scan the whole document for paths (fallback)
            extract_paths_from_text(&content, &source, &mut configs, &mut seen_paths);
        }
    }

    (configs, precedence)
}

/// Extract a section from man page content
fn extract_man_section(content: &str, section_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_section = false;
    let mut section_content = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        // Section headers are typically ALL CAPS at the start of a line
        if trimmed == section_name || trimmed == section_name.to_uppercase() {
            in_section = true;
            continue;
        }

        // End section when we hit another header (all caps, no indent)
        if in_section && !trimmed.is_empty() && trimmed.chars().all(|c| c.is_uppercase() || c.is_whitespace())
            && trimmed.len() > 2 && !trimmed.starts_with('/') {
            break;
        }

        if in_section {
            section_content.push(line);
        }
    }

    if section_content.is_empty() {
        None
    } else {
        Some(section_content.join("\n"))
    }
}

/// Extract config paths from text
/// v7.6.1: Added identity parameter for filtering
fn extract_paths_from_text(
    text: &str,
    source: &str,
    configs: &mut Vec<ConfigFile>,
    seen: &mut HashMap<String, bool>,
) {
    extract_paths_from_text_for_identity(text, source, configs, seen, None)
}

/// Extract config paths from text, filtered by identity - v7.8.0
fn extract_paths_from_text_for_identity(
    text: &str,
    source: &str,
    configs: &mut Vec<ConfigFile>,
    seen: &mut HashMap<String, bool>,
    identity: Option<&str>,
) {
    // Regex for system paths
    let sys_re = Regex::new(r"(/etc/[a-zA-Z0-9_./-]+)").unwrap();
    let usr_share_re = Regex::new(r"(/usr/share/[a-zA-Z0-9_./-]+\.(conf|ini|yaml|yml|toml|cfg))").unwrap();

    // Regex for user paths
    let user_re = Regex::new(r"(~/[a-zA-Z0-9_./-]+|\$HOME/[a-zA-Z0-9_./-]+|\$XDG_CONFIG_HOME/[a-zA-Z0-9_./-]+|~\.[a-zA-Z0-9_]+)").unwrap();

    // Check if source indicates recommendation
    let is_recommended_source = source.contains("Arch Wiki") || source.contains("recommended");

    // System paths from /etc/
    for cap in sys_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if looks_like_config_path(path) && !seen.contains_key(path) {
            // v7.6.1: Filter by identity
            if let Some(id) = identity {
                if !path_belongs_to_identity(path, id) {
                    continue;
                }
            }
            let exists = Path::new(path).exists();
            seen.insert(path.to_string(), false);
            configs.push(ConfigFile {
                path: path.to_string(),
                source: source.to_string(),
                exists,
                is_user_config: false,
                is_directory: path.ends_with('/') || Path::new(path).is_dir(),
                status: if exists { ConfigStatus::Present } else if is_recommended_source { ConfigStatus::Recommended } else { ConfigStatus::Missing },
                category: ConfigCategory::System,
                is_recommended: is_recommended_source && !exists,
            });
        }
    }

    // Templates in /usr/share/
    for cap in usr_share_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if !seen.contains_key(path) {
            // v7.6.1: Filter by identity
            if let Some(id) = identity {
                if !path_belongs_to_identity(path, id) {
                    continue;
                }
            }
            let exists = Path::new(path).exists();
            seen.insert(path.to_string(), false);
            configs.push(ConfigFile {
                path: path.to_string(),
                source: source.to_string(),
                exists,
                is_user_config: false,
                is_directory: false,
                status: if exists { ConfigStatus::Present } else { ConfigStatus::Missing },
                category: ConfigCategory::Other, // Templates go to Other
                is_recommended: false,
            });
        }
    }

    // User paths
    for cap in user_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if looks_like_user_config_path(path) && !seen.contains_key(path) {
            // v7.6.1: Filter by identity
            if let Some(id) = identity {
                if !path_belongs_to_identity(path, id) {
                    continue;
                }
            }
            let normalized = normalize_user_path(path);
            let exists = resolve_user_path(&normalized)
                .map(|p| p.exists())
                .unwrap_or(false);
            seen.insert(normalized.clone(), true);
            configs.push(ConfigFile {
                path: normalized,
                source: source.to_string(),
                exists,
                is_user_config: true,
                is_directory: false,
                status: if exists { ConfigStatus::Present } else if is_recommended_source { ConfigStatus::Recommended } else { ConfigStatus::Missing },
                category: ConfigCategory::User,
                is_recommended: is_recommended_source && !exists,
            });
        }
    }
}

/// Check if a path belongs to a specific identity - v7.6.1
/// Uses word boundary matching to avoid "nvim" matching "vim"
fn path_belongs_to_identity(path: &str, identity: &str) -> bool {
    let path_lower = path.to_lowercase();
    let id_lower = identity.to_lowercase();

    // v7.6.1: Exclude paths that belong to other well-known tools
    // Even if they contain the identity name (e.g., uwsm/env-hyprland)
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

    // Known aliases
    let aliases = get_identity_aliases(&id_lower);
    for alias in aliases {
        if path_contains_identity_segment(&path_lower, &alias) {
            return true;
        }
    }

    false
}

/// Check if path belongs to a different well-known tool - v7.6.1
/// Used to filter out paths like /uwsm/env-hyprland from hyprland results
fn is_path_for_other_tool(path: &str, current_identity: &str) -> bool {
    // List of well-known tools that might appear in docs but are separate software
    let other_tools = [
        "uwsm", "mako", "waybar", "dunst", "rofi", "wofi", "swaylock",
        "swayidle", "wlogout", "eww", "ags", "nwg", "wlr", "sway",
        "kitty", "alacritty", "foot", "wezterm",
    ];

    for tool in &other_tools {
        // Skip if the tool is the current identity or an alias of it
        if *tool == current_identity {
            continue;
        }
        // Check if path contains this other tool as a directory segment
        if path.contains(&format!("/{}/", tool)) || path.contains(&format!("/.{}", tool)) {
            return true;
        }
    }

    false
}

/// Check if path is clearly not a config file - v7.6.1
fn is_non_config_path(path: &str) -> bool {
    // Paths that are clearly not config files
    let non_config_patterns = [
        "/scripts/",
        "/wallpapers/",
        "/wallpapers",
        "/icons/",
        "/themes/",
        "/backgrounds/",
        "/screenshots/",
        "/cache/",
        "/logs/",
    ];

    for pattern in &non_config_patterns {
        if path.contains(pattern) {
            return true;
        }
    }

    // Also exclude paths ending in common non-config extensions
    if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") ||
       path.ends_with(".svg") || path.ends_with(".gif") || path.ends_with(".ico") ||
       path.ends_with(".sh") || path.ends_with(".bash") || path.ends_with(".zsh") {
        return true;
    }

    false
}

/// Check if path contains identity as a distinct segment
/// Matches: /vim/, /vim., .vimrc, /vim at end, but NOT /nvim/
fn path_contains_identity_segment(path: &str, identity: &str) -> bool {
    // Find all occurrences of identity in path
    let mut start = 0;
    while let Some(pos) = path[start..].find(identity) {
        let abs_pos = start + pos;
        let end_pos = abs_pos + identity.len();

        // Check character before (should be / or . or start of string or ~)
        let before_ok = abs_pos == 0 ||
            path.chars().nth(abs_pos - 1).map(|c| c == '/' || c == '.' || c == '-' || c == '_' || c == '~').unwrap_or(false);

        // Check character after (should be / or . or - or _ or rc or end of string)
        let after_char = path.chars().nth(end_pos);
        let after_ok = end_pos >= path.len() ||
            after_char.map(|c| c == '/' || c == '.' || c == '-' || c == '_').unwrap_or(false) ||
            // Also match *rc pattern (e.g., .vimrc)
            (after_char == Some('r') && path.chars().nth(end_pos + 1) == Some('c'));

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
fn get_identity_aliases(identity: &str) -> Vec<String> {
    match identity {
        "hyprland" => vec!["hypr".to_string()],
        "vim" => vec!["vi".to_string(), "gvim".to_string()],
        "neovim" => vec!["nvim".to_string()],
        "networkmanager" => vec!["nm".to_string(), "network-manager".to_string()],
        "pulseaudio" => vec!["pulse".to_string()],
        "pipewire" => vec!["pw".to_string()],
        "bluetooth" | "bluez" => vec!["bt".to_string(), "bluetooth".to_string(), "bluez".to_string()],
        _ => vec![],
    }
}

/// Normalize user path to tilde form
fn normalize_user_path(path: &str) -> String {
    if let Some(stripped) = path.strip_prefix("$HOME/") {
        format!("~/{}", stripped)
    } else if let Some(stripped) = path.strip_prefix("$XDG_CONFIG_HOME/") {
        format!("~/.config/{}", stripped)
    } else {
        path.to_string()
    }
}

/// Extract precedence rules from text
fn extract_precedence_from_text(text: &str, source: &str, rules: &mut Vec<PrecedenceRule>) {
    let text_lower = text.to_lowercase();

    // Look for explicit precedence statements
    let patterns = [
        ("override", "overrides"),
        ("takes precedence", "takes precedence over"),
        ("falls back", "falls back to"),
        ("read first", "is read first"),
        ("read last", "is read last"),
        ("user.* config.* system", "user configuration overrides system"),
        ("system.* user", "system configuration is overridden by user"),
    ];

    for (pattern, _desc_hint) in patterns {
        if text_lower.contains(pattern) {
            // Find the sentence containing this pattern
            for sentence in text.split('.') {
                if sentence.to_lowercase().contains(pattern) {
                    let trimmed = sentence.trim();
                    if !trimmed.is_empty() && trimmed.len() < 200 {
                        rules.push(PrecedenceRule {
                            description: trimmed.to_string(),
                            source: source.to_string(),
                            is_conventional: false,
                        });
                        break;
                    }
                }
            }
        }
    }
}

// ============================================================================
// Source 3: Arch Wiki local mirror
// ============================================================================

/// Discover config files from Arch Wiki (if local mirror available)
/// v7.6.1: Now uses identity-filtered extraction
pub fn discover_from_arch_wiki(name: &str) -> (Vec<ConfigFile>, Vec<PrecedenceRule>) {
    let wiki = ArchWikiIndex::detect();
    if !wiki.enabled {
        return (Vec::new(), Vec::new());
    }

    let candidates = wiki.find_candidates(name, 3);
    let mut configs = Vec::new();
    let mut precedence = Vec::new();
    let mut seen_paths: HashMap<String, bool> = HashMap::new();

    for file_path in candidates {
        // v7.6.1: Use identity-filtered extraction
        let hints = wiki.extract_config_paths_for_identity(&file_path, Some(name));
        for hint in hints {
            if seen_paths.contains_key(&hint.path) {
                continue;
            }

            let (exists, is_user, category) = match hint.path_type {
                PathType::System => {
                    let exists = Path::new(&hint.path).exists();
                    let cat = if hint.path.starts_with("/etc/") {
                        ConfigCategory::System
                    } else {
                        ConfigCategory::Other
                    };
                    (exists, false, cat)
                },
                PathType::User => {
                    let exists = resolve_user_path(&hint.path)
                        .map(|p| p.exists())
                        .unwrap_or(false);
                    (exists, true, ConfigCategory::User)
                }
            };

            seen_paths.insert(hint.path.clone(), is_user);
            configs.push(ConfigFile {
                path: hint.path,
                source: "Arch Wiki".to_string(),
                exists,
                is_user_config: is_user,
                is_directory: false,
                status: if exists { ConfigStatus::Present } else { ConfigStatus::Recommended },
                category,
                is_recommended: !exists,
            });
        }

        // Also extract precedence rules from wiki content (HTML stripped internally)
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            let clean_content = super::arch_wiki::strip_html(&content);
            extract_precedence_from_text(&clean_content, "Arch Wiki", &mut precedence);
        }
    }

    (configs, precedence)
}

// ============================================================================
// Source 4: systemd unit files
// ============================================================================

/// Discover config for a systemd service
pub fn discover_service_config(unit_name: &str) -> ServiceConfigInfo {
    let mut info = ServiceConfigInfo::default();

    // Ensure unit name has .service suffix
    let unit = if unit_name.ends_with(".service") {
        unit_name.to_string()
    } else {
        format!("{}.service", unit_name)
    };

    // Get unit file path using systemctl show
    let output = Command::new("systemctl")
        .args(["show", "-p", "FragmentPath,DropInPaths,UnitFileState", &unit])
        .output();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);

            for line in stdout.lines() {
                if let Some(path) = line.strip_prefix("FragmentPath=") {
                    if !path.is_empty() {
                        info.unit_file = Some(ConfigFile {
                            path: path.to_string(),
                            source: "systemctl show".to_string(),
                            exists: Path::new(path).exists(),
                            is_user_config: false,
                            is_directory: false,
                            status: if Path::new(path).exists() { ConfigStatus::Present } else { ConfigStatus::Missing },
                            category: ConfigCategory::System,
                            is_recommended: false,
                        });

                        // Parse the unit file for EnvironmentFile and other configs
                        parse_unit_file_for_configs(path, &mut info);
                    }
                }

                if let Some(paths) = line.strip_prefix("DropInPaths=") {
                    if !paths.is_empty() {
                        for path in paths.split_whitespace() {
                            if Path::new(path).is_dir() {
                                info.drop_in_dir = Some(ConfigFile {
                                    path: path.to_string(),
                                    source: "systemctl show".to_string(),
                                    exists: true,
                                    is_user_config: false,
                                    is_directory: true,
                                    status: ConfigStatus::Present,
                                    category: ConfigCategory::System,
                                    is_recommended: false,
                                });

                                // List files in drop-in directory
                                if let Ok(entries) = std::fs::read_dir(path) {
                                    for entry in entries.flatten() {
                                        let file_path = entry.path();
                                        if file_path.extension().map(|e| e == "conf").unwrap_or(false) {
                                            info.drop_in_files.push(ConfigFile {
                                                path: file_path.display().to_string(),
                                                source: "drop-in directory".to_string(),
                                                exists: true,
                                                is_user_config: false,
                                                is_directory: false,
                                                status: ConfigStatus::Present,
                                                category: ConfigCategory::System,
                                                is_recommended: false,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for override unit in /etc/systemd/system/
    let override_path = format!("/etc/systemd/system/{}", unit);
    if Path::new(&override_path).exists() {
        info.override_unit = Some(ConfigFile {
            path: override_path,
            source: "filesystem".to_string(),
            exists: true,
            is_user_config: false,
            is_directory: false,
            status: ConfigStatus::Present,
            category: ConfigCategory::System,
            is_recommended: false,
        });
    }

    // Check for drop-in directory if not already found
    if info.drop_in_dir.is_none() {
        let drop_in_path = format!("/etc/systemd/system/{}.d", unit);
        let exists = Path::new(&drop_in_path).is_dir();
        info.drop_in_dir = Some(ConfigFile {
            path: drop_in_path.clone(),
            source: "conventional".to_string(),
            exists,
            is_user_config: false,
            is_directory: true,
            status: if exists { ConfigStatus::Present } else { ConfigStatus::Recommended },
            category: ConfigCategory::System,
            is_recommended: !exists,
        });

        if exists {
            if let Ok(entries) = std::fs::read_dir(&drop_in_path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if file_path.extension().map(|e| e == "conf").unwrap_or(false) {
                        info.drop_in_files.push(ConfigFile {
                            path: file_path.display().to_string(),
                            source: "drop-in directory".to_string(),
                            exists: true,
                            is_user_config: false,
                            is_directory: false,
                            status: ConfigStatus::Present,
                            category: ConfigCategory::System,
                            is_recommended: false,
                        });
                    }
                }
            }
        }
    }

    // Get package configs for the base name
    let base_name = unit_name.trim_end_matches(".service");
    info.package_configs = discover_config_info(base_name);

    info
}

/// Parse a systemd unit file for EnvironmentFile and other config references
fn parse_unit_file_for_configs(path: &str, info: &mut ServiceConfigInfo) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    // Look for EnvironmentFile=
    let env_re = Regex::new(r"EnvironmentFile=(-?)(.+)").unwrap();
    for cap in env_re.captures_iter(&content) {
        let env_path = cap.get(2).unwrap().as_str().trim();
        if !env_path.is_empty() && env_path.starts_with('/') {
            let exists = Path::new(env_path).exists();
            info.related_configs.push(ConfigFile {
                path: env_path.to_string(),
                source: "unit file EnvironmentFile=".to_string(),
                exists,
                is_user_config: false,
                is_directory: false,
                status: if exists { ConfigStatus::Present } else { ConfigStatus::Missing },
                category: ConfigCategory::System,
                is_recommended: false,
            });
        }
    }

    // Look for ExecStart with config file arguments
    let exec_re = Regex::new(r"ExecStart=.+?(-c|--config|--conf)\s+(\S+)").unwrap();
    for cap in exec_re.captures_iter(&content) {
        let config_path = cap.get(2).unwrap().as_str().trim();
        if config_path.starts_with('/') {
            let exists = Path::new(config_path).exists();
            info.related_configs.push(ConfigFile {
                path: config_path.to_string(),
                source: "unit file ExecStart".to_string(),
                exists,
                is_user_config: false,
                is_directory: false,
                status: if exists { ConfigStatus::Present } else { ConfigStatus::Missing },
                category: ConfigCategory::System,
                is_recommended: false,
            });
        }
    }
}

// ============================================================================
// Combined discovery
// ============================================================================

/// Maximum config paths to show per category - v7.8.0
const MAX_SYSTEM_PATHS: usize = 6;
const MAX_USER_PATHS: usize = 6;
const MAX_OTHER_PATHS: usize = 4;

/// Config entry for merging - v7.8.0
#[derive(Clone)]
struct ConfigEntry {
    sources: Vec<String>,
    exists: bool,
    is_user_config: bool,
    is_directory: bool,
    category: ConfigCategory,
    is_recommended: bool,
}

/// Discover all config files for an object from all sources
/// v7.8.0: Filesystem first, proper category separation, precise sources
pub fn discover_config_info(name: &str) -> ConfigInfo {
    let mut path_map: HashMap<String, ConfigEntry> = HashMap::new();
    let mut all_precedence = Vec::new();

    // Source 0: Filesystem (most trusted, always first) - v7.8.0
    for cfg in discover_from_filesystem(name) {
        if !path_belongs_to_identity(&cfg.path, name) {
            continue;
        }
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| ConfigEntry {
                sources: Vec::new(),
                exists: cfg.exists,
                is_user_config: cfg.is_user_config,
                is_directory: cfg.is_directory,
                category: cfg.category,
                is_recommended: cfg.is_recommended,
            });
        if !entry.sources.contains(&cfg.source) {
            entry.sources.push(cfg.source);
        }
        entry.exists = entry.exists || cfg.exists;
    }

    // Source 1: pacman -Ql
    for cfg in discover_from_pacman(name) {
        if !path_belongs_to_identity(&cfg.path, name) {
            continue;
        }
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| ConfigEntry {
                sources: Vec::new(),
                exists: cfg.exists,
                is_user_config: cfg.is_user_config,
                is_directory: cfg.is_directory,
                category: cfg.category,
                is_recommended: cfg.is_recommended,
            });
        if !entry.sources.iter().any(|s| s.starts_with("pacman")) {
            entry.sources.push(cfg.source);
        }
        entry.exists = entry.exists || cfg.exists;
    }

    // Source 2: man pages (filtered by identity)
    let (man_configs, man_precedence) = discover_from_man_filtered(name);
    for cfg in man_configs {
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| ConfigEntry {
                sources: Vec::new(),
                exists: cfg.exists,
                is_user_config: cfg.is_user_config,
                is_directory: cfg.is_directory,
                category: cfg.category,
                is_recommended: cfg.is_recommended,
            });
        if !entry.sources.iter().any(|s| s.starts_with("man ")) {
            entry.sources.push(cfg.source);
        }
        entry.exists = entry.exists || cfg.exists;
        entry.is_recommended = entry.is_recommended || cfg.is_recommended;
    }
    all_precedence.extend(man_precedence);

    // Source 3: Arch Wiki (already filtered by identity)
    let (wiki_configs, wiki_precedence) = discover_from_arch_wiki(name);
    for cfg in wiki_configs {
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| ConfigEntry {
                sources: Vec::new(),
                exists: cfg.exists,
                is_user_config: cfg.is_user_config,
                is_directory: cfg.is_directory,
                category: cfg.category,
                is_recommended: cfg.is_recommended,
            });
        if !entry.sources.contains(&"Arch Wiki".to_string()) {
            entry.sources.push(cfg.source);
        }
        entry.exists = entry.exists || cfg.exists;
        entry.is_recommended = entry.is_recommended || cfg.is_recommended;
    }
    all_precedence.extend(wiki_precedence);

    // Convert to categorized lists - v7.8.0
    let mut system_configs = Vec::new();
    let mut user_configs = Vec::new();
    let mut other_configs = Vec::new();

    for (path, entry) in path_map {
        let status = if entry.exists {
            ConfigStatus::Present
        } else if entry.is_recommended {
            ConfigStatus::Recommended
        } else {
            ConfigStatus::Missing
        };

        let cfg = ConfigFile {
            path,
            source: entry.sources.join(", "),
            exists: entry.exists,
            is_user_config: entry.is_user_config,
            is_directory: entry.is_directory,
            status,
            category: entry.category,
            is_recommended: entry.is_recommended,
        };

        match entry.category {
            ConfigCategory::System => system_configs.push(cfg),
            ConfigCategory::User => user_configs.push(cfg),
            ConfigCategory::Other => other_configs.push(cfg),
        }
    }

    // Sort: existing first, then by path
    let sort_fn = |a: &ConfigFile, b: &ConfigFile| -> std::cmp::Ordering {
        match (a.exists, b.exists) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path.cmp(&b.path),
        }
    };
    system_configs.sort_by(sort_fn);
    user_configs.sort_by(sort_fn);
    other_configs.sort_by(sort_fn);

    // v7.8.0: Apply limits per category
    system_configs = limit_configs(&system_configs, name, MAX_SYSTEM_PATHS);
    user_configs = limit_configs(&user_configs, name, MAX_USER_PATHS);
    other_configs = limit_configs(&other_configs, name, MAX_OTHER_PATHS);

    // Add conventional precedence if we have both user and system configs
    if !system_configs.is_empty() && !user_configs.is_empty() {
        all_precedence.push(PrecedenceRule {
            description: "User configs (~/) typically override system configs (/etc/)".to_string(),
            source: "conventional".to_string(),
            is_conventional: true,
        });
    }

    // Deduplicate precedence rules
    let mut seen_desc: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_precedence.retain(|r| seen_desc.insert(r.description.clone()));

    let has_configs = !system_configs.is_empty() || !user_configs.is_empty() || !other_configs.is_empty();
    let source_description = get_source_description();

    ConfigInfo {
        system_configs,
        user_configs,
        other_configs,
        precedence_rules: all_precedence,
        has_configs,
        source_description,
    }
}

/// Limit configs to a maximum count, preferring existing and recommended - v7.8.0
fn limit_configs(configs: &[ConfigFile], _identity: &str, limit: usize) -> Vec<ConfigFile> {
    if configs.len() <= limit {
        return configs.to_vec();
    }

    // Priority: existing first, then recommended, then others
    let mut existing: Vec<_> = configs.iter().filter(|c| c.exists).cloned().collect();
    let recommended: Vec<_> = configs.iter().filter(|c| !c.exists && c.is_recommended).cloned().collect();
    let missing: Vec<_> = configs.iter().filter(|c| !c.exists && !c.is_recommended).cloned().collect();

    // Take as many existing as we can
    existing.truncate(limit);
    let remaining = limit.saturating_sub(existing.len());

    // Fill remaining with recommended
    let mut result = existing;
    for cfg in recommended.into_iter().take(remaining) {
        result.push(cfg);
    }
    let remaining = limit.saturating_sub(result.len());

    // Fill any still remaining with missing
    for cfg in missing.into_iter().take(remaining) {
        result.push(cfg);
    }

    result
}

/// Discover from man pages with identity filtering - v7.6.1
fn discover_from_man_filtered(name: &str) -> (Vec<ConfigFile>, Vec<PrecedenceRule>) {
    let output = Command::new("man")
        .args(["-P", "cat", name])
        .env("MANWIDTH", "1000")
        .output();

    let mut configs = Vec::new();
    let mut precedence = Vec::new();
    let mut seen_paths: HashMap<String, bool> = HashMap::new();

    if let Ok(result) = output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let source = format!("man {}", name);
            let content = stdout.to_string();

            // Extract from FILES section
            if let Some(files_section) = extract_man_section(&content, "FILES") {
                extract_paths_from_text_for_identity(&files_section, &source, &mut configs, &mut seen_paths, Some(name));
                extract_precedence_from_text(&files_section, &source, &mut precedence);
            }

            // Extract from CONFIGURATION section
            if let Some(config_section) = extract_man_section(&content, "CONFIGURATION") {
                extract_paths_from_text_for_identity(&config_section, &source, &mut configs, &mut seen_paths, Some(name));
                extract_precedence_from_text(&config_section, &source, &mut precedence);
            }
        }
    }

    (configs, precedence)
}

/// Legacy function for backwards compatibility
pub fn discover_config_files(name: &str) -> Vec<ConfigFile> {
    let info = discover_config_info(name);
    let mut all = info.system_configs;
    all.extend(info.user_configs);
    all
}

// ============================================================================
// Validation helpers
// ============================================================================

/// Check if a path looks like a valid system config file path
pub fn looks_like_config_path(path: &str) -> bool {
    if !path.starts_with("/etc/") {
        return false;
    }

    let after_etc = &path[5..];
    if after_etc.is_empty() {
        return false;
    }

    // Directories are OK for drop-ins
    // But filter out obvious non-configs
    if path.contains('*') || path.contains('$') || path.contains('{') || path.contains('<') {
        return false;
    }

    if let Some(c) = after_etc.chars().next() {
        if !c.is_alphanumeric() && c != '.' {
            return false;
        }
    }

    true
}

/// Check if a user path looks valid
pub fn looks_like_user_config_path(path: &str) -> bool {
    if !path.starts_with("~/") && !path.starts_with("$HOME/") && !path.starts_with("$XDG_CONFIG_HOME/") {
        return false;
    }

    if path.len() < 3 {
        return false;
    }

    if path.contains('*') || path.contains('{') || path.contains('<') {
        return false;
    }

    true
}

// ============================================================================
// Config highlights for KDB overview
// ============================================================================

/// Summary of config status across all packages
#[derive(Debug, Clone, Default)]
pub struct ConfigHighlights {
    /// Packages with user configs present
    pub user_configs_present: Vec<String>,
    /// Services with drop-in overrides
    pub services_with_overrides: Vec<(String, String)>, // (service, override path)
    /// Packages using only default config
    pub default_config_only: Vec<String>,
}

/// Get config highlights for KDB overview
pub fn get_config_highlights(packages: &[String], services: &[String]) -> ConfigHighlights {
    let mut highlights = ConfigHighlights::default();

    // Check packages for user configs
    for pkg in packages.iter().take(50) { // Limit for performance
        let info = discover_config_info(pkg);
        if info.user_configs.iter().any(|c| c.exists) {
            highlights.user_configs_present.push(pkg.clone());
        } else if info.system_configs.iter().any(|c| c.exists) && !info.user_configs.iter().any(|c| c.exists) {
            highlights.default_config_only.push(pkg.clone());
        }
    }

    // Check services for overrides
    for svc in services.iter().take(30) { // Limit for performance
        let info = discover_service_config(svc);

        // Check for override unit or drop-ins
        if let Some(ref override_unit) = info.override_unit {
            if override_unit.exists {
                highlights.services_with_overrides.push((
                    svc.clone(),
                    format!("override in {}", override_unit.path),
                ));
                continue;
            }
        }

        if !info.drop_in_files.is_empty() {
            let drop_in_path = info.drop_in_dir
                .as_ref()
                .map(|d| d.path.clone())
                .unwrap_or_else(|| format!("/etc/systemd/system/{}.service.d/", svc));
            highlights.services_with_overrides.push((
                svc.clone(),
                format!("drop-in in {}", drop_in_path),
            ));
        }
    }

    // Sort and limit
    highlights.user_configs_present.sort();
    highlights.user_configs_present.truncate(10);
    highlights.default_config_only.sort();
    highlights.default_config_only.truncate(10);
    highlights.services_with_overrides.sort_by(|a, b| a.0.cmp(&b.0));
    highlights.services_with_overrides.truncate(10);

    highlights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_looks_like_config_path() {
        assert!(looks_like_config_path("/etc/pacman.conf"));
        assert!(looks_like_config_path("/etc/ssh/sshd_config"));
        assert!(looks_like_config_path("/etc/X11/xorg.conf"));
        assert!(looks_like_config_path("/etc/systemd/system/foo.service.d/"));

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
        assert!(looks_like_user_config_path("$HOME/.bashrc"));
        assert!(looks_like_user_config_path("$XDG_CONFIG_HOME/hypr/hyprland.conf"));

        // Invalid
        assert!(!looks_like_user_config_path("~/"));
        assert!(!looks_like_user_config_path("~"));
        assert!(!looks_like_user_config_path("~/.config/*"));
    }

    #[test]
    fn test_normalize_user_path() {
        assert_eq!(normalize_user_path("$HOME/.bashrc"), "~/.bashrc");
        assert_eq!(normalize_user_path("$XDG_CONFIG_HOME/nvim"), "~/.config/nvim");
        assert_eq!(normalize_user_path("~/.vimrc"), "~/.vimrc");
    }

    #[test]
    fn test_pacman_discovery() {
        // Test with a common package
        let configs = discover_from_pacman("pacman");
        for cfg in &configs {
            assert!(cfg.path.starts_with("/etc/") || cfg.path.starts_with("/usr/"));
            assert!(cfg.source.contains("pacman -Ql"));
            assert!(!cfg.is_user_config);
        }
    }

    #[test]
    fn test_is_config_template() {
        assert!(is_config_template("/usr/share/vim/vimrc.conf"));
        assert!(is_config_template("/usr/share/foo/defaults.conf"));
        assert!(is_config_template("/usr/lib/tlp/defaults.conf"));
        assert!(!is_config_template("/usr/bin/vim"));
    }

    // =========================================================================
    // Snow Leopard Config Hygiene Tests v7.6.1
    // =========================================================================

    #[test]
    fn test_identity_segment_matching_vim() {
        // vim should match .vimrc, /vim/, vim.conf
        assert!(path_contains_identity_segment("~/.vimrc", "vim"));
        assert!(path_contains_identity_segment("/etc/vim/vimrc", "vim"));
        assert!(path_contains_identity_segment("~/.vim/", "vim"));
        assert!(path_contains_identity_segment("/usr/share/vim/vimfiles", "vim"));

        // nvim directory should NOT match vim (nvim != vim)
        assert!(!path_contains_identity_segment("~/.config/nvim/init.lua", "vim"));

        // But sysinit.vim SHOULD match vim because the filename ends with .vim
        // This is correct - the filename genuinely contains "vim"
        assert!(path_contains_identity_segment("/etc/xdg/nvim/sysinit.vim", "vim"));
    }

    #[test]
    fn test_identity_segment_matching_nvim() {
        // nvim should match nvim paths
        assert!(path_contains_identity_segment("~/.config/nvim/init.lua", "nvim"));
        assert!(path_contains_identity_segment("/etc/xdg/nvim/sysinit.vim", "nvim"));

        // vim should NOT match nvim (no false negatives)
        assert!(!path_contains_identity_segment("~/.vimrc", "nvim"));
        assert!(!path_contains_identity_segment("/etc/vim/vimrc", "nvim"));
    }

    #[test]
    fn test_identity_segment_matching_hyprland() {
        // hyprland should match hypr* paths
        assert!(path_contains_identity_segment("~/.config/hypr/hyprland.conf", "hyprland"));
        assert!(path_belongs_to_identity("~/.config/hypr/hyprland.conf", "hyprland"));

        // unrelated wayland tools should NOT match
        assert!(!path_contains_identity_segment("~/.config/mako/config", "hyprland"));
        assert!(!path_contains_identity_segment("~/.config/uwsm/config", "hyprland"));
        assert!(!path_contains_identity_segment("~/.config/waybar/config", "hyprland"));

        // Verify identity filtering rejects unrelated paths
        assert!(!path_belongs_to_identity("~/.config/mako/config", "hyprland"));
        assert!(!path_belongs_to_identity("~/.config/uwsm/config", "hyprland"));
    }

    #[test]
    fn test_identity_segment_boundary_chars() {
        // Test various boundary characters
        assert!(path_contains_identity_segment("/etc/ssh/sshd_config", "ssh"));
        assert!(path_contains_identity_segment("~/.ssh/config", "ssh"));
        assert!(path_contains_identity_segment("/etc/ssh.conf", "ssh"));

        // Ensure partial matches don't work
        assert!(!path_contains_identity_segment("/etc/sshx/config", "ssh"));
        assert!(!path_contains_identity_segment("/etc/xssh/config", "ssh"));
    }

    #[test]
    fn test_identity_aliases() {
        // hyprland should also match hypr alias
        assert!(path_belongs_to_identity("~/.config/hypr/hyprland.conf", "hyprland"));

        // vim should match vi and gvim
        let vim_aliases = get_identity_aliases("vim");
        assert!(vim_aliases.contains(&"vi".to_string()));
        assert!(vim_aliases.contains(&"gvim".to_string()));

        // neovim should match nvim
        let neovim_aliases = get_identity_aliases("neovim");
        assert!(neovim_aliases.contains(&"nvim".to_string()));
    }

    #[test]
    fn test_limit_configs_prefers_existing() {
        let configs = vec![
            ConfigFile {
                path: "/etc/random/vim.conf".to_string(),
                source: "wiki".to_string(),
                exists: false,
                is_user_config: false,
                is_directory: false,
                status: ConfigStatus::Missing,
                category: ConfigCategory::System,
                is_recommended: false,
            },
            ConfigFile {
                path: "/etc/vim/vimrc".to_string(),
                source: "pacman -Ql vim".to_string(),
                exists: true,
                is_user_config: false,
                is_directory: false,
                status: ConfigStatus::Present,
                category: ConfigCategory::System,
                is_recommended: false,
            },
        ];

        let limited = limit_configs(&configs, "vim", 2);

        // Existing file should be included
        assert!(!limited.is_empty());
        assert!(limited.iter().any(|c| c.exists && c.path.contains("/etc/vim/")));
    }

    #[test]
    fn test_limit_configs_respects_limit() {
        let configs = vec![
            ConfigFile {
                path: "/etc/vim/vimrc".to_string(),
                source: "pacman".to_string(),
                exists: true,
                is_user_config: false,
                is_directory: false,
                status: ConfigStatus::Present,
                category: ConfigCategory::System,
                is_recommended: false,
            },
            ConfigFile {
                path: "/etc/pacman.conf".to_string(),
                source: "pacman".to_string(),
                exists: true,
                is_user_config: false,
                is_directory: false,
                status: ConfigStatus::Present,
                category: ConfigCategory::System,
                is_recommended: false,
            },
            ConfigFile {
                path: "/etc/third.conf".to_string(),
                source: "pacman".to_string(),
                exists: true,
                is_user_config: false,
                is_directory: false,
                status: ConfigStatus::Present,
                category: ConfigCategory::System,
                is_recommended: false,
            },
        ];

        let limited = limit_configs(&configs, "test", 2);

        // Should be limited to 2
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_config_discovery_limits_output() {
        // This is an integration test - actual discovery may vary by system
        // v7.8.0: Verify limits are applied
        let info = discover_config_info("nonexistent_package_xyz123");

        // No configs should be found for non-existent package
        assert!(!info.has_configs || info.system_configs.len() <= MAX_SYSTEM_PATHS);
        assert!(!info.has_configs || info.user_configs.len() <= MAX_USER_PATHS);
        assert!(!info.has_configs || info.other_configs.len() <= MAX_OTHER_PATHS);
    }

    // =========================================================================
    // Snow Leopard Config Hygiene Tests v7.6.1 - Extended
    // =========================================================================

    #[test]
    fn test_other_tool_filtering() {
        // uwsm/env-hyprland should NOT belong to hyprland (it's a uwsm config)
        assert!(!path_belongs_to_identity("~/.config/uwsm/env-hyprland", "hyprland"));
        assert!(!path_belongs_to_identity("/home/user/.config/uwsm/env-hyprland", "hyprland"));

        // mako configs should not show for hyprland
        assert!(!path_belongs_to_identity("~/.config/mako/config", "hyprland"));

        // waybar configs should not show for hyprland
        assert!(!path_belongs_to_identity("~/.config/waybar/config", "hyprland"));
        assert!(!path_belongs_to_identity("~/.config/waybar/style.css", "hyprland"));
    }

    #[test]
    fn test_non_config_path_filtering() {
        // Scripts should be filtered
        assert!(is_non_config_path("/home/user/.config/hypr/scripts/volume"));
        assert!(is_non_config_path("~/.config/hypr/scripts/backlight"));

        // Wallpapers should be filtered
        assert!(is_non_config_path("/home/user/.config/hypr/wallpapers"));
        assert!(is_non_config_path("~/.config/hypr/wallpapers/"));

        // Icons should be filtered
        assert!(is_non_config_path("/usr/share/icons/hicolor/icon.png"));

        // Config files should NOT be filtered
        assert!(!is_non_config_path("/home/user/.config/hypr/hyprland.conf"));
        assert!(!is_non_config_path("~/.vimrc"));
        assert!(!is_non_config_path("/etc/pacman.conf"));
    }

    #[test]
    fn test_path_belongs_excludes_scripts_and_wallpapers() {
        // Even if path contains identity, scripts/wallpapers should be excluded
        assert!(!path_belongs_to_identity("~/.config/hypr/scripts/volume", "hyprland"));
        assert!(!path_belongs_to_identity("~/.config/hypr/wallpapers/bg.png", "hyprland"));

        // But actual config files should still match
        assert!(path_belongs_to_identity("~/.config/hypr/hyprland.conf", "hyprland"));
    }

    #[test]
    fn test_hyprland_specific_hygiene() {
        // These paths must NOT match hyprland
        let bad_paths = [
            "~/.config/uwsm/env-hyprland",
            "~/.config/mako/config",
            "~/.config/waybar/config",
            "~/.config/dunst/dunstrc",
            "~/.config/rofi/config.rasi",
            "~/.config/hypr/scripts/volume",
            "~/.config/hypr/wallpapers/bg.png",
        ];

        for path in &bad_paths {
            assert!(
                !path_belongs_to_identity(path, "hyprland"),
                "Path '{}' should NOT match hyprland",
                path
            );
        }

        // These paths MUST match hyprland
        let good_paths = [
            "~/.config/hypr/hyprland.conf",
            "/usr/share/hypr/hyprland.conf",
            "~/.config/hypr/hypridle.conf",
            "~/.config/hypr/hyprpaper.conf",
        ];

        for path in &good_paths {
            assert!(
                path_belongs_to_identity(path, "hyprland"),
                "Path '{}' should match hyprland",
                path
            );
        }
    }

    #[test]
    fn test_vim_nvim_isolation() {
        // vim paths should match vim, not nvim
        assert!(path_belongs_to_identity("~/.vimrc", "vim"));
        assert!(path_belongs_to_identity("/etc/vim/vimrc", "vim"));
        assert!(!path_belongs_to_identity("~/.vimrc", "nvim"));

        // nvim paths should match nvim, not vim
        assert!(path_belongs_to_identity("~/.config/nvim/init.lua", "nvim"));
        assert!(!path_belongs_to_identity("~/.config/nvim/init.lua", "vim"));

        // nvim directory paths should not show up for vim queries
        assert!(!path_belongs_to_identity("/etc/xdg/nvim/sysinit.vim", "vim"));
    }
}
