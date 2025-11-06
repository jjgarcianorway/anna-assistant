//! GNOME Desktop Environment Configuration Intelligence
//!
//! Analyzes GNOME installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct GnomeConfig {
    pub gnome_version: Option<String>,
    pub has_gnome_tweaks: bool,
    pub has_extensions_app: bool,
    pub has_gtk_theme: bool,
    pub has_icon_theme: bool,
    pub shell_extensions_enabled: bool,
    pub installed_extensions: Vec<String>,
    pub has_dconf_editor: bool,
    pub using_wayland: bool,
}

/// Analyze GNOME installation and configuration
pub fn analyze_gnome() -> Option<GnomeConfig> {
    info!("Analyzing GNOME desktop environment");

    // Check if GNOME is installed
    if !Command::new("which")
        .arg("gnome-shell")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let gnome_version = get_gnome_version();
    let has_gnome_tweaks = package_installed("gnome-tweaks");
    let has_extensions_app = package_installed("gnome-shell-extensions")
        || package_installed("gnome-extensions-app");
    let has_gtk_theme = check_gtk_theme();
    let has_icon_theme = check_icon_theme();
    let shell_extensions_enabled = check_extensions_enabled();
    let installed_extensions = get_installed_extensions();
    let has_dconf_editor = package_installed("dconf-editor");
    let using_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|s| s.to_lowercase() == "wayland")
        .unwrap_or(false);

    Some(GnomeConfig {
        gnome_version,
        has_gnome_tweaks,
        has_extensions_app,
        has_gtk_theme,
        has_icon_theme,
        shell_extensions_enabled,
        installed_extensions,
        has_dconf_editor,
        using_wayland,
    })
}

/// Generate GNOME-specific recommendations
pub fn generate_gnome_recommendations(config: &GnomeConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating GNOME recommendations");

    // GNOME Tweaks - essential for customization
    if !config.has_gnome_tweaks {
        recommendations.push(Advice::new(
            "gnome-tweaks".to_string(),
            "Install GNOME Tweaks for customization".to_string(),
            "GNOME Tweaks provides access to many settings not available in the default Settings app, including:\n\
             - Window title bar buttons\n\
             - Fonts and themes\n\
             - Top bar customization\n\
             - Extensions management\n\
             - Power settings\n\
             - Startup applications".to_string(),
            "Install gnome-tweaks".to_string(),
            Some("sudo pacman -S --noconfirm gnome-tweaks".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/GNOME#Customization".to_string()],
            "desktop".to_string(),
        ));
    }

    // Extensions App for GNOME 40+
    if !config.has_extensions_app && config.gnome_version.as_ref().map(|v| v.starts_with("4")).unwrap_or(false) {
        recommendations.push(Advice::new(
            "gnome-extensions-app".to_string(),
            "Install GNOME Extensions App".to_string(),
            "GNOME Extensions App allows you to manage shell extensions without browser integration. \
             Essential for customizing GNOME with extensions like:\n\
             - Dash to Dock\n\
             - AppIndicator Support\n\
             - Blur My Shell\n\
             - User Themes".to_string(),
            "Install gnome-shell-extensions".to_string(),
            Some("sudo pacman -S --noconfirm gnome-shell-extensions".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/GNOME#Extensions".to_string()],
            "desktop".to_string(),
        ));
    }

    // dconf-editor - power user tool
    if !config.has_dconf_editor {
        recommendations.push(Advice::new(
            "dconf-editor".to_string(),
            "Install dconf Editor for advanced settings".to_string(),
            "dconf Editor lets you modify hidden GNOME settings not available in the GUI. \
             Useful for power users who want complete control over their desktop.".to_string(),
            "Install dconf-editor".to_string(),
            Some("sudo pacman -S --noconfirm dconf-editor".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/GNOME#dconf".to_string()],
            "desktop".to_string(),
        ));
    }

    // GTK Theme recommendation
    if !config.has_gtk_theme {
        recommendations.push(Advice::new(
            "gnome-gtk-theme".to_string(),
            "Install additional GTK themes".to_string(),
            "Enhance GNOME's appearance with popular GTK themes. Consider:\n\
             - Arc Theme (clean, modern)\n\
             - Adapta (Material Design)\n\
             - Materia (Material inspired)\n\
             - Orchis (rounded, colorful)".to_string(),
            "Install popular GTK themes".to_string(),
            Some("sudo pacman -S --noconfirm arc-gtk-theme adapta-gtk-theme materia-gtk-theme".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/GTK#Themes".to_string()],
            "beautification".to_string(),
        ));
    }

    // Icon theme recommendation
    if !config.has_icon_theme {
        recommendations.push(Advice::new(
            "gnome-icon-theme".to_string(),
            "Install modern icon themes".to_string(),
            "Improve GNOME's visual appeal with modern icon themes. Popular choices:\n\
             - Papirus (colorful, comprehensive)\n\
             - Numix Circle (circular, modern)\n\
             - Tela (macOS-inspired)\n\
             - Colloid (Material Design)".to_string(),
            "Install popular icon themes".to_string(),
            Some("sudo pacman -S --noconfirm papirus-icon-theme".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Icons#Icon_themes".to_string()],
            "beautification".to_string(),
        ));
    }

    // Recommend essential GNOME extensions
    if config.shell_extensions_enabled && config.installed_extensions.is_empty() {
        recommendations.push(Advice::new(
            "gnome-essential-extensions".to_string(),
            "Install essential GNOME Shell extensions".to_string(),
            "Enhance GNOME functionality with essential extensions:\n\
             1. **Dash to Dock** - Transforms dash into a dock for quick app access\n\
             2. **AppIndicator Support** - Shows tray icons in top bar\n\
             3. **Blur My Shell** - Adds blur effects for aesthetics\n\
             4. **User Themes** - Allows custom shell themes\n\
             5. **Clipboard Indicator** - Clipboard manager in top bar\n\n\
             Install via GNOME Extensions website or AUR packages.".to_string(),
            "Install gnome-shell-extensions package".to_string(),
            Some("sudo pacman -S --noconfirm gnome-shell-extensions".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec![
                "https://wiki.archlinux.org/title/GNOME#Extensions".to_string(),
                "https://extensions.gnome.org/".to_string(),
            ],
            "desktop".to_string(),
        ));
    }

    // Wayland-specific recommendations
    if config.using_wayland {
        recommendations.push(Advice::new(
            "gnome-wayland-tools".to_string(),
            "Install Wayland-specific tools for GNOME".to_string(),
            "You're using GNOME on Wayland. Consider these tools:\n\
             - **wl-clipboard** - Command-line clipboard for Wayland\n\
             - **wtype** - xdotool alternative for Wayland\n\
             - **grim** - Screenshot utility\n\
             - **slurp** - Region selection tool".to_string(),
            "Install Wayland utilities".to_string(),
            Some("sudo pacman -S --noconfirm wl-clipboard wtype grim slurp".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Wayland#Utilities".to_string()],
            "desktop".to_string(),
        ));
    }

    // Performance optimization
    recommendations.push(Advice::new(
        "gnome-performance-tweaks".to_string(),
        "Optimize GNOME performance".to_string(),
        "Improve GNOME performance with these tweaks:\n\
         1. Disable unused animations\n\
         2. Reduce window effects\n\
         3. Limit background apps\n\
         4. Use fractional scaling carefully (performance hit)\n\n\
         Use GNOME Tweaks to adjust these settings.".to_string(),
        "Performance recommendations (manual)".to_string(),
        None, // Manual tweaks, no automated command
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/GNOME#Performance".to_string()],
        "performance".to_string(),
    ));

    recommendations
}

/// Get GNOME version
fn get_gnome_version() -> Option<String> {
    let output = Command::new("gnome-shell")
        .arg("--version")
        .output()
        .ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        Some(version_str.trim().to_string())
    } else {
        None
    }
}

/// Check if a package is installed
fn package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if GTK theme is configured
fn check_gtk_theme() -> bool {
    let gtk3_settings = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config/gtk-3.0/settings.ini");

    if !gtk3_settings.exists() {
        return false;
    }

    // Check if a theme is set
    if let Ok(content) = std::fs::read_to_string(&gtk3_settings) {
        content.contains("gtk-theme-name")
    } else {
        false
    }
}

/// Check if icon theme is configured
fn check_icon_theme() -> bool {
    // Check for popular icon theme packages
    package_installed("papirus-icon-theme")
        || package_installed("numix-circle-icon-theme")
        || package_installed("tela-icon-theme")
}

/// Check if GNOME Shell extensions are enabled
fn check_extensions_enabled() -> bool {
    // Check if user-extensions directory exists
    let extensions_dir = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/gnome-shell/extensions");

    extensions_dir.exists()
}

/// Get list of installed GNOME Shell extensions
fn get_installed_extensions() -> Vec<String> {
    let extensions_dir = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/gnome-shell/extensions");

    if !extensions_dir.exists() {
        return vec![];
    }

    match std::fs::read_dir(extensions_dir) {
        Ok(entries) => {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        }
        Err(_) => vec![],
    }
}
