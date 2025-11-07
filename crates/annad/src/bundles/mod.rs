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

use anna_common::types::{Advice, Priority, Requirement, RiskLevel, SystemFacts};
use std::collections::HashMap;

pub mod wayland_compositors;
pub mod tiling_wms;
pub mod stacking_wms;
pub mod minimal_wms;
pub mod classic_wms;

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
    pub image_viewer: Option<String>,
    pub pdf_viewer: Option<String>,

    // System Tools
    pub network_manager: Option<String>,
    pub bluetooth_manager: Option<String>,
    pub audio_control: Option<String>,
    pub brightness_control: Option<String>,
    pub screen_sharing: Option<String>, // xdg-desktop-portal backend for screen sharing
    pub audio_server: Option<String>, // pipewire for modern audio/video

    // Theming
    pub gtk_theme: Option<String>,
    pub icon_theme: Option<String>,
    pub cursor_theme: Option<String>,
    pub font: Option<String>,
    pub color_scheme_generator: Option<String>, // pywal, etc.

    // Hardware-specific
    pub gpu_driver_nvidia: Option<String>,
    pub gpu_driver_amd: Option<String>,

    // Keybindings (key → description)
    pub keybindings: HashMap<String, String>,

    // Configuration files (component_id → template_subpath)
    // e.g., "hyprland" → ".config/hypr", "waybar" → ".config/waybar"
    pub config_files: HashMap<String, String>,
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
            image_viewer: None,
            pdf_viewer: None,
            network_manager: None,
            bluetooth_manager: None,
            audio_control: None,
            brightness_control: None,
            screen_sharing: None,
            audio_server: None,
            gtk_theme: None,
            icon_theme: None,
            cursor_theme: None,
            font: None,
            color_scheme_generator: None,
            gpu_driver_nvidia: None,
            gpu_driver_amd: None,
            keybindings: HashMap::new(),
            config_files: HashMap::new(),
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

    /// Set audio control tool
    pub fn audio_control(mut self, tool: &str) -> Self {
        self.components.audio_control = Some(tool.to_string());
        self
    }

    /// Set brightness control tool
    pub fn brightness_control(mut self, tool: &str) -> Self {
        self.components.brightness_control = Some(tool.to_string());
        self
    }

    /// Set media player
    pub fn media_player(mut self, player: &str) -> Self {
        self.components.media_player = Some(player.to_string());
        self
    }

    /// Set image viewer
    pub fn image_viewer(mut self, viewer: &str) -> Self {
        self.components.image_viewer = Some(viewer.to_string());
        self
    }

    /// Set PDF viewer
    pub fn pdf_viewer(mut self, viewer: &str) -> Self {
        self.components.pdf_viewer = Some(viewer.to_string());
        self
    }

    /// Set text editor
    pub fn text_editor(mut self, editor: &str) -> Self {
        self.components.text_editor = Some(editor.to_string());
        self
    }

    /// Set GTK theme
    pub fn gtk_theme(mut self, theme: &str) -> Self {
        self.components.gtk_theme = Some(theme.to_string());
        self
    }

    /// Set icon theme
    pub fn icon_theme(mut self, theme: &str) -> Self {
        self.components.icon_theme = Some(theme.to_string());
        self
    }

    /// Set cursor theme
    pub fn cursor_theme(mut self, theme: &str) -> Self {
        self.components.cursor_theme = Some(theme.to_string());
        self
    }

    /// Set color scheme generator (pywal, etc.)
    pub fn color_scheme_generator(mut self, generator: &str) -> Self {
        self.components.color_scheme_generator = Some(generator.to_string());
        self
    }

    /// Set screen sharing portal (for Teams/Zoom)
    pub fn screen_sharing(mut self, portal: &str) -> Self {
        self.components.screen_sharing = Some(portal.to_string());
        self
    }

    /// Set audio server (pipewire for modern audio/video)
    pub fn audio_server(mut self, server: &str) -> Self {
        self.components.audio_server = Some(server.to_string());
        self
    }

    /// Add a keybinding
    pub fn keybind(mut self, key: &str, description: &str) -> Self {
        self.components.keybindings.insert(key.to_string(), description.to_string());
        self
    }

    /// Add a configuration file mapping
    /// Example: .config("hyprland", ".config/hypr")
    pub fn config(mut self, component_id: &str, template_subpath: &str) -> Self {
        self.components.config_files.insert(component_id.to_string(), template_subpath.to_string());
        self
    }

    /// Helper to create a bundle advice item
    fn make_advice(
        &self,
        id_suffix: &str,
        title: String,
        reason: String,
        command: String,
        category: String,
        priority: Priority,
    ) -> Advice {
        Advice::new(
            format!("{}-{}", self.bundle_id, id_suffix),
            title,
            reason,
            command.clone(), // action = human readable description
            Some(command), // command = executable shell command
            RiskLevel::Low, // bundles are typically safe
            priority,
            vec![], // wiki_refs
            category,
        )
        .with_bundle(self.bundle_id.clone())
    }

    /// Create advice with system requirements (RC.6)
    /// Only show this component if system meets the requirements
    fn make_advice_with_requirements(
        &self,
        id_suffix: &str,
        title: String,
        reason: String,
        command: String,
        category: String,
        priority: Priority,
        requirements: Vec<Requirement>,
    ) -> Advice {
        self.make_advice(id_suffix, title, reason, command, category, priority)
            .with_requirements(requirements)
    }

    /// Build the bundle into a list of Advice items
    pub fn build(self, facts: &SystemFacts) -> Vec<Advice> {
        let mut advice = Vec::new();
        let bundle_name = &self.bundle_id;

        // IMPORTANT: Only show bundle if WM is already installed!
        // Don't recommend 20+ different WMs - only show components for what user has
        let wm_installed = self.is_package_installed(&self.components.wm_package, facts);

        if !wm_installed {
            // WM not installed - don't show this bundle at all
            return advice;
        }

        // WM IS installed - show recommended complementary components
        // (No need to recommend installing the WM itself since it's already installed)

        // 1. Launcher
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

        // 2. Status Bar
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

        // 3. Terminal
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

        // 4. File Manager (GUI)
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

        // 5. Notification Daemon
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

        // 6. Network Manager
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

        // 7. Bluetooth Manager (RC.6 - Only if Bluetooth hardware present)
        if let Some(bt) = &self.components.bluetooth_manager {
            if !self.is_package_installed(bt, facts) {
                advice.push(
                    self.make_advice_with_requirements(
                        "bluetooth",
                        format!("Install {} bluetooth manager", bt),
                        format!("{} provides bluetooth device management.", bt),
                        format!("pacman -S --noconfirm bluez bluez-utils {}", bt),
                        "system".to_string(),
                        Priority::Optional,
                        vec![Requirement::Bluetooth],
                    )
                );
            }
        }

        // 7a. Audio Control Tool (RC.6 - Only if audio system present)
        if let Some(audio) = &self.components.audio_control {
            if !self.is_package_installed(audio, facts) {
                advice.push(
                    self.make_advice_with_requirements(
                        "audio",
                        format!("Install {} audio control", audio),
                        format!("{} provides volume control for multimedia keys.", audio),
                        format!("pacman -S --noconfirm {}", audio),
                        "system".to_string(),
                        Priority::Recommended,
                        vec![Requirement::AudioSystem],
                    )
                );
            }
        }

        // 7b. Brightness Control Tool (RC.6 - Only on laptops)
        if let Some(brightness) = &self.components.brightness_control {
            // Only install brightness control on laptops
            if facts.user_preferences.uses_laptop && !self.is_package_installed(brightness, facts) {
                advice.push(
                    self.make_advice_with_requirements(
                        "brightness",
                        format!("Install {} brightness control", brightness),
                        format!("{} provides brightness control for laptop multimedia keys.", brightness),
                        format!("pacman -S --noconfirm {}", brightness),
                        "system".to_string(),
                        Priority::Recommended,
                        vec![Requirement::Laptop],
                    )
                );
            }
        }

        // 7c. Media Player (RC.6 - Needs display server AND audio system)
        if let Some(player) = &self.components.media_player {
            if !self.is_package_installed(player, facts) {
                advice.push(
                    self.make_advice_with_requirements(
                        "media-player",
                        format!("Install {} video player", player),
                        format!("{} is a powerful media player for videos and music.", player),
                        format!("pacman -S --noconfirm {}", player),
                        "applications".to_string(),
                        Priority::Recommended,
                        vec![Requirement::DisplayServer, Requirement::AudioSystem],
                    )
                );
            }
        }

        // 7d. Image Viewer (Beta.113)
        if let Some(viewer) = &self.components.image_viewer {
            if !self.is_package_installed(viewer, facts) {
                advice.push(
                    self.make_advice(
                        "image-viewer",
                        format!("Install {} image viewer", viewer),
                        format!("{} is a fast image viewer for viewing photos.", viewer),
                        format!("pacman -S --noconfirm {}", viewer),
                        "applications".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 7e. PDF Viewer (Beta.113)
        if let Some(pdf) = &self.components.pdf_viewer {
            if !self.is_package_installed(pdf, facts) {
                advice.push(
                    self.make_advice(
                        "pdf-viewer",
                        format!("Install {} PDF viewer", pdf),
                        format!("{} allows you to read PDF documents.", pdf),
                        format!("pacman -S --noconfirm {}", pdf),
                        "applications".to_string(),
                        Priority::Optional,
                    )
                );
            }
        }

        // 7f. Text Editor (Beta.113)
        if let Some(editor) = &self.components.text_editor {
            if !self.is_package_installed(editor, facts) {
                advice.push(
                    self.make_advice(
                        "text-editor",
                        format!("Install {} text editor", editor),
                        format!("{} is a modern text editor.", editor),
                        format!("pacman -S --noconfirm {}", editor),
                        "applications".to_string(),
                        Priority::Optional,
                    )
                );
            }
        }

        // 7g. Color Scheme Generator - Pywal! (Beta.113)
        if let Some(generator) = &self.components.color_scheme_generator {
            if !self.is_package_installed(generator, facts) {
                advice.push(
                    self.make_advice(
                        "color-scheme",
                        format!("Install {} - Auto color themes from wallpaper!", generator),
                        format!("{} automatically generates beautiful color schemes from your wallpaper and applies them system-wide to terminal, waybar, rofi, etc. Your desktop will always match your wallpaper!", generator),
                        format!("pacman -S --noconfirm {}", generator),
                        "beautification".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 7h. GTK Theme (Beta.113)
        if let Some(theme) = &self.components.gtk_theme {
            if !self.is_package_installed(theme, facts) {
                advice.push(
                    self.make_advice(
                        "gtk-theme",
                        format!("Install {} GTK theme", theme),
                        format!("{} provides beautiful theming for GTK applications.", theme),
                        format!("pacman -S --noconfirm {}", theme),
                        "beautification".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 7i. Icon Theme (Beta.113)
        if let Some(icons) = &self.components.icon_theme {
            if !self.is_package_installed(icons, facts) {
                advice.push(
                    self.make_advice(
                        "icon-theme",
                        format!("Install {} icon theme", icons),
                        format!("{} provides beautiful system-wide icons.", icons),
                        format!("pacman -S --noconfirm {}", icons),
                        "beautification".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 7j. Cursor Theme (Beta.113)
        if let Some(cursor) = &self.components.cursor_theme {
            if !self.is_package_installed(cursor, facts) {
                advice.push(
                    self.make_advice(
                        "cursor-theme",
                        format!("Install {} cursor theme", cursor),
                        format!("{} provides a modern cursor design.", cursor),
                        format!("pacman -S --noconfirm {}", cursor),
                        "beautification".to_string(),
                        Priority::Optional,
                    )
                );
            }
        }

        // 7k. Audio Server - Pipewire! (Beta.114)
        if let Some(audio_server) = &self.components.audio_server {
            if !self.is_package_installed(audio_server, facts) {
                advice.push(
                    self.make_advice(
                        "audio-server",
                        format!("Install {} - Modern audio/video server", audio_server),
                        format!("{} provides modern audio routing and is required for screen sharing in Teams, Zoom, and other video conferencing apps. Also includes wireplumber session manager.", audio_server),
                        format!("pacman -S --noconfirm {} wireplumber", audio_server),
                        "system".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 7l. Screen Sharing Portal (Beta.114)
        if let Some(portal) = &self.components.screen_sharing {
            if !self.is_package_installed(portal, facts) {
                advice.push(
                    self.make_advice(
                        "screen-sharing",
                        format!("Install {} - Screen sharing for Teams/Zoom", portal),
                        format!("{} enables screen sharing in video conferencing apps like Teams, Zoom, Google Meet, etc. Required for xdg-desktop-portal screen capture.", portal),
                        format!("pacman -S --noconfirm {} xdg-desktop-portal", portal),
                        "system".to_string(),
                        Priority::Recommended,
                    )
                );
            }
        }

        // 8. Configuration Files (Beta.111)
        // Copy configuration templates from /usr/share/anna/templates/ to user's ~/.config/
        for (component_id, template_subpath) in &self.components.config_files {
            let target_dir = format!("$HOME/{}", template_subpath);
            let source_path = format!("/usr/share/anna/templates/{}", template_subpath);

            // Extract parent directory name for display (e.g., ".config/hypr" → "hypr")
            let display_name = template_subpath.split('/').last().unwrap_or(component_id);

            let command = format!(
                "mkdir -p {} && cp -r {}/* {} 2>/dev/null || echo 'Config template not found at {}, skipping...'",
                target_dir, source_path, target_dir, source_path
            );

            advice.push(
                self.make_advice(
                    &format!("config-{}", component_id),
                    format!("Setup {} configuration", display_name),
                    format!("Copies configuration files from {} to {}", source_path, target_dir),
                    command,
                    "configuration".to_string(),
                    Priority::Recommended,
                )
            );
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

    // Classic WMs (3 WMs: windowmaker, fvwm, enlightenment)
    advice.extend(classic_wms::generate_bundles(facts));

    advice
}
