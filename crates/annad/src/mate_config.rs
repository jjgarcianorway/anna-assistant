//! MATE Desktop Environment Configuration Intelligence
//!
//! Analyzes MATE installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct MateConfig {
    pub has_caja: bool,
    pub has_mate_terminal: bool,
    pub has_pluma: bool,
    pub has_mate_control_center: bool,
    pub has_mate_screensaver: bool,
    pub has_mate_power_manager: bool,
}

/// Analyze MATE installation and configuration
pub fn analyze_mate() -> Option<MateConfig> {
    info!("Analyzing MATE desktop environment");

    // Check if MATE is installed
    if !Command::new("which")
        .arg("mate-session")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let has_caja = package_installed("caja");
    let has_mate_terminal = package_installed("mate-terminal");
    let has_pluma = package_installed("pluma");
    let has_mate_control_center = package_installed("mate-control-center");
    let has_mate_screensaver = package_installed("mate-screensaver");
    let has_mate_power_manager = package_installed("mate-power-manager");

    Some(MateConfig {
        has_caja,
        has_mate_terminal,
        has_pluma,
        has_mate_control_center,
        has_mate_screensaver,
        has_mate_power_manager,
    })
}

/// Generate MATE-specific recommendations
pub fn generate_mate_recommendations(config: &MateConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating MATE recommendations");

    // Caja file manager
    if !config.has_caja {
        recommendations.push(Advice::new(
            "caja".to_string(),
            "Install Caja file manager for MATE".to_string(),
            "Caja is MATE's default file manager, forked from Nautilus 2.x:\\n\\
             - Traditional two-pane layout\\n\\
             - Spatial and browser modes\\n\\
             - Thumbnail generation\\n\\
             - Extension support\\n\\
             - Integrated with MATE desktop".to_string(),
            "Install Caja".to_string(),
            Some("sudo pacman -S --noconfirm caja".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "desktop".to_string(),
        ));
    }

    // MATE Terminal
    if !config.has_mate_terminal {
        recommendations.push(Advice::new(
            "mate-terminal".to_string(),
            "Install MATE Terminal emulator".to_string(),
            "MATE Terminal is the default terminal emulator for MATE:\\n\\
             - Multiple tabs and profiles\\n\\
             - Custom colors and fonts\\n\\
             - Background transparency\\n\\
             - Traditional GNOME 2 terminal features\\n\\
             - Lightweight and familiar".to_string(),
            "Install MATE Terminal".to_string(),
            Some("sudo pacman -S --noconfirm mate-terminal".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "desktop".to_string(),
        ));
    }

    // Pluma text editor
    if !config.has_pluma {
        recommendations.push(Advice::new(
            "pluma".to_string(),
            "Install Pluma text editor for MATE".to_string(),
            "Pluma is MATE's default text editor, forked from gedit:\\n\\
             - Syntax highlighting for many languages\\n\\
             - Plugin system for extensions\\n\\
             - Multiple document interface\\n\\
             - Search and replace\\n\\
             - Undo/redo with history".to_string(),
            "Install Pluma".to_string(),
            Some("sudo pacman -S --noconfirm pluma".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "desktop".to_string(),
        ));
    }

    // MATE Control Center
    if !config.has_mate_control_center {
        recommendations.push(Advice::new(
            "mate-control-center".to_string(),
            "Install MATE Control Center (System Settings)".to_string(),
            "MATE Control Center is essential for system configuration:\\n\\
             - Appearance settings (themes, fonts, icons)\\n\\
             - Hardware configuration (keyboard, mouse, display)\\n\\
             - Window manager settings\\n\\
             - Network and sound configuration\\n\\
             - User account management".to_string(),
            "Install MATE Control Center".to_string(),
            Some("sudo pacman -S --noconfirm mate-control-center".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "desktop".to_string(),
        ));
    }

    // MATE Screensaver
    if !config.has_mate_screensaver {
        recommendations.push(Advice::new(
            "mate-screensaver".to_string(),
            "Install MATE Screensaver for screen locking".to_string(),
            "MATE Screensaver provides screen locking functionality:\\n\\
             - Lock screen on idle\\n\\
             - Password protection\\n\\
             - Multiple screensaver modes\\n\\
             - Integration with MATE session\\n\\
             - Power management integration".to_string(),
            "Install MATE Screensaver".to_string(),
            Some("sudo pacman -S --noconfirm mate-screensaver".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "desktop".to_string(),
        ));
    }

    // MATE Power Manager
    if !config.has_mate_power_manager {
        recommendations.push(Advice::new(
            "mate-power-manager".to_string(),
            "Install MATE Power Manager for laptops".to_string(),
            "MATE Power Manager is essential for laptop power management:\\n\\
             - Battery monitoring\\n\\
             - CPU frequency scaling\\n\\
             - Suspend/hibernate support\\n\\
             - Screen dimming\\n\\
             - Power profiles (performance/balanced/power-saver)".to_string(),
            "Install MATE Power Manager".to_string(),
            Some("sudo pacman -S --noconfirm mate-power-manager".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/MATE".to_string()],
            "power".to_string(),
        ));
    }

    // GTK themes for MATE
    recommendations.push(Advice::new(
        "mate-gtk-themes".to_string(),
        "Install GTK themes for MATE".to_string(),
        "MATE uses GTK2 and GTK3, supporting classic themes:\\n\\
         - **Mint-Y themes** - Modern flat theme from Linux Mint\\n\\
         - **Arc Theme** - Flat theme with transparent elements\\n\\
         - **Adapta** - Material Design inspired\\n\\
         - **Materia** - Material Design GTK theme\\n\\
         - **TraditionalOk** - Classic GNOME 2 look\\n\\n\\\
         Install themes, then apply in:\\n\\\
         System → Preferences → Appearance → Theme".to_string(),
        "Install GTK themes".to_string(),
        Some("sudo pacman -S --noconfirm arc-gtk-theme adapta-gtk-theme materia-gtk-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/GTK#Themes".to_string()],
        "beautification".to_string(),
    ));

    // Icon themes
    recommendations.push(Advice::new(
        "mate-icon-themes".to_string(),
        "Install modern icon themes for MATE".to_string(),
        "Icon themes enhance MATE's visual appeal:\\n\\
         - **Papirus** - Modern, colorful icon theme\\n\\
         - **Numix Circle** - Circular icons\\n\\
         - **MATE icons** - Traditional MATE icon set\\n\\n\\\
         Install icons, then apply in:\\n\\\
         System → Preferences → Appearance → Theme".to_string(),
        "Install icon themes".to_string(),
        Some("sudo pacman -S --noconfirm papirus-icon-theme mate-icon-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Icons".to_string()],
        "beautification".to_string(),
    ));

    // Caja extensions
    recommendations.push(Advice::new(
        "caja-extensions".to_string(),
        "Install Caja extensions for enhanced functionality".to_string(),
        "Caja extensions add useful features to the file manager:\\n\\
         - **caja-extensions-common** - Extension framework\\n\\
         - **caja-open-terminal** - Open terminal in folder\\n\\
         - **caja-sendto** - Send files via email/IM\\n\\
         - **caja-share** - Samba share management\\n\\
         - **caja-wallpaper** - Set image as wallpaper\\n\\
         - **caja-xattr-tags** - Extended attributes support".to_string(),
        "Install Caja extensions".to_string(),
        Some("sudo pacman -S --noconfirm caja-extensions-common".to_string()),
        RiskLevel::Low,
        Priority::Optional,
        vec!["https://wiki.archlinux.org/title/MATE".to_string()],
        "desktop".to_string(),
    ));

    // MATE applications suite
    recommendations.push(Advice::new(
        "mate-applications".to_string(),
        "Install additional MATE applications".to_string(),
        "MATE comes with a suite of traditional applications:\\n\\
         - **eom** (Eye of MATE) - Image viewer\\n\\
         - **atril** - Document viewer (PDF, PostScript, DVI)\\n\\
         - **engrampa** - Archive manager\\n\\
         - **mate-calc** - Calculator\\n\\
         - **mate-screenshot** - Screenshot tool\\n\\
         - **mate-system-monitor** - System resource monitor\\n\\n\\\
         These apps maintain the traditional GNOME 2 experience.".to_string(),
        "MATE applications suite".to_string(),
        None, // User choice
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/MATE#Applications".to_string()],
        "desktop".to_string(),
    ));

    // Panel applets
    recommendations.push(Advice::new(
        "mate-panel-applets".to_string(),
        "Explore MATE panel applets".to_string(),
        "The MATE panel is highly customizable with applets:\\n\\
         - Right-click panel → Add to Panel\\n\\
         - Popular applets:\\n\\\
           • Window List - Show all open windows\\n\\\
           • Workspace Switcher - Multiple desktops\\n\\\
           • System Monitor - CPU/RAM/Network graphs\\n\\\
           • Weather Report - Weather forecast\\n\\\
           • Dictionary Applet - Quick word lookup\\n\\\
           • Command - Run custom commands\\n\\\
           • Sticky Notes - Desktop notes\\n\\n\\\
         Configure in: System → Preferences → MATE Tweak".to_string(),
        "Panel applets guide".to_string(),
        None, // Manual configuration
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/MATE#Panel".to_string()],
        "desktop".to_string(),
    ));

    // MATE Tweak
    recommendations.push(Advice::new(
        "mate-tweak".to_string(),
        "Install MATE Tweak for advanced customization".to_string(),
        "MATE Tweak provides advanced configuration options:\\n\\
         - Desktop layout presets (Traditional, Modern, Cupertino)\\n\\
         - Panel layout customization\\n\\
         - Window manager selection (Marco, Compiz, Metacity)\\n\\
         - Interface customization\\n\\
         - Font settings\\n\\n\\\
         Note: MATE Tweak is primarily for Linux Mint but works on Arch.".to_string(),
        "Install MATE Tweak".to_string(),
        Some("yay -S --noconfirm mate-tweak".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/MATE".to_string()],
        "desktop".to_string(),
    ));

    // Desktop effects (Compiz)
    recommendations.push(Advice::new(
        "mate-compiz-effects".to_string(),
        "Consider Compiz for desktop effects".to_string(),
        "MATE supports Compiz for advanced desktop effects:\\n\\
         - 3D cube desktop switching\\n\\
         - Wobbly windows\\n\\
         - Window animations\\n\\
         - Transparency effects\\n\\
         - Expo view (show all workspaces)\\n\\n\\\
         Install Compiz and switch window manager in MATE Tweak.\\n\\n\\\
         Note: Compiz is resource-intensive, not recommended for low-end hardware.".to_string(),
        "Desktop effects with Compiz".to_string(),
        None, // Optional
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Compiz".to_string()],
        "beautification".to_string(),
    ));

    // Performance tips
    recommendations.push(Advice::new(
        "mate-performance-tips".to_string(),
        "Optimize MATE performance".to_string(),
        "Improve MATE performance with these tips:\\n\\
         1. **Use Marco** - MATE's default window manager (lightweight)\\n\\
         2. **Disable animations** - System → Preferences → Windows → Enable animations\\n\\
         3. **Reduce applets** - Remove unused panel applets\\n\\
         4. **Limit startup apps** - System → Preferences → Startup Applications\\n\\
         5. **Use lighter themes** - Avoid themes with transparency\\n\\
         6. **Disable compositor** - For better performance on older hardware\\n\\n\\\
         MATE is already lightweight, inheriting GNOME 2's efficiency!".to_string(),
        "Performance optimization tips".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/MATE#Performance".to_string()],
        "performance".to_string(),
    ));

    recommendations
}

/// Check if a package is installed
fn package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
