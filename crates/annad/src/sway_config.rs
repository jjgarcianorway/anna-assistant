//! Sway configuration analysis and recommendations
//!
//! Sway is a tiling Wayland compositor that's compatible with i3 configuration.
//! This module analyzes Sway configurations and recommends missing components.

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct SwayConfig {
    #[allow(dead_code)]
    pub sway_installed: bool,
    pub config_path: Option<PathBuf>,
    pub has_volume_keys: bool,
    pub has_brightness_keys: bool,
    pub has_screenshot_keys: bool,
    pub has_media_keys: bool,
    pub has_launcher: bool,
    pub has_status_bar: bool,
    pub has_wallpaper: bool,
    pub has_lock_screen: bool,
    pub has_notification_daemon: bool,
    pub has_idle_management: bool,
}

impl Default for SwayConfig {
    fn default() -> Self {
        Self {
            sway_installed: false,
            config_path: None,
            has_volume_keys: false,
            has_brightness_keys: false,
            has_screenshot_keys: false,
            has_media_keys: false,
            has_launcher: false,
            has_status_bar: false,
            has_wallpaper: false,
            has_lock_screen: false,
            has_notification_daemon: false,
            has_idle_management: false,
        }
    }
}

/// Analyze Sway installation and configuration
pub fn analyze_sway() -> Option<SwayConfig> {
    info!("Analyzing Sway configuration");

    // Check if Sway is installed
    let sway_installed = Command::new("which")
        .arg("sway")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !sway_installed {
        info!("Sway not installed, skipping analysis");
        return None;
    }

    let mut config = SwayConfig {
        sway_installed: true,
        ..Default::default()
    };

    // Find config file
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let config_candidates = vec![
        PathBuf::from(format!("{}/.config/sway/config", home)),
        PathBuf::from(format!("{}/sway/config", home)),
    ];

    for candidate in config_candidates {
        if candidate.exists() {
            config.config_path = Some(candidate.clone());
            info!("Found Sway config at: {:?}", candidate);
            break;
        }
    }

    // Analyze config file if found
    if let Some(ref config_path) = config.config_path {
        if let Ok(content) = fs::read_to_string(config_path) {
            analyze_sway_config_content(&content, &mut config);
        }
    }

    // Detect installed packages
    detect_sway_packages(&mut config);

    Some(config)
}

/// Analyze Sway config file content
fn analyze_sway_config_content(content: &str, config: &mut SwayConfig) {
    let content_lower = content.to_lowercase();

    // Check for volume controls (wpctl, pactl, pamixer)
    if content_lower.contains("wpctl")
        || content_lower.contains("pactl")
        || content_lower.contains("pamixer")
        || content_lower.contains("amixer")
    {
        if content_lower.contains("volume") || content_lower.contains("audioraisevolume") {
            config.has_volume_keys = true;
        }
    }

    // Check for brightness controls
    if content_lower.contains("brightnessctl") || content_lower.contains("light") {
        config.has_brightness_keys = true;
    }

    // Check for screenshot tools (Wayland: grim/slurp/grimblast)
    if content_lower.contains("grim")
        || content_lower.contains("slurp")
        || content_lower.contains("grimblast")
        || content_lower.contains("wayshot")
    {
        config.has_screenshot_keys = true;
    }

    // Check for media controls
    if content_lower.contains("playerctl") || content_lower.contains("mpc") {
        config.has_media_keys = true;
    }

    // Check for launchers (Wayland-compatible: wofi, rofi, tofi, fuzzel)
    if content_lower.contains("wofi")
        || content_lower.contains("rofi")
        || content_lower.contains("tofi")
        || content_lower.contains("fuzzel")
        || content_lower.contains("bemenu")
    {
        config.has_launcher = true;
    }

    // Check for status bars (waybar, i3status, yambar, swaybar)
    if content_lower.contains("waybar")
        || content_lower.contains("i3status")
        || content_lower.contains("yambar")
        || content_lower.contains("swaybar")
    {
        config.has_status_bar = true;
    }

    // Check for wallpaper (Wayland: swaybg, swww, mpvpaper)
    if content_lower.contains("swaybg")
        || content_lower.contains("swww")
        || content_lower.contains("mpvpaper")
        || content_lower.contains("wpaperd")
    {
        config.has_wallpaper = true;
    }

    // Check for lock screen (swaylock)
    if content_lower.contains("swaylock") {
        config.has_lock_screen = true;
    }

    // Check for notification daemon (mako, dunst)
    if content_lower.contains("mako") || content_lower.contains("dunst") {
        config.has_notification_daemon = true;
    }

    // Check for idle management (swayidle)
    if content_lower.contains("swayidle") {
        config.has_idle_management = true;
    }
}

/// Detect installed packages related to Sway
fn detect_sway_packages(_config: &mut SwayConfig) {
    // Package detection is not needed for Sway since we detect everything
    // from the config file itself. This function is kept for potential
    // future enhancements where we might want to detect installed packages
    // even if they're not referenced in the config.
}

/// Generate Sway-specific recommendations
pub fn generate_sway_recommendations(config: &SwayConfig) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    // Volume controls
    if !config.has_volume_keys {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-volume-control".to_string(),
            "Add volume control keybindings to Sway".to_string(),
            "Your Sway config doesn't have volume control keybindings. Most keyboards have dedicated media keys (XF86AudioRaiseVolume, etc.). This adds wpctl-based volume controls.".to_string(),
            "Install wireplumber and add volume keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm wireplumber && cat >> {} <<'EOF'

# Volume controls (wpctl)
bindsym XF86AudioRaiseVolume exec wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%+
bindsym XF86AudioLowerVolume exec wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindsym XF86AudioMute exec wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
bindsym XF86AudioMicMute exec wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Volume_control".to_string()],
            "desktop".to_string(),
        ));
    }

    // Brightness controls
    if !config.has_brightness_keys {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-brightness-control".to_string(),
            "Add brightness control keybindings to Sway".to_string(),
            "Your Sway config doesn't have brightness controls. Essential for laptops with brightness keys.".to_string(),
            "Install brightnessctl and add brightness keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm brightnessctl && cat >> {} <<'EOF'

# Brightness controls
bindsym XF86MonBrightnessUp exec brightnessctl set 5%+
bindsym XF86MonBrightnessDown exec brightnessctl set 5%-
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Backlight".to_string()],
            "desktop".to_string(),
        ));
    }

    // Screenshot functionality
    if !config.has_screenshot_keys {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-screenshot-tools".to_string(),
            "Add screenshot functionality to Sway".to_string(),
            "Your Sway config doesn't have screenshot keybindings. grim and slurp provide Wayland-native screenshot capabilities.".to_string(),
            "Install grim/slurp and add screenshot keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm grim slurp && cat >> {} <<'EOF'

# Screenshots (grim + slurp)
bindsym Print exec grim ~/Pictures/screenshot-$(date +%Y%m%d-%H%M%S).png
bindsym Shift+Print exec grim -g "$(slurp)" ~/Pictures/screenshot-$(date +%Y%m%d-%H%M%S).png
bindsym Control+Print exec grim -g "$(slurp)" - | wl-copy
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Screen_capture".to_string()],
            "desktop".to_string(),
        ));
    }

    // Media controls
    if !config.has_media_keys {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-media-controls".to_string(),
            "Add media playback controls to Sway".to_string(),
            "Your Sway config doesn't have media control keybindings. playerctl provides universal media controls for music/video players.".to_string(),
            "Install playerctl and add media keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm playerctl && cat >> {} <<'EOF'

# Media controls
bindsym XF86AudioPlay exec playerctl play-pause
bindsym XF86AudioNext exec playerctl next
bindsym XF86AudioPrev exec playerctl previous
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Sway#Media_controls".to_string()],
            "desktop".to_string(),
        ));
    }

    // Application launcher
    if !config.has_launcher {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-launcher".to_string(),
            "Add application launcher to Sway".to_string(),
            "Your Sway config doesn't have an application launcher configured. wofi is a native Wayland launcher designed for wlroots compositors like Sway.".to_string(),
            "Install wofi and add launcher keybinding".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm wofi && cat >> {} <<'EOF'

# Application launcher
bindsym $mod+d exec wofi --show drun
bindsym $mod+Shift+d exec wofi --show run
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Application_launchers".to_string()],
            "desktop".to_string(),
        ));
    }

    // Status bar
    if !config.has_status_bar {
        recommendations.push(Advice::new(
            "sway-status-bar".to_string(),
            "Install status bar for Sway".to_string(),
            "Your Sway config doesn't have a status bar configured. Waybar is a highly customizable status bar for Wayland compositors with great Sway integration.".to_string(),
            "Install Waybar and enable it in Sway".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm waybar && cat >> {} <<'EOF'

# Status bar
bar {{
    swaybar_command waybar
}}
EOF

# Create basic Waybar config
mkdir -p ~/.config/waybar
cat > ~/.config/waybar/config <<'WAYBAR_EOF'
{{
    "layer": "top",
    "modules-left": ["sway/workspaces", "sway/mode"],
    "modules-center": ["sway/window"],
    "modules-right": ["cpu", "memory", "disk", "pulseaudio", "network", "battery", "clock"],

    "cpu": {{
        "format": " {{usage}}%"
    }},
    "memory": {{
        "format": " {{}}%"
    }},
    "disk": {{
        "format": " {{percentage_used}}%",
        "path": "/"
    }},
    "pulseaudio": {{
        "format": "{{icon}} {{volume}}%",
        "format-icons": ["", "", ""]
    }},
    "network": {{
        "format-wifi": " {{essid}}",
        "format-ethernet": " {{ifname}}",
        "format-disconnected": "⚠ Disconnected"
    }},
    "battery": {{
        "format": "{{icon}} {{capacity}}%",
        "format-icons": ["", "", "", "", ""]
    }},
    "clock": {{
        "format": " {{:%H:%M   %Y-%m-%d}}"
    }}
}}
WAYBAR_EOF
"#,
                config
                    .config_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "~/.config/sway/config".to_string())
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
            "desktop".to_string(),
        ));
    }

    // Wallpaper
    if !config.has_wallpaper {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-wallpaper".to_string(),
            "Configure wallpaper for Sway".to_string(),
            "Your Sway config doesn't set a wallpaper. swaybg is the native wallpaper tool for Sway.".to_string(),
            "Install swaybg and configure wallpaper".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm swaybg && cat >> {} <<'EOF'

# Wallpaper (replace with your image path)
output * bg ~/Pictures/wallpaper.jpg fill
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Cosmetic,
            vec!["https://wiki.archlinux.org/title/Sway#Wallpaper".to_string()],
            "desktop".to_string(),
        ));
    }

    // Lock screen
    if !config.has_lock_screen {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-lock-screen".to_string(),
            "Add screen lock to Sway".to_string(),
            "Your Sway config doesn't have a screen lock configured. swaylock provides a simple, secure lock screen for Sway.".to_string(),
            "Install swaylock and add lock keybinding".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm swaylock && cat >> {} <<'EOF'

# Lock screen
bindsym $mod+l exec swaylock -f -c 000000
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Screen_locking".to_string()],
            "desktop".to_string(),
        ));
    }

    // Notification daemon
    if !config.has_notification_daemon {
        recommendations.push(Advice::new(
            "sway-notifications".to_string(),
            "Install notification daemon for Sway".to_string(),
            "Your Sway setup doesn't have a notification daemon. mako is a lightweight notification daemon designed for Wayland.".to_string(),
            "Install mako notification daemon".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm mako && cat >> {} <<'EOF'

# Notification daemon
exec mako
EOF
"#,
                config
                    .config_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "~/.config/sway/config".to_string())
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Sway#Notifications".to_string()],
            "desktop".to_string(),
        ));
    }

    // Idle management
    if !config.has_idle_management {
        let config_path = config
            .config_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "~/.config/sway/config".to_string());

        recommendations.push(Advice::new(
            "sway-idle-management".to_string(),
            "Add idle management to Sway".to_string(),
            "Your Sway config doesn't have idle management. swayidle can automatically lock your screen, turn off displays, and suspend the system when idle.".to_string(),
            "Install swayidle and configure idle actions".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm swayidle && cat >> {} <<'EOF'

# Idle management
exec swayidle -w \
    timeout 300 'swaylock -f -c 000000' \
    timeout 600 'swaymsg "output * dpms off"' \
    resume 'swaymsg "output * dpms on"' \
    before-sleep 'swaylock -f -c 000000'
EOF
"#,
                config_path
            )),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Sway#Idle".to_string()],
            "desktop".to_string(),
        ));
    }

    recommendations
}
