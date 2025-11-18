//! Desktop Environment Detection
//!
//! Detects the current desktop environment and session type to enable
//! context-aware automation (wallpapers, configs, etc.)

use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

/// Desktop environment type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DesktopEnvironment {
    /// Hyprland (Wayland compositor)
    Hyprland,
    /// i3 window manager (X11)
    I3,
    /// Sway window manager (Wayland, i3-compatible)
    Sway,
    /// KDE Plasma
    Kde,
    /// GNOME
    Gnome,
    /// Xfce
    Xfce,
    /// Other/Unknown desktop environment
    Other(String),
    /// No desktop environment detected (headless/TTY)
    None,
}

/// Session type (display server)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// Wayland session
    Wayland,
    /// X11 session
    X11,
    /// TTY/console (no display server)
    Tty,
    /// Unknown
    Unknown,
}

/// Complete desktop environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopInfo {
    /// Desktop environment
    pub environment: DesktopEnvironment,
    /// Session type
    pub session_type: SessionType,
    /// Config directory (e.g., ~/.config/hypr)
    pub config_dir: Option<PathBuf>,
    /// Primary config file (e.g., ~/.config/hypr/hyprland.conf)
    pub config_file: Option<PathBuf>,
}

impl DesktopEnvironment {
    /// Get human-readable name
    pub fn name(&self) -> &str {
        match self {
            Self::Hyprland => "Hyprland",
            Self::I3 => "i3",
            Self::Sway => "Sway",
            Self::Kde => "KDE Plasma",
            Self::Gnome => "GNOME",
            Self::Xfce => "Xfce",
            Self::Other(name) => name,
            Self::None => "None",
        }
    }

    /// Get config directory name
    pub fn config_dir_name(&self) -> Option<&str> {
        match self {
            Self::Hyprland => Some("hypr"),
            Self::I3 => Some("i3"),
            Self::Sway => Some("sway"),
            Self::Kde => Some("kde"),
            Self::Gnome => Some("gnome"),
            Self::Xfce => Some("xfce4"),
            _ => None,
        }
    }

    /// Get primary config file name
    pub fn config_file_name(&self) -> Option<&str> {
        match self {
            Self::Hyprland => Some("hyprland.conf"),
            Self::I3 => Some("config"),
            Self::Sway => Some("config"),
            Self::Kde => None,   // KDE uses many config files
            Self::Gnome => None, // GNOME uses dconf/gsettings
            Self::Xfce => None,  // Xfce uses many XML files
            _ => None,
        }
    }
}

impl DesktopInfo {
    /// Detect current desktop environment
    pub fn detect() -> Self {
        let session_type = Self::detect_session_type();
        let environment = Self::detect_environment();
        let (config_dir, config_file) = Self::find_config_paths(&environment);

        Self {
            environment,
            session_type,
            config_dir,
            config_file,
        }
    }

    /// Detect session type (Wayland/X11/TTY)
    fn detect_session_type() -> SessionType {
        // Check XDG_SESSION_TYPE
        if let Ok(session_type) = env::var("XDG_SESSION_TYPE") {
            match session_type.to_lowercase().as_str() {
                "wayland" => return SessionType::Wayland,
                "x11" => return SessionType::X11,
                "tty" => return SessionType::Tty,
                _ => {}
            }
        }

        // Check WAYLAND_DISPLAY
        if env::var("WAYLAND_DISPLAY").is_ok() {
            return SessionType::Wayland;
        }

        // Check DISPLAY (X11)
        if env::var("DISPLAY").is_ok() {
            return SessionType::X11;
        }

        // No display server
        SessionType::Tty
    }

    /// Detect desktop environment
    fn detect_environment() -> DesktopEnvironment {
        // Check HYPRLAND_INSTANCE_SIGNATURE (most specific)
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return DesktopEnvironment::Hyprland;
        }

        // Check XDG_CURRENT_DESKTOP
        if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
            let desktop_lower = desktop.to_lowercase();

            if desktop_lower.contains("hyprland") {
                return DesktopEnvironment::Hyprland;
            }
            if desktop_lower.contains("sway") {
                return DesktopEnvironment::Sway;
            }
            if desktop_lower.contains("i3") {
                return DesktopEnvironment::I3;
            }
            if desktop_lower.contains("kde") || desktop_lower.contains("plasma") {
                return DesktopEnvironment::Kde;
            }
            if desktop_lower.contains("gnome") {
                return DesktopEnvironment::Gnome;
            }
            if desktop_lower.contains("xfce") {
                return DesktopEnvironment::Xfce;
            }

            // Return as "Other" if not recognized
            return DesktopEnvironment::Other(desktop);
        }

        // Check DESKTOP_SESSION as fallback
        if let Ok(session) = env::var("DESKTOP_SESSION") {
            let session_lower = session.to_lowercase();

            if session_lower.contains("hyprland") {
                return DesktopEnvironment::Hyprland;
            }
            if session_lower.contains("sway") {
                return DesktopEnvironment::Sway;
            }
            if session_lower.contains("i3") {
                return DesktopEnvironment::I3;
            }
            if session_lower.contains("kde") || session_lower.contains("plasma") {
                return DesktopEnvironment::Kde;
            }
            if session_lower.contains("gnome") {
                return DesktopEnvironment::Gnome;
            }
            if session_lower.contains("xfce") {
                return DesktopEnvironment::Xfce;
            }
        }

        // Check SWAYSOCK for Sway
        if env::var("SWAYSOCK").is_ok() {
            return DesktopEnvironment::Sway;
        }

        // Check I3SOCK for i3
        if env::var("I3SOCK").is_ok() {
            return DesktopEnvironment::I3;
        }

        DesktopEnvironment::None
    }

    /// Find config directory and file paths
    fn find_config_paths(env: &DesktopEnvironment) -> (Option<PathBuf>, Option<PathBuf>) {
        let config_dir_name = match env.config_dir_name() {
            Some(name) => name,
            None => return (None, None),
        };

        // Get XDG_CONFIG_HOME or fallback to ~/.config
        let config_base = env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_else(|_| String::from("/root"));
                PathBuf::from(home).join(".config")
            });

        let config_dir = config_base.join(config_dir_name);

        // Check if directory exists
        if !config_dir.exists() {
            return (None, None);
        }

        // Find primary config file
        let config_file = match env.config_file_name() {
            Some(filename) => {
                let file_path = config_dir.join(filename);
                if file_path.exists() {
                    Some(file_path)
                } else {
                    None
                }
            }
            None => None,
        };

        (Some(config_dir), config_file)
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "{} on {} (config: {})",
            self.environment.name(),
            match self.session_type {
                SessionType::Wayland => "Wayland",
                SessionType::X11 => "X11",
                SessionType::Tty => "TTY",
                SessionType::Unknown => "Unknown",
            },
            self.config_file
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "not found".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_detection() {
        // This will vary by environment, just ensure it doesn't crash
        let info = DesktopInfo::detect();
        println!("Detected: {}", info.summary());
    }

    #[test]
    fn test_environment_names() {
        assert_eq!(DesktopEnvironment::Hyprland.name(), "Hyprland");
        assert_eq!(DesktopEnvironment::I3.name(), "i3");
        assert_eq!(DesktopEnvironment::Sway.name(), "Sway");
    }
}
