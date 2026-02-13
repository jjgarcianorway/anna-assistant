//! DE/WM-aware configuration intelligence.
//!
//! Answers: "How do I change X on this specific desktop environment?"
//!
//! - Detects the exact DE/WM via processes, env vars, and session files
//! - Determines the correct change method (gsettings, config file, dconf, etc.)
//! - Follows modular config includes (source=, include=, Import=) recursively
//! - Locates the exact file and line where a setting lives

use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::debug;

/// Detected desktop environment or window manager
#[derive(Debug, Clone, PartialEq)]
pub enum DE {
    Hyprland,
    Sway,
    I3,
    Gnome,
    Kde,
    Xfce,
    Bspwm,
    Awesome,
    Dwm,
    Openbox,
    Other(String),
    Unknown,
}

/// How to make a configuration change for this DE
#[derive(Debug, Clone)]
pub enum ChangeMethod {
    /// Edit a config file directly
    ConfigFile { main_config: PathBuf },
    /// GNOME gsettings command
    GSettings { schema: &'static str },
    /// dconf command
    DConf { path: &'static str },
    /// KDE plasma config tool
    KConfig,
    /// X11 xkbmap/setxkbmap (session-only)
    XSetKeyboard,
    /// Systemd/locale config
    SystemLocale,
    /// Not yet determined, need investigation
    Unknown,
}

/// Full DE environment context
#[derive(Debug, Clone)]
pub struct DesktopContext {
    pub de: DE,
    pub session_type: String,   // "wayland" or "x11"
    pub username: String,
}

impl DesktopContext {
    /// Detect the current DE/WM by probing the running system
    pub fn detect(username: &str) -> Self {
        let de = detect_de();
        let session_type = detect_session_type();

        debug!("Detected DE: {:?}, session: {}", de, session_type);

        Self {
            de,
            session_type,
            username: username.to_string(),
        }
    }

    /// Determine the right method to change keyboard layout on this DE
    pub fn keyboard_layout_method(&self) -> ChangeMethod {
        match &self.de {
            DE::Hyprland => {
                let cfg = self.find_hyprland_config();
                ChangeMethod::ConfigFile { main_config: cfg }
            }
            DE::Sway => {
                let cfg = self.home_path(".config/sway/config");
                ChangeMethod::ConfigFile { main_config: cfg }
            }
            DE::I3 => {
                let cfg = self.home_path(".config/i3/config");
                ChangeMethod::ConfigFile { main_config: cfg }
            }
            DE::Gnome => ChangeMethod::GSettings {
                schema: "org.gnome.desktop.input-sources",
            },
            DE::Kde => ChangeMethod::KConfig,
            DE::Xfce => ChangeMethod::DConf {
                path: "/org/gnome/desktop/input-sources/",
            },
            _ => {
                // X11 fallback: setxkbmap (session-only, suggest /etc/X11/xorg.conf.d/)
                if self.session_type == "x11" {
                    ChangeMethod::XSetKeyboard
                } else {
                    ChangeMethod::Unknown
                }
            }
        }
    }

    fn home_path(&self, rel: &str) -> PathBuf {
        PathBuf::from(format!("/home/{}/{}", self.username, rel))
    }

    fn find_hyprland_config(&self) -> PathBuf {
        // Standard location
        let standard = self.home_path(".config/hypr/hyprland.conf");
        if standard.exists() {
            return standard;
        }
        // XDG_CONFIG_HOME fallback
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(format!("{}/hypr/hyprland.conf", xdg));
            if path.exists() {
                return path;
            }
        }
        standard // return even if doesn't exist yet
    }
}

/// Detect the running DE/WM
fn detect_de() -> DE {
    // 1. XDG_CURRENT_DESKTOP is most reliable when set
    if let Ok(de_env) = std::env::var("XDG_CURRENT_DESKTOP") {
        let de_lower = de_env.to_lowercase();
        if de_lower.contains("hyprland") { return DE::Hyprland; }
        if de_lower.contains("sway") { return DE::Sway; }
        if de_lower.contains("i3") { return DE::I3; }
        if de_lower.contains("gnome") { return DE::Gnome; }
        if de_lower.contains("kde") || de_lower.contains("plasma") { return DE::Kde; }
        if de_lower.contains("xfce") { return DE::Xfce; }
    }

    // 2. HYPRLAND_INSTANCE_SIGNATURE means hyprland is running
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return DE::Hyprland;
    }

    // 3. Swaysock means sway is running
    if std::env::var("SWAYSOCK").is_ok() {
        return DE::Sway;
    }

    // 4. Check running processes (works when env vars aren't set)
    let procs = running_processes();
    if procs.iter().any(|p| p.contains("Hyprland")) { return DE::Hyprland; }
    if procs.iter().any(|p| p == "sway") { return DE::Sway; }
    if procs.iter().any(|p| p == "i3") { return DE::I3; }
    if procs.iter().any(|p| p.contains("gnome-shell")) { return DE::Gnome; }
    if procs.iter().any(|p| p.contains("plasmashell")) { return DE::Kde; }
    if procs.iter().any(|p| p.contains("xfce4-session")) { return DE::Xfce; }
    if procs.iter().any(|p| p == "bspwm") { return DE::Bspwm; }
    if procs.iter().any(|p| p == "awesome") { return DE::Awesome; }
    if procs.iter().any(|p| p == "openbox") { return DE::Openbox; }

    // 5. Check DESKTOP_SESSION
    if let Ok(session) = std::env::var("DESKTOP_SESSION") {
        let s = session.to_lowercase();
        if s.contains("hyprland") { return DE::Hyprland; }
        if s.contains("sway") { return DE::Sway; }
        if s.contains("gnome") { return DE::Gnome; }
        if s.contains("plasma") { return DE::Kde; }
        if !s.is_empty() { return DE::Other(session); }
    }

    DE::Unknown
}

fn detect_session_type() -> String {
    std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_else(|_| {
            // Ask loginctl
            Command::new("loginctl")
                .args(["show-session", "auto", "-p", "Type", "--value"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        })
        .to_lowercase()
}

fn running_processes() -> Vec<String> {
    Command::new("ps")
        .args(["-eo", "comm"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().map(|l| l.trim().to_string()).collect())
        .unwrap_or_default()
}

// ─── Modular config resolver ────────────────────────────────────────────────

/// Resolve all config files for a DE, following source/include directives.
/// Returns an ordered list of all files that contribute to the config.
pub fn resolve_config_files(main_config: &Path) -> Vec<PathBuf> {
    let mut visited = Vec::new();
    let mut queue = vec![main_config.to_path_buf()];

    while let Some(path) = queue.first().cloned() {
        queue.remove(0);

        if visited.contains(&path) {
            continue;
        }

        if !path.exists() {
            debug!("Config file not found: {}", path.display());
            visited.push(path);
            continue;
        }

        visited.push(path.clone());

        // Parse file for include directives and add to queue
        let includes = find_includes(&path);
        debug!("Found {} includes in {}", includes.len(), path.display());
        for inc in includes {
            if !visited.contains(&inc) {
                queue.push(inc);
            }
        }
    }

    visited
}

/// Parse a config file and return all paths it includes/sources.
/// Handles: hyprland `source =`, sway/i3 `include`, and generic `include`/`Import`
fn find_includes(config_path: &Path) -> Vec<PathBuf> {
    let content = match std::fs::read_to_string(config_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let base_dir = config_path.parent().unwrap_or(Path::new("/"));
    let mut includes = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Hyprland: source = ./input.conf  or  source = ~/.config/hypr/input.conf
        if let Some(rest) = trimmed.strip_prefix("source") {
            let path_str = rest.trim_start_matches([' ', '=', '\t']).trim();
            if let Some(p) = expand_path(path_str, base_dir) {
                includes.push(p);
            }
        }

        // Sway/i3/generic: include path  or  include ./path
        else if let Some(rest) = trimmed.strip_prefix("include") {
            let path_str = rest.trim();
            // Expand globs (sway uses include ~/.config/sway/config.d/*)
            if path_str.contains('*') {
                includes.extend(expand_glob(path_str, base_dir));
            } else if let Some(p) = expand_path(path_str, base_dir) {
                includes.push(p);
            }
        }

        // KDE / generic: Import=path
        else if let Some(rest) = trimmed.strip_prefix("Import=") {
            if let Some(p) = expand_path(rest.trim(), base_dir) {
                includes.push(p);
            }
        }
    }

    includes
}

fn expand_path(raw: &str, base_dir: &Path) -> Option<PathBuf> {
    let expanded = if raw.starts_with("~/") {
        // Expand ~ using HOME env
        let home = std::env::var("HOME").ok()?;
        raw.replacen('~', &home, 1)
    } else if raw.starts_with("./") || (!raw.starts_with('/') && !raw.is_empty()) {
        base_dir.join(raw).to_string_lossy().to_string()
    } else {
        raw.to_string()
    };

    if expanded.is_empty() { None } else { Some(PathBuf::from(expanded)) }
}

fn expand_glob(pattern: &str, base_dir: &Path) -> Vec<PathBuf> {
    let expanded = if pattern.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_default();
        pattern.replacen('~', &home, 1)
    } else if pattern.starts_with("./") {
        base_dir.join(&pattern[2..]).to_string_lossy().to_string()
    } else {
        pattern.to_string()
    };

    // Use find to expand globs rather than depending on a glob crate
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("ls -1 {} 2>/dev/null", expanded))
        .output()
        .ok();

    output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.lines().filter(|l| !l.is_empty()).map(PathBuf::from).collect())
        .unwrap_or_default()
}

// ─── Setting locator ─────────────────────────────────────────────────────────

/// Find which file and line contains a specific setting across all config files.
pub fn find_setting_in_configs(configs: &[PathBuf], search_term: &str) -> Option<(PathBuf, usize, String)> {
    for config in configs {
        let content = match std::fs::read_to_string(config) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            if line.to_lowercase().contains(&search_term.to_lowercase()) {
                debug!("Found '{}' in {} at line {}", search_term, config.display(), line_num + 1);
                return Some((config.clone(), line_num + 1, line.to_string()));
            }
        }
    }
    None
}

/// Build a full investigation report for a config change request.
/// This is injected into the plan generation prompt so Anna knows exactly
/// what's on this system.
pub fn build_config_investigation(de_ctx: &DesktopContext, topic: &str) -> String {
    let mut report = format!(
        "=== DESKTOP ENVIRONMENT INVESTIGATION ===\n\
         DE/WM: {:?}\n\
         Session type: {}\n\
         User: {}\n\n",
        de_ctx.de, de_ctx.session_type, de_ctx.username
    );

    // Get keyboard layout method
    let method = de_ctx.keyboard_layout_method();
    report.push_str(&format!("Change method for '{}': {:?}\n\n", topic, method));

    match &method {
        ChangeMethod::ConfigFile { main_config } => {
            report.push_str(&format!("Main config: {}\n", main_config.display()));

            // Follow all includes/sources
            let all_configs = resolve_config_files(main_config);
            report.push_str(&format!("All config files ({} total):\n", all_configs.len()));
            for cfg in &all_configs {
                report.push_str(&format!("  - {}\n", cfg.display()));
            }

            // Find where the relevant setting currently lives
            let search_terms = config_search_terms(topic);
            for term in &search_terms {
                if let Some((file, line, content)) = find_setting_in_configs(&all_configs, term) {
                    report.push_str(&format!(
                        "\nCurrent '{}' setting found:\n  File: {}\n  Line: {}\n  Content: {}\n",
                        term, file.display(), line, content.trim()
                    ));
                }
            }
        }
        ChangeMethod::GSettings { schema } => {
            report.push_str(&format!("Use gsettings with schema: {}\n", schema));
            // Show current value
            if let Some(current) = get_gsetting(schema, "sources") {
                report.push_str(&format!("Current value: {}\n", current));
            }
        }
        ChangeMethod::KConfig => {
            report.push_str("Use KDE plasma keyboard settings (kcmshell5 kcm_keyboard or localectl)\n");
        }
        ChangeMethod::XSetKeyboard => {
            report.push_str("X11 session: setxkbmap for session, /etc/X11/xorg.conf.d/ for persistence\n");
            if let Some(current) = get_current_xkb_layout() {
                report.push_str(&format!("Current XKB layout: {}\n", current));
            }
        }
        _ => {}
    }

    // Always include: what localectl says (system-wide default)
    if let Some(locale) = get_localectl_keymap() {
        report.push_str(&format!("\nSystem keyboard (localectl): {}\n", locale));
    }

    report
}

/// Get search terms relevant to a config topic
fn config_search_terms(topic: &str) -> Vec<&'static str> {
    let topic_lower = topic.to_lowercase();
    if topic_lower.contains("keyboard") || topic_lower.contains("layout") || topic_lower.contains("kbd") {
        vec!["kb_layout", "xkb_layout", "input", "keyboard"]
    } else if topic_lower.contains("monitor") || topic_lower.contains("display") || topic_lower.contains("resolution") {
        vec!["monitor", "resolution", "refresh", "output"]
    } else if topic_lower.contains("mouse") || topic_lower.contains("cursor") || topic_lower.contains("pointer") {
        vec!["mouse", "cursor", "pointer", "sensitivity"]
    } else if topic_lower.contains("font") {
        vec!["font", "font_size", "font_family"]
    } else if topic_lower.contains("gap") || topic_lower.contains("border") {
        vec!["gaps", "border", "border_size"]
    } else {
        vec![]
    }
}

fn get_gsetting(schema: &str, key: &str) -> Option<String> {
    Command::new("gsettings")
        .args(["get", schema, key])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn get_current_xkb_layout() -> Option<String> {
    Command::new("setxkbmap")
        .arg("-query")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("layout:"))
                .map(|l| l.replacen("layout:", "", 1).trim().to_string())
        })
}

fn get_localectl_keymap() -> Option<String> {
    Command::new("localectl")
        .arg("status")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find(|l| l.contains("X11 Layout") || l.contains("VC Keymap"))
                .map(|l| l.trim().to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_includes_hyprland() {
        // Test that source = directives are parsed
        let content = "source = ~/.config/hypr/input.conf\nsource=./keybinds.conf\n";
        let tmp = std::env::temp_dir().join("hyprland_test.conf");
        std::fs::write(&tmp, content).unwrap();
        let includes = find_includes(&tmp);
        assert_eq!(includes.len(), 2);
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_find_includes_sway() {
        let content = "include ~/.config/sway/config.d/input\ninclude ./extra.conf\n";
        let tmp = std::env::temp_dir().join("sway_test.conf");
        std::fs::write(&tmp, content).unwrap();
        let includes = find_includes(&tmp);
        assert!(!includes.is_empty());
        std::fs::remove_file(&tmp).unwrap();
    }

    #[test]
    fn test_expand_path_home() {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
        let result = expand_path("~/.config/hypr/input.conf", Path::new("/base"));
        assert_eq!(result, Some(PathBuf::from(format!("{}/.config/hypr/input.conf", home))));
    }

    #[test]
    fn test_config_search_terms() {
        let terms = config_search_terms("keyboard layout");
        assert!(terms.contains(&"kb_layout"));
        assert!(terms.contains(&"input"));
    }
}
