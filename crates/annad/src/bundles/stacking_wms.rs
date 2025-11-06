//! Stacking/Floating Window Manager Bundles
//!
//! Complete desktop setups for traditional stacking window managers:
//! - Openbox - Lightweight, highly configurable
//! - Fluxbox - Fast and lightweight
//! - IceWM - Windows 95-like interface

use super::{DisplayServer, WMBundleBuilder};
use anna_common::types::{Advice, SystemFacts};

/// Generate all stacking WM bundles
pub fn generate_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    advice.extend(openbox_bundle(facts));
    advice.extend(fluxbox_bundle(facts));
    advice.extend(icewm_bundle(facts));

    advice
}

/// Openbox bundle - Lightweight, highly configurable stacking WM
fn openbox_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("openbox")
        .display_server(DisplayServer::X11)
        .wm_package("openbox")
        .launcher("rofi")
        .status_bar("tint2")
        .terminal("alacritty")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("i3lock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+Escape", "Show window menu")
        .keybind("SUPER+D", "Toggle show desktop")
        .keybind("F11", "Toggle fullscreen")
        .keybind("ALT+F5", "Un-maximize window")
        .keybind("ALT+F9", "Minimize window")
        .keybind("ALT+F10", "Maximize window")
        // Window Switching
        .keybind("ALT+Tab", "Next window")
        .keybind("ALT+Shift+Tab", "Previous window")
        // Workspaces (Desktops in Openbox)
        .keybind("CTRL+ALT+Left", "Previous desktop")
        .keybind("CTRL+ALT+Right", "Next desktop")
        .keybind("CTRL+ALT+Up", "Desktop up")
        .keybind("CTRL+ALT+Down", "Desktop down")
        // Applications
        .keybind("SUPER+Space", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        // System
        .keybind("CTRL+ALT+Delete", "System menu")
        .keybind("Right-click desktop", "Show desktop menu")
        .build(facts)
}

/// Fluxbox bundle - Fast and lightweight stacking WM
fn fluxbox_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("fluxbox")
        .display_server(DisplayServer::X11)
        .wm_package("fluxbox")
        .launcher("rofi")
        .status_bar("") // fluxbox has built-in toolbar
        .terminal("xterm")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlockmore")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+F9", "Minimize window")
        .keybind("ALT+F10", "Maximize window")
        .keybind("ALT+F11", "Toggle fullscreen")
        .keybind("ALT+F12", "Stick window to all workspaces")
        // Window Switching
        .keybind("ALT+Tab", "Next window")
        .keybind("ALT+Shift+Tab", "Previous window")
        // Workspaces
        .keybind("CTRL+F1-F12", "Switch to workspace 1-12")
        .keybind("CTRL+ALT+Left", "Previous workspace")
        .keybind("CTRL+ALT+Right", "Next workspace")
        // Applications
        .keybind("Right-click desktop", "Show menu")
        .keybind("Middle-click desktop", "Show window list")
        // System
        .keybind("CTRL+ALT+Delete", "Exit fluxbox")
        .keybind("CTRL+ALT+R", "Restart fluxbox")
        .build(facts)
}

/// IceWM bundle - Windows 95-like lightweight WM
fn icewm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("icewm")
        .display_server(DisplayServer::X11)
        .wm_package("icewm")
        .launcher("rofi")
        .status_bar("") // icewm has built-in taskbar
        .terminal("xterm")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlockmore")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+F5", "Restore window")
        .keybind("ALT+F9", "Minimize window")
        .keybind("ALT+F10", "Maximize window")
        .keybind("ALT+F11", "Toggle fullscreen")
        .keybind("ALT+F12", "Rollup window")
        // Window Switching
        .keybind("ALT+Tab", "Quick switch window")
        .keybind("ALT+Shift+Tab", "Reverse quick switch")
        .keybind("ALT+Escape", "Window list menu")
        // Workspaces
        .keybind("CTRL+ALT+Left", "Previous workspace")
        .keybind("CTRL+ALT+Right", "Next workspace")
        .keybind("CTRL+ALT+Shift+Left", "Move window to prev workspace")
        .keybind("CTRL+ALT+Shift+Right", "Move window to next workspace")
        // Applications
        .keybind("Click taskbar", "Show start menu")
        .keybind("Right-click desktop", "Show window list")
        // System
        .keybind("CTRL+ALT+Delete", "Logout dialog")
        .keybind("CTRL+ALT+R", "Restart icewm")
        .build(facts)
}
