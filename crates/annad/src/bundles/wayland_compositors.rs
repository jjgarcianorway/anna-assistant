//! Wayland Compositor Bundles
//!
//! Complete desktop setups for Wayland compositors:
//! - Hyprland - Dynamic tiling Wayland compositor
//! - Sway - i3-compatible Wayland compositor
//! - Wayfire - 3D Wayland compositor
//! - River - Dynamic tiling Wayland compositor

use super::{DisplayServer, WMBundleBuilder};
use anna_common::types::{Advice, SystemFacts};

/// Generate all Wayland compositor bundles
pub fn generate_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    // Only generate bundles if Wayland is available/relevant
    if is_wayland_system(facts) {
        advice.extend(hyprland_bundle(facts));
        // Only Hyprland supported for v1.0
        // Other compositors (sway, wayfire, river) may return in v2.0
    }

    advice
}

/// Check if this is a Wayland system or could use Wayland
fn is_wayland_system(_facts: &SystemFacts) -> bool {
    // For now, always return true - users can choose
    // Later: detect if GPU supports Wayland, etc.
    true
}

/// Hyprland bundle - Dynamic tiling Wayland compositor
/// THE PERFECT HYPRLAND SETUP! (Beta.114 + RC.9 Telemetry Intelligence)
fn hyprland_bundle(facts: &SystemFacts) -> Vec<Advice> {
    let mut builder = WMBundleBuilder::new("hyprland")
        .display_server(DisplayServer::Wayland)
        .wm_package("hyprland")
        // UI Components
        .launcher("rofi-wayland")
        .status_bar("waybar")
        .terminal("kitty")
        .file_manager("nautilus", "ranger")
        .notification_daemon("mako")
        .wallpaper_manager("hyprpaper")
        .lock_screen("swaylock")
        // System Tools
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Screen Sharing (Beta.114) - Teams/Zoom support!
        .audio_server("pipewire") // Modern audio/video routing
        .screen_sharing("xdg-desktop-portal-hyprland") // Screen sharing for video calls
        // Multimedia Tools (Beta.112) - Make multimedia keys work!
        .audio_control("pamixer")
        .brightness_control("brightnessctl"); // Only installed on laptops

    // RC.9: SMART APPLICATION SELECTION - Issue #0 Fix
    // Only recommend apps if user doesn't already have and use them!
    // Fixes: "what is the point of installing nano when I have vim?"

    // Text editor - only if user doesn't already have one
    if let Some(editor) = super::smart_select_text_editor(facts) {
        builder = builder.text_editor(&editor);
    }

    // Media player - only if user has videos and no player
    if let Some(player) = super::smart_select_media_player(facts) {
        builder = builder.media_player(&player);
    }

    // Image viewer - only if user has images and no viewer
    if let Some(viewer) = super::smart_select_image_viewer(facts) {
        builder = builder.image_viewer(&viewer);
    }

    // PDF viewer - only if user doesn't have one
    if let Some(pdf_viewer) = super::smart_select_pdf_viewer(facts) {
        builder = builder.pdf_viewer(&pdf_viewer);
    }

    // Continue with rest of configuration
    builder
        // BEAUTIFICATION! (Beta.113) - Make it look AMAZING!
        .color_scheme_generator("python-pywal") // Auto colors from wallpaper!
        .gtk_theme("arc-gtk-theme") // Modern GTK theme
        .icon_theme("papirus-icon-theme") // Beautiful icons
        .cursor_theme("bibata-cursor-theme") // Modern cursor
        // Configuration Files (Beta.111)
        .config("hyprland", ".config/hypr")
        .config("waybar", ".config/waybar")
        .config("kitty", ".config/kitty")
        .config("rofi", ".config/rofi")
        .config("mako", ".config/mako")
        // Window Management
        .keybind("SUPER+Q", "Close window")
        .keybind("SUPER+M", "Exit Hyprland")
        .keybind("SUPER+V", "Toggle floating")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+P", "Toggle pseudo-tiling")
        .keybind("SUPER+J", "Toggle split direction")
        // Workspaces
        .keybind("SUPER+1-9", "Switch to workspace 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to workspace 1-9")
        .keybind("SUPER+Left", "Focus left window")
        .keybind("SUPER+Right", "Focus right window")
        .keybind("SUPER+Up", "Focus up window")
        .keybind("SUPER+Down", "Focus down window")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        .keybind("SUPER+E", "Launch file manager")
        // Media & System
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .keybind("XF86MonBrightnessUp", "Increase brightness")
        .keybind("XF86MonBrightnessDown", "Decrease brightness")
        .keybind("Print", "Screenshot (full screen)")
        .keybind("SUPER+Print", "Screenshot (area select)")
        .build(facts)
}

/// Sway bundle - i3-compatible Wayland compositor
fn sway_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("sway")
        .display_server(DisplayServer::Wayland)
        .wm_package("sway")
        .launcher("wofi")
        .status_bar("waybar")
        .terminal("foot")
        .file_manager("thunar", "ranger")
        .notification_daemon("mako")
        .wallpaper_manager("swaybg")
        .lock_screen("swaylock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+Q", "Close window")
        .keybind("SUPER+Shift+E", "Exit sway")
        .keybind("SUPER+Shift+Space", "Toggle floating")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+E", "Toggle split horizontal/vertical")
        .keybind("SUPER+S", "Switch to stacking layout")
        .keybind("SUPER+W", "Switch to tabbed layout")
        .keybind("SUPER+T", "Switch to tiling layout")
        // Workspaces
        .keybind("SUPER+1-9", "Switch to workspace 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to workspace 1-9")
        .keybind("SUPER+Left/H", "Focus left window")
        .keybind("SUPER+Right/L", "Focus right window")
        .keybind("SUPER+Up/K", "Focus up window")
        .keybind("SUPER+Down/J", "Focus down window")
        // Applications
        .keybind("SUPER+D", "Launch application menu (wofi)")
        .keybind("SUPER+Return", "Launch terminal")
        // Media & System
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .keybind("XF86MonBrightnessUp", "Increase brightness")
        .keybind("XF86MonBrightnessDown", "Decrease brightness")
        .keybind("Print", "Screenshot")
        .build(facts)
}

/// Wayfire bundle - 3D Wayland compositor
fn wayfire_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("wayfire")
        .display_server(DisplayServer::Wayland)
        .wm_package("wayfire")
        .launcher("wofi")
        .status_bar("waybar")
        .terminal("alacritty")
        .file_manager("pcmanfm-gtk3", "lf")
        .notification_daemon("mako")
        .wallpaper_manager("swaybg")
        .lock_screen("swaylock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Q", "Close window")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+Space", "Toggle floating")
        .keybind("SUPER+M", "Maximize window")
        // Workspaces
        .keybind("SUPER+1-9", "Switch to workspace 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to workspace 1-9")
        .keybind("SUPER+Left", "Focus left window")
        .keybind("SUPER+Right", "Focus right window")
        .keybind("SUPER+Up", "Focus up window")
        .keybind("SUPER+Down", "Focus down window")
        // Applications
        .keybind("SUPER+D", "Launch application menu (wofi)")
        .keybind("SUPER+Return", "Launch terminal")
        .keybind("SUPER+E", "Launch file manager")
        // Effects (Wayfire-specific)
        .keybind("SUPER+Tab", "Window switcher effect")
        .keybind("CTRL+ALT+Left/Right", "Cube desktop rotation")
        .keybind("SUPER+F1", "Toggle expo (workspace overview)")
        // Media & System
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .keybind("XF86MonBrightnessUp", "Increase brightness")
        .keybind("XF86MonBrightnessDown", "Decrease brightness")
        .build(facts)
}

/// River bundle - Dynamic tiling Wayland compositor
fn river_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("river")
        .display_server(DisplayServer::Wayland)
        .wm_package("river")
        .launcher("fuzzel")
        .status_bar("waybar")
        .terminal("foot")
        .file_manager("thunar", "nnn")
        .notification_daemon("mako")
        .wallpaper_manager("swaybg")
        .lock_screen("swaylock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+C", "Close window")
        .keybind("SUPER+Shift+E", "Exit river")
        .keybind("SUPER+Space", "Toggle floating")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+H/L", "Adjust main window size")
        // Workspaces (Tags in River)
        .keybind("SUPER+1-9", "Switch to tag 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to tag 1-9")
        .keybind("SUPER+J", "Focus next window")
        .keybind("SUPER+K", "Focus previous window")
        // Applications
        .keybind("SUPER+D", "Launch application menu (fuzzel)")
        .keybind("SUPER+Return", "Launch terminal")
        // Layout
        .keybind("SUPER+T", "Switch to tiled layout")
        .keybind("SUPER+M", "Switch to monocle layout")
        // Media & System
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .keybind("XF86MonBrightnessUp", "Increase brightness")
        .keybind("XF86MonBrightnessDown", "Decrease brightness")
        .build(facts)
}
