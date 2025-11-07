//! i3 Window Manager configuration intelligence
//!
//! Analyzes i3 installation and configuration, suggests improvements

use anna_common::{Advice, Priority, RiskLevel};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

#[derive(Debug, Clone)]
pub struct I3Config {
    #[allow(dead_code)]
    pub i3_installed: bool,
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
    pub has_compositor: bool,
}

impl Default for I3Config {
    fn default() -> Self {
        Self {
            i3_installed: false,
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
            has_compositor: false,
        }
    }
}

/// Detect and analyze i3 configuration
pub fn analyze_i3() -> Option<I3Config> {
    // Check if i3 is installed
    let i3_installed = Command::new("which")
        .arg("i3")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !i3_installed {
        info!("i3 not installed, skipping i3 config analysis");
        return None;
    }

    info!("i3 detected, analyzing configuration");

    // Find i3 config
    let home = std::env::var("HOME").ok()?;
    let config_candidates = vec![
        PathBuf::from(format!("{}/.config/i3/config", home)),
        PathBuf::from(format!("{}/.i3/config", home)),
    ];

    let config_path = config_candidates
        .iter()
        .find(|p| p.exists())
        .cloned();

    if config_path.is_none() {
        info!("i3 config not found");
    }

    let mut config = I3Config {
        i3_installed: true,
        config_path: config_path.clone(),
        ..Default::default()
    };

    // Analyze config if found
    if let Some(ref path) = config_path {
        if let Ok(content) = fs::read_to_string(path) {
            analyze_i3_config_content(&content, &mut config);
        }
    }

    // Check for installed packages
    detect_i3_packages(&mut config);

    Some(config)
}

/// Analyze i3 config content
fn analyze_i3_config_content(content: &str, config: &mut I3Config) {
    info!("Analyzing i3 config content");

    // Volume control detection
    config.has_volume_keys = content.contains("pactl")
        || content.contains("pamixer")
        || content.contains("amixer")
        || content.contains("wpctl");

    // Brightness control detection
    config.has_brightness_keys = content.contains("brightnessctl")
        || content.contains("light")
        || content.contains("xbacklight");

    // Screenshot detection
    config.has_screenshot_keys = content.contains("maim")
        || content.contains("scrot")
        || content.contains("flameshot")
        || content.contains("spectacle");

    // Media control detection
    config.has_media_keys = content.contains("playerctl")
        || content.contains("mpc");

    // Launcher detection
    config.has_launcher = content.contains("rofi")
        || content.contains("dmenu")
        || content.contains("j4-dmenu");

    // Status bar detection
    config.has_status_bar = content.contains("i3status")
        || content.contains("i3blocks")
        || content.contains("polybar")
        || content.contains("yambar");

    // Wallpaper detection
    config.has_wallpaper = content.contains("feh")
        || content.contains("nitrogen")
        || content.contains("xwallpaper");

    // Lock screen detection
    config.has_lock_screen = content.contains("i3lock")
        || content.contains("betterlockscreen")
        || content.contains("xsecurelock");

    // Compositor detection
    config.has_compositor = content.contains("picom")
        || content.contains("compton");
}

/// Detect installed i3-related packages
fn detect_i3_packages(config: &mut I3Config) {
    // Check notification daemon
    config.has_notification_daemon = check_package_installed("dunst")
        || check_package_installed("mako");
}

/// Check if a package is installed
fn check_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate i3 configuration recommendations
pub fn generate_i3_recommendations(config: &I3Config) -> Vec<Advice> {
    let mut recommendations = Vec::new();

    // Volume controls
    if !config.has_volume_keys {
        recommendations.push(Advice::new(
            "i3-volume-control".to_string(),
            "Add volume control keybindings to i3".to_string(),
            "Your i3 config doesn't have volume control keybindings. This adds F11/F12 keys for volume control using pactl.".to_string(),
            "Install wireplumber and add volume keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm wireplumber && cat >> {} <<'EOF'

# Volume controls (pactl)
bindsym XF86AudioRaiseVolume exec --no-startup-id pactl set-sink-volume @DEFAULT_SINK@ +5%
bindsym XF86AudioLowerVolume exec --no-startup-id pactl set-sink-volume @DEFAULT_SINK@ -5%
bindsym XF86AudioMute exec --no-startup-id pactl set-sink-mute @DEFAULT_SINK@ toggle
bindsym XF86AudioMicMute exec --no-startup-id pactl set-source-mute @DEFAULT_SOURCE@ toggle

# Alternative: F11/F12 for laptops without media keys
bindsym $mod+F12 exec --no-startup-id pactl set-sink-volume @DEFAULT_SINK@ +5%
bindsym $mod+F11 exec --no-startup-id pactl set-sink-volume @DEFAULT_SINK@ -5%
bindsym $mod+F10 exec --no-startup-id pactl set-sink-mute @DEFAULT_SINK@ toggle
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/I3#Volume_control".to_string()],
            "desktop".to_string(),
        ));
    }

    // Brightness controls
    if !config.has_brightness_keys {
        recommendations.push(Advice::new(
            "i3-brightness-control".to_string(),
            "Add brightness control keybindings to i3".to_string(),
            "Your i3 config doesn't have brightness control keybindings. This adds F5/F6 keys for screen brightness using brightnessctl.".to_string(),
            "Install brightnessctl and add brightness keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm brightnessctl && cat >> {} <<'EOF'

# Brightness controls (brightnessctl)
bindsym XF86MonBrightnessUp exec --no-startup-id brightnessctl set +5%
bindsym XF86MonBrightnessDown exec --no-startup-id brightnessctl set 5%-

# Alternative: F5/F6 for laptops without media keys
bindsym $mod+F6 exec --no-startup-id brightnessctl set +5%
bindsym $mod+F5 exec --no-startup-id brightnessctl set 5%-
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Backlight".to_string()],
            "Power Management".to_string(),
        ));
    }

    // Screenshot tools
    if !config.has_screenshot_keys {
        recommendations.push(Advice::new(
            "i3-screenshot-tool".to_string(),
            "Add screenshot functionality to i3".to_string(),
            "Your i3 config doesn't have screenshot keybindings. This installs maim+xclip for screenshots and adds keybindings.".to_string(),
            "Install maim and add screenshot keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm maim xclip && cat >> {} <<'EOF'

# Screenshot keybindings (maim)
# Full screen screenshot
bindsym Print exec --no-startup-id maim ~/Pictures/screenshot-$(date +%Y%m%d-%H%M%S).png

# Select area screenshot
bindsym $mod+Print exec --no-startup-id maim -s ~/Pictures/screenshot-$(date +%Y%m%d-%H%M%S).png

# Screenshot to clipboard
bindsym Shift+Print exec --no-startup-id maim | xclip -selection clipboard -t image/png

# Select area to clipboard
bindsym $mod+Shift+Print exec --no-startup-id maim -s | xclip -selection clipboard -t image/png
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Screen_capture".to_string()],
            "desktop".to_string(),
        ));
    }

    // Media control
    if !config.has_media_keys {
        recommendations.push(Advice::new(
            "i3-media-control".to_string(),
            "Add media control keybindings to i3".to_string(),
            "Your i3 config doesn't have media control keybindings. This installs playerctl for play/pause/next/previous controls.".to_string(),
            "Install playerctl and add media keybindings".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm playerctl && cat >> {} <<'EOF'

# Media player controls (playerctl)
bindsym XF86AudioPlay exec --no-startup-id playerctl play-pause
bindsym XF86AudioPause exec --no-startup-id playerctl play-pause
bindsym XF86AudioNext exec --no-startup-id playerctl next
bindsym XF86AudioPrev exec --no-startup-id playerctl previous

# Alternative: Mod+arrow keys
bindsym $mod+p exec --no-startup-id playerctl play-pause
bindsym $mod+bracketright exec --no-startup-id playerctl next
bindsym $mod+bracketleft exec --no-startup-id playerctl previous
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/I3#Media_control".to_string()],
            "desktop".to_string(),
        ));
    }

    // Application launcher
    if !config.has_launcher {
        recommendations.push(Advice::new(
            "i3-app-launcher".to_string(),
            "Install application launcher for i3".to_string(),
            "You don't have an application launcher configured. Rofi is a modern, feature-rich launcher that replaces dmenu with better UI and fuzzy search.".to_string(),
            "Install rofi and add launcher keybinding".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm rofi && cat >> {} <<'EOF'

# Application launcher (rofi)
bindsym $mod+d exec --no-startup-id rofi -show drun -show-icons
bindsym $mod+Tab exec --no-startup-id rofi -show window
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Rofi".to_string()],
            "desktop".to_string(),
        ));
    }

    // Status bar
    if !config.has_status_bar {
        recommendations.push(Advice::new(
            "i3-status-bar".to_string(),
            "Install a status bar for i3".to_string(),
            "You don't have a status bar configured. i3status-rust is a modern, feature-rich status bar written in Rust with great defaults.".to_string(),
            "Install i3status-rust and configure it".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm i3status-rust && cat >> {} <<'EOF'

# Status bar (i3status-rust)
bar {{
    position top
    status_command i3status-rs
    colors {{
        background #1e1e2e
        statusline #cdd6f4
        separator #45475a

        focused_workspace  #89b4fa #89b4fa #1e1e2e
        active_workspace   #45475a #45475a #cdd6f4
        inactive_workspace #1e1e2e #1e1e2e #cdd6f4
        urgent_workspace   #f38ba8 #f38ba8 #1e1e2e
    }}
}}
EOF

mkdir -p ~/.config/i3status-rust
cat > ~/.config/i3status-rust/config.toml <<'TOML'
[theme]
theme = "nord-dark"

[icons]
icons = "awesome5"

[[block]]
block = "cpu"

[[block]]
block = "memory"
format = " $icon $mem_used_percents "

[[block]]
block = "disk_space"
path = "/"
info_type = "available"
alert = 10.0
format = " $icon $available "

[[block]]
block = "net"
format = " $icon $ip "

[[block]]
block = "sound"

[[block]]
block = "battery"
format = " $icon $percentage "

[[block]]
block = "time"
format = " $icon $timestamp.datetime(f:'%a %m/%d %I:%M %p') "
interval = 60
TOML
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/I3#Status_bar".to_string()],
            "desktop".to_string(),
        ));
    }

    // Wallpaper manager
    if !config.has_wallpaper {
        recommendations.push(Advice::new(
            "i3-wallpaper".to_string(),
            "Set up wallpaper management for i3".to_string(),
            "You don't have wallpaper management configured. feh is the standard tool for setting wallpapers on X11/i3.".to_string(),
            "Install feh and set up wallpaper".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm feh && cat >> {} <<'EOF'

# Wallpaper (feh)
exec --no-startup-id feh --bg-scale ~/.config/wallpaper.jpg
EOF

# Download a nice wallpaper
mkdir -p ~/.config
curl -L https://images.unsplash.com/photo-1557683316-973673baf926 -o ~/.config/wallpaper.jpg || echo "Download wallpaper manually to ~/.config/wallpaper.jpg"
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Feh".to_string()],
            "desktop".to_string(),
        ));
    }

    // Lock screen
    if !config.has_lock_screen {
        recommendations.push(Advice::new(
            "i3-lock-screen".to_string(),
            "Install screen lock for i3".to_string(),
            "You don't have a screen lock configured. i3lock-color is an improved version of i3lock with customizable colors and effects.".to_string(),
            "Install i3lock-color and add lock keybinding".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm i3lock && cat >> {} <<'EOF'

# Lock screen (i3lock)
bindsym $mod+Escape exec --no-startup-id i3lock -c 000000
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/I3#Shutdown.2C_reboot.2C_lock_screen".to_string()],
            "desktop".to_string(),
        ));
    }

    // Notification daemon
    if !config.has_notification_daemon {
        recommendations.push(Advice::new(
            "i3-notification-daemon".to_string(),
            "Install notification daemon for i3".to_string(),
            "You don't have a notification daemon installed. Dunst is a lightweight, customizable notification daemon that works great with i3.".to_string(),
            "Install and enable dunst".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm dunst && cat >> {} <<'EOF'

# Notification daemon (dunst)
exec --no-startup-id dunst
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Recommended,
            vec!["https://wiki.archlinux.org/title/Dunst".to_string()],
            "desktop".to_string(),
        ));
    }

    // Compositor for transparency and effects
    if !config.has_compositor {
        recommendations.push(Advice::new(
            "i3-compositor".to_string(),
            "Install compositor for i3 (transparency & effects)".to_string(),
            "You don't have a compositor configured. Picom provides transparency, shadows, and fade effects for i3, making it look more polished.".to_string(),
            "Install picom and enable it".to_string(),
            Some(format!(
                r#"sudo pacman -S --noconfirm picom && cat >> {} <<'EOF'

# Compositor (picom) - transparency, shadows, fade effects
exec --no-startup-id picom -b
EOF
"#,
                config.config_path.as_ref().unwrap().display()
            )),
            RiskLevel::Low,
            Priority::Optional,
            vec!["https://wiki.archlinux.org/title/Picom".to_string()],
            "desktop".to_string(),
        ));
    }

    recommendations
}
