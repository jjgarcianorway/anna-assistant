//! Desktop Environment and Window Manager Detection
//! v6.40.0: Multi-layered detection with confidence scoring
//!
//! Detection layers (in order):
//! 1. Environment variables (XDG_CURRENT_DESKTOP, DESKTOP_SESSION)
//! 2. Running processes (compositor, WM, DE session)
//! 3. Installed packages (pacman -Q)
//! 4. Config files in ~/.config
//! 5. X11 properties (xprop for X11 systems)

use std::env;
use std::process::Command;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct DeWmInfo {
    pub name: String,
    pub de_type: DeType,
    pub confidence: Confidence,
    pub detection_method: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeType {
    DesktopEnvironment,  // GNOME, KDE, XFCE, etc.
    WindowManager,       // i3, sway, bspwm, etc.
    Compositor,          // Hyprland, wayfire (standalone Wayland)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,     // Only one weak signal
    Medium,  // Multiple signals OR one strong signal
    High,    // Multiple strong signals
}

/// Detect desktop environment or window manager
pub fn detect_de_wm() -> DeWmInfo {
    // Layer 1: Environment variables (highest confidence)
    if let Some(info) = check_environment_variables() {
        return info;
    }

    // Layer 2: Running processes (high confidence)
    if let Some(info) = check_running_processes() {
        return info;
    }

    // Layer 3: Installed packages (medium confidence)
    if let Some(info) = check_installed_packages() {
        return info;
    }

    // Layer 4: Config directories (low-medium confidence)
    if let Some(info) = check_config_directories() {
        return info;
    }

    // Layer 5: X11 properties (for X11 systems)
    if let Some(info) = check_x11_properties() {
        return info;
    }

    // Fallback: Could not detect
    DeWmInfo {
        name: "Unable to detect".to_string(),
        de_type: DeType::WindowManager,
        confidence: Confidence::Low,
        detection_method: "No detection methods succeeded".to_string(),
    }
}

fn check_environment_variables() -> Option<DeWmInfo> {
    // XDG_CURRENT_DESKTOP is the most reliable
    if let Ok(desktop) = env::var("XDG_CURRENT_DESKTOP") {
        if !desktop.is_empty() && desktop != "unknown" {
            return Some(DeWmInfo {
                name: normalize_de_name(&desktop),
                de_type: classify_de_type(&desktop),
                confidence: Confidence::High,
                detection_method: "XDG_CURRENT_DESKTOP environment variable".to_string(),
            });
        }
    }

    // DESKTOP_SESSION as fallback
    if let Ok(session) = env::var("DESKTOP_SESSION") {
        if !session.is_empty() && session != "unknown" {
            return Some(DeWmInfo {
                name: normalize_de_name(&session),
                de_type: classify_de_type(&session),
                confidence: Confidence::High,
                detection_method: "DESKTOP_SESSION environment variable".to_string(),
            });
        }
    }

    None
}

fn check_running_processes() -> Option<DeWmInfo> {
    let output = Command::new("ps")
        .args(["aux"])
        .output()
        .ok()?;

    let processes = String::from_utf8_lossy(&output.stdout);

    // Check for DE processes (high priority)
    let de_patterns = [
        ("gnome-shell", "GNOME", DeType::DesktopEnvironment),
        ("plasmashell", "KDE Plasma", DeType::DesktopEnvironment),
        ("xfce4-session", "XFCE", DeType::DesktopEnvironment),
        ("cinnamon-session", "Cinnamon", DeType::DesktopEnvironment),
        ("mate-session", "MATE", DeType::DesktopEnvironment),
        ("lxsession", "LXDE", DeType::DesktopEnvironment),
        ("lxqt-session", "LXQt", DeType::DesktopEnvironment),
    ];

    for (process, name, de_type) in &de_patterns {
        if processes.contains(process) {
            return Some(DeWmInfo {
                name: name.to_string(),
                de_type: de_type.clone(),
                confidence: Confidence::High,
                detection_method: format!("Running process: {}", process),
            });
        }
    }

    // Check for Wayland compositors
    let wayland_patterns = [
        ("sway", "Sway", DeType::Compositor),
        ("hyprland", "Hyprland", DeType::Compositor),
        ("river", "River", DeType::Compositor),
        ("wayfire", "Wayfire", DeType::Compositor),
        ("labwc", "Labwc", DeType::Compositor),
    ];

    for (process, name, de_type) in &wayland_patterns {
        if processes.contains(process) {
            return Some(DeWmInfo {
                name: name.to_string(),
                de_type: de_type.clone(),
                confidence: Confidence::High,
                detection_method: format!("Running Wayland compositor: {}", process),
            });
        }
    }

    // Check for X11 window managers
    let wm_patterns = [
        ("i3", "i3", DeType::WindowManager),
        ("bspwm", "bspwm", DeType::WindowManager),
        ("awesome", "Awesome", DeType::WindowManager),
        ("openbox", "Openbox", DeType::WindowManager),
        ("fluxbox", "Fluxbox", DeType::WindowManager),
        ("dwm", "dwm", DeType::WindowManager),
        ("xmonad", "XMonad", DeType::WindowManager),
        ("qtile", "Qtile", DeType::WindowManager),
    ];

    for (process, name, de_type) in &wm_patterns {
        if processes.contains(process) {
            return Some(DeWmInfo {
                name: name.to_string(),
                de_type: de_type.clone(),
                confidence: Confidence::Medium,
                detection_method: format!("Running window manager: {}", process),
            });
        }
    }

    None
}

fn check_installed_packages() -> Option<DeWmInfo> {
    let output = Command::new("pacman")
        .args(["-Qq"])
        .output()
        .ok()?;

    let packages = String::from_utf8_lossy(&output.stdout);

    // Check for major DE packages
    let package_patterns = [
        ("gnome-shell", "GNOME", DeType::DesktopEnvironment),
        ("plasma-desktop", "KDE Plasma", DeType::DesktopEnvironment),
        ("xfce4-session", "XFCE", DeType::DesktopEnvironment),
        ("cinnamon", "Cinnamon", DeType::DesktopEnvironment),
        ("mate-desktop", "MATE", DeType::DesktopEnvironment),
        ("sway", "Sway", DeType::Compositor),
        ("hyprland", "Hyprland", DeType::Compositor),
        ("i3-wm", "i3", DeType::WindowManager),
        ("bspwm", "bspwm", DeType::WindowManager),
    ];

    for (package, name, de_type) in &package_patterns {
        if packages.lines().any(|line| line == *package) {
            return Some(DeWmInfo {
                name: name.to_string(),
                de_type: de_type.clone(),
                confidence: Confidence::Medium,
                detection_method: format!("Installed package: {}", package),
            });
        }
    }

    None
}

fn check_config_directories() -> Option<DeWmInfo> {
    let home = env::var("HOME").ok()?;
    let config_base = format!("{}/.config", home);

    // Check for config directories
    let config_patterns = [
        ("i3", "i3", DeType::WindowManager),
        ("sway", "Sway", DeType::Compositor),
        ("hypr", "Hyprland", DeType::Compositor),
        ("bspwm", "bspwm", DeType::WindowManager),
        ("awesome", "Awesome", DeType::WindowManager),
        ("qtile", "Qtile", DeType::WindowManager),
    ];

    for (dir, name, de_type) in &config_patterns {
        let path = format!("{}/{}", config_base, dir);
        if Path::new(&path).exists() {
            return Some(DeWmInfo {
                name: name.to_string(),
                de_type: de_type.clone(),
                confidence: Confidence::Low,
                detection_method: format!("Config directory: ~/.config/{}", dir),
            });
        }
    }

    None
}

fn check_x11_properties() -> Option<DeWmInfo> {
    // Try xprop to get window manager name (X11 only)
    let output = Command::new("xprop")
        .args(["-root", "_NET_SUPPORTING_WM_CHECK"])
        .output()
        .ok()?;

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        if result.contains("window id") {
            // Extract WM name from xprop output
            let wm_output = Command::new("xprop")
                .args(["-root", "_NET_WM_NAME"])
                .output()
                .ok()?;

            let wm_name = String::from_utf8_lossy(&wm_output.stdout);
            if let Some(name) = extract_wm_name(&wm_name) {
                return Some(DeWmInfo {
                    name: normalize_de_name(&name),
                    de_type: classify_de_type(&name),
                    confidence: Confidence::Medium,
                    detection_method: "X11 properties (xprop)".to_string(),
                });
            }
        }
    }

    None
}

fn normalize_de_name(raw_name: &str) -> String {
    let name = raw_name.to_lowercase();

    if name.contains("gnome") {
        "GNOME".to_string()
    } else if name.contains("kde") || name.contains("plasma") {
        "KDE Plasma".to_string()
    } else if name.contains("xfce") {
        "XFCE".to_string()
    } else if name.contains("cinnamon") {
        "Cinnamon".to_string()
    } else if name.contains("mate") {
        "MATE".to_string()
    } else if name.contains("lxqt") {
        "LXQt".to_string()
    } else if name.contains("lxde") {
        "LXDE".to_string()
    } else if name.contains("sway") {
        "Sway".to_string()
    } else if name.contains("hyprland") || name.contains("hypr") {
        "Hyprland".to_string()
    } else if name.contains("i3") {
        "i3".to_string()
    } else if name.contains("bspwm") {
        "bspwm".to_string()
    } else if name.contains("awesome") {
        "Awesome".to_string()
    } else {
        // Capitalize first letter
        let mut chars = raw_name.chars();
        match chars.next() {
            None => raw_name.to_string(),
            Some(f) => f.to_uppercase().chain(chars).collect(),
        }
    }
}

fn classify_de_type(name: &str) -> DeType {
    let name_lower = name.to_lowercase();

    if name_lower.contains("gnome")
        || name_lower.contains("kde")
        || name_lower.contains("xfce")
        || name_lower.contains("cinnamon")
        || name_lower.contains("mate")
        || name_lower.contains("lxqt")
        || name_lower.contains("lxde")
    {
        DeType::DesktopEnvironment
    } else if name_lower.contains("sway")
        || name_lower.contains("hyprland")
        || name_lower.contains("wayfire")
        || name_lower.contains("river")
    {
        DeType::Compositor
    } else {
        DeType::WindowManager
    }
}

fn extract_wm_name(xprop_output: &str) -> Option<String> {
    // Extract quoted string from xprop output
    // Example: _NET_WM_NAME(UTF8_STRING) = "i3"
    xprop_output
        .split('"')
        .nth(1)
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_de_wm_does_not_crash() {
        let result = detect_de_wm();
        // Just ensure it doesn't crash
        assert!(!result.name.is_empty());
    }

    #[test]
    fn test_normalize_gnome() {
        assert_eq!(normalize_de_name("gnome"), "GNOME");
        assert_eq!(normalize_de_name("GNOME"), "GNOME");
    }

    #[test]
    fn test_normalize_kde() {
        assert_eq!(normalize_de_name("kde"), "KDE Plasma");
        assert_eq!(normalize_de_name("plasma"), "KDE Plasma");
    }

    #[test]
    fn test_classify_de() {
        assert_eq!(classify_de_type("gnome"), DeType::DesktopEnvironment);
        assert_eq!(classify_de_type("KDE Plasma"), DeType::DesktopEnvironment);
    }

    #[test]
    fn test_classify_wm() {
        assert_eq!(classify_de_type("i3"), DeType::WindowManager);
        assert_eq!(classify_de_type("bspwm"), DeType::WindowManager);
    }

    #[test]
    fn test_classify_compositor() {
        assert_eq!(classify_de_type("sway"), DeType::Compositor);
        assert_eq!(classify_de_type("hyprland"), DeType::Compositor);
    }
}
