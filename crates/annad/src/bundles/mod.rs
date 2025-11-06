//! Window Manager Bundle Generation System
//!
//! This module provides a declarative framework for generating complete window manager
//! setup bundles. Each bundle includes all necessary components for a functional desktop:
//! - Window manager installation and configuration
//! - Application launcher, status bar, notification daemon
//! - Essential applications (terminal, file manager, browser, etc.)
//! - System utilities (network, bluetooth, audio, brightness)
//! - Hardware-specific drivers and configurations
//! - Theme and appearance setup
//! - Keybinding reference documentation
//!
//! ## Architecture
//!
//! Bundles are organized by WM category:
//! - `wayland_compositors`: Hyprland, sway, Wayfire, River
//! - `tiling_wms`: i3, bspwm, dwm, xmonad, herbstluftwm, etc.
//! - `stacking_wms`: Openbox, Fluxbox, IceWM, etc.
//! - `de_wms`: KWin, Mutter, Marco, Xfwm, etc.
//!
//! ## Usage
//!
//! ```rust
//! use bundles::WMBundleBuilder;
//!
//! let hyprland = WMBundleBuilder::new("hyprland")
//!     .display_server("wayland")
//!     .launcher("rofi-wayland")
//!     .status_bar("waybar")
//!     .terminal("kitty")
//!     .file_manager("nautilus", "ranger")
//!     .keybind("SUPER+D", "Launch application menu")
//!     .build();
//! ```

use anna_common::types::{Advice, Priority, RiskLevel, SystemFacts};
use std::collections::HashMap;

pub mod wayland_compositors;
pub mod tiling_wms;
pub mod stacking_wms;
pub mod minimal_wms;

/// Bundle variant types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleVariant {
    /// Minimal resources - lightweight, fast, low RAM/CPU
    Minimal,
    /// Terminal/CLI focused - TUI tools, keyboard-driven
    Terminal,
    /// GTK ecosystem - GNOME tools and GTK apps
    Gtk,
    /// Qt/KDE ecosystem - KDE tools and Qt apps
    Qt,
    /// Full-featured - best of everything, "unixporn ready"
    Full,
    /// Functional (default) - balanced, everything works
    Functional,
}

/// Display server type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Both,
}

/// Component categories for a complete desktop setup
#[derive(Debug, Clone)]
pub struct WMComponents {
    // Core WM
    pub wm_package: String,

    // UI Components
    pub launcher: Option<String>,
    pub status_bar: Option<String>,
    pub notification_daemon: Option<String>,
    pub wallpaper_manager: Option<String>,
    pub lock_screen: Option<String>,

    // Applications
    pub terminal: Option<String>,
    pub file_manager_gui: Option<String>,
    pub file_manager_tui: Option<String>,
    pub text_editor: Option<String>,
    pub browser: Option<String>,
    pub media_player: Option<String>,

    // System Tools
    pub network_manager: Option<String>,
    pub bluetooth_manager: Option<String>,
    pub audio_control: Option<String>,
    pub brightness_control: Option<String>,

    // Theming
    pub gtk_theme: Option<String>,
    pub icon_theme: Option<String>,
    pub cursor_theme: Option<String>,
    pub font: Option<String>,

    // Hardware-specific
    pub gpu_driver_nvidia: Option<String>,
    pub gpu_driver_amd: Option<String>,

    // Keybindings (key → description)
    pub keybindings: HashMap<String, String>,
}

impl Default for WMComponents {
    fn default() -> Self {
        Self {
            wm_package: String::new(),
            launcher: None,
            status_bar: None,
            notification_daemon: None,
            wallpaper_manager: None,
            lock_screen: None,
            terminal: None,
            file_manager_gui: None,
            file_manager_tui: None,
            text_editor: None,
            browser: None,
            media_player: None,
            network_manager: None,
            bluetooth_manager: None,
            audio_control: None,
            brightness_control: None,
            gtk_theme: None,
            icon_theme: None,
            cursor_theme: None,
            font: None,
            gpu_driver_nvidia: None,
            gpu_driver_amd: None,
            keybindings: HashMap::new(),
        }
    }
}

/// Builder for window manager bundles
pub struct WMBundleBuilder {
    wm_name: String,
    bundle_id: String,
    display_server: DisplayServer,
    variant: BundleVariant,
    components: WMComponents,
}

impl WMBundleBuilder {
    /// Create a new bundle builder for a window manager
    pub fn new(wm_name: &str) -> Self {
        Self {
            wm_name: wm_name.to_string(),
            bundle_id: format!("{}-setup", wm_name),
            display_server: DisplayServer::Both,
            variant: BundleVariant::Functional,
            components: WMComponents::default(),
        }
    }

    /// Set display server type
    pub fn display_server(mut self, server: DisplayServer) -> Self {
        self.display_server = server;
        self
    }

    /// Set bundle variant
    pub fn variant(mut self, variant: BundleVariant) -> Self {
        self.variant = variant;
        match variant {
            BundleVariant::Minimal => self.bundle_id = format!("{}-minimal", self.wm_name),
            BundleVariant::Terminal => self.bundle_id = format!("{}-terminal", self.wm_name),
            BundleVariant::Gtk => self.bundle_id = format!("{}-gtk", self.wm_name),
            BundleVariant::Qt => self.bundle_id = format!("{}-qt", self.wm_name),
            BundleVariant::Full => self.bundle_id = format!("{}-full", self.wm_name),
            BundleVariant::Functional => self.bundle_id = format!("{}-setup", self.wm_name),
        }
        self
    }

    /// Set WM package name
    pub fn wm_package(mut self, package: &str) -> Self {
        self.components.wm_package = package.to_string();
        self
    }

    /// Set application launcher
    pub fn launcher(mut self, launcher: &str) -> Self {
        self.components.launcher = Some(launcher.to_string());
        self
    }

    /// Set status bar
    pub fn status_bar(mut self, bar: &str) -> Self {
        self.components.status_bar = Some(bar.to_string());
        self
    }

    /// Set terminal emulator
    pub fn terminal(mut self, term: &str) -> Self {
        self.components.terminal = Some(term.to_string());
        self
    }

    /// Set file managers (GUI and TUI)
    pub fn file_manager(mut self, gui: &str, tui: &str) -> Self {
        self.components.file_manager_gui = Some(gui.to_string());
        self.components.file_manager_tui = Some(tui.to_string());
        self
    }

    /// Set notification daemon
    pub fn notification_daemon(mut self, daemon: &str) -> Self {
        self.components.notification_daemon = Some(daemon.to_string());
        self
    }

    /// Set wallpaper manager
    pub fn wallpaper_manager(mut self, manager: &str) -> Self {
        self.components.wallpaper_manager = Some(manager.to_string());
        self
    }

    /// Set lock screen
    pub fn lock_screen(mut self, locker: &str) -> Self {
        self.components.lock_screen = Some(locker.to_string());
        self
    }

    /// Set network manager
    pub fn network_manager(mut self, manager: &str) -> Self {
        self.components.network_manager = Some(manager.to_string());
        self
    }

    /// Set bluetooth manager
    pub fn bluetooth_manager(mut self, manager: &str) -> Self {
        self.components.bluetooth_manager = Some(manager.to_string());
        self
    }

    /// Add a keybinding
    pub fn keybind(mut self, key: &str, description: &str) -> Self {
        self.components.keybindings.insert(key.to_string(), description.to_string());
        self
    }

    /// Helper to create a bundle advice item
    fn make_advice(
        &self,
        id_suffix: &str,
        title: String,
        reason: String,
        action: String,
        category: String,
        priority: Priority,
    ) -> Advice {
        Advice::new(
            format!("{}-{}", self.bundle_id, id_suffix),
            title,
            reason,
            action,
            None, // command - actions are direct
            RiskLevel::Low, // bundles are typically safe
            priority,
            vec![], // wiki_refs
            category,
        )
        .with_bundle(self.bundle_id.clone())
    }

    /// Build the bundle into a list of Advice items
    pub fn build(self, facts: &SystemFacts) -> Vec<Advice> {
        let mut advice = Vec::new();
        let bundle_name = &self.bundle_id;

        // 1. Window Manager
        if !self.is_package_installed(&self.components.wm_package, facts) {
            advice.push(
                self.make_advice(
                    "wm",
                    format!("Install {} window manager", self.wm_name),
                    format!(
                        "{} is a popular {} window manager. This bundle will set up a complete desktop environment with all necessary components.",
                        self.wm_name,
                        match self.display_server {
                            DisplayServer::Wayland => "Wayland",
                            DisplayServer::X11 => "X11",
                            DisplayServer::Both => "X11/Wayland",
                        }
                    ),
                    format!("pacman -S --noconfirm {}", self.components.wm_package),
                    "desktop".to_string(),
                    Priority::Recommended,
                )
            );
        }

        // 2. Launcher
        if let Some(launcher) = &self.components.launcher {
            if !self.is_package_installed(launcher, facts) {
                advice.push(
                    self.make_advice(
                        "launcher",
                        format!("Install {} application launcher", launcher),
                        format!("{} provides a fast application launcher for your desktop.", launcher),
                        format!("pacman -S --noconfirm {}", launcher),
                        "desktop".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 3. Status Bar
        if let Some(bar) = &self.components.status_bar {
            if !self.is_package_installed(bar, facts) {
                advice.push(
                    self.make_advice(
                        "statusbar",
                        format!("Install {} status bar", bar),
                        format!("{} displays system information, workspaces, and notifications.", bar),
                        format!("pacman -S --noconfirm {}", bar),
                        "desktop".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 4. Terminal
        if let Some(terminal) = &self.components.terminal {
            if !self.is_package_installed(terminal, facts) {
                advice.push(
                    self.make_advice(
                        "terminal",
                        format!("Install {} terminal emulator", terminal),
                        format!("{} is a modern terminal emulator with good performance.", terminal),
                        format!("pacman -S --noconfirm {}", terminal),
                        "desktop".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 5. File Manager (GUI)
        if let Some(fm) = &self.components.file_manager_gui {
            if !self.is_package_installed(fm, facts) {
                advice.push(
                    self.make_advice(
                        "filemanager",
                        format!("Install {} file manager", fm),
                        format!("{} provides graphical file management.", fm),
                        format!("pacman -S --noconfirm {}", fm),
                        "desktop".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 6. Notification Daemon
        if let Some(daemon) = &self.components.notification_daemon {
            if !self.is_package_installed(daemon, facts) {
                advice.push(
                    self.make_advice(
                        "notifications",
                        format!("Install {} notification daemon", daemon),
                        format!("{} displays desktop notifications.", daemon),
                        format!("pacman -S --noconfirm {}", daemon),
                        "desktop".to_string(),
                        Priority::Optional,
                    )
                );
            }
        }

        // 7. Network Manager
        if let Some(nm) = &self.components.network_manager {
            if !self.is_package_installed(nm, facts) {
                advice.push(
                    self.make_advice(
                        "network",
                        format!("Install {} network manager", nm),
                        format!("{} provides easy network configuration.", nm),
                        format!("pacman -S --noconfirm {}", nm),
                        "system".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 8. Bluetooth Manager
        if let Some(bt) = &self.components.bluetooth_manager {
            if !self.is_package_installed(bt, facts) {
                advice.push(
                    self.make_advice(
                        "bluetooth",
                        format!("Install {} bluetooth manager", bt),
                        format!("{} provides bluetooth device management.", bt),
                        format!("pacman -S --noconfirm bluez bluez-utils {}", bt),
                        "system".to_string(),
                        Priority::Optional,
                    )
                );
            }
        }

        // 9. Keybinding Reference
        if !self.components.keybindings.is_empty() {
            let mut keybind_text = format!("## {} Keyboard Shortcuts\n\n", self.wm_name);

            // Group by category
            let mut window_mgmt = Vec::new();
            let mut apps = Vec::new();
            let mut media = Vec::new();
            let mut system = Vec::new();

            for (key, desc) in &self.components.keybindings {
                let entry = format!("- **{}**: {}", key, desc);
                if desc.contains("window") || desc.contains("workspace") || desc.contains("focus") {
                    window_mgmt.push(entry);
                } else if desc.contains("Launch") || desc.contains("Open") {
                    apps.push(entry);
                } else if desc.contains("volume") || desc.contains("brightness") || desc.contains("media") {
                    media.push(entry);
                } else {
                    system.push(entry);
                }
            }

            if !window_mgmt.is_empty() {
                keybind_text.push_str("### Window Management\n");
                for entry in window_mgmt {
                    keybind_text.push_str(&format!("{}\n", entry));
                }
                keybind_text.push('\n');
            }

            if !apps.is_empty() {
                keybind_text.push_str("### Applications\n");
                for entry in apps {
                    keybind_text.push_str(&format!("{}\n", entry));
                }
                keybind_text.push('\n');
            }

            if !media.is_empty() {
                keybind_text.push_str("### Media & System\n");
                for entry in media {
                    keybind_text.push_str(&format!("{}\n", entry));
                }
                keybind_text.push('\n');
            }

            advice.push(
                self.make_advice(
                    "keybindings",
                    format!("View {} keyboard shortcuts", self.wm_name),
                    keybind_text,
                    "echo 'Keybinding reference shown above'".to_string(),
                    "documentation".to_string(),
                    Priority::Cosmetic,
                )
            );
        }

        advice
    }

    /// Helper to check if package is installed
    fn is_package_installed(&self, package: &str, _facts: &SystemFacts) -> bool {
        use std::process::Command;
        Command::new("pacman")
            .args(&["-Q", package])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Generate all window manager bundles
pub fn generate_all_wm_bundles(facts: &SystemFacts) -> Vec<Advice> {
    let mut advice = Vec::new();

    // Wayland compositors (4 WMs)
    advice.extend(wayland_compositors::generate_bundles(facts));

    // Tiling WMs (9 WMs: i3, bspwm, dwm, xmonad, herbstluftwm, awesome, qtile, leftwm, spectrwm)
    advice.extend(tiling_wms::generate_bundles(facts));

    // Stacking WMs (3 WMs: openbox, fluxbox, icewm)
    advice.extend(stacking_wms::generate_bundles(facts));

    // Minimal WMs (3 WMs: ratpoison, wmii, evilwm)
    advice.extend(minimal_wms::generate_bundles(facts));

    advice
}
