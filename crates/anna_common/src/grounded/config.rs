//! Config Discovery v7.4.0 - Honest Multi-source Config Discovery
//!
//! Sources (in order):
//! 1. pacman -Ql <package> - Files under /etc/ and config templates
//! 2. man pages - FILES and CONFIGURATION sections
//! 3. Arch Wiki local mirror - Config paths from documentation
//! 4. systemd unit files - For services: unit paths, drop-ins, EnvironmentFile
//!
//! Rules:
//! - Every config file must cite its source
//! - No guessing or "common paths"
//! - Paths from multiple sources are merged with combined attribution
//! - Precedence rules only stated when documented
//! - Missing files are shown as [not present] if documented

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use regex::Regex;

use super::arch_wiki::{ArchWikiIndex, PathType, resolve_user_path};

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
    /// Documented precedence rules
    pub precedence_rules: Vec<PrecedenceRule>,
    /// Whether any config was discovered
    pub has_configs: bool,
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
                    // /etc files (config files)
                    if path.starts_with("/etc/") && !path.ends_with('/') {
                        configs.push(ConfigFile {
                            path: path.to_string(),
                            source: source.clone(),
                            exists: Path::new(path).exists(),
                            is_user_config: false,
                            is_directory: false,
                        });
                    }
                    // Config templates in /usr/share/<pkg>/ or /usr/lib/<pkg>/
                    else if (path.starts_with("/usr/share/") || path.starts_with("/usr/lib/"))
                        && is_config_template(path)
                    {
                        configs.push(ConfigFile {
                            path: path.to_string(),
                            source: source.clone(),
                            exists: Path::new(path).exists(),
                            is_user_config: false,
                            is_directory: false,
                        });
                    }
                }
            }
        }
    }

    configs
}

/// Check if a path looks like a config template
fn is_config_template(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".conf")
        || lower.ends_with(".ini")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
        || lower.ends_with(".toml")
        || lower.ends_with(".cfg")
        || lower.ends_with("rc")
        || lower.contains("/defaults")
        || lower.contains("/config")
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
fn extract_paths_from_text(
    text: &str,
    source: &str,
    configs: &mut Vec<ConfigFile>,
    seen: &mut HashMap<String, bool>,
) {
    // Regex for system paths
    let sys_re = Regex::new(r"(/etc/[a-zA-Z0-9_./-]+)").unwrap();
    let usr_share_re = Regex::new(r"(/usr/share/[a-zA-Z0-9_./-]+\.(conf|ini|yaml|yml|toml|cfg))").unwrap();

    // Regex for user paths
    let user_re = Regex::new(r"(~/[a-zA-Z0-9_./-]+|\$HOME/[a-zA-Z0-9_./-]+|\$XDG_CONFIG_HOME/[a-zA-Z0-9_./-]+|~\.[a-zA-Z0-9_]+)").unwrap();

    // System paths from /etc/
    for cap in sys_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if looks_like_config_path(path) && !seen.contains_key(path) {
            seen.insert(path.to_string(), false);
            configs.push(ConfigFile {
                path: path.to_string(),
                source: source.to_string(),
                exists: Path::new(path).exists(),
                is_user_config: false,
                is_directory: path.ends_with('/') || Path::new(path).is_dir(),
            });
        }
    }

    // Templates in /usr/share/
    for cap in usr_share_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if !seen.contains_key(path) {
            seen.insert(path.to_string(), false);
            configs.push(ConfigFile {
                path: path.to_string(),
                source: source.to_string(),
                exists: Path::new(path).exists(),
                is_user_config: false,
                is_directory: false,
            });
        }
    }

    // User paths
    for cap in user_re.captures_iter(text) {
        let path = cap.get(1).unwrap().as_str();
        if looks_like_user_config_path(path) && !seen.contains_key(path) {
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
            });
        }
    }
}

/// Normalize user path to tilde form
fn normalize_user_path(path: &str) -> String {
    if path.starts_with("$HOME/") {
        format!("~/{}", &path[6..])
    } else if path.starts_with("$XDG_CONFIG_HOME/") {
        format!("~/.config/{}", &path[17..])
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
                is_directory: false,
            });
        }

        // Also extract precedence rules from wiki content
        if let Ok(content) = std::fs::read_to_string(&file_path) {
            extract_precedence_from_text(&content, "Arch Wiki", &mut precedence);
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
            info.related_configs.push(ConfigFile {
                path: env_path.to_string(),
                source: "unit file EnvironmentFile=".to_string(),
                exists: Path::new(env_path).exists(),
                is_user_config: false,
                is_directory: false,
            });
        }
    }

    // Look for ExecStart with config file arguments
    let exec_re = Regex::new(r"ExecStart=.+?(-c|--config|--conf)\s+(\S+)").unwrap();
    for cap in exec_re.captures_iter(&content) {
        let config_path = cap.get(2).unwrap().as_str().trim();
        if config_path.starts_with('/') {
            info.related_configs.push(ConfigFile {
                path: config_path.to_string(),
                source: "unit file ExecStart".to_string(),
                exists: Path::new(config_path).exists(),
                is_user_config: false,
                is_directory: false,
            });
        }
    }
}

// ============================================================================
// Combined discovery
// ============================================================================

/// Discover all config files for an object from all sources
pub fn discover_config_info(name: &str) -> ConfigInfo {
    let mut path_map: HashMap<String, (Vec<String>, bool, bool, bool)> = HashMap::new();
    let mut all_precedence = Vec::new();

    // Source 1: pacman -Ql
    for cfg in discover_from_pacman(name) {
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config, cfg.is_directory));
        entry.0.push(cfg.source);
        entry.1 = entry.1 || cfg.exists;
    }

    // Source 2: man pages
    let (man_configs, man_precedence) = discover_from_man(name);
    for cfg in man_configs {
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config, cfg.is_directory));
        if !entry.0.iter().any(|s| s.starts_with("man ")) {
            entry.0.push(cfg.source);
        }
        entry.1 = entry.1 || cfg.exists;
    }
    all_precedence.extend(man_precedence);

    // Source 3: Arch Wiki
    let (wiki_configs, wiki_precedence) = discover_from_arch_wiki(name);
    for cfg in wiki_configs {
        let entry = path_map.entry(cfg.path.clone())
            .or_insert_with(|| (Vec::new(), cfg.exists, cfg.is_user_config, cfg.is_directory));
        if !entry.0.contains(&"Arch Wiki".to_string()) {
            entry.0.push(cfg.source);
        }
        entry.1 = entry.1 || cfg.exists;
    }
    all_precedence.extend(wiki_precedence);

    // Convert to lists
    let mut system_configs = Vec::new();
    let mut user_configs = Vec::new();

    for (path, (sources, exists, is_user, is_dir)) in path_map {
        let cfg = ConfigFile {
            path,
            source: sources.join(", "),
            exists,
            is_user_config: is_user,
            is_directory: is_dir,
        };

        if is_user {
            user_configs.push(cfg);
        } else {
            system_configs.push(cfg);
        }
    }

    // Sort alphabetically
    system_configs.sort_by(|a, b| a.path.cmp(&b.path));
    user_configs.sort_by(|a, b| a.path.cmp(&b.path));

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

    let has_configs = !system_configs.is_empty() || !user_configs.is_empty();

    ConfigInfo {
        system_configs,
        user_configs,
        precedence_rules: all_precedence,
        has_configs,
    }
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
}
