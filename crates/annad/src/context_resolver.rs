//! Context resolver - detects missing references and auto-locates config files.
//!
//! Reduces wasted iterations by catching ambiguous questions upfront:
//! - "show me that file" → which file?
//! - "check the service" → which service?
//!
//! Auto-locates config files by extracting filename patterns from the question
//! and running `find` across standard config locations - no hardcoded paths.
//!
//! Examples:
//!   "show my vimrc"         → extracts "vimrc" → find -name "*vimrc*"
//!   "check hyprland config" → extracts "hyprland" → find -name "*hyprland*"
//!   "edit alacritty.toml"   → extracts "alacritty.toml" → find -name "alacritty.toml"

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;
use tracing::debug;

/// Reference patterns that indicate missing context
const AMBIGUOUS_REFERENCES: &[&str] = &[
    "this file",
    "that file",
    "the file",
    "this service",
    "that service",
    "the service",
    "this error",
    "that error",
    "the error",
    "the issue",
    "the problem",
    "the warning",
    "this process",
    "that process",
    "the process",
    "it ",
    "its ",
    " it.",
    " it?",
];

/// File extensions that indicate config/script files
const CONFIG_EXTENSIONS: &[&str] = &[
    ".conf", ".toml", ".yml", ".yaml", ".lua", ".vim",
    ".ini", ".cfg", ".json", ".jsonc", ".sh", ".fish",
    ".env", ".xml", ".zsh", ".bash",
];

/// Result of context resolution
#[derive(Debug)]
pub enum ContextResolution {
    /// Question is clear, proceed normally
    Clear,
    /// Missing context detected, need clarification
    NeedsClarification(String),
    /// Config file found and resolved
    Resolved {
        original_question: String,
        resolved_question: String,
        found_path: PathBuf,
    },
}

/// Detect missing references and resolve config files
pub fn resolve_context(question: &str, username: &str) -> Result<ContextResolution> {
    let q_lower = question.to_lowercase();

    // Step 1: Try to extract and find config file from the question
    if let Some(resolution) = try_resolve_config_file(question, &q_lower, username)? {
        return Ok(resolution);
    }

    // Step 2: Check for ambiguous references that survived config resolution
    if has_ambiguous_reference(&q_lower) {
        let clarification = generate_clarification(&q_lower);
        return Ok(ContextResolution::NeedsClarification(clarification));
    }

    Ok(ContextResolution::Clear)
}

/// Extract a filename pattern from the question, then find it using:
/// 1. pacman -Ql (package manager knows exactly where configs live)
/// 2. find as fallback across standard config locations
fn try_resolve_config_file(
    original_question: &str,
    q_lower: &str,
    username: &str,
) -> Result<Option<ContextResolution>> {
    let pattern = match extract_filename_pattern(q_lower) {
        Some(p) => p,
        None => return Ok(None),
    };

    debug!("Extracted filename pattern from question: {:?}", pattern);

    // Extract app name for package manager lookup
    // e.g., "vimrc" → "vim", "*hyprland*" → "hyprland"
    let app_name = extract_app_name(&pattern);

    // Try 1: Ask the package manager — most accurate
    let found_path = if let Some(name) = &app_name {
        find_config_via_package(name)
            .or_else(|| find_file_on_system(&pattern, username))
    } else {
        find_file_on_system(&pattern, username)
    };

    if let Some(path) = found_path {
        debug!("Resolved config to: {}", path.display());
        let resolved_question = format!(
            "{} (file located at: {})",
            original_question,
            path.display()
        );
        return Ok(Some(ContextResolution::Resolved {
            original_question: original_question.to_string(),
            resolved_question,
            found_path: path,
        }));
    }

    Ok(None)
}

/// Query pacman (or dpkg/rpm) for config files belonging to a package.
/// This is the authoritative source — no guessing needed.
fn find_config_via_package(app_name: &str) -> Option<PathBuf> {
    // Try pacman first (Arch)
    if let Some(path) = query_pacman(app_name) {
        return Some(path);
    }
    // Try dpkg (Debian/Ubuntu)
    if let Some(path) = query_dpkg(app_name) {
        return Some(path);
    }
    // Try rpm (Fedora/RHEL)
    if let Some(path) = query_rpm(app_name) {
        return Some(path);
    }
    None
}

fn query_pacman(app_name: &str) -> Option<PathBuf> {
    let output = Command::new("pacman")
        .args(["-Ql", app_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_package_file_list(&output.stdout)
}

fn query_dpkg(app_name: &str) -> Option<PathBuf> {
    let output = Command::new("dpkg")
        .args(["-L", app_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_package_file_list(&output.stdout)
}

fn query_rpm(app_name: &str) -> Option<PathBuf> {
    let output = Command::new("rpm")
        .args(["-ql", app_name])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    parse_package_file_list(&output.stdout)
}

/// Parse file list output and return the most relevant config file.
/// Priority: ~/.config paths > /etc paths > other
fn parse_package_file_list(raw: &[u8]) -> Option<PathBuf> {
    let text = String::from_utf8_lossy(raw);

    // Extract just the file paths (pacman format: "pkg /path/to/file")
    let paths: Vec<&str> = text.lines()
        .filter_map(|line| {
            // pacman: "pkgname /path", dpkg/rpm: just "/path"
            let path = if let Some(idx) = line.find('/') {
                &line[idx..]
            } else {
                line.trim()
            };
            if path.starts_with('/') { Some(path) } else { None }
        })
        .collect();

    // Priority 1: files in /etc with config extensions
    let config_exts = [".conf", ".cfg", ".ini", ".toml", ".yml", ".yaml"];
    for path in &paths {
        if path.starts_with("/etc") {
            for ext in &config_exts {
                if path.ends_with(ext) && std::path::Path::new(path).exists() {
                    debug!("Found via package manager (etc): {}", path);
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    // Priority 2: example configs in /usr/share (user can copy these)
    for path in &paths {
        if path.contains("/share/") {
            for ext in &config_exts {
                if path.ends_with(ext) && std::path::Path::new(path).exists() {
                    debug!("Found via package manager (share): {}", path);
                    return Some(PathBuf::from(path));
                }
            }
        }
    }

    None
}

/// Extract the probable app name from a file pattern.
/// "*hyprland*" → "hyprland", "vimrc" → "vim", "alacritty.toml" → "alacritty"
fn extract_app_name(pattern: &str) -> Option<String> {
    let clean = pattern.trim_matches('*');

    // Remove common suffixes/prefixes that aren't the app name
    let name = clean
        .trim_end_matches(".conf")
        .trim_end_matches(".toml")
        .trim_end_matches(".yml")
        .trim_end_matches(".yaml")
        .trim_end_matches(".lua")
        .trim_end_matches(".vim")
        .trim_end_matches(".ini")
        .trim_end_matches("rc")  // vimrc → vim, bashrc → bash
        .trim();

    if name.len() >= 2 {
        Some(name.to_string())
    } else {
        None
    }
}

/// Extract a file name or search pattern from the question.
///
/// Handles three cases:
///   1. Explicit extension: "alacritty.toml" → "alacritty.toml"
///   2. *rc suffix: "vimrc", "bashrc", "zshrc" → "*vimrc"
///   3. "<name> config/conf": "hyprland config" → "*hyprland*"
fn extract_filename_pattern(q_lower: &str) -> Option<String> {
    // Case 1: word with a known config extension
    for word in q_lower.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_' && c != '-');
        for ext in CONFIG_EXTENSIONS {
            if clean.ends_with(ext) && clean.len() > ext.len() {
                return Some(clean.to_string());
            }
        }
    }

    // Case 2: *rc word (vimrc, bashrc, zshrc, tmuxrc, etc.)
    for word in q_lower.split_whitespace() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric());
        if clean.ends_with("rc") && clean.len() > 3 && clean.chars().all(|c| c.is_alphanumeric()) {
            return Some(clean.to_string());
        }
    }

    // Case 3: "<keyword> config" or "<keyword> conf" — extract the keyword
    // e.g., "hyprland config", "i3 config", "sway conf"
    for trigger in &["config", "conf", "configuration"] {
        if let Some(idx) = q_lower.find(trigger) {
            // Get the word immediately before the trigger
            let before = q_lower[..idx].trim();
            if let Some(keyword) = before.split_whitespace().last() {
                let clean = keyword.trim_matches(|c: char| !c.is_alphanumeric());
                // Must be a meaningful keyword (not "my", "the", etc.)
                if clean.len() >= 2 && !is_stop_word(clean) {
                    return Some(format!("*{}*", clean));
                }
            }
        }
    }

    None
}

/// Words that don't identify a file
fn is_stop_word(word: &str) -> bool {
    matches!(word, "my" | "the" | "a" | "an" | "its" | "their" | "your" | "our" | "this" | "that" | "which" | "what" | "how" | "where" | "show" | "edit" | "open" | "check" | "see")
}

/// Run find(1) across standard config locations for the given pattern
fn find_file_on_system(name_pattern: &str, username: &str) -> Option<PathBuf> {
    let home = format!("/home/{}", username);

    // Search locations in priority order
    let search_roots = [
        format!("{}/.config", home),   // XDG: most modern app configs
        home.clone(),                   // home: dotfiles (.vimrc, .tmux.conf, etc.)
        "/etc".to_string(),             // system-wide configs
    ];

    for root in &search_roots {
        let output = Command::new("find")
            .arg(root)
            .arg("-maxdepth")
            .arg("4")
            .arg("-name")
            .arg(name_pattern)
            .arg("-type")
            .arg("f")
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut matches: Vec<&str> = stdout.lines().collect();

        if matches.is_empty() {
            continue;
        }

        // Prefer shallower paths (fewer path components = more likely the right file)
        matches.sort_by_key(|p| p.matches('/').count());

        debug!("find({}, {}) → {:?}", root, name_pattern, &matches[..matches.len().min(3)]);
        return Some(PathBuf::from(matches[0]));
    }

    None
}

/// Check if question has ambiguous references
fn has_ambiguous_reference(q_lower: &str) -> bool {
    // Skip if question is very specific (contains file path or service name)
    if has_specific_reference(q_lower) {
        return false;
    }

    for pattern in AMBIGUOUS_REFERENCES {
        if q_lower.contains(pattern) {
            return true;
        }
    }

    false
}

/// Check if question already has specific references (paths, known services)
fn has_specific_reference(q_lower: &str) -> bool {
    // Has explicit file path
    if q_lower.contains('/') {
        return true;
    }

    // Has file extension (user typed full filename)
    for ext in CONFIG_EXTENSIONS {
        if q_lower.contains(ext) {
            return true;
        }
    }

    // Has a well-known service name
    const KNOWN_SERVICES: &[&str] = &[
        "systemd", "nginx", "apache", "postgresql", "mysql", "docker",
        "ssh", "sshd", "ollama", "gdm", "lightdm", "networkmanager",
        "bluetooth", "pipewire", "pulseaudio", "firewalld",
    ];
    for service in KNOWN_SERVICES {
        if q_lower.contains(service) {
            return true;
        }
    }

    false
}

/// Generate clarification question based on detected ambiguity
fn generate_clarification(q_lower: &str) -> String {
    if q_lower.contains("file") {
        "Which file are you referring to? Please provide the file path or name.".to_string()
    } else if q_lower.contains("service") {
        "Which service? Please provide the service name (e.g., nginx.service).".to_string()
    } else if q_lower.contains("error") || q_lower.contains("issue") || q_lower.contains("problem") {
        "Which error or issue? Please paste the error message or describe it more specifically.".to_string()
    } else if q_lower.contains("process") {
        "Which process? Please provide the process name or PID.".to_string()
    } else {
        "Could you be more specific? I need more context to answer accurately.".to_string()
    }
}

/// Track recent context from session (files, services mentioned)
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub recent_files: Vec<String>,
    pub recent_services: Vec<String>,
}

impl SessionContext {
    pub fn new() -> Self {
        Self {
            recent_files: Vec::new(),
            recent_services: Vec::new(),
        }
    }

    pub fn track_file(&mut self, path: String) {
        if !self.recent_files.contains(&path) {
            self.recent_files.push(path);
            if self.recent_files.len() > 10 {
                self.recent_files.remove(0);
            }
        }
    }

    pub fn track_service(&mut self, service: String) {
        if !self.recent_services.contains(&service) {
            self.recent_services.push(service);
            if self.recent_services.len() > 10 {
                self.recent_services.remove(0);
            }
        }
    }

    pub fn resolve_file_reference(&self) -> Option<&str> {
        self.recent_files.last().map(|s| s.as_str())
    }

    pub fn resolve_service_reference(&self) -> Option<&str> {
        self.recent_services.last().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_explicit_extension() {
        assert_eq!(
            extract_filename_pattern("show me my alacritty.toml"),
            Some("alacritty.toml".to_string())
        );
        assert_eq!(
            extract_filename_pattern("edit hyprland.conf"),
            Some("hyprland.conf".to_string())
        );
        assert_eq!(
            extract_filename_pattern("what's in my init.lua"),
            Some("init.lua".to_string())
        );
    }

    #[test]
    fn test_extract_rc_suffix() {
        assert_eq!(
            extract_filename_pattern("show my vimrc"),
            Some("vimrc".to_string())
        );
        assert_eq!(
            extract_filename_pattern("open my bashrc"),
            Some("bashrc".to_string())
        );
        assert_eq!(
            extract_filename_pattern("check zshrc please"),
            Some("zshrc".to_string())
        );
    }

    #[test]
    fn test_extract_config_keyword() {
        assert_eq!(
            extract_filename_pattern("show my hyprland config"),
            Some("*hyprland*".to_string())
        );
        assert_eq!(
            extract_filename_pattern("open the kitty config"),
            Some("*kitty*".to_string())
        );
        assert_eq!(
            extract_filename_pattern("check i3 configuration"),
            Some("*i3*".to_string())
        );
    }

    #[test]
    fn test_stop_words_not_extracted() {
        assert_eq!(extract_filename_pattern("show my config"), None);
        assert_eq!(extract_filename_pattern("open the config"), None);
    }

    #[test]
    fn test_ambiguous_references() {
        assert!(has_ambiguous_reference("show me that file"));
        assert!(has_ambiguous_reference("check the service status"));
        assert!(has_ambiguous_reference("fix the error"));
        assert!(has_ambiguous_reference("what is it doing"));
        assert!(has_ambiguous_reference("show me the problem"));
    }

    #[test]
    fn test_specific_references_not_ambiguous() {
        assert!(!has_ambiguous_reference("show me /etc/fstab"));
        assert!(!has_ambiguous_reference("check nginx.service status"));
        assert!(!has_ambiguous_reference("restart ollama service"));
    }
}
