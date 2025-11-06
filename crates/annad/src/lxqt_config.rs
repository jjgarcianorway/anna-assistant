//! LXQt Desktop Environment Configuration Intelligence
//!
//! Analyzes LXQt installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct LxqtConfig {
    pub has_pcmanfm_qt: bool,
    pub has_lxqt_panel: bool,
    pub has_qterminal: bool,
    pub has_lxqt_config: bool,
    pub has_lxqt_powermanagement: bool,
    pub has_openbox: bool,
}

/// Analyze LXQt installation and configuration
pub fn analyze_lxqt() -> Option<LxqtConfig> {
    info!("Analyzing LXQt desktop environment");

    // Check if LXQt is installed
    if !Command::new("which")
        .arg("startlxqt")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let has_pcmanfm_qt = package_installed("pcmanfm-qt");
    let has_lxqt_panel = package_installed("lxqt-panel");
    let has_qterminal = package_installed("qterminal");
    let has_lxqt_config = package_installed("lxqt-config");
    let has_lxqt_powermanagement = package_installed("lxqt-powermanagement");
    let has_openbox = package_installed("openbox");

    Some(LxqtConfig {
        has_pcmanfm_qt,
        has_lxqt_panel,
        has_qterminal,
        has_lxqt_config,
        has_lxqt_powermanagement,
        has_openbox,
    })
}

/// Generate LXQt-specific recommendations
pub fn generate_lxqt_recommendations(config: &LxqtConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating LXQt recommendations");

    // PCManFM-Qt file manager
    if !config.has_pcmanfm_qt {
        recommendations.push(Advice::new(
            "pcmanfm-qt".to_string(),
            "Install PCManFM-Qt file manager for LXQt".to_string(),
            "PCManFM-Qt is LXQt's default file manager:\\n\\
             - Qt-based file manager (lightweight)\\n\\
             - Tabbed browsing\\n\\
             - Built-in archiver support\\n\\
             - Customizable view modes\\n\\
             - Integrated with LXQt desktop".to_string(),
            "Install PCManFM-Qt".to_string(),
            Some("sudo pacman -S --noconfirm pcmanfm-qt".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/LXQt".to_string()],
            "desktop".to_string(),
        ));
    }

    // LXQt Panel
    if !config.has_lxqt_panel {
        recommendations.push(Advice::new(
            "lxqt-panel".to_string(),
            "Install LXQt Panel (taskbar)".to_string(),
            "LXQt Panel is the desktop panel/taskbar:\\n\\
             - Lightweight and customizable\\n\\
             - Plugin-based architecture\\n\\
             - Multiple panel widgets available\\n\\
             - Application menu, taskbar, system tray\\n\\
             - Quick launcher and desktop switcher".to_string(),
            "Install LXQt Panel".to_string(),
            Some("sudo pacman -S --noconfirm lxqt-panel".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/LXQt".to_string()],
            "desktop".to_string(),
        ));
    }

    // QTerminal
    if !config.has_qterminal {
        recommendations.push(Advice::new(
            "qterminal".to_string(),
            "Install QTerminal emulator for LXQt".to_string(),
            "QTerminal is LXQt's default terminal emulator:\\n\\
             - Lightweight Qt-based terminal\\n\\
             - Multiple tabs support\\n\\
             - Split terminal views\\n\\
             - Customizable appearance\\n\\
             - Dropdown terminal mode".to_string(),
            "Install QTerminal".to_string(),
            Some("sudo pacman -S --noconfirm qterminal".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/LXQt".to_string()],
            "desktop".to_string(),
        ));
    }

    // LXQt Configuration Center
    if !config.has_lxqt_config {
        recommendations.push(Advice::new(
            "lxqt-config".to_string(),
            "Install LXQt Configuration Center".to_string(),
            "LXQt Configuration Center provides system settings:\\n\\
             - Appearance settings (themes, icons, fonts)\\n\\
             - Desktop configuration\\n\\
             - Session settings\\n\\
             - Keyboard and mouse configuration\\n\\
             - Monitor settings".to_string(),
            "Install LXQt Configuration Center".to_string(),
            Some("sudo pacman -S --noconfirm lxqt-config".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/LXQt".to_string()],
            "desktop".to_string(),
        ));
    }

    // LXQt Power Management
    if !config.has_lxqt_powermanagement {
        recommendations.push(Advice::new(
            "lxqt-powermanagement".to_string(),
            "Install LXQt Power Management for laptops".to_string(),
            "LXQt Power Management is essential for laptop users:\\n\\
             - Battery monitoring\\n\\
             - Power profiles (performance/balanced/power-saver)\\n\\
             - Screen brightness control\\n\\
             - Suspend/hibernate support\\n\\
             - Lid close actions".to_string(),
            "Install LXQt Power Management".to_string(),
            Some("sudo pacman -S --noconfirm lxqt-powermanagement".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/LXQt".to_string()],
            "power".to_string(),
        ));
    }

    // Openbox window manager
    if !config.has_openbox {
        recommendations.push(Advice::new(
            "openbox".to_string(),
            "Install Openbox (LXQt's default window manager)".to_string(),
            "Openbox is LXQt's default window manager:\\n\\
             - Extremely lightweight\\n\\
             - Highly configurable\\n\\
             - Right-click menu for applications\\n\\
             - Keyboard-driven workflow\\n\\
             - Themeable appearance\\n\\n\\\n\
             Configure via: ~/.config/openbox/rc.xml".to_string(),
            "Install Openbox".to_string(),
            Some("sudo pacman -S --noconfirm openbox".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Openbox".to_string()],
            "desktop".to_string(),
        ));
    }

    // Qt themes
    recommendations.push(Advice::new(
        "lxqt-qt-themes".to_string(),
        "Install Qt themes for LXQt".to_string(),
        "LXQt uses Qt5/Qt6 themes for visual appearance:\\n\\
         - **Kvantum** - Highly customizable Qt theme engine\\n\\
         - **Breeze** - KDE's default theme (clean, modern)\\n\\
         - **Adwaita-qt** - GNOME-style theme for Qt\\n\\
         - **Arc Theme** - Flat theme with transparent elements\\n\\n\\\n\
         Install themes, then apply in:\\n\\\n\
         LXQt Configuration → Appearance → Widget Style".to_string(),
        "Install Qt themes".to_string(),
        Some("sudo pacman -S --noconfirm kvantum breeze".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Qt#Themes".to_string()],
        "beautification".to_string(),
    ));

    // Icon themes
    recommendations.push(Advice::new(
        "lxqt-icon-themes".to_string(),
        "Install modern icon themes for LXQt".to_string(),
        "Icon themes enhance LXQt's visual appeal:\\n\\
         - **Papirus** - Modern, colorful icon theme\\n\\
         - **Numix Circle** - Circular icons\\n\\
         - **Breeze** - KDE's icon theme\\n\\n\\\n\
         Install icons, then apply in:\\n\\\n\
         LXQt Configuration → Appearance → Icons Theme".to_string(),
        "Install icon themes".to_string(),
        Some("sudo pacman -S --noconfirm papirus-icon-theme breeze-icons".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Icons".to_string()],
        "beautification".to_string(),
    ));

    // LXQt applications suite
    recommendations.push(Advice::new(
        "lxqt-applications".to_string(),
        "Install additional LXQt applications".to_string(),
        "LXQt comes with a suite of lightweight Qt applications:\\n\\
         - **LXImage-Qt** - Image viewer\\n\\
         - **FeatherPad** - Lightweight text editor\\n\\
         - **qpdfview** - PDF viewer\\n\\
         - **LXQt Archiver** - Archive manager\\n\\
         - **Trojitá** - Lightweight email client\\n\\
         - **ScreenGrab** - Screenshot tool\\n\\n\\\n\
         These apps maintain LXQt's lightweight philosophy.".to_string(),
        "LXQt applications suite".to_string(),
        None, // User choice
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/LXQt#Applications".to_string()],
        "desktop".to_string(),
    ));

    // Panel plugins/widgets
    recommendations.push(Advice::new(
        "lxqt-panel-plugins".to_string(),
        "Explore LXQt panel plugins".to_string(),
        "The LXQt panel is highly customizable with plugins:\\n\\
         - Right-click panel → Manage Widgets → Add widgets\\n\\
         - Popular plugins:\\n\\\n\
           • Application Menu - Show all applications\\n\\\n\
           • Desktop Switcher - Multiple desktops\\n\\\n\
           • Quick Launch - Favorite apps\\n\\\n\
           • Task Manager - Window list\\n\\\n\
           • System Tray - Background apps\\n\\\n\
           • Volume - Audio control\\n\\\n\
           • CPU Monitor - System resources\\n\\\n\
           • Network Monitor - Network activity\\n\\n\\\n\
         Configure in: Right-click panel → Configure Panel".to_string(),
        "Panel plugins guide".to_string(),
        None, // Manual configuration
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/LXQt#Panel".to_string()],
        "desktop".to_string(),
    ));

    // Kvantum theme engine
    recommendations.push(Advice::new(
        "kvantum-theme-engine".to_string(),
        "Install Kvantum for advanced Qt theming".to_string(),
        "Kvantum is a powerful Qt theme engine for LXQt:\\n\\
         - SVG-based themes\\n\\
         - Blur effects and transparency\\n\\
         - Highly customizable\\n\\
         - Many pre-made themes available\\n\\
         - Better than default Qt theming\\n\\n\\\n\
         After installing:\\n\\\n\
         1. Install: sudo pacman -S kvantum\\n\\\n\
         2. Install themes: yay -S kvantum-theme-arc\\n\\\n\
         3. Apply: kvantummanager (GUI) or lxqt-config".to_string(),
        "Install Kvantum theme engine".to_string(),
        Some("sudo pacman -S --noconfirm kvantum".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/LXQt#Kvantum".to_string()],
        "beautification".to_string(),
    ));

    // Alternative window managers
    recommendations.push(Advice::new(
        "lxqt-alternative-wm".to_string(),
        "Consider alternative window managers for LXQt".to_string(),
        "LXQt is modular and works with various window managers:\\n\\
         - **Openbox** - Default, lightweight (already using)\\n\\
         - **KWin** - KDE's WM with compositing effects\\n\\
         - **i3** - Tiling window manager\\n\\
         - **bspwm** - Binary space partitioning WM\\n\\n\\\n\
         To switch window manager:\\n\\\n\
         Edit: ~/.config/lxqt/session.conf\\n\\\n\
         Change: window_manager=openbox\\n\\n\\\n\
         Note: Openbox is recommended for best performance!".to_string(),
        "Alternative window managers".to_string(),
        None, // Optional
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/LXQt#Window_managers".to_string()],
        "desktop".to_string(),
    ));

    // Compositor for effects
    recommendations.push(Advice::new(
        "lxqt-compositor".to_string(),
        "Install compositor for desktop effects".to_string(),
        "Add a compositor for transparency and effects:\\n\\
         - **Picom** - Lightweight compositor (recommended)\\n\\
         - **Compton** - Fork of xcompmgr (legacy)\\n\\n\\\n\
         Features with Picom:\\n\\
         - Window transparency\\n\\
         - Shadows and fading\\n\\
         - Blur effects\\n\\
         - VSync for smooth rendering\\n\\n\\\n\
         Install and configure:\\n\\\n\
         sudo pacman -S picom\\n\\\n\
         Auto-start: LXQt Configuration → Session Settings → Autostart".to_string(),
        "Install Picom compositor".to_string(),
        Some("sudo pacman -S --noconfirm picom".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Picom".to_string()],
        "beautification".to_string(),
    ));

    // Performance tips
    recommendations.push(Advice::new(
        "lxqt-performance-tips".to_string(),
        "Optimize LXQt performance".to_string(),
        "Improve LXQt performance with these tips:\\n\\
         1. **Keep Openbox** - Use default WM for best performance\\n\\
         2. **Disable compositor** - If you don't need transparency\\n\\
         3. **Reduce panel widgets** - Remove unused widgets\\n\\
         4. **Limit autostart apps** - LXQt Configuration → Session Settings\\n\\
         5. **Use lightweight apps** - Prefer Qt apps over GTK\\n\\
         6. **Disable animations** - In Openbox configuration\\n\\n\\\n\
         LXQt is already one of the lightest DEs - great for older hardware!".to_string(),
        "Performance optimization tips".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/LXQt#Performance".to_string()],
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
