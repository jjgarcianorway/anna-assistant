//! Tiling Window Manager Bundles
//!
//! Complete desktop setups for X11 tiling window managers:
//! - i3 - Popular tiling WM
//! - bspwm - Binary space partitioning WM
//! - dwm - Dynamic WM from suckless
//! - xmonad - Haskell-based tiling WM
//! - herbstluftwm - Manual tiling WM

use super::{DisplayServer, WMBundleBuilder};
use anna_common::types::{Advice, SystemFacts};

/// Generate all tiling WM bundles
pub fn generate_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    advice.extend(i3_bundle(facts));
    advice.extend(bspwm_bundle(facts));
    advice.extend(dwm_bundle(facts));
    advice.extend(xmonad_bundle(facts));
    advice.extend(herbstluftwm_bundle(facts));
    advice.extend(awesome_bundle(facts));
    advice.extend(qtile_bundle(facts));

    advice
}

/// i3 bundle - Popular tiling window manager
fn i3_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("i3")
        .display_server(DisplayServer::X11)
        .wm_package("i3-wm")
        .launcher("rofi")
        .status_bar("i3status")
        .terminal("alacritty")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("i3lock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+Q", "Close window")
        .keybind("SUPER+Shift+E", "Exit i3")
        .keybind("SUPER+Shift+Space", "Toggle floating")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+E", "Toggle split horizontal/vertical")
        .keybind("SUPER+S", "Switch to stacking layout")
        .keybind("SUPER+W", "Switch to tabbed layout")
        .keybind("SUPER+T", "Switch to tiling layout")
        // Workspaces
        .keybind("SUPER+1-9", "Switch to workspace 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to workspace 1-9")
        .keybind("SUPER+H/Left", "Focus left window")
        .keybind("SUPER+L/Right", "Focus right window")
        .keybind("SUPER+K/Up", "Focus up window")
        .keybind("SUPER+J/Down", "Focus down window")
        // Resize Mode
        .keybind("SUPER+R", "Enter resize mode")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
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

/// bspwm bundle - Binary space partitioning window manager
fn bspwm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("bspwm")
        .display_server(DisplayServer::X11)
        .wm_package("bspwm")
        .launcher("rofi")
        .status_bar("polybar")
        .terminal("kitty")
        .file_manager("thunar", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("betterlockscreen")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+Q", "Close window")
        .keybind("SUPER+Alt+Q", "Exit bspwm")
        .keybind("SUPER+Space", "Toggle floating")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+T", "Toggle tiled mode")
        .keybind("SUPER+Shift+T", "Toggle pseudo-tiled mode")
        .keybind("SUPER+M", "Toggle monocle layout")
        // Workspaces (Desktops in bspwm)
        .keybind("SUPER+1-9", "Switch to desktop 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to desktop 1-9")
        .keybind("SUPER+H/Left", "Focus left window")
        .keybind("SUPER+L/Right", "Focus right window")
        .keybind("SUPER+K/Up", "Focus up window")
        .keybind("SUPER+J/Down", "Focus down window")
        // Window Swapping
        .keybind("SUPER+Shift+H/Left", "Swap window left")
        .keybind("SUPER+Shift+L/Right", "Swap window right")
        .keybind("SUPER+Shift+K/Up", "Swap window up")
        .keybind("SUPER+Shift+J/Down", "Swap window down")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        // Media & System
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .keybind("XF86MonBrightnessUp", "Increase brightness")
        .keybind("XF86MonBrightnessDown", "Decrease brightness")
        .build(facts)
}

/// dwm bundle - Dynamic window manager (suckless)
fn dwm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("dwm")
        .display_server(DisplayServer::X11)
        .wm_package("dwm")
        .launcher("dmenu")
        .status_bar("") // dwm has built-in status bar
        .terminal("st")
        .file_manager("pcmanfm", "lf")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("slock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+Shift+C", "Close window")
        .keybind("ALT+Shift+Q", "Exit dwm")
        .keybind("ALT+Space", "Toggle floating")
        .keybind("ALT+F", "Toggle fullscreen")
        .keybind("ALT+M", "Toggle monocle layout")
        .keybind("ALT+T", "Switch to tiled layout")
        // Master Area
        .keybind("ALT+H", "Decrease master area")
        .keybind("ALT+L", "Increase master area")
        .keybind("ALT+Return", "Zoom window to master")
        // Workspaces (Tags in dwm)
        .keybind("ALT+1-9", "View tag 1-9")
        .keybind("ALT+Shift+1-9", "Move window to tag 1-9")
        .keybind("ALT+J", "Focus next window")
        .keybind("ALT+K", "Focus previous window")
        .keybind("ALT+Tab", "Toggle last tag")
        // Applications
        .keybind("ALT+P", "Launch application menu (dmenu)")
        .keybind("ALT+Shift+Return", "Launch terminal")
        // System
        .keybind("ALT+B", "Toggle status bar")
        .build(facts)
}

/// xmonad bundle - Haskell-based tiling window manager
fn xmonad_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("xmonad")
        .display_server(DisplayServer::X11)
        .wm_package("xmonad")
        .launcher("rofi")
        .status_bar("xmobar")
        .terminal("alacritty")
        .file_manager("thunar", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("xscreensaver")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("ALT+Shift+C", "Close window")
        .keybind("ALT+Shift+Q", "Exit xmonad")
        .keybind("ALT+Space", "Cycle layouts")
        .keybind("ALT+Shift+Space", "Reset layout")
        .keybind("ALT+T", "Sink floating window back to tiling")
        // Master Area
        .keybind("ALT+H", "Shrink master area")
        .keybind("ALT+L", "Expand master area")
        .keybind("ALT+Return", "Swap with master window")
        // Workspaces
        .keybind("ALT+1-9", "Switch to workspace 1-9")
        .keybind("ALT+SHIFT+1-9", "Move window to workspace 1-9")
        .keybind("ALT+J", "Focus next window")
        .keybind("ALT+K", "Focus previous window")
        .keybind("ALT+Shift+J", "Swap with next window")
        .keybind("ALT+Shift+K", "Swap with previous window")
        // Applications
        .keybind("ALT+P", "Launch application menu (rofi)")
        .keybind("ALT+Shift+Return", "Launch terminal")
        // System
        .keybind("ALT+Q", "Restart xmonad (recompile config)")
        .build(facts)
}

/// herbstluftwm bundle - Manual tiling window manager
fn herbstluftwm_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("herbstluftwm")
        .display_server(DisplayServer::X11)
        .wm_package("herbstluftwm")
        .launcher("rofi")
        .status_bar("polybar")
        .terminal("kitty")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("i3lock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+Q", "Close window")
        .keybind("SUPER+Shift+E", "Exit herbstluftwm")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+Space", "Toggle floating")
        .keybind("SUPER+M", "Toggle maximize")
        // Frame Splitting
        .keybind("SUPER+U", "Split frame horizontally")
        .keybind("SUPER+O", "Split frame vertically")
        .keybind("SUPER+R", "Remove current frame")
        // Workspaces (Tags)
        .keybind("SUPER+1-9", "Switch to tag 1-9")
        .keybind("SUPER+SHIFT+1-9", "Move window to tag 1-9")
        .keybind("SUPER+H/Left", "Focus left")
        .keybind("SUPER+L/Right", "Focus right")
        .keybind("SUPER+K/Up", "Focus up")
        .keybind("SUPER+J/Down", "Focus down")
        // Layout
        .keybind("SUPER+T", "Toggle frame layout")
        .keybind("SUPER+Shift+F", "Toggle frame fullscreen")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        .build(facts)
}

/// awesome bundle - Dynamic window manager with Lua configuration
fn awesome_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("awesome")
        .display_server(DisplayServer::X11)
        .wm_package("awesome")
        .launcher("rofi")
        .status_bar("") // awesome has built-in wibar
        .terminal("alacritty")
        .file_manager("thunar", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("i3lock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+Shift+C", "Close window")
        .keybind("SUPER+Shift+Q", "Exit awesome")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+Space", "Toggle floating")
        .keybind("SUPER+M", "Toggle maximize")
        .keybind("SUPER+N", "Toggle minimize")
        // Layouts
        .keybind("SUPER+L", "Increase master width")
        .keybind("SUPER+H", "Decrease master width")
        .keybind("SUPER+Shift+H", "Increase master count")
        .keybind("SUPER+Shift+L", "Decrease master count")
        .keybind("SUPER+Ctrl+Space", "Cycle layouts")
        // Workspaces (Tags in Awesome)
        .keybind("SUPER+1-9", "View tag 1-9")
        .keybind("SUPER+Shift+1-9", "Move window to tag 1-9")
        .keybind("SUPER+J", "Focus next window")
        .keybind("SUPER+K", "Focus previous window")
        .keybind("SUPER+Shift+J", "Swap with next window")
        .keybind("SUPER+Shift+K", "Swap with previous window")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        // System
        .keybind("SUPER+Ctrl+R", "Restart awesome")
        .keybind("SUPER+B", "Toggle wibar visibility")
        .build(facts)
}

/// qtile bundle - Python-based tiling window manager
fn qtile_bundle(facts: &SystemFacts) -> Vec<Advice> {
    WMBundleBuilder::new("qtile")
        .display_server(DisplayServer::Both)
        .wm_package("qtile")
        .launcher("rofi")
        .status_bar("") // qtile has built-in bar
        .terminal("alacritty")
        .file_manager("pcmanfm", "ranger")
        .notification_daemon("dunst")
        .wallpaper_manager("feh")
        .lock_screen("i3lock")
        .network_manager("networkmanager")
        .bluetooth_manager("blueman")
        // Window Management
        .keybind("SUPER+W", "Close window")
        .keybind("SUPER+Ctrl+Q", "Exit qtile")
        .keybind("SUPER+F", "Toggle fullscreen")
        .keybind("SUPER+Shift+Space", "Toggle floating")
        .keybind("SUPER+M", "Toggle maximize")
        // Layouts
        .keybind("SUPER+Space", "Cycle layouts")
        .keybind("SUPER+H", "Shrink master area")
        .keybind("SUPER+L", "Expand master area")
        .keybind("SUPER+N", "Reset window sizes")
        // Workspaces (Groups in Qtile)
        .keybind("SUPER+1-9", "Switch to group 1-9")
        .keybind("SUPER+Shift+1-9", "Move window to group 1-9")
        .keybind("SUPER+J", "Focus next window")
        .keybind("SUPER+K", "Focus previous window")
        .keybind("SUPER+Shift+J", "Shuffle window down")
        .keybind("SUPER+Shift+K", "Shuffle window up")
        // Applications
        .keybind("SUPER+D", "Launch application menu (rofi)")
        .keybind("SUPER+Return", "Launch terminal")
        .keybind("SUPER+B", "Launch browser")
        // System
        .keybind("SUPER+Ctrl+R", "Restart qtile")
        .keybind("XF86AudioRaiseVolume", "Increase volume")
        .keybind("XF86AudioLowerVolume", "Decrease volume")
        .keybind("XF86AudioMute", "Toggle mute")
        .build(facts)
}
