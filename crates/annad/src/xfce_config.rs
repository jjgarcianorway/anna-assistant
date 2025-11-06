//! XFCE Desktop Environment Configuration Intelligence
//!
//! Analyzes XFCE installation and provides intelligent recommendations

use anna_common::{Advice, Priority, RiskLevel};
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct XfceConfig {
    pub has_thunar: bool,
    pub has_xfce4_terminal: bool,
    pub has_whisker_menu: bool,
    pub has_compositor: bool,
    pub has_thunar_plugins: bool,
    pub has_panel_plugins: bool,
    pub has_power_manager: bool,
    pub has_screenshooter: bool,
}

/// Analyze XFCE installation and configuration
pub fn analyze_xfce() -> Option<XfceConfig> {
    info!("Analyzing XFCE desktop environment");

    // Check if XFCE is installed
    if !Command::new("which")
        .arg("xfce4-session")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return None;
    }

    let has_thunar = package_installed("thunar");
    let has_xfce4_terminal = package_installed("xfce4-terminal");
    let has_whisker_menu = package_installed("xfce4-whiskermenu-plugin");
    let has_compositor = check_compositor();
    let has_thunar_plugins = package_installed("thunar-archive-plugin")
        || package_installed("thunar-media-tags-plugin");
    let has_panel_plugins = check_panel_plugins();
    let has_power_manager = package_installed("xfce4-power-manager");
    let has_screenshooter = package_installed("xfce4-screenshooter");

    Some(XfceConfig {
        has_thunar,
        has_xfce4_terminal,
        has_whisker_menu,
        has_compositor,
        has_thunar_plugins,
        has_panel_plugins,
        has_power_manager,
        has_screenshooter,
    })
}

/// Generate XFCE-specific recommendations
pub fn generate_xfce_recommendations(config: &XfceConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    info!("Generating XFCE recommendations");

    // Thunar - XFCE file manager
    if !config.has_thunar {
        recommendations.push(Advice::new(
            "thunar".to_string(),
            "Install Thunar file manager for XFCE".to_string(),
            "Thunar is the default file manager for XFCE, designed to be fast and easy to use:\\n\\\
             - Lightweight and fast\\n\\\
             - Bulk renaming\\n\\\
             - Custom actions\\n\\\
             - Plugin support\\n\\\
             - Network share browsing".to_string(),
            "Install Thunar".to_string(),
            Some("sudo pacman -S --noconfirm thunar".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Thunar".to_string()],
            "desktop".to_string(),
        ));
    }

    // Thunar plugins
    if config.has_thunar && !config.has_thunar_plugins {
        recommendations.push(Advice::new(
            "thunar-plugins".to_string(),
            "Install Thunar plugins for enhanced functionality".to_string(),
            "Thunar plugins extend file manager capabilities:\\n\\\
             - **thunar-archive-plugin** - Extract/create archives from context menu\\n\\\
             - **thunar-media-tags-plugin** - Edit media file tags (MP3, OGG)\\n\\\
             - **thunar-volman** - Automatic management of removable drives\\n\\\
             - **tumbler** - Thumbnail generator for images and videos".to_string(),
            "Install Thunar plugins".to_string(),
            Some("sudo pacman -S --noconfirm thunar-archive-plugin thunar-media-tags-plugin thunar-volman tumbler".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Thunar#Plugins_and_addons".to_string()],
            "desktop".to_string(),
        ));
    }

    // XFCE Terminal
    if !config.has_xfce4_terminal {
        recommendations.push(Advice::new(
            "xfce4-terminal".to_string(),
            "Install XFCE Terminal emulator".to_string(),
            "XFCE Terminal is the default terminal for XFCE:\\n\\\
             - Lightweight and fast\\n\\\
             - Drop-down mode (Quake-style)\\n\\\
             - Tabs and transparency\\n\\\
             - Color schemes\\n\\\
             - Integrates perfectly with XFCE".to_string(),
            "Install xfce4-terminal".to_string(),
            Some("sudo pacman -S --noconfirm xfce4-terminal".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Xfce#Terminal_emulator".to_string()],
            "desktop".to_string(),
        ));
    }

    // Whisker Menu - Modern application launcher
    if !config.has_whisker_menu {
        recommendations.push(Advice::new(
            "xfce4-whiskermenu-plugin".to_string(),
            "Install Whisker Menu - modern application launcher".to_string(),
            "Whisker Menu is a modern, fast application launcher for XFCE:\\n\\\
             - Search applications by name\\n\\\
             - Favorites list\\n\\\
             - Recent items\\n\\\
             - Custom commands\\n\\\
             - Much faster than default XFCE menu\\n\\n\\\
             After installing, add it to your panel:\\n\\\
             Right-click panel → Panel → Add New Items → Whisker Menu".to_string(),
            "Install Whisker Menu".to_string(),
            Some("sudo pacman -S --noconfirm xfce4-whiskermenu-plugin".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Xfce#Whisker_Menu".to_string()],
            "desktop".to_string(),
        ));
    }

    // Compositor effects (xfwm4 compositor)
    if !config.has_compositor {
        recommendations.push(Advice::new(
            "xfwm4-compositor".to_string(),
            "Enable xfwm4 compositor for window effects".to_string(),
            "The xfwm4 compositor provides window effects for XFCE:\\n\\\
             - Window shadows\\n\\\
             - Transparency\\n\\\
             - Smooth animations\\n\\\
             - Anti-tearing (vsync)\\n\\n\\\
             Enable in: Settings → Window Manager Tweaks → Compositor tab\\n\\\
             Check 'Enable display compositing'".to_string(),
            "Enable compositor (manual)".to_string(),
            None, // Manual configuration
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Xfce#Compositor".to_string()],
            "beautification".to_string(),
        ));
    }

    // Panel plugins
    if !config.has_panel_plugins {
        recommendations.push(Advice::new(
            "xfce4-panel-plugins".to_string(),
            "Install useful XFCE panel plugins".to_string(),
            "XFCE panel plugins add functionality to your desktop:\\n\\\
             - **xfce4-pulseaudio-plugin** - Volume control\\n\\\
             - **xfce4-cpugraph-plugin** - CPU usage graph\\n\\\
             - **xfce4-netload-plugin** - Network usage monitor\\n\\\
             - **xfce4-weather-plugin** - Weather forecast\\n\\\
             - **xfce4-sensors-plugin** - Hardware sensors (temp, fan)\\n\\\
             - **xfce4-systemload-plugin** - System load monitor\\n\\\
             - **xfce4-clipman-plugin** - Clipboard manager\\n\\n\\\
             After installing, add them to your panel:\\n\\\
             Right-click panel → Panel → Add New Items → Choose plugin".to_string(),
            "Install XFCE panel plugins".to_string(),
            Some("sudo pacman -S --noconfirm xfce4-pulseaudio-plugin xfce4-cpugraph-plugin xfce4-netload-plugin xfce4-weather-plugin xfce4-clipman-plugin".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Xfce#Panel_plugins".to_string()],
            "desktop".to_string(),
        ));
    }

    // Power Manager
    if !config.has_power_manager {
        recommendations.push(Advice::new(
            "xfce4-power-manager".to_string(),
            "Install XFCE Power Manager".to_string(),
            "XFCE Power Manager handles power settings and battery management:\\n\\\
             - Battery monitoring\\n\\\
             - Display brightness control\\n\\\
             - Suspend and hibernate\\n\\\
             - Power button actions\\n\\\
             - Laptop lid close actions\\n\\\
             - Automatic screen blanking".to_string(),
            "Install xfce4-power-manager".to_string(),
            Some("sudo pacman -S --noconfirm xfce4-power-manager".to_string()),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Xfce#Power_manager".to_string()],
            "desktop".to_string(),
        ));
    }

    // Screenshooter
    if !config.has_screenshooter {
        recommendations.push(Advice::new(
            "xfce4-screenshooter".to_string(),
            "Install XFCE Screenshooter".to_string(),
            "XFCE Screenshooter is a screenshot utility:\\n\\\
             - Capture full screen, window, or region\\n\\\
             - Delay option\\n\\\
             - Save or upload screenshots\\n\\\
             - Integrates with panel (screenshot button)\\n\\\
             - Keyboard shortcut support (usually Print Screen)".to_string(),
            "Install xfce4-screenshooter".to_string(),
            Some("sudo pacman -S --noconfirm xfce4-screenshooter".to_string()),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Xfce#Screenshooter".to_string()],
            "desktop".to_string(),
        ));
    }

    // GTK theme recommendations
    recommendations.push(Advice::new(
        "xfce-gtk-themes".to_string(),
        "Install beautiful GTK themes for XFCE".to_string(),
        "XFCE uses GTK for its interface. Popular modern themes:\\n\\\
         - **Arc Theme** - Flat theme with transparent elements\\n\\\
         - **Adapta** - Material Design inspired\\n\\\
         - **Materia (Formerly Flat-Plat)** - Material Design\\n\\\
         - **Gruvbox** - Retro groove color scheme\\n\\n\\\
         Install themes, then apply in:\\n\\\
         Settings → Appearance → Style".to_string(),
        "Install GTK themes".to_string(),
        Some("sudo pacman -S --noconfirm arc-gtk-theme adapta-gtk-theme materia-gtk-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/GTK#Themes".to_string()],
        "beautification".to_string(),
    ));

    // Icon theme recommendations
    recommendations.push(Advice::new(
        "xfce-icon-themes".to_string(),
        "Install modern icon themes for XFCE".to_string(),
        "Icon themes enhance the visual appeal of XFCE:\\n\\\
         - **Papirus** - Modern, colorful icon theme\\n\\\
         - **Numix Circle** - Circular icons with vibrant colors\\n\\\
         - **Adwaita** - GNOME's default clean icons\\n\\n\\\
         Install icons, then apply in:\\n\\\
         Settings → Appearance → Icons".to_string(),
        "Install icon themes".to_string(),
        Some("sudo pacman -S --noconfirm papirus-icon-theme".to_string()),
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Icons#Icon_themes".to_string()],
        "beautification".to_string(),
    ));

    // Notification daemon
    recommendations.push(Advice::new(
        "xfce4-notifyd".to_string(),
        "Ensure XFCE notification daemon is installed".to_string(),
        "xfce4-notifyd displays desktop notifications for XFCE:\\n\\\
         - Application notifications\\n\\\
         - System alerts\\n\\\
         - Customizable position and duration\\n\\\
         - Notification history (Settings → Notifications)".to_string(),
        "Install xfce4-notifyd".to_string(),
        Some("sudo pacman -S --noconfirm xfce4-notifyd".to_string()),
        RiskLevel::Low,
        Priority::Recommended,
        vec!["https://wiki.archlinux.org/title/Desktop_notifications#Xfce".to_string()],
        "desktop".to_string(),
    ));

    // XFCE Goodies - Collection of apps
    recommendations.push(Advice::new(
        "xfce4-goodies".to_string(),
        "Install XFCE Goodies - collection of extra apps".to_string(),
        "xfce4-goodies is a package group containing useful XFCE applications:\\n\\\
         - Task Manager\\n\\\
         - Notes plugin\\n\\\
         - Disk usage plugin\\n\\\
         - Timer plugin\\n\\\
         - And many more panel plugins and utilities\\n\\n\\\
         This is a meta-package that installs many extras at once.".to_string(),
        "Install xfce4-goodies".to_string(),
        Some("sudo pacman -S --noconfirm xfce4-goodies".to_string()),
        RiskLevel::Low,
        Priority::Optional,
        vec!["https://wiki.archlinux.org/title/Xfce#Installation".to_string()],
        "desktop".to_string(),
    ));

    // XFCE customization tips
    recommendations.push(Advice::new(
        "xfce-customization-tips".to_string(),
        "XFCE customization tips and tricks".to_string(),
        "XFCE is highly customizable. Key tips:\\n\\\
         - **Panel:** Right-click → Panel → Panel Preferences (customize layout)\\n\\\
         - **Keyboard shortcuts:** Settings → Keyboard → Application Shortcuts\\n\\\
         - **Auto-start apps:** Settings → Session and Startup → Application Autostart\\n\\\
         - **Window behavior:** Settings → Window Manager → Advanced\\n\\\
         - **Desktop:** Right-click desktop → Desktop Settings (wallpaper, icons)\\n\\\
         - **Workspaces:** Settings → Workspaces (number and names)\\n\\n\\\
         **Pro tip:** Settings Manager has all configuration tools in one place!".to_string(),
        "XFCE customization guide".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Xfce#Tips_and_tricks".to_string()],
        "desktop".to_string(),
    ));

    // Performance tips for XFCE
    recommendations.push(Advice::new(
        "xfce-performance-tips".to_string(),
        "XFCE performance optimization tips".to_string(),
        "Make XFCE even faster with these tweaks:\\n\\\
         1. **Disable compositor:** Settings → Window Manager Tweaks → Compositor (if not needed)\\n\\\
         2. **Reduce window animations:** Settings → Window Manager → Style → Theme\\n\\\
         3. **Disable panel shadows:** Panel Preferences → Appearance\\n\\\
         4. **Use lighter themes:** Choose themes without transparency\\n\\\
         5. **Disable startup services:** Settings → Session and Startup\\n\\\
         6. **Limit panel plugins:** Only use essential plugins\\n\\n\\\
         XFCE is already lightweight - these are for ultra-low-spec systems!".to_string(),
        "Performance optimization tips".to_string(),
        None, // Informational
        RiskLevel::Low,
        Priority::Cosmetic,
        vec!["https://wiki.archlinux.org/title/Xfce#Tips_and_tricks".to_string()],
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

/// Check if xfwm4 compositor is enabled
fn check_compositor() -> bool {
    // Check xfconf for compositor setting
    Command::new("xfconf-query")
        .args(["-c", "xfwm4", "-p", "/general/use_compositing"])
        .output()
        .map(|o| {
            if o.status.success() {
                String::from_utf8_lossy(&o.stdout).trim() == "true"
            } else {
                false
            }
        })
        .unwrap_or(false)
}

/// Check if common panel plugins are installed
fn check_panel_plugins() -> bool {
    package_installed("xfce4-pulseaudio-plugin")
        || package_installed("xfce4-cpugraph-plugin")
        || package_installed("xfce4-weather-plugin")
        || package_installed("xfce4-clipman-plugin")
}
