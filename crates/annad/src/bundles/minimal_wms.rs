//! Minimal/Terminal-focused Window Manager Bundles
//!
//! Complete desktop setups for extremely minimal, keyboard-driven window managers:
//! - Ratpoison - Screen-like keybindings, no mouse required
//! - Wmii - Minimalist, scriptable with 9P filesystem
//! - evilwm - Minimalist, very lightweight, pure functionality

use super::{DisplayServer, WMBundleBuilder};
use anna_common::types::{Advice, SystemFacts};

/// Generate all minimal WM bundles
pub fn generate_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    advice.extend(ratpoison_bundle(facts));
    advice.extend(wmii_bundle(facts));
    advice.extend(evilwm_bundle(facts));

    advice
}

/// Ratpoison bundle - GNU Screen-like window manager (no mouse required!)
fn ratpoison_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("ratpoison")
        .display_server(DisplayServer::X11)
        .wm_package("ratpoison")
        .launcher("dmenu")
        .status_bar("") // ratpoison displays info in status messages
        .terminal("xterm")
        .file_manager("", "ranger") // Terminal-focused, no GUI file manager
        .notification_daemon("dunst")
        .wallpaper_manager("") // Minimalist - no wallpaper
        .lock_screen("xlock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Core Commands (all start with CTRL+T prefix, like GNU Screen)
        .keybind("CTRL+T, C", "Launch terminal")
        .keybind("CTRL+T, K", "Close window")
        .keybind("CTRL+T, Q", "Exit ratpoison")
        .keybind("CTRL+T, !", "Execute shell command")
        .keybind("CTRL+T, :", "Execute ratpoison command")
        // Window Navigation
        .keybind("CTRL+T, Tab", "Next window")
        .keybind("CTRL+T, N", "Next window")
        .keybind("CTRL+T, P", "Previous window")
        .keybind("CTRL+T, 0-9", "Switch to window 0-9")
        .keybind("CTRL+T, W", "Show window list")
        // Frame Management
        .keybind("CTRL+T, S", "Split frame horizontally")
        .keybind("CTRL+T, V", "Split frame vertically")
        .keybind("CTRL+T, R", "Remove current frame")
        .keybind("CTRL+T, Q", "Remove all frames except current")
        .keybind("CTRL+T, F", "Make frame fullscreen")
        // Frame Navigation
        .keybind("CTRL+T, Tab", "Next frame")
        .keybind("CTRL+T, Arrow", "Navigate to frame in direction")
        // Groups (Workspaces)
        .keybind("CTRL+T, G, N", "Create new group")
        .keybind("CTRL+T, G, Tab", "Next group")
        .keybind("CTRL+T, G, 0-9", "Switch to group 0-9")
        // Info
        .keybind("CTRL+T, T", "Show current time")
        .keybind("CTRL+T, I", "Show window info")
        .keybind("CTRL+T, ?", "Show help")
        .build(facts)
}

/// Wmii bundle - Minimalist, scriptable window manager with 9P filesystem
fn wmii_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("wmii")
        .display_server(DisplayServer::X11)
        .wm_package("wmii")
        .launcher("wimenu") // wmii's own dmenu-like tool
        .status_bar("") // wmii has built-in bar
        .terminal("xterm")
        .file_manager("", "ranger") // Terminal-focused
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+C", "Close window")
        .keybind("SUPER+D", "Launch wimenu")
        .keybind("SUPER+Return", "Launch terminal")
        // Layouts
        .keybind("SUPER+S", "Stacking layout")
        .keybind("SUPER+D", "Default (column) layout")
        .keybind("SUPER+M", "Max (fullscreen) layout")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+Shift+Space", "Toggle floating")
        // Column Management
        .keybind("SUPER+Shift+H", "Move window to left column")
        .keybind("SUPER+Shift+L", "Move window to right column")
        // Window Navigation
        .keybind("SUPER+J", "Focus next window")
        .keybind("SUPER+K", "Focus previous window")
        .keybind("SUPER+H", "Focus left column")
        .keybind("SUPER+L", "Focus right column")
        // Workspaces (Tags in wmii)
        .keybind("SUPER+1-9", "View tag 1-9")
        .keybind("SUPER+Shift+1-9", "Move window to tag 1-9")
        .keybind("SUPER+T", "Retag current window")
        // System
        .keybind("SUPER+A", "Execute action")
        .keybind("SUPER+P", "Launch program")
        .build(facts)
}

/// evilwm bundle - Minimalist window manager, pure functionality
fn evilwm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("evilwm")
        .display_server(DisplayServer::X11)
        .wm_package("evilwm")
        .launcher("dmenu")
        .status_bar("") // evilwm has no status bar
        .terminal("xterm")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management (mouse-focused by default, but has keyboard shortcuts)
        .keybind("CTRL+ALT+Return", "Launch new terminal")
        .keybind("CTRL+ALT+Escape", "Close window")
        .keybind("CTRL+ALT+H", "Move window left")
        .keybind("CTRL+ALT+J", "Move window down")
        .keybind("CTRL+ALT+K", "Move window up")
        .keybind("CTRL+ALT+L", "Move window right")
        // Workspaces (Virtual Desktops)
        .keybind("CTRL+ALT+1-8", "Switch to desktop 1-8")
        .keybind("CTRL+ALT+Left", "Previous desktop")
        .keybind("CTRL+ALT+Right", "Next desktop")
        // Window Info
        .keybind("CTRL+ALT+I", "Show window info")
        // Mouse Operations
        .keybind("ALT+Left-drag", "Move window")
        .keybind("ALT+Right-drag", "Resize window")
        .keybind("ALT+Middle-click", "Lower window")
        .keybind("CTRL+ALT+Right-drag", "Resize window keeping aspect ratio")
        .build(facts)
}
