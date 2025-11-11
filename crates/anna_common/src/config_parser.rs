//! Universal configuration file parser for window managers and applications
//!
//! Provides intelligent parsing of config files for:
//! - Window Managers (Hyprland, Sway, i3, bspwm, awesome, qtile, etc.)
//! - Shell configs (bashrc, zshrc, fish)
//! - SSH configs
//! - Application configs
//!
//! Architecture based on Arch Wiki documentation:
//! - https://wiki.archlinux.org/title/Window_manager
//! - https://wiki.archlinux.org/title/Wayland
//! - https://wiki.archlinux.org/title/Xorg

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

/// Window manager types with their config locations
#[derive(Debug, Clone, PartialEq)]
pub enum WindowManager {
    // Wayland compositors
    Hyprland,
    Sway,
    Wayfire,
    River,

    // X11 window managers
    I3,
    Bspwm,
    Awesome,
    Qtile,
    Openbox,
    Xmonad,
    Dwm,

    Unknown,
}

impl WindowManager {
    /// Get the primary config file path for this window manager
    pub fn config_path(&self) -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;

        match self {
            WindowManager::Hyprland => Some(PathBuf::from(format!(
                "{}/.config/hypr/hyprland.conf",
                home
            ))),
            WindowManager::Sway => Some(PathBuf::from(format!("{}/.config/sway/config", home))),
            WindowManager::Wayfire => Some(PathBuf::from(format!("{}/.config/wayfire.ini", home))),
            WindowManager::River => Some(PathBuf::from(format!("{}/.config/river/init", home))),
            WindowManager::I3 => {
                // i3 can be in two locations
                let new_path = PathBuf::from(format!("{}/.config/i3/config", home));
                let old_path = PathBuf::from(format!("{}/.i3/config", home));
                if new_path.exists() {
                    Some(new_path)
                } else if old_path.exists() {
                    Some(old_path)
                } else {
                    Some(new_path) // Return new path as default
                }
            }
            WindowManager::Bspwm => Some(PathBuf::from(format!("{}/.config/bspwm/bspwmrc", home))),
            WindowManager::Awesome => {
                Some(PathBuf::from(format!("{}/.config/awesome/rc.lua", home)))
            }
            WindowManager::Qtile => {
                Some(PathBuf::from(format!("{}/.config/qtile/config.py", home)))
            }
            WindowManager::Openbox => {
                Some(PathBuf::from(format!("{}/.config/openbox/rc.xml", home)))
            }
            WindowManager::Xmonad => Some(PathBuf::from(format!("{}/.xmonad/xmonad.hs", home))),
            WindowManager::Dwm => None, // dwm is compiled, no config file
            WindowManager::Unknown => None,
        }
    }

    /// Get the config file format for parsing
    pub fn config_format(&self) -> ConfigFormat {
        match self {
            WindowManager::Hyprland => ConfigFormat::HyprlandConf,
            WindowManager::Sway | WindowManager::I3 => ConfigFormat::I3Style,
            WindowManager::Wayfire | WindowManager::Openbox => ConfigFormat::Ini,
            WindowManager::Awesome => ConfigFormat::Lua,
            WindowManager::Qtile => ConfigFormat::Python,
            WindowManager::Xmonad => ConfigFormat::Haskell,
            WindowManager::Bspwm | WindowManager::River => ConfigFormat::Shell,
            _ => ConfigFormat::Unknown,
        }
    }
}

/// Config file format types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFormat {
    HyprlandConf, // Hyprland-specific format
    I3Style,      // i3/Sway config format
    Ini,          // INI-style (Wayfire, etc.)
    Shell,        // Shell script (bspwm, river)
    Lua,          // Lua (awesome)
    Python,       // Python (qtile)
    Haskell,      // Haskell (xmonad)
    Unknown,
}

/// Configuration parser for window managers
pub struct ConfigParser {
    wm: WindowManager,
    content: Option<String>,
}

impl ConfigParser {
    /// Create a new parser for the given window manager
    pub fn new(wm: WindowManager) -> Self {
        Self { wm, content: None }
    }

    /// Load the config file content
    pub fn load(&mut self) -> Result<()> {
        if let Some(path) = self.wm.config_path() {
            if path.exists() {
                self.content = Some(fs::read_to_string(&path)?);
            }
        }
        Ok(())
    }

    /// Check if an environment variable is set in the config
    pub fn has_env_var(&self, var_name: &str) -> bool {
        let Some(content) = &self.content else {
            return false;
        };

        match self.wm.config_format() {
            ConfigFormat::HyprlandConf => {
                // Hyprland format: env = VARNAME,value
                content.lines().any(|line| {
                    let trimmed = line.trim();
                    trimmed.starts_with("env")
                        && trimmed.contains(var_name)
                        && !trimmed.starts_with('#')
                })
            }
            ConfigFormat::I3Style => {
                // Sway/i3 format: set $varname value  or  exec export VARNAME=value
                content.lines().any(|line| {
                    let trimmed = line.trim();
                    if trimmed.starts_with('#') {
                        return false;
                    }

                    (trimmed.starts_with("set") && trimmed.contains(var_name))
                        || (trimmed.contains("export") && trimmed.contains(var_name))
                })
            }
            ConfigFormat::Shell => {
                // Shell script: export VARNAME=value
                content.lines().any(|line| {
                    let trimmed = line.trim();
                    !trimmed.starts_with('#')
                        && trimmed.contains("export")
                        && trimmed.contains(var_name)
                })
            }
            _ => false, // Other formats not implemented yet
        }
    }

    /// Check if a specific setting exists in the config
    pub fn has_setting(&self, setting: &str) -> bool {
        let Some(content) = &self.content else {
            return false;
        };

        content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.contains(setting)
        })
    }

    /// Get the raw config content
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }
}

/// Detect the currently running window manager
pub fn detect_window_manager() -> WindowManager {
    // Try $XDG_CURRENT_DESKTOP first
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        let desktop_lower = desktop.to_lowercase();
        if desktop_lower.contains("hyprland") {
            return WindowManager::Hyprland;
        } else if desktop_lower.contains("sway") {
            return WindowManager::Sway;
        } else if desktop_lower.contains("i3") {
            return WindowManager::I3;
        }
    }

    // Try $WAYLAND_DISPLAY for Wayland compositors
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // Check running processes for Wayland compositors
        if is_process_running("Hyprland") {
            return WindowManager::Hyprland;
        } else if is_process_running("sway") {
            return WindowManager::Sway;
        } else if is_process_running("wayfire") {
            return WindowManager::Wayfire;
        } else if is_process_running("river") {
            return WindowManager::River;
        }
    }

    // Check for X11 window managers
    if is_process_running("i3") {
        return WindowManager::I3;
    } else if is_process_running("bspwm") {
        return WindowManager::Bspwm;
    } else if is_process_running("awesome") {
        return WindowManager::Awesome;
    } else if is_process_running("qtile") {
        return WindowManager::Qtile;
    } else if is_process_running("openbox") {
        return WindowManager::Openbox;
    } else if is_process_running("xmonad") {
        return WindowManager::Xmonad;
    } else if is_process_running("dwm") {
        return WindowManager::Dwm;
    }

    WindowManager::Unknown
}

/// Check if a process is running
fn is_process_running(process_name: &str) -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg(process_name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_paths() {
        let hypr = WindowManager::Hyprland;
        assert!(hypr
            .config_path()
            .unwrap()
            .to_string_lossy()
            .contains("hypr/hyprland.conf"));

        let sway = WindowManager::Sway;
        assert!(sway
            .config_path()
            .unwrap()
            .to_string_lossy()
            .contains("sway/config"));
    }

    #[test]
    fn test_env_var_detection() {
        let content = r#"
            # Hyprland config
            env = XCURSOR_SIZE,24
            env = LIBVA_DRIVER_NAME,nvidia
            # env = DISABLED_VAR,value
        "#;

        let mut parser = ConfigParser::new(WindowManager::Hyprland);
        parser.content = Some(content.to_string());

        assert!(parser.has_env_var("XCURSOR_SIZE"));
        assert!(parser.has_env_var("LIBVA_DRIVER_NAME"));
        assert!(!parser.has_env_var("DISABLED_VAR")); // Commented out
        assert!(!parser.has_env_var("NONEXISTENT"));
    }
}
