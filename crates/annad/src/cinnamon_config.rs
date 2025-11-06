//! Cinnamon Desktop Environment Configuration Intelligence
//!
//! Analyzes Cinnamon installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct CinnamonConfig {
    pub has_nemo: bool,
    pub has_cinnamon_control_center: bool,
    pub has_cinnamon_screensaver: bool,
    pub has_spices: bool,
    pub has_nemo_extensions: bool,
}

/// Analyze Cinnamon installation and configuration
pub fn analyze_cinnamon() -> Option<CinnamonConfig> {
    info!("Analyzing Cinnamon desktop environment");

    // Check if Cinnamon is installed
    if !Command::new("which")
        .arg("cinnamon")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let has_nemo = package_installed("nemo");
    let has_cinnamon_control_center = package_installed("cinnamon-control-center");
    let has_cinnamon_screensaver = package_installed("cinnamon-screensaver");
    let has_spices = check_spices_installed();
    let has_nemo_extensions = check_nemo_extensions();

    Some(CinnamonConfig {
        has_nemo,
        has_cinnamon_control_center,
        has_cinnamon_screensaver,
        has_spices,
        has_nemo_extensions,
    })
}

/// Generate Cinnamon-specific recommendations
pub fn generate_cinnamon_recommendations(config: &CinnamonConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating Cinnamon recommendations");

    // Nemo file manager
    if !config.has_nemo {
        recommendations.push(Advice::new(
            "nemo".to_string(),
            "Install Nemo file manager for Cinnamon".to_string(),
            "Nemo is Cinnamon's default file manager, forked from Nautilus:\\n\\\
             - Feature-rich and user-friendly\\n\\\
             - Context menu actions\\n\\\
             - Multiple tabs and split view\\n\\\
             - Plugin/extension support\\n\\\
             - Thumbnail generation".to_string(),
            "Install Nemo".to_string(),
            Some("sudo pacman -S --noconfirm nemo".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Cinnamon#Nemo".to_string()],
            "desktop".to_string(),
        ));
    }

    // Nemo extensions
    if config.has_nemo && !config.has_nemo_extensions {
        recommendations.push(Advice::new(
            "nemo-extensions".to_string(),
            "Install Nemo extensions for enhanced functionality".to_string(),
            "Nemo extensions add useful features to the file manager:\\n\\\
             - **nemo-fileroller** - Archive integration (extract/compress from context menu)\\n\\\
             - **nemo-preview** - Quick file preview (spacebar to preview)\\n\\\
             - **nemo-share** - Samba share management\\n\\\
             - **nemo-image-converter** - Batch image resize/rotate\\n\\\
             - **nemo-audio-tab** - Show audio file metadata".to_string(),
            "Install Nemo extensions".to_string(),
            Some("sudo pacman -S --noconfirm nemo-fileroller nemo-preview".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Cinnamon#Extensions".to_string()],
            "desktop".to_string(),
        ));
    }

    // Cinnamon Control Center
    if !config.has_cinnamon_control_center {
        recommendations.push(Advice::new(
            "cinnamon-control-center".to_string(),
            "Install Cinnamon Control Center (System Settings)".to_string(),
            "The Control Center is Cinnamon's settings application:\\n\\\
             - System settings management\\n\\\
             - Appearance customization\\n\\\
             - Hardware configuration\\n\\\
             - User account management\\n\\\
             - Essential for configuring Cinnamon".to_string(),
            "Install cinnamon-control-center".to_string(),
            Some("sudo pacman -S --noconfirm cinnamon-control-center".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Cinnamon".to_string()],
            "desktop".to_string(),
        ));
    }

    // Cinnamon Screensaver
    if !config.has_cinnamon_screensaver {
        recommendations.push(Advice::new(
            "cinnamon-screensaver".to_string(),
            "Install Cinnamon Screensaver for screen locking".to_string(),
            "Cinnamon Screensaver provides screen locking functionality:\\n\\\
             - Lock screen on idle\\n\\\
             - Password protection\\n\\\
             - Customizable lock screen\\n\\\
             - Integration with Cinnamon session".to_string(),
            "Install cinnamon-screensaver".to_string(),
            Some("sudo pacman -S --noconfirm cinnamon-screensaver".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Cinnamon#Screen_locking".to_string()],
            "desktop".to_string(),
        ));
    }

    // Cinnamon Spices (applets, desklets, extensions)
    if !config.has_spices {
        recommendations.push(Advice::new(
            "cinnamon-spices".to_string(),
            "Explore Cinnamon Spices for customization".to_string(),
            "Cinnamon Spices are community-created extensions:\\n\\\
             - **Applets** - Panel widgets (weather, system monitor, calendar)\\n\\\
             - **Desklets** - Desktop widgets (notes, calendar, system info)\\n\\\
             - **Extensions** - Additional functionality\\n\\\
             - **Themes** - Visual appearance customization\\n\\n\\\
             Browse and install from:\\n\\\
             System Settings → Extensions/Applets/Desklets → Download tab\\n\\n\\\
             Popular applets:\\n\\\
             - Weather applet\\n\\\
             - System monitor (CPU/RAM/Net)\\n\\\
             - Calendar events\\n\\\
             - Color picker".to_string(),
            "Explore Cinnamon Spices".to_string(),
            None, // Installed via GUI
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://cinnamon-spices.linuxmint.com/".to_string()],
            "desktop".to_string(),
        ));
    }

    // GTK themes for Cinnamon
    recommendations.push(Advice::new(
        "cinnamon-gtk-themes".to_string(),
        "Install beautiful GTK themes for Cinnamon".to_string(),
        "Cinnamon uses GTK and supports many themes. Popular themes:\\n\\\
         - **Mint-Y themes** - Official Linux Mint themes (if available)\\n\\\
         - **Arc Theme** - Flat theme with transparent elements\\n\\\
         - **Adapta** - Material Design inspired\\n\\\
         - **Materia** - Material Design GTK theme\\n\\\
         - **Nordic** - Dark bluish Nord-based theme\\n\\n\\\
         Install themes, then apply in:\\n\\\
         System Settings → Themes → Window borders, Controls, Desktop".to_string(),
        "Install GTK themes".to_string(),
        Some("sudo pacman -S --noconfirm arc-gtk-theme adapta-gtk-theme materia-gtk-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/GTK#Themes".to_string()],
        "beautification".to_string(),
    ));

    // Icon themes
    recommendations.push(Advice::new(
        "cinnamon-icon-themes".to_string(),
        "Install modern icon themes for Cinnamon".to_string(),
        "Icon themes enhance Cinnamon's visual appeal:\\n\\\
         - **Papirus** - Modern, colorful icon theme\\n\\\
         - **Numix Circle** - Circular icons\\n\\\
         - **Mint-Y icons** - Official Linux Mint icons (if available)\\n\\n\\\
         Install icons, then apply in:\\n\\\
         System Settings → Themes → Icons".to_string(),
        "Install icon themes".to_string(),
        Some("sudo pacman -S --noconfirm papirus-icon-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Icons".to_string()],
        "beautification".to_string(),
    ));

    // Cinnamon applications
    recommendations.push(Advice::new(
        "cinnamon-applications".to_string(),
        "Consider installing Cinnamon applications".to_string(),
        "Cinnamon comes with a suite of applications that integrate well:\\n\\\
         - **gnome-terminal** or **xterm** - Terminal emulator\\n\\\
         - **xviewer** - Image viewer (fork of Eye of GNOME)\\n\\\
         - **xreader** - Document viewer (fork of Evince)\\n\\\
         - **xed** - Text editor (fork of gedit)\\n\\\
         - **gnome-calculator** - Calculator\\n\\\
         - **gnome-screenshot** - Screenshot tool\\n\\n\\\
         These apps are lightweight and integrate seamlessly with Cinnamon.".to_string(),
        "Cinnamon applications suite".to_string(),
        None, // User choice
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Cinnamon#Applications".to_string()],
        "desktop".to_string(),
    ));

    // Effects and animations
    recommendations.push(Advice::new(
        "cinnamon-effects".to_string(),
        "Customize Cinnamon effects and animations".to_string(),
        "Cinnamon has built-in effects you can customize:\\n\\\
         - **Window effects** - Minimize, maximize animations\\n\\\
         - **Desktop effects** - Workspace switching, overview\\n\\\
         - **Menu animations** - Fade, slide effects\\n\\n\\\
         Configure in: System Settings → Effects\\n\\n\\\
         Tips:\\n\\\
         - Disable effects for better performance on low-end hardware\\n\\\
         - Enable compositor for transparency and shadows\\n\\\
         - Adjust animation speed for snappier feel".to_string(),
        "Effects configuration (manual)".to_string(),
        None, // Manual configuration
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Cinnamon#Desktop_effects".to_string()],
        "beautification".to_string(),
    ));

    // Hot corners and gestures
    recommendations.push(Advice::new(
        "cinnamon-hot-corners".to_string(),
        "Configure hot corners for productivity".to_string(),
        "Hot corners provide quick actions by moving mouse to screen corners:\\n\\\
         - **Show all windows** (Expo view)\\n\\\
         - **Show desktop** (minimize all)\\n\\\
         - **Switch workspaces**\\n\\\
         - **Launch applications**\\n\\n\\\
         Configure in: System Settings → Hot Corners\\n\\n\\\
         Also explore:**\\n\\\
         - **Edge tiling** - Drag windows to screen edges to tile\\n\\\
         - **Keyboard shortcuts** - System Settings → Keyboard → Shortcuts".to_string(),
        "Hot corners setup".to_string(),
        None, // Manual configuration
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Cinnamon#Configuration".to_string()],
        "desktop".to_string(),
    ));

    // Panel customization
    recommendations.push(Advice::new(
        "cinnamon-panel-customization".to_string(),
        "Customize Cinnamon panel and applets".to_string(),
        "The Cinnamon panel is highly customizable:\\n\\\
         - **Panel edit mode** - Right-click panel → Panel edit mode\\n\\\
         - **Add applets** - Right-click panel → Add applets to the panel\\n\\\
         - **Rearrange** - Drag applets to reorder\\n\\\
         - **Multiple panels** - Add additional panels if desired\\n\\\
         - **Panel settings** - Height, position, auto-hide\\n\\n\\\
         Popular applets:\\n\\\
         - Grouped window list\\n\\\
         - System tray\\n\\\
         - Calendar\\n\\\
         - Sound controls\\n\\\
         - Network manager".to_string(),
        "Panel customization guide".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Cinnamon#Panel".to_string()],
        "desktop".to_string(),
    ));

    // Performance tips
    recommendations.push(Advice::new(
        "cinnamon-performance-tips".to_string(),
        "Optimize Cinnamon performance".to_string(),
        "Improve Cinnamon performance with these tips:\\n\\\
         1. **Disable effects** - System Settings → Effects (if on low-end hardware)\\n\\\
         2. **Reduce applets** - Remove unused panel applets\\n\\\
         3. **Limit startup apps** - System Settings → Startup Applications\\n\\\
         4. **Use lighter themes** - Avoid themes with transparency\\n\\\
         5. **Disable file indexing** - If not needed\\n\\\
         6. **GPU drivers** - Ensure proper graphics drivers are installed\\n\\n\\\
         Cinnamon is reasonably lightweight but these help on older systems.".to_string(),
        "Performance optimization tips".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Cinnamon#Performance".to_string()],
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

/// Check if Cinnamon spices (applets/desklets) are installed
fn check_spices_installed() -> bool {
    // Check common spices directories
    let home = std::env::var("HOME").unwrap_or_default();
    let applets_dir = format!("{}/.local/share/cinnamon/applets", home);
    let desklets_dir = format!("{}/.local/share/cinnamon/desklets", home);

    std::path::Path::new(&applets_dir).exists() || std::path::Path::new(&desklets_dir).exists()
}

/// Check if Nemo extensions are installed
fn check_nemo_extensions() -> bool {
    package_installed("nemo-fileroller") || package_installed("nemo-preview")
}
