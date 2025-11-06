//! Hyprland Configuration Intelligence
//!
//! Detects, analyzes, and improves Hyprland configurations
//! Implements the roadmap vision for comprehensive WM configuration

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Hyprland configuration analysis
#[derive(Debug, Clone)]
pub struct HyprlandConfig {
    pub config_path: PathBuf,
    #[allow(dead_code)]
    pub config_content: String,
    pub has_volume_control: bool,
    pub has_brightness_control: bool,
    pub has_mute_toggle: bool,
    pub has_screenshot_tool: bool,
    pub has_media_controls: bool,
    pub has_app_launcher: bool,
    pub detected_launcher: Option<String>,
    pub has_waybar: bool,
    pub has_wallpaper_manager: bool,
    pub has_lock_screen: bool,
    pub has_notification_daemon: bool,
}

impl Default for HyprlandConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::new(),
            config_content: String::new(),
            has_volume_control: false,
            has_brightness_control: false,
            has_mute_toggle: false,
            has_screenshot_tool: false,
            has_media_controls: false,
            has_app_launcher: false,
            detected_launcher: None,
            has_waybar: false,
            has_wallpaper_manager: false,
            has_lock_screen: false,
            has_notification_daemon: false,
        }
    }
}

/// Required packages for Hyprland features
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct HyprlandPackages {
    volume_tools: Vec<&'static str>,
    brightness_tools: Vec<&'static str>,
    screenshot_tools: Vec<&'static str>,
    media_tools: Vec<&'static str>,
    launchers: Vec<&'static str>,
    status_bars: Vec<&'static str>,
    wallpaper_managers: Vec<&'static str>,
    lock_screens: Vec<&'static str>,
    notification_daemons: Vec<&'static str>,
}

impl Default for HyprlandPackages {
    fn default() -> Self {
        Self {
            volume_tools: vec!["wireplumber", "pamixer", "pulsemixer"],
            brightness_tools: vec!["brightnessctl", "light"],
            screenshot_tools: vec!["grim", "slurp", "grimblast"],
            media_tools: vec!["playerctl"],
            launchers: vec!["rofi-wayland", "wofi", "tofi", "fuzzel", "bemenu"],
            status_bars: vec!["waybar", "yambar", "eww"],
            wallpaper_managers: vec!["swaybg", "hyprpaper", "wpaperd"],
            lock_screens: vec!["swaylock", "hyprlock"],
            notification_daemons: vec!["mako", "dunst", "swaync"],
        }
    }
}

/// Detect and analyze Hyprland configuration
pub fn analyze_hyprland() -> Option<HyprlandConfig> {
    info!("Analyzing Hyprland configuration");

    // Check if Hyprland is installed
    if !is_hyprland_installed() {
        debug!("Hyprland not installed");
        return None;
    }

    // Find hyprland.conf
    let config_path = find_hyprland_config()?;
    info!("Found Hyprland config at: {}", config_path.display());

    // Read config content
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            tracing::error!("Failed to read Hyprland config: {}", e);
            return None;
        }
    };

    let mut config = HyprlandConfig {
        config_path: config_path.clone(),
        config_content: config_content.clone(),
        ..Default::default()
    };

    // Analyze keybindings
    analyze_keybindings(&config_content, &mut config);

    // Detect installed components
    detect_hyprland_components(&mut config);

    Some(config)
}

/// Check if Hyprland is installed
fn is_hyprland_installed() -> bool {
    std::process::Command::new("which")
        .arg("Hyprland")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Find Hyprland config file
fn find_hyprland_config() -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME/hypr/hyprland.conf
    if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(xdg_config).join("hypr/hyprland.conf");
        if path.exists() {
            return Some(path);
        }
    }

    // Try ~/.config/hypr/hyprland.conf
    if let Ok(home) = std::env::var("HOME") {
        let path = PathBuf::from(home).join(".config/hypr/hyprland.conf");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// Analyze keybindings in the config
fn analyze_keybindings(content: &str, config: &mut HyprlandConfig) {
    let content_lower = content.to_lowercase();

    // Volume control (wpctl, pamixer, pactl, amixer)
    config.has_volume_control = content_lower.contains("wpctl")
        || content_lower.contains("pamixer")
        || content_lower.contains("pactl")
        || content_lower.contains("amixer")
        || content_lower.contains("volume");

    // Brightness control (brightnessctl, light)
    config.has_brightness_control = content_lower.contains("brightnessctl")
        || content_lower.contains("light ")
        || content_lower.contains("brightness");

    // Mute toggle
    config.has_mute_toggle = content_lower.contains("mute")
        || (content_lower.contains("wpctl") && content_lower.contains("toggle"))
        || (content_lower.contains("pamixer") && content_lower.contains("toggle"));

    // Screenshot (grim, grimblast, flameshot)
    config.has_screenshot_tool = content_lower.contains("grim")
        || content_lower.contains("slurp")
        || content_lower.contains("grimblast")
        || content_lower.contains("flameshot")
        || content_lower.contains("screenshot");

    // Media controls (playerctl)
    config.has_media_controls = content_lower.contains("playerctl")
        || content_lower.contains("media-play")
        || content_lower.contains("play-pause");

    // Application launcher
    config.has_app_launcher = content_lower.contains("rofi")
        || content_lower.contains("wofi")
        || content_lower.contains("tofi")
        || content_lower.contains("fuzzel")
        || content_lower.contains("bemenu");

    // Detect which launcher
    if content_lower.contains("rofi") {
        config.detected_launcher = Some("rofi-wayland".to_string());
    } else if content_lower.contains("wofi") {
        config.detected_launcher = Some("wofi".to_string());
    } else if content_lower.contains("tofi") {
        config.detected_launcher = Some("tofi".to_string());
    } else if content_lower.contains("fuzzel") {
        config.detected_launcher = Some("fuzzel".to_string());
    } else if content_lower.contains("bemenu") {
        config.detected_launcher = Some("bemenu".to_string());
    }
}

/// Detect installed Hyprland components
fn detect_hyprland_components(config: &mut HyprlandConfig) {
    // Check for waybar
    config.has_waybar = is_package_installed("waybar");

    // Check for wallpaper managers
    config.has_wallpaper_manager = is_package_installed("swaybg")
        || is_package_installed("hyprpaper")
        || is_package_installed("wpaperd");

    // Check for lock screens
    config.has_lock_screen = is_package_installed("swaylock")
        || is_package_installed("hyprlock");

    // Check for notification daemons
    config.has_notification_daemon = is_package_installed("mako")
        || is_package_installed("dunst")
        || is_package_installed("swaync");
}

/// Check if a package is installed
fn is_package_installed(package: &str) -> bool {
    std::process::Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Generate Hyprland improvement recommendations
pub fn generate_hyprland_recommendations(config: &HyprlandConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();
    let packages = HyprlandPackages::default();

    info!("Generating Hyprland improvement recommendations");

    // Volume control
    if !config.has_volume_control {
        let has_volume_tool = packages
            .volume_tools
            .iter()
            .any(|pkg| is_package_installed(pkg));

        if !has_volume_tool {
            recommendations.push(
                Advice::new(
                    "hyprland-volume-control".to_string(),
                    "Add volume control keybindings to Hyprland".to_string(),
                    "Your Hyprland config is missing volume control keybindings. This will install wireplumber (PipeWire volume control) and add SUPER+F11/F12 keybindings for volume down/up.".to_string(),
                    "Set up volume controls for Hyprland".to_string(),
                    Some(format!(
                        r#"sudo pacman -S --noconfirm wireplumber && \
echo '
# Volume controls
bind = SUPER, F11, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = SUPER, F12, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = SUPER, F10, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                )
                .with_satisfies(vec!["hyprland-mute-toggle".to_string()]), // This also adds mute
            );
        } else {
            // Package exists but not configured
            recommendations.push(
                Advice::new(
                    "hyprland-volume-keybindings".to_string(),
                    "Add volume control keybindings to Hyprland".to_string(),
                    "You have volume control tools installed but no keybindings in your Hyprland config. This will add SUPER+F11/F12 for volume control.".to_string(),
                    "Add volume keybindings to hyprland.conf".to_string(),
                    Some(format!(
                        r#"echo '
# Volume controls
bind = SUPER, F11, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bind = SUPER, F12, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bind = SUPER, F10, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                )
                .with_satisfies(vec!["hyprland-mute-toggle".to_string()]),
            );
        }
    }

    // Brightness control (laptop-specific)
    if !config.has_brightness_control {
        let has_brightness_tool = packages
            .brightness_tools
            .iter()
            .any(|pkg| is_package_installed(pkg));

        if !has_brightness_tool {
            recommendations.push(
                Advice::new(
                    "hyprland-brightness-control".to_string(),
                    "Add brightness control for Hyprland".to_string(),
                    "Your Hyprland config is missing brightness controls. This will install brightnessctl and add SUPER+F5/F6 keybindings for brightness down/up.".to_string(),
                    "Set up brightness controls for Hyprland".to_string(),
                    Some(format!(
                        r#"sudo pacman -S --noconfirm brightnessctl && \
echo '
# Brightness controls
bind = SUPER, F5, exec, brightnessctl set 5%-
bind = SUPER, F6, exec, brightnessctl set 5%+
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        } else {
            recommendations.push(
                Advice::new(
                    "hyprland-brightness-keybindings".to_string(),
                    "Add brightness keybindings to Hyprland".to_string(),
                    "You have brightnessctl installed but no keybindings. This will add SUPER+F5/F6 for brightness control.".to_string(),
                    "Add brightness keybindings to hyprland.conf".to_string(),
                    Some(format!(
                        r#"echo '
# Brightness controls
bind = SUPER, F5, exec, brightnessctl set 5%-
bind = SUPER, F6, exec, brightnessctl set 5%+
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        }
    }

    // Screenshot tool
    if !config.has_screenshot_tool {
        let has_screenshot = is_package_installed("grim") && is_package_installed("slurp");

        if !has_screenshot {
            recommendations.push(
                Advice::new(
                    "hyprland-screenshot".to_string(),
                    "Add screenshot functionality to Hyprland".to_string(),
                    "Your Hyprland config is missing screenshot capabilities. This will install grim (screenshot tool) and slurp (region selector) and add SUPER+SHIFT+S keybinding for screenshots.".to_string(),
                    "Set up screenshots for Hyprland".to_string(),
                    Some(format!(
                        r#"sudo pacman -S --noconfirm grim slurp && \
echo '
# Screenshot controls
bind = SUPER_SHIFT, S, exec, grim -g \"$(slurp)\" - | wl-copy && notify-send \"Screenshot copied to clipboard\"
bind = SUPER_SHIFT, P, exec, grim - | wl-copy && notify-send \"Full screenshot copied to clipboard\"
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        } else {
            recommendations.push(
                Advice::new(
                    "hyprland-screenshot-keybindings".to_string(),
                    "Add screenshot keybindings to Hyprland".to_string(),
                    "You have grim and slurp installed but no screenshot keybindings. This will add SUPER+SHIFT+S for region screenshots.".to_string(),
                    "Add screenshot keybindings to hyprland.conf".to_string(),
                    Some(format!(
                        r#"echo '
# Screenshot controls
bind = SUPER_SHIFT, S, exec, grim -g \"$(slurp)\" - | wl-copy && notify-send \"Screenshot copied to clipboard\"
bind = SUPER_SHIFT, P, exec, grim - | wl-copy && notify-send \"Full screenshot copied to clipboard\"
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        }
    }

    // Media controls
    if !config.has_media_controls {
        if !is_package_installed("playerctl") {
            recommendations.push(
                Advice::new(
                    "hyprland-media-controls".to_string(),
                    "Add media playback controls to Hyprland".to_string(),
                    "Your Hyprland config is missing media controls. This will install playerctl and add keybindings for play/pause, next, and previous track.".to_string(),
                    "Set up media controls for Hyprland".to_string(),
                    Some(format!(
                        r#"sudo pacman -S --noconfirm playerctl && \
echo '
# Media controls
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        } else {
            recommendations.push(
                Advice::new(
                    "hyprland-media-keybindings".to_string(),
                    "Add media control keybindings to Hyprland".to_string(),
                    "You have playerctl installed but no media control keybindings. This will add bindings for media keys.".to_string(),
                    "Add media keybindings to hyprland.conf".to_string(),
                    Some(format!(
                        r#"echo '
# Media controls
bind = , XF86AudioPlay, exec, playerctl play-pause
bind = , XF86AudioNext, exec, playerctl next
bind = , XF86AudioPrev, exec, playerctl previous
' >> {}"#,
                        config.config_path.display()
                    )),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec!["https://wiki.archlinux.org/title/Hyprland".to_string()],
                    "Desktop Customization".to_string(),
                ),
            );
        }
    }

    // Application launcher
    if !config.has_app_launcher {
        recommendations.push(
            Advice::new(
                "hyprland-app-launcher".to_string(),
                "Install application launcher for Hyprland".to_string(),
                "Your Hyprland setup is missing an application launcher. Rofi is a popular choice for Wayland. This will install rofi-wayland and add SUPER+D keybinding to launch it.".to_string(),
                "Install and configure rofi for Hyprland".to_string(),
                Some(format!(
                    r#"sudo pacman -S --noconfirm rofi-wayland && \
echo '
# Application launcher
bind = SUPER, D, exec, rofi -show drun
bind = SUPER, R, exec, rofi -show run
' >> {}"#,
                    config.config_path.display()
                )),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Rofi".to_string()],
                "Desktop Customization".to_string(),
            ),
        );
    }

    // Waybar status bar
    if !config.has_waybar {
        recommendations.push(
            Advice::new(
                "hyprland-waybar".to_string(),
                "Install Waybar status bar for Hyprland".to_string(),
                "Waybar is a highly customizable Wayland bar that shows system information, workspaces, and tray icons. Perfect companion for Hyprland.".to_string(),
                "Install Waybar for Hyprland".to_string(),
                Some("sudo pacman -S --noconfirm waybar".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
                "Desktop Customization".to_string(),
            ),
        );
    }

    // Wallpaper manager
    if !config.has_wallpaper_manager {
        recommendations.push(
            Advice::new(
                "hyprland-wallpaper".to_string(),
                "Install wallpaper manager for Hyprland".to_string(),
                "swaybg is a simple wallpaper manager for Wayland that works great with Hyprland.".to_string(),
                "Install swaybg for wallpapers".to_string(),
                Some("sudo pacman -S --noconfirm swaybg".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec!["https://wiki.archlinux.org/title/Sway#Wallpaper".to_string()],
                "Desktop Customization".to_string(),
            ),
        );
    }

    // Lock screen
    if !config.has_lock_screen {
        recommendations.push(
            Advice::new(
                "hyprland-lock-screen".to_string(),
                "Install screen locker for Hyprland".to_string(),
                "swaylock is a simple screen locker for Wayland. This will also add SUPER+L keybinding to lock your screen.".to_string(),
                "Install swaylock and add keybinding".to_string(),
                Some(format!(
                    r#"sudo pacman -S --noconfirm swaylock && \
echo '
# Lock screen
bind = SUPER, L, exec, swaylock -f
' >> {}"#,
                    config.config_path.display()
                )),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Sway#Screen_locking".to_string()],
                "Security & Privacy".to_string(),
            ),
        );
    }

    // Notification daemon
    if !config.has_notification_daemon {
        recommendations.push(
            Advice::new(
                "hyprland-notifications".to_string(),
                "Install notification daemon for Hyprland".to_string(),
                "mako is a lightweight notification daemon for Wayland that integrates beautifully with Hyprland.".to_string(),
                "Install mako for notifications".to_string(),
                Some("sudo pacman -S --noconfirm mako".to_string()),
                RiskLevel::Low,
                Priority::Recommended,
                vec!["https://wiki.archlinux.org/title/Desktop_notifications".to_string()],
                "Desktop Customization".to_string(),
            ),
        );
    }

    recommendations
}
