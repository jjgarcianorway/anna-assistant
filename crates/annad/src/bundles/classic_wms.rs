//! Classic/Traditional Window Manager Bundles
//!
//! Complete desktop setups for classic, nostalgic, and highly-configurable window managers:
//! - Window Maker - GNUstep/NeXTSTEP-like interface
//! - FVWM - Classic, infinitely configurable
//! - Enlightenment - Beautiful, feature-rich
//!
//! NOTE: Disabled for v1.0 (Hyprland-only), may return in v2.0

#![allow(dead_code)]

use super::{DisplayServer, WMBundleBuilder};
use anna_common::types::{Advice, SystemFacts};

/// Generate all classic WM bundles
pub fn generate_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    advice.extend(windowmaker_bundle(facts));
    advice.extend(fvwm_bundle(facts));
    advice.extend(enlightenment_bundle(facts));

    advice
}

/// Window Maker bundle - GNUstep/NeXTSTEP-like window manager
fn windowmaker_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("windowmaker")
        .display_server(DisplayServer::X11)
        .wm_package("windowmaker")
        .launcher("") // Window Maker has built-in app menu
        .status_bar("") // Window Maker has dock and clip
        .terminal("xterm")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+F1", "Open applications menu")
        .keybind("ALT+F2", "Run command")
        .keybind("ALT+F3", "Open window list")
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+F9", "Minimize window")
        .keybind("ALT+F10", "Maximize window")
        .keybind("ALT+F11", "Shade window (roll up)")
        .keybind("ALT+F12", "Keep window on top")
        // Window Navigation
        .keybind("ALT+Tab", "Next window")
        .keybind("ALT+Shift+Tab", "Previous window")
        .keybind("ALT+Ctrl+Tab", "Next window (all workspaces)")
        // Workspaces
        .keybind("CTRL+F1-F10", "Switch to workspace 1-10")
        .keybind("CTRL+ALT+Right", "Next workspace")
        .keybind("CTRL+ALT+Left", "Previous workspace")
        // Dock Management
        .keybind("Right-click dock", "Dock menu")
        .keybind("Drag to dock", "Add application to dock")
        .keybind("Middle-click icon", "Hide application")
        // Session
        .keybind("ALT+Ctrl+Escape", "Exit Window Maker")
        .keybind("ALT+Ctrl+R", "Restart Window Maker")
        // Mouse Operations
        .keybind("ALT+Left-drag titlebar", "Move window")
        .keybind("ALT+Right-drag titlebar", "Resize window")
        .keybind("Double-click titlebar", "Shade window")
        .build(facts)
}

/// FVWM bundle - Highly configurable classic window manager
fn fvwm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("fvwm")
        .display_server(DisplayServer::X11)
        .wm_package("fvwm")
        .launcher("") // FVWM has built-in menus
        .status_bar("") // FVWM can have custom modules
        .terminal("xterm")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xlock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+F1", "Open main menu")
        .keybind("ALT+F2", "Open window ops menu")
        .keybind("ALT+F3", "Lower window")
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+F5", "CirculateUp (previous window)")
        .keybind("ALT+F6", "CirculateDown (next window)")
        .keybind("ALT+F7", "Move window")
        .keybind("ALT+F8", "Resize window")
        .keybind("ALT+F9", "Iconify (minimize)")
        .keybind("ALT+F10", "Maximize")
        .keybind("ALT+F11", "Fullscreen")
        .keybind("ALT+F12", "Stick (all desktops)")
        // Desktop Navigation
        .keybind("CTRL+ALT+Left", "Previous desk")
        .keybind("CTRL+ALT+Right", "Next desk")
        .keybind("CTRL+ALT+Up", "Previous page")
        .keybind("CTRL+ALT+Down", "Next page")
        // Pager
        .keybind("Click on pager", "Switch to desk/page")
        .keybind("Drag window on pager", "Move window to desk")
        // Mouse Operations
        .keybind("Left-click title", "Raise/focus window")
        .keybind("Right-click title", "Window operations menu")
        .keybind("Middle-click title", "Lower window")
        .keybind("Right-click desktop", "Open menu")
        // System
        .keybind("ALT+Shift+Q", "Quit FVWM")
        .keybind("ALT+Shift+R", "Restart FVWM")
        .build(facts)
}

/// Enlightenment bundle - Beautiful, feature-rich window manager/compositor
fn enlightenment_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("enlightenment")
        .display_server(DisplayServer::Both) // Supports both X11 and Wayland
        .wm_package("enlightenment")
        .launcher("") // Enlightenment has built-in Everything launcher
        .status_bar("") // Enlightenment has integrated shelf
        .terminal("terminology") // Enlightenment's own terminal
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("") // Enlightenment has built-in notifications
        .wallpaper_manager("") // Enlightenment manages wallpapers
        .lock_screen("") // Enlightenment has built-in screen locking
        .network_manager("connman") // Enlightenment recommends ConnMan
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+Escape", "Show main menu")
        .keybind("ALT+F4", "Close window")
        .keybind("ALT+F10", "Maximize window")
        .keybind("ALT+F11", "Toggle fullscreen")
        .keybind("CTRL+ALT+F", "Toggle fullscreen")
        .keybind("CTRL+ALT+S", "Shade window")
        .keybind("CTRL+ALT+I", "Iconify window")
        .keybind("CTRL+ALT+K", "Close window")
        .keybind("CTRL+ALT+X", "Kill window")
        // Desktop Navigation
        .keybind("CTRL+ALT+Left", "Previous desktop")
        .keybind("CTRL+ALT+Right", "Next desktop")
        .keybind("CTRL+ALT+Up", "Flip desktop up")
        .keybind("CTRL+ALT+Down", "Flip desktop down")
        .keybind("CTRL+F1-F12", "Switch to desktop 1-12")
        // Window Switching
        .keybind("ALT+Tab", "Next window")
        .keybind("ALT+Shift+Tab", "Previous window")
        .keybind("CTRL+ALT+Tab", "Next window (all desktops)")
        // Launcher
        .keybind("ALT+Space", "Show Everything launcher")
        .keybind("CTRL+ALT+Space", "Show Everything launcher (alternate)")
        // Window Placement
        .keybind("CTRL+ALT+H", "Move window left")
        .keybind("CTRL+ALT+L", "Move window right")
        .keybind("CTRL+ALT+J", "Move window down")
        .keybind("CTRL+ALT+K", "Move window up")
        // System
        .keybind("CTRL+ALT+Delete", "Logout dialog")
        .keybind("CTRL+ALT+End", "Shutdown dialog")
        .keybind("CTRL+ALT+Insert", "Lock screen")
        .keybind("CTRL+ALT+R", "Restart Enlightenment")
        // Mouse Operations
        .keybind("Middle-click desktop", "Show window list")
        .keybind("Right-click desktop", "Show desktop menu")
        .keybind("Left-click shelf", "Launch applications")
        .build(facts)
}
