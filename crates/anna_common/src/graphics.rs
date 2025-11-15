//! Graphics and display detection
//!
//! Detects graphics capabilities and display configuration:
//! - Vulkan support and devices
//! - OpenGL version and renderer
//! - Display session type (Wayland vs X11)
//! - Compositor detection

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Session type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    /// Wayland session
    Wayland,
    /// X11/Xorg session
    X11,
    /// TTY (no graphical session)
    Tty,
    /// Unknown
    Unknown,
}

/// Graphics capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsInfo {
    /// Session type (Wayland, X11, TTY)
    pub session_type: SessionType,
    /// Vulkan support available
    pub vulkan_available: bool,
    /// Vulkan devices (GPU names)
    pub vulkan_devices: Vec<String>,
    /// Vulkan API version (if available)
    pub vulkan_version: Option<String>,
    /// OpenGL support available
    pub opengl_available: bool,
    /// OpenGL version string
    pub opengl_version: Option<String>,
    /// OpenGL renderer (GPU name from OpenGL)
    pub opengl_renderer: Option<String>,
    /// Compositor name (if detected)
    pub compositor: Option<String>,
    /// Display server protocol details
    pub display_server: Option<String>,
}

impl GraphicsInfo {
    /// Detect graphics capabilities
    pub fn detect() -> Self {
        let session_type = detect_session_type();
        let (vulkan_available, vulkan_devices, vulkan_version) = detect_vulkan();
        let (opengl_available, opengl_version, opengl_renderer) = detect_opengl();
        let compositor = detect_compositor(&session_type);
        let display_server = get_display_server_info(&session_type);

        Self {
            session_type,
            vulkan_available,
            vulkan_devices,
            vulkan_version,
            opengl_available,
            opengl_version,
            opengl_renderer,
            compositor,
            display_server,
        }
    }
}

/// Detect session type (Wayland, X11, TTY)
fn detect_session_type() -> SessionType {
    // Check XDG_SESSION_TYPE environment variable
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        match session_type.to_lowercase().as_str() {
            "wayland" => return SessionType::Wayland,
            "x11" => return SessionType::X11,
            "tty" => return SessionType::Tty,
            _ => {}
        }
    }

    // Check WAYLAND_DISPLAY
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return SessionType::Wayland;
    }

    // Check DISPLAY for X11
    if std::env::var("DISPLAY").is_ok() {
        return SessionType::X11;
    }

    // Check loginctl for session type
    if let Ok(output) = Command::new("loginctl").arg("show-session").arg("self").arg("-p").arg("Type").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Type=wayland") {
                return SessionType::Wayland;
            } else if stdout.contains("Type=x11") {
                return SessionType::X11;
            } else if stdout.contains("Type=tty") {
                return SessionType::Tty;
            }
        }
    }

    SessionType::Unknown
}

/// Detect Vulkan support and devices
fn detect_vulkan() -> (bool, Vec<String>, Option<String>) {
    let mut available = false;
    let mut devices = Vec::new();
    let mut version = None;

    // Try vulkaninfo command
    if let Ok(output) = Command::new("vulkaninfo").arg("--summary").output() {
        if output.status.success() {
            available = true;
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Extract API version
            for line in stdout.lines() {
                if line.contains("Vulkan Instance Version:") || line.contains("apiVersion") {
                    if let Some(ver) = line.split(':').nth(1) {
                        version = Some(ver.trim().to_string());
                        break;
                    }
                }
            }

            // Extract device names
            for line in stdout.lines() {
                if line.contains("deviceName") {
                    if let Some(device) = line.split('=').nth(1) {
                        let device_name = device.trim().trim_matches('"').to_string();
                        if !devices.contains(&device_name) {
                            devices.push(device_name);
                        }
                    }
                } else if line.contains("GPU") && !line.contains("GPU id") {
                    // Fallback: look for "GPU0:", "GPU1:", etc.
                    if let Some(device) = line.split(':').nth(1) {
                        let device_name = device.trim().to_string();
                        if !device_name.is_empty() && !devices.contains(&device_name) {
                            devices.push(device_name);
                        }
                    }
                }
            }
        }
    }

    // Fallback: check if vulkan packages are installed
    if !available {
        if let Ok(output) = Command::new("pacman").arg("-Q").arg("vulkan-icd-loader").output() {
            if output.status.success() {
                available = true;
            }
        }
    }

    (available, devices, version)
}

/// Detect OpenGL support and version
fn detect_opengl() -> (bool, Option<String>, Option<String>) {
    let mut available = false;
    let mut version = None;
    let mut renderer = None;

    // Try glxinfo command (X11)
    if let Ok(output) = Command::new("glxinfo").arg("-B").output() {
        if output.status.success() {
            available = true;
            let stdout = String::from_utf8_lossy(&output.stdout);

            for line in stdout.lines() {
                if line.starts_with("OpenGL version string:") {
                    if let Some(ver) = line.split(':').nth(1) {
                        version = Some(ver.trim().to_string());
                    }
                } else if line.starts_with("OpenGL renderer string:") {
                    if let Some(rend) = line.split(':').nth(1) {
                        renderer = Some(rend.trim().to_string());
                    }
                }
            }
        }
    }

    // Try eglinfo command (Wayland/EGL)
    if !available {
        if let Ok(output) = Command::new("eglinfo").output() {
            if output.status.success() {
                available = true;
                let stdout = String::from_utf8_lossy(&output.stdout);

                for line in stdout.lines() {
                    if line.contains("EGL version:") || line.contains("OpenGL ES") {
                        version = Some(line.split(':').nth(1).unwrap_or("").trim().to_string());
                        break;
                    }
                }
            }
        }
    }

    // Fallback: check if mesa or proprietary drivers are installed
    if !available {
        let mesa_check = Command::new("pacman").arg("-Q").arg("mesa").output();
        let nvidia_check = Command::new("pacman").arg("-Q").arg("nvidia").output();

        if mesa_check.is_ok() || nvidia_check.is_ok() {
            available = true;
        }
    }

    (available, version, renderer)
}

/// Detect compositor
fn detect_compositor(session_type: &SessionType) -> Option<String> {
    // Check common environment variables
    if let Ok(compositor) = std::env::var("WAYLAND_COMPOSITOR") {
        return Some(compositor);
    }

    // For Wayland sessions
    if matches!(session_type, SessionType::Wayland) {
        // Check for common Wayland compositors
        let compositors = vec![
            "hyprland",
            "sway",
            "wayfire",
            "river",
            "labwc",
            "kwin_wayland",
            "gnome-shell",
            "mutter",
        ];

        for comp in compositors {
            if let Ok(output) = Command::new("pgrep").arg("-x").arg(comp).output() {
                if output.status.success() && !output.stdout.is_empty() {
                    return Some(comp.to_string());
                }
            }
        }
    }

    // For X11 sessions
    if matches!(session_type, SessionType::X11) {
        // Check for X11 compositors
        let compositors = vec!["picom", "compton", "xcompmgr", "kwin", "mutter", "compiz"];

        for comp in compositors {
            if let Ok(output) = Command::new("pgrep").arg("-x").arg(comp).output() {
                if output.status.success() && !output.stdout.is_empty() {
                    return Some(comp.to_string());
                }
            }
        }
    }

    None
}

/// Get display server information
fn get_display_server_info(session_type: &SessionType) -> Option<String> {
    match session_type {
        SessionType::Wayland => {
            if let Ok(display) = std::env::var("WAYLAND_DISPLAY") {
                Some(format!("Wayland ({})", display))
            } else {
                Some("Wayland".to_string())
            }
        }
        SessionType::X11 => {
            if let Ok(display) = std::env::var("DISPLAY") {
                // Try to get X server version
                if let Ok(output) = Command::new("X").arg("-version").output() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    for line in stderr.lines() {
                        if line.contains("X.Org") || line.contains("X Server") {
                            return Some(format!("{} ({})", line.trim(), display));
                        }
                    }
                }
                Some(format!("X11 ({})", display))
            } else {
                Some("X11".to_string())
            }
        }
        SessionType::Tty => Some("TTY (no graphical session)".to_string()),
        SessionType::Unknown => None,
    }
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionType::Wayland => write!(f, "Wayland"),
            SessionType::X11 => write!(f, "X11"),
            SessionType::Tty => write!(f, "TTY"),
            SessionType::Unknown => write!(f, "Unknown"),
        }
    }
}
