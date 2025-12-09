//! Desktop environment configuration recipes (v0.0.257).
//!
//! Provides recipes for common desktop configuration tasks like:
//! - Setting wallpaper
//! - Enabling/disabling dark mode
//! - Changing themes
//! - Display settings
//!
//! Detects the desktop environment and generates appropriate commands.

use serde::{Deserialize, Serialize};

/// Supported desktop environments
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    Gnome,
    Kde,
    Xfce,
    Cinnamon,
    Mate,
    Unknown,
}

impl DesktopEnvironment {
    /// Detect current desktop environment from environment variables
    pub fn detect() -> Self {
        // Check XDG_CURRENT_DESKTOP first (most reliable)
        if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
            let desktop_lower = desktop.to_lowercase();
            if desktop_lower.contains("gnome") {
                return Self::Gnome;
            }
            if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
                return Self::Kde;
            }
            if desktop_lower.contains("xfce") {
                return Self::Xfce;
            }
            if desktop_lower.contains("cinnamon") {
                return Self::Cinnamon;
            }
            if desktop_lower.contains("mate") {
                return Self::Mate;
            }
        }

        // Fallback to DESKTOP_SESSION
        if let Ok(session) = std::env::var("DESKTOP_SESSION") {
            let session_lower = session.to_lowercase();
            if session_lower.contains("gnome") {
                return Self::Gnome;
            }
            if session_lower.contains("plasma") || session_lower.contains("kde") {
                return Self::Kde;
            }
            if session_lower.contains("xfce") {
                return Self::Xfce;
            }
        }

        Self::Unknown
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Gnome => "GNOME",
            Self::Kde => "KDE Plasma",
            Self::Xfce => "Xfce",
            Self::Cinnamon => "Cinnamon",
            Self::Mate => "MATE",
            Self::Unknown => "Unknown",
        }
    }
}

/// Desktop configuration action types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopAction {
    SetWallpaper { path: String },
    EnableDarkMode,
    DisableDarkMode,
    SetTheme { theme: String },
    SetIconTheme { theme: String },
    SetCursorTheme { theme: String },
}

/// A desktop configuration recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopRecipe {
    pub id: String,
    pub description: String,
    pub action: DesktopAction,
    pub desktop: DesktopEnvironment,
    pub command: String,
    pub rollback_command: Option<String>,
}

impl DesktopRecipe {
    /// Generate wallpaper command for detected desktop
    pub fn set_wallpaper(path: &str) -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                format!(
                    "gsettings set org.gnome.desktop.background picture-uri 'file://{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Kde => (
                format!(
                    r#"qdbus org.kde.plasmashell /PlasmaShell org.kde.PlasmaShell.evaluateScript 'var allDesktops = desktops();for (i=0;i<allDesktops.length;i++){{d = allDesktops[i];d.wallpaperPlugin = "org.kde.image";d.currentConfigGroup = Array("Wallpaper","org.kde.image","General");d.writeConfig("Image","file://{}");}}'"#,
                    path
                ),
                None,
            ),
            DesktopEnvironment::Xfce => (
                format!(
                    "xfconf-query -c xfce4-desktop -p /backdrop/screen0/monitor0/workspace0/last-image -s '{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Cinnamon => (
                format!(
                    "gsettings set org.cinnamon.desktop.background picture-uri 'file://{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Mate => (
                format!(
                    "gsettings set org.mate.background picture-filename '{}'",
                    path
                ),
                None,
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_wallpaper".to_string(),
            description: format!("Set wallpaper to {}", path),
            action: DesktopAction::SetWallpaper {
                path: path.to_string(),
            },
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate dark mode command for detected desktop
    pub fn enable_dark_mode() -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                "gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'".to_string(),
                Some(
                    "gsettings set org.gnome.desktop.interface color-scheme 'default'".to_string(),
                ),
            ),
            DesktopEnvironment::Kde => (
                "lookandfeeltool -a org.kde.breezedark.desktop".to_string(),
                Some("lookandfeeltool -a org.kde.breeze.desktop".to_string()),
            ),
            DesktopEnvironment::Xfce => (
                "xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita-dark'".to_string(),
                Some("xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita'".to_string()),
            ),
            DesktopEnvironment::Cinnamon => (
                "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita-dark'".to_string(),
                Some(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita'".to_string(),
                ),
            ),
            DesktopEnvironment::Mate => (
                "gsettings set org.mate.interface gtk-theme 'Adwaita-dark'".to_string(),
                Some("gsettings set org.mate.interface gtk-theme 'Adwaita'".to_string()),
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_dark_mode".to_string(),
            description: "Enable dark mode".to_string(),
            action: DesktopAction::EnableDarkMode,
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate disable dark mode command
    pub fn disable_dark_mode() -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let (command, rollback) = match desktop {
            DesktopEnvironment::Gnome => (
                "gsettings set org.gnome.desktop.interface color-scheme 'default'".to_string(),
                Some(
                    "gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'"
                        .to_string(),
                ),
            ),
            DesktopEnvironment::Kde => (
                "lookandfeeltool -a org.kde.breeze.desktop".to_string(),
                Some("lookandfeeltool -a org.kde.breezedark.desktop".to_string()),
            ),
            DesktopEnvironment::Xfce => (
                "xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita'".to_string(),
                Some("xfconf-query -c xsettings -p /Net/ThemeName -s 'Adwaita-dark'".to_string()),
            ),
            DesktopEnvironment::Cinnamon => (
                "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita'".to_string(),
                Some(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme 'Adwaita-dark'"
                        .to_string(),
                ),
            ),
            DesktopEnvironment::Mate => (
                "gsettings set org.mate.interface gtk-theme 'Adwaita'".to_string(),
                Some("gsettings set org.mate.interface gtk-theme 'Adwaita-dark'".to_string()),
            ),
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_light_mode".to_string(),
            description: "Disable dark mode (enable light mode)".to_string(),
            action: DesktopAction::DisableDarkMode,
            desktop,
            command,
            rollback_command: rollback,
        })
    }

    /// Generate theme change command
    pub fn set_theme(theme: &str) -> Option<Self> {
        let desktop = DesktopEnvironment::detect();
        let command = match desktop {
            DesktopEnvironment::Gnome => {
                format!(
                    "gsettings set org.gnome.desktop.interface gtk-theme '{}'",
                    theme
                )
            }
            DesktopEnvironment::Kde => {
                format!("lookandfeeltool -a {}", theme)
            }
            DesktopEnvironment::Xfce => {
                format!(
                    "xfconf-query -c xsettings -p /Net/ThemeName -s '{}'",
                    theme
                )
            }
            DesktopEnvironment::Cinnamon => {
                format!(
                    "gsettings set org.cinnamon.desktop.interface gtk-theme '{}'",
                    theme
                )
            }
            DesktopEnvironment::Mate => {
                format!("gsettings set org.mate.interface gtk-theme '{}'", theme)
            }
            DesktopEnvironment::Unknown => return None,
        };

        Some(Self {
            id: "desktop_theme".to_string(),
            description: format!("Set theme to {}", theme),
            action: DesktopAction::SetTheme {
                theme: theme.to_string(),
            },
            desktop,
            command,
            rollback_command: None,
        })
    }
}

/// Detect desktop configuration request from query
pub fn detect_desktop_intent(query: &str) -> Option<DesktopRecipe> {
    let query_lower = query.to_lowercase();

    // Wallpaper detection
    if query_lower.contains("wallpaper") || query_lower.contains("background") {
        // Try to extract path from query
        if let Some(path) = extract_file_path(&query_lower) {
            return DesktopRecipe::set_wallpaper(&path);
        }
        // No path provided - return None, will need clarification
        return None;
    }

    // Dark mode detection
    if (query_lower.contains("dark") && query_lower.contains("mode"))
        || (query_lower.contains("dark") && query_lower.contains("theme"))
    {
        if query_lower.contains("disable")
            || query_lower.contains("turn off")
            || query_lower.contains("light")
        {
            return DesktopRecipe::disable_dark_mode();
        }
        return DesktopRecipe::enable_dark_mode();
    }

    // Light mode detection
    if query_lower.contains("light") && query_lower.contains("mode") {
        return DesktopRecipe::disable_dark_mode();
    }

    // Theme detection
    if query_lower.contains("theme") && !query_lower.contains("dark") {
        if let Some(theme) = extract_theme_name(&query_lower) {
            return DesktopRecipe::set_theme(&theme);
        }
    }

    None
}

/// Check if query is a desktop config request
pub fn is_desktop_config_request(query: &str) -> bool {
    let query_lower = query.to_lowercase();
    let keywords = [
        "wallpaper",
        "background",
        "dark mode",
        "light mode",
        "theme",
        "desktop",
        "appearance",
    ];
    let actions = ["set", "change", "enable", "disable", "switch", "toggle"];

    let has_keyword = keywords.iter().any(|k| query_lower.contains(k));
    let has_action = actions.iter().any(|a| query_lower.contains(a));

    has_keyword && has_action
}

/// Extract file path from query (simple heuristic)
fn extract_file_path(query: &str) -> Option<String> {
    // Look for paths starting with / or ~
    let words: Vec<&str> = query.split_whitespace().collect();
    for word in words {
        let clean = word.trim_matches(|c| c == '\'' || c == '"');
        if clean.starts_with('/') || clean.starts_with('~') {
            // Expand ~ to $HOME
            let expanded = if clean.starts_with('~') {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                clean.replacen('~', &home, 1)
            } else {
                clean.to_string()
            };
            return Some(expanded);
        }
    }
    None
}

/// Extract theme name from query (simple heuristic)
fn extract_theme_name(query: &str) -> Option<String> {
    // Common theme patterns - longer names first to match "arc-dark" before "arc"
    let known_themes = [
        "adwaita-dark",
        "adwaita",
        "arc-dark",
        "arc",
        "breeze-dark",
        "breeze",
        "dracula",
        "nord",
        "gruvbox",
        "materia",
        "numix",
    ];

    let query_lower = query.to_lowercase();
    for theme in known_themes {
        if query_lower.contains(theme) {
            // Capitalize properly
            return Some(theme.split('-').map(capitalize_first).collect::<Vec<_>>().join("-"));
        }
    }
    None
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_desktop_environment() {
        // Just test that detection doesn't panic
        let _de = DesktopEnvironment::detect();
    }

    #[test]
    fn test_is_desktop_config_request() {
        assert!(is_desktop_config_request("set my wallpaper"));
        assert!(is_desktop_config_request("enable dark mode"));
        assert!(is_desktop_config_request("change theme to dracula"));
        assert!(!is_desktop_config_request("what is my disk usage"));
        assert!(!is_desktop_config_request("install vim"));
    }

    #[test]
    fn test_detect_dark_mode_intent() {
        // These will return None in test environment (no DE detected)
        // but we test the logic path
        let _result = detect_desktop_intent("enable dark mode");
        let _result = detect_desktop_intent("disable dark mode");
        let _result = detect_desktop_intent("switch to light mode");
    }

    #[test]
    fn test_extract_file_path() {
        assert_eq!(
            extract_file_path("set wallpaper to /home/user/pic.jpg"),
            Some("/home/user/pic.jpg".to_string())
        );
        assert!(extract_file_path("set my wallpaper").is_none());
    }

    #[test]
    fn test_extract_theme_name() {
        assert_eq!(extract_theme_name("change theme to dracula"), Some("Dracula".to_string()));
        assert_eq!(extract_theme_name("set arc-dark theme"), Some("Arc-Dark".to_string()));
        assert!(extract_theme_name("change theme").is_none());
    }

    #[test]
    fn test_desktop_environment_display_name() {
        assert_eq!(DesktopEnvironment::Gnome.display_name(), "GNOME");
        assert_eq!(DesktopEnvironment::Kde.display_name(), "KDE Plasma");
    }
}
