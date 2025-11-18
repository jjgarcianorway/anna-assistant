//! Desktop Config File Parser
//!
//! Parses desktop environment config files (Hyprland, i3, Sway) to extract
//! settings like wallpapers, themes, keybindings, and other configurations.
//!
//! This enables Anna to understand user preferences and make intelligent
//! suggestions for config modifications.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::desktop::{DesktopEnvironment, DesktopInfo};

/// Parsed desktop configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    /// Desktop environment
    pub environment: DesktopEnvironment,
    /// Config file path
    pub config_file: PathBuf,
    /// Wallpaper settings
    pub wallpaper: Option<WallpaperConfig>,
    /// Theme settings
    pub theme: Option<ThemeConfig>,
    /// Startup applications
    pub startup_apps: Vec<String>,
    /// Additional key-value settings
    pub settings: HashMap<String, String>,
}

/// Wallpaper configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperConfig {
    /// Wallpaper setter command/tool (hyprpaper, feh, nitrogen, etc.)
    pub setter: String,
    /// Current wallpaper paths
    pub paths: Vec<PathBuf>,
    /// Wallpaper config file (if separate)
    pub config_file: Option<PathBuf>,
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color scheme name
    pub name: Option<String>,
    /// Theme colors (key: color name, value: hex color)
    pub colors: HashMap<String, String>,
    /// GTK theme
    pub gtk_theme: Option<String>,
    /// Icon theme
    pub icon_theme: Option<String>,
}

impl DesktopConfig {
    /// Parse desktop config from detected environment
    pub fn parse() -> Option<Self> {
        let desktop_info = DesktopInfo::detect();

        let config_file = desktop_info.config_file.as_ref()?;

        Self::parse_file(&desktop_info.environment, config_file)
    }

    /// Parse specific config file
    pub fn parse_file(env: &DesktopEnvironment, config_file: &Path) -> Option<Self> {
        let content = fs::read_to_string(config_file).ok()?;

        match env {
            DesktopEnvironment::Hyprland => Self::parse_hyprland(&content, config_file),
            DesktopEnvironment::I3 | DesktopEnvironment::Sway => {
                Self::parse_i3_sway(&content, config_file, env)
            }
            _ => None,
        }
    }

    /// Parse Hyprland config
    fn parse_hyprland(content: &str, config_file: &Path) -> Option<Self> {
        let mut wallpaper = None;
        let mut theme = None;
        let mut startup_apps = Vec::new();
        let mut settings = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse exec-once (startup applications)
            if line.starts_with("exec-once") {
                if let Some(cmd) = Self::extract_value(line, "exec-once") {
                    startup_apps.push(cmd.to_string());

                    // Check for wallpaper setter
                    if cmd.contains("hyprpaper") {
                        wallpaper = Self::parse_hyprpaper();
                    } else if cmd.contains("swaybg") {
                        if let Some(path) = Self::extract_wallpaper_from_cmd(cmd) {
                            wallpaper = Some(WallpaperConfig {
                                setter: "swaybg".to_string(),
                                paths: vec![PathBuf::from(path)],
                                config_file: None,
                            });
                        }
                    }
                }
            }

            // Parse exec (regular exec commands)
            if line.starts_with("exec ") || line.starts_with("exec=") {
                if let Some(cmd) = Self::extract_value(line, "exec") {
                    startup_apps.push(cmd.to_string());
                }
            }

            // Parse general settings (key = value)
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                // Skip if it's a section header or special keyword
                if !key.contains(' ') && !key.is_empty() && !value.is_empty() {
                    settings.insert(key.to_string(), value.to_string());
                }
            }
        }

        Some(DesktopConfig {
            environment: DesktopEnvironment::Hyprland,
            config_file: config_file.to_path_buf(),
            wallpaper,
            theme,
            startup_apps,
            settings,
        })
    }

    /// Parse hyprpaper config
    fn parse_hyprpaper() -> Option<WallpaperConfig> {
        let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            format!("{}/.config", home)
        });

        let hyprpaper_conf = PathBuf::from(config_home)
            .join("hypr")
            .join("hyprpaper.conf");

        if !hyprpaper_conf.exists() {
            return None;
        }

        let content = fs::read_to_string(&hyprpaper_conf).ok()?;
        let mut paths = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("preload") {
                if let Some(value) = Self::extract_value(line, "preload") {
                    paths.push(Self::expand_path(value));
                }
            } else if line.starts_with("wallpaper") {
                // wallpaper = monitor, path
                if let Some(value) = line.split_once('=').map(|(_, v)| v.trim()) {
                    if let Some(path) = value.split(',').nth(1) {
                        let path = path.trim();
                        paths.push(Self::expand_path(path));
                    }
                }
            }
        }

        if paths.is_empty() {
            return None;
        }

        Some(WallpaperConfig {
            setter: "hyprpaper".to_string(),
            paths,
            config_file: Some(hyprpaper_conf),
        })
    }

    /// Parse i3/Sway config
    fn parse_i3_sway(content: &str, config_file: &Path, env: &DesktopEnvironment) -> Option<Self> {
        let mut wallpaper = None;
        let mut theme = None;
        let mut startup_apps = Vec::new();
        let mut settings = HashMap::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse exec (startup applications)
            if line.starts_with("exec ") || line.starts_with("exec_always ") {
                let cmd = if line.starts_with("exec_always ") {
                    line.strip_prefix("exec_always ").unwrap_or("")
                } else {
                    line.strip_prefix("exec ").unwrap_or("")
                };

                let cmd = cmd.trim();
                startup_apps.push(cmd.to_string());

                // Check for wallpaper setter
                if cmd.contains("feh") && cmd.contains("--bg") {
                    if let Some(path) = Self::extract_wallpaper_from_cmd(cmd) {
                        wallpaper = Some(WallpaperConfig {
                            setter: "feh".to_string(),
                            paths: vec![PathBuf::from(path)],
                            config_file: None,
                        });
                    }
                } else if cmd.contains("nitrogen") {
                    wallpaper = Some(WallpaperConfig {
                        setter: "nitrogen".to_string(),
                        paths: Vec::new(),
                        config_file: None,
                    });
                } else if cmd.contains("swaybg") {
                    if let Some(path) = Self::extract_wallpaper_from_cmd(cmd) {
                        wallpaper = Some(WallpaperConfig {
                            setter: "swaybg".to_string(),
                            paths: vec![PathBuf::from(path)],
                            config_file: None,
                        });
                    }
                }
            }

            // Parse set variables (color scheme)
            if line.starts_with("set ") {
                if let Some(rest) = line.strip_prefix("set ") {
                    if let Some((var, value)) = rest.split_once(' ') {
                        settings.insert(var.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }

        Some(DesktopConfig {
            environment: env.clone(),
            config_file: config_file.to_path_buf(),
            wallpaper,
            theme,
            startup_apps,
            settings,
        })
    }

    /// Extract value from "key = value" or "key value"
    fn extract_value<'a>(line: &'a str, key: &str) -> Option<&'a str> {
        let line = line.trim();

        // Try "key = value"
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim();
            if let Some(value) = rest.strip_prefix('=') {
                return Some(value.trim());
            }
            // Try "key value" (space-separated)
            if !rest.is_empty() {
                return Some(rest);
            }
        }

        None
    }

    /// Extract wallpaper path from command
    fn extract_wallpaper_from_cmd(cmd: &str) -> Option<&str> {
        // Look for common patterns:
        // feh --bg-scale /path/to/wallpaper.jpg
        // swaybg -i /path/to/wallpaper.jpg

        if cmd.contains("feh") {
            // Find last argument (usually the path)
            cmd.split_whitespace().last()
        } else if cmd.contains("swaybg") {
            // Look for -i flag
            let parts: Vec<&str> = cmd.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "-i" && i + 1 < parts.len() {
                    return Some(parts[i + 1]);
                }
            }
            None
        } else {
            None
        }
    }

    /// Expand ~ to home directory
    fn expand_path(path: &str) -> PathBuf {
        if path.starts_with('~') {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            PathBuf::from(path.replacen('~', &home, 1))
        } else {
            PathBuf::from(path)
        }
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Config: {}", self.config_file.display()));

        if let Some(ref wp) = self.wallpaper {
            if !wp.paths.is_empty() {
                parts.push(format!(
                    "Wallpaper ({}): {}",
                    wp.setter,
                    wp.paths[0].display()
                ));
            } else {
                parts.push(format!("Wallpaper setter: {}", wp.setter));
            }
        }

        if !self.startup_apps.is_empty() {
            parts.push(format!("{} startup apps", self.startup_apps.len()));
        }

        parts.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hyprland_config() {
        let config = r#"
# Hyprland config
exec-once = hyprpaper
exec-once = waybar

monitor = eDP-1,1920x1080@60,0x0,1

general {
    gaps_in = 5
    gaps_out = 10
}
"#;

        let temp_file = PathBuf::from("/tmp/test_hyprland.conf");
        std::fs::write(&temp_file, config).unwrap();

        let parsed = DesktopConfig::parse_file(&DesktopEnvironment::Hyprland, &temp_file);
        assert!(parsed.is_some());

        let parsed = parsed.unwrap();
        assert_eq!(parsed.startup_apps.len(), 2);
        assert!(parsed.startup_apps.contains(&"hyprpaper".to_string()));

        std::fs::remove_file(&temp_file).ok();
    }

    #[test]
    fn test_parse_i3_config() {
        let config = r#"
# i3 config
set $mod Mod4

exec --no-startup-id feh --bg-scale ~/Pictures/wallpaper.jpg
exec --no-startup-id picom

bindsym $mod+Return exec alacritty
"#;

        let temp_file = PathBuf::from("/tmp/test_i3.conf");
        std::fs::write(&temp_file, config).unwrap();

        let parsed = DesktopConfig::parse_file(&DesktopEnvironment::I3, &temp_file);
        assert!(parsed.is_some());

        let parsed = parsed.unwrap();
        assert!(parsed.wallpaper.is_some());

        let wp = parsed.wallpaper.unwrap();
        assert_eq!(wp.setter, "feh");
        assert!(!wp.paths.is_empty());

        std::fs::remove_file(&temp_file).ok();
    }
}
