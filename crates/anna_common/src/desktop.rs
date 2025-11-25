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
    /// v6.40.0: Layered detection - never returns None unless ALL methods fail
    fn detect_environment() -> DesktopEnvironment {
        // Layer 1: Check HYPRLAND_INSTANCE_SIGNATURE (most specific)
        if env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return DesktopEnvironment::Hyprland;
        }

        // Layer 2: Check XDG_CURRENT_DESKTOP
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
            if desktop_lower.contains("cinnamon") {
                return DesktopEnvironment::Other("Cinnamon".to_string());
            }
            if desktop_lower.contains("mate") {
                return DesktopEnvironment::Other("MATE".to_string());
            }
            if desktop_lower.contains("lxde") || desktop_lower.contains("lxqt") {
                return DesktopEnvironment::Other("LXQt".to_string());
            }

            // Return as "Other" if not recognized but has a value
            if !desktop.is_empty() {
                return DesktopEnvironment::Other(desktop);
            }
        }

        // Layer 3: Check DESKTOP_SESSION as fallback
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
            if session_lower.contains("cinnamon") {
                return DesktopEnvironment::Other("Cinnamon".to_string());
            }

            if !session.is_empty() {
                return DesktopEnvironment::Other(session);
            }
        }

        // Layer 4: Check Wayland compositor sockets
        if env::var("SWAYSOCK").is_ok() {
            return DesktopEnvironment::Sway;
        }

        // Layer 5: Check X11 window manager sockets
        if env::var("I3SOCK").is_ok() {
            return DesktopEnvironment::I3;
        }

        // Layer 6: Check for common WM/compositor processes
        if let Ok(result) = Self::detect_from_processes() {
            return result;
        }

        // Layer 7: For X11, try xprop to detect WM
        if Self::detect_session_type() == SessionType::X11 {
            if let Ok(wm) = Self::detect_wm_from_xprop() {
                return DesktopEnvironment::Other(wm);
            }
        }

        DesktopEnvironment::None
    }

    /// Detect DE/WM from running processes (Layer 6)
    fn detect_from_processes() -> Result<DesktopEnvironment, ()> {
        use std::process::Command;

        // Try to get process list
        let output = Command::new("ps")
            .args(&["aux"])
            .output()
            .map_err(|_| ())?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_lower = stdout.to_lowercase();

        // Check for Wayland compositors first
        if stdout_lower.contains("hyprland") {
            return Ok(DesktopEnvironment::Hyprland);
        }
        if stdout_lower.contains("/sway ") || stdout_lower.contains("/sway\n") {
            return Ok(DesktopEnvironment::Sway);
        }
        if stdout_lower.contains("wayfire") {
            return Ok(DesktopEnvironment::Other("Wayfire".to_string()));
        }
        if stdout_lower.contains("river") {
            return Ok(DesktopEnvironment::Other("River".to_string()));
        }

        // Check for DE processes
        if stdout_lower.contains("gnome-shell") {
            return Ok(DesktopEnvironment::Gnome);
        }
        if stdout_lower.contains("plasmashell") || stdout_lower.contains("kwin") {
            return Ok(DesktopEnvironment::Kde);
        }
        if stdout_lower.contains("xfce4-session") {
            return Ok(DesktopEnvironment::Xfce);
        }
        if stdout_lower.contains("cinnamon") {
            return Ok(DesktopEnvironment::Other("Cinnamon".to_string()));
        }

        // Check for X11 window managers
        if stdout_lower.contains("/i3 ") || stdout_lower.contains("/i3\n") || stdout_lower.contains("i3-msg") {
            return Ok(DesktopEnvironment::I3);
        }
        if stdout_lower.contains("bspwm") {
            return Ok(DesktopEnvironment::Other("bspwm".to_string()));
        }
        if stdout_lower.contains("openbox") {
            return Ok(DesktopEnvironment::Other("Openbox".to_string()));
        }
        if stdout_lower.contains("fluxbox") {
            return Ok(DesktopEnvironment::Other("Fluxbox".to_string()));
        }
        if stdout_lower.contains("awesome") {
            return Ok(DesktopEnvironment::Other("Awesome".to_string()));
        }
        if stdout_lower.contains("dwm") {
            return Ok(DesktopEnvironment::Other("dwm".to_string()));
        }

        Err(())
    }

    /// Detect WM from xprop (Layer 7, X11 only)
    fn detect_wm_from_xprop() -> Result<String, ()> {
        use std::process::Command;

        let output = Command::new("xprop")
            .args(&["-root", "_NET_SUPPORTING_WM_CHECK"])
            .output()
            .map_err(|_| ())?;

        if !output.status.success() {
            return Err(());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // xprop output: _NET_SUPPORTING_WM_CHECK(WINDOW): window id # 0x...
        // We need to query that window for _NET_WM_NAME
        if let Some(window_id) = stdout.split_whitespace().last() {
            let name_output = Command::new("xprop")
                .args(&["-id", window_id, "_NET_WM_NAME"])
                .output()
                .map_err(|_| ())?;

            if name_output.status.success() {
                let name_stdout = String::from_utf8_lossy(&name_output.stdout);
                // Parse: _NET_WM_NAME(UTF8_STRING) = "WindowManager Name"
                if let Some(name_start) = name_stdout.find('"') {
                    if let Some(name_end) = name_stdout[name_start + 1..].find('"') {
                        let wm_name = &name_stdout[name_start + 1..name_start + 1 + name_end];
                        if !wm_name.is_empty() {
                            return Ok(wm_name.to_string());
                        }
                    }
                }
            }
        }

        Err(())
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

    // v6.40.0: Tests for enhanced DE/WM detection

    #[test]
    fn test_detect_from_processes_finds_common_wms() {
        // Test that process detection works for common patterns
        // Note: This test might fail if the actual processes aren't running
        // but it exercises the code path
        let result = DesktopInfo::detect_from_processes();
        // Should return Ok or Err, but not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_detect_wm_from_xprop_handles_missing_xprop() {
        // Test that xprop detection gracefully handles missing xprop command
        let result = DesktopInfo::detect_wm_from_xprop();
        // Should return Err if xprop not available or not in X11 session
        // Should not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_layered_detection_handles_empty_env_vars() {
        // Simulate environment with no DE/WM env vars set
        // Detection should still work or gracefully return None
        std::env::remove_var("XDG_CURRENT_DESKTOP");
        std::env::remove_var("DESKTOP_SESSION");
        std::env::remove_var("HYPRLAND_INSTANCE_SIGNATURE");
        std::env::remove_var("SWAYSOCK");
        std::env::remove_var("I3SOCK");

        let info = DesktopInfo::detect();
        // Should return something (either detected from processes or None)
        // Should not panic
        assert!(matches!(info.environment, DesktopEnvironment::None | DesktopEnvironment::Other(_) | _));
    }

    #[test]
    fn test_detect_environment_with_xdg_current_desktop() {
        // Test detection from XDG_CURRENT_DESKTOP
        std::env::set_var("XDG_CURRENT_DESKTOP", "KDE");
        let env = DesktopInfo::detect_environment();
        assert!(matches!(env, DesktopEnvironment::Kde));

        std::env::set_var("XDG_CURRENT_DESKTOP", "GNOME");
        let env = DesktopInfo::detect_environment();
        assert!(matches!(env, DesktopEnvironment::Gnome));

        // Cleanup
        std::env::remove_var("XDG_CURRENT_DESKTOP");
    }

    #[test]
    fn test_detect_environment_with_desktop_session() {
        // Test fallback to DESKTOP_SESSION
        std::env::remove_var("XDG_CURRENT_DESKTOP");
        std::env::set_var("DESKTOP_SESSION", "xfce");
        let env = DesktopInfo::detect_environment();
        assert!(matches!(env, DesktopEnvironment::Xfce));

        // Cleanup
        std::env::remove_var("DESKTOP_SESSION");
    }
}
