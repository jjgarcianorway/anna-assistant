//! KDE Plasma Desktop Environment Configuration Intelligence
//!
//! Analyzes KDE Plasma installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct KdeConfig {
    pub has_kde_connect: bool,
    pub has_kvantum: bool,
    pub has_gtk_theme_integration: bool,
    pub has_latte_dock: bool,
    pub has_kwin_effects: bool,
    pub plasma_widgets_installed: Vec<String>,
    pub using_wayland: bool,
}

/// Analyze KDE Plasma installation and configuration
pub fn analyze_kde() -> Option<KdeConfig> {
    info!("Analyzing KDE Plasma desktop environment");

    // Check if KDE Plasma is installed
    if !Command::new("which")
        .arg("plasmashell")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let has_kde_connect = package_installed("kdeconnect");
    let has_kvantum = package_installed("kvantum") || package_installed("kvantum-qt5");
    let has_gtk_theme_integration = package_installed("kde-gtk-config");
    let has_latte_dock = package_installed("latte-dock");
    let has_kwin_effects = check_kwin_effects();
    let plasma_widgets_installed = get_installed_widgets();
    let using_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|s| s.to_lowercase() == "wayland")
        .unwrap_or(false);

    Some(KdeConfig {
        has_kde_connect,
        has_kvantum,
        has_gtk_theme_integration,
        has_latte_dock,
        has_kwin_effects,
        plasma_widgets_installed,
        using_wayland,
    })
}

/// Generate KDE Plasma-specific recommendations
pub fn generate_kde_recommendations(config: &KdeConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating KDE Plasma recommendations");

    // KDE Connect - essential for mobile integration
    if !config.has_kde_connect {
        recommendations.push(Advice::new(
            "kde-connect".to_string(),
            "Install KDE Connect for mobile device integration".to_string(),
            "KDE Connect allows seamless integration between your computer and Android/iOS devices:\n\
             - Share files and links\n\
             - Sync notifications\n\
             - Use phone as remote control\n\
             - Answer SMS from desktop\n\
             - Share clipboard\n\
             - Remote file browsing".to_string(),
            "Install kdeconnect".to_string(),
            Some("sudo pacman -S --noconfirm kdeconnect".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/KDE#KDE_Connect".to_string()],
            "desktop".to_string(),
        ));
    }

    // Kvantum - Advanced Qt/KDE theming
    if !config.has_kvantum {
        recommendations.push(Advice::new(
            "kvantum".to_string(),
            "Install Kvantum for advanced KDE theming".to_string(),
            "Kvantum is a powerful SVG-based theme engine for Qt/KDE applications:\n\
             - Beautiful, modern themes\n\
             - Transparency and blur effects\n\
             - Customizable colors and fonts\n\
             - Popular themes: Layan, Sweet, Orchis\n\
             - Better looking than default Breeze".to_string(),
            "Install kvantum theme engine".to_string(),
            Some("sudo pacman -S --noconfirm kvantum".to_string()),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Uniform_look_for_Qt_and_GTK_applications#Kvantum".to_string()],
            "beautification".to_string(),
        ));
    }

    // GTK Theme Integration - Consistent appearance
    if !config.has_gtk_theme_integration {
        recommendations.push(Advice::new(
            "kde-gtk-config".to_string(),
            "Install GTK theme integration for KDE".to_string(),
            "This package allows you to configure GTK application appearance from KDE System Settings:\n\
             - Makes GTK apps (Firefox, GIMP, etc.) match KDE theme\n\
             - Ensures consistent look across Qt and GTK apps\n\
             - Configure from System Settings > Appearance > Application Style > GTK".to_string(),
            "Install kde-gtk-config".to_string(),
            Some("sudo pacman -S --noconfirm kde-gtk-config".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Uniform_look_for_Qt_and_GTK_applications".to_string()],
            "desktop".to_string(),
        ));
    }

    // Latte Dock - macOS-like dock
    if !config.has_latte_dock {
        recommendations.push(Advice::new(
            "latte-dock".to_string(),
            "Install Latte Dock for enhanced panel experience".to_string(),
            "Latte Dock is a beautiful, feature-rich dock/panel for KDE Plasma:\n\
             - macOS-like dock with animations\n\
             - Multiple docks and panels\n\
             - Auto-hide and dodge windows\n\
             - Customizable appearance\n\
             - Widget support\n\
             - Very popular in KDE community".to_string(),
            "Install latte-dock".to_string(),
            Some("sudo pacman -S --noconfirm latte-dock".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/KDE#Latte_Dock".to_string()],
            "desktop".to_string(),
        ));
    }

    // KDE Applications recommendations
    recommendations.push(Advice::new(
        "kde-applications".to_string(),
        "Consider installing useful KDE applications".to_string(),
        "KDE offers many high-quality applications that integrate perfectly with Plasma:\n\
         - **Dolphin** - Advanced file manager (usually installed)\n\
         - **Konsole** - Powerful terminal emulator\n\
         - **Spectacle** - Screenshot utility\n\
         - **Okular** - Universal document viewer (PDF, ebooks)\n\
         - **Gwenview** - Image viewer\n\
         - **Kate** - Advanced text editor\n\
         - **Ark** - Archive manager\n\
         - **KCalc** - Calculator\n\n\
         Install the kde-applications group for all KDE apps:\n\
         sudo pacman -S kde-applications\n\n\
         Or install individually based on your needs.".to_string(),
        "KDE applications suite".to_string(),
        None, // Manual selection
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/KDE#Applications".to_string()],
        "desktop".to_string(),
    ));

    // KWin effects and desktop effects
    if !config.has_kwin_effects {
        recommendations.push(Advice::new(
            "kwin-effects".to_string(),
            "Enable KWin desktop effects for better experience".to_string(),
            "KWin (KDE's window manager) includes beautiful desktop effects:\n\
             - Window animations (minimize, maximize, close)\n\
             - Desktop cube or desktop grid\n\
             - Blur behind transparent windows\n\
             - Magic Lamp minimize effect\n\
             - Wobbly windows\n\
             - Dim inactive windows\n\n\
             Enable effects in: System Settings > Workspace Behavior > Desktop Effects".to_string(),
            "Enable KWin effects (manual)".to_string(),
            None, // Manual configuration
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/KDE#Desktop_effects".to_string()],
            "beautification".to_string(),
        ));
    }

    // Plasma Widgets recommendations
    if config.plasma_widgets_installed.is_empty() {
        recommendations.push(Advice::new(
            "plasma-widgets".to_string(),
            "Explore useful Plasma widgets".to_string(),
            "Plasma widgets (plasmoids) extend your desktop functionality. Popular widgets:\n\
             - **Event Calendar** - Google Calendar integration\n\
             - **NetSpeed Widget** - Network speed monitor\n\
             - **Simple Weather** - Weather forecast\n\
             - **Thermal Monitor** - CPU/GPU temperatures\n\
             - **Window Title Applet** - Show window title in panel\n\n\
             Discover and install widgets from:\n\
             - Right-click desktop → Add Widgets → Get New Widgets\n\
             - Or install from AUR (search for 'plasma' or 'plasmoid')".to_string(),
            "Explore Plasma widgets".to_string(),
            None, // User choice from store
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/KDE#Plasma_widgets".to_string()],
            "desktop".to_string(),
        ));
    }

    // Wayland-specific recommendations
    if config.using_wayland {
        recommendations.push(Advice::new(
            "kde-wayland-tools".to_string(),
            "Install Wayland-specific tools for KDE Plasma".to_string(),
            "You're using KDE Plasma on Wayland. Consider these tools:\n\
             - **wl-clipboard** - Command-line clipboard for Wayland\n\
             - **xwaylandvideobridge** - Screen sharing for X11 apps on Wayland\n\
             - **plasma-wayland-protocols** - Additional Wayland protocols\n\
             - **kwayland-integration** - Qt Wayland integration".to_string(),
            "Install Wayland utilities for KDE".to_string(),
            Some("sudo pacman -S --noconfirm wl-clipboard xwaylandvideobridge".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/KDE#Wayland".to_string()],
            "desktop".to_string(),
        ));
    } else {
        // X11 session - recommend trying Wayland
        recommendations.push(Advice::new(
            "kde-try-wayland".to_string(),
            "Consider trying KDE Plasma on Wayland".to_string(),
            "You're currently using KDE Plasma on X11. KDE Plasma has excellent Wayland support:\n\
             - Better performance and lower latency\n\
             - Improved security (app isolation)\n\
             - Better multi-monitor support\n\
             - HDR support (on supported hardware)\n\
             - Fractional scaling without blur\n\n\
             To try Wayland: Log out, and select 'Plasma (Wayland)' from the session menu at login screen.\n\
             You can always switch back to X11 if needed.".to_string(),
            "Try Plasma Wayland session".to_string(),
            None, // Manual session selection
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/KDE#Wayland".to_string()],
            "desktop".to_string(),
        ));
    }

    // Performance optimization
    recommendations.push(Advice::new(
        "kde-performance-tweaks".to_string(),
        "Optimize KDE Plasma performance".to_string(),
        "Improve KDE Plasma performance with these tweaks:\n\
         1. **Reduce animations** - System Settings > Workspace Behavior > General Behavior → Animation Speed\n\
         2. **Disable compositor** - For gaming: Alt+Shift+F12 (toggle)\n\
         3. **Limit background services** - System Settings → Background Services\n\
         4. **Use lighter effects** - Disable wobbly windows, desktop cube if laggy\n\
         5. **Hardware acceleration** - Ensure GPU drivers are installed\n\
         6. **Baloo file indexing** - Disable if on SSD: System Settings → Search → File Search".to_string(),
        "Performance recommendations (manual)".to_string(),
        None, // Manual tweaks
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/KDE#Performance".to_string()],
        "performance".to_string(),
    ));

    // KDE System Settings tips
    recommendations.push(Advice::new(
        "kde-system-settings-tour".to_string(),
        "Explore KDE System Settings for customization".to_string(),
        "KDE Plasma is incredibly customizable via System Settings. Key sections:\n\
         - **Appearance** - Themes, colors, fonts, icons\n\
         - **Workspace Behavior** - Desktop effects, window management, activities\n\
         - **Window Management** - Window rules, window behavior, KWin scripts\n\
         - **Shortcuts** - Customize keyboard shortcuts\n\
         - **Startup and Shutdown** - Autostart apps, login screen\n\
         - **Display and Monitor** - Resolution, scaling, night color\n\n\
         Tip: Use the search bar in System Settings to quickly find options!".to_string(),
        "System Settings overview".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/KDE#Configuration".to_string()],
        "desktop".to_string(),
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

/// Check if KWin effects are enabled
fn check_kwin_effects() -> bool {
    // Check kwinrc config for enabled effects
    let kwinrc_path = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".config/kwinrc");

    if !kwinrc_path.exists() {
        return false;
    }

    // If kwinrc exists and has [Plugins] section, assume some effects are enabled
    if let Ok(content) = std::fs::read_to_string(&kwinrc_path) {
        content.contains("[Plugins]")
    } else {
        false
    }
}

/// Get list of installed Plasma widgets
fn get_installed_widgets() -> Vec<String> {
    let widgets_dir = PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join(".local/share/plasma/plasmoids");

    if !widgets_dir.exists() {
        return vec![];
    }

    match std::fs::read_dir(widgets_dir) {
        Ok(entries) => {
            entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .collect()
        }
        Err(_) => vec![],
    }
}
