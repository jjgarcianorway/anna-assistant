//! Desktop recommendations

use super::{check_command_usage, is_package_installed};

use anna_common::{Advice, Alternative, Priority, RiskLevel, SystemFacts};
use std::path::Path;
use std::process::Command;

pub(crate) fn check_status_bar(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if user has a WM (not DE with built-in bar) via package groups
    let wms_needing_bar = vec!["i3", "sway", "hyprland", "bspwm", "dwm", "xmonad"];
    let has_wm_needing_bar = facts.package_groups.iter().any(|group| {
        wms_needing_bar
            .iter()
            .any(|wm| group.to_lowercase().contains(wm))
    });

    if !has_wm_needing_bar {
        return result;
    }

    // Check for waybar (Wayland) - check if command exists
    let has_waybar = Command::new("which")
        .arg("waybar")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for i3status/polybar (X11)
    let has_i3status = Command::new("which")
        .arg("i3status")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_polybar = Command::new("which")
        .arg("polybar")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_waybar && !has_i3status && !has_polybar {
        // Recommend based on display server
        if facts.display_server.as_deref() == Some("wayland") {
            result.push(Advice {
                id: "install-waybar".to_string(),
                title: "Install Waybar for your Wayland WM".to_string(),
                reason: "You're using a window manager that doesn't come with a status bar. Waybar is the best status bar for Wayland compositors - it's highly customizable, shows system info, workspaces, and looks beautiful. Without it, you won't see battery, network, or time!".to_string(),
                action: "Install and configure Waybar".to_string(),
                command: Some("pacman -S --noconfirm waybar && mkdir -p ~/.config/waybar && cp /etc/xdg/waybar/* ~/.config/waybar/".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "yambar".to_string(),
                        description: "Lightweight Wayland bar".to_string(),
                        install_command: "pacman -S yambar".to_string(),
                    },
                    Alternative {
                        name: "eww".to_string(),
                        description: "Widgets for any window manager".to_string(),
                        install_command: "yay -S --noconfirm eww".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("wayland-essentials".to_string()),
            satisfies: Vec::new(),
                popularity: 80,
            requires: Vec::new(),
            });
        } else {
            result.push(Advice {
                id: "install-polybar".to_string(),
                title: "Install Polybar for your window manager".to_string(),
                reason: "Your WM doesn't have a built-in status bar. Polybar is a fast, customizable bar for X11 window managers. It shows workspaces, system info, and can be themed to match your setup. Essential for i3/bspwm users!".to_string(),
                action: "Install Polybar".to_string(),
                command: Some("pacman -S --noconfirm polybar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "i3status".to_string(),
                        description: "Simple, built-in status for i3".to_string(),
                        install_command: "pacman -S i3status".to_string(),
                    },
                    Alternative {
                        name: "lemonbar".to_string(),
                        description: "Minimal bar for scripting enthusiasts".to_string(),
                        install_command: "yay -S --noconfirm lemonbar".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Polybar".to_string()],
                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: Some("wm-essentials".to_string()),
            satisfies: Vec::new(),
                popularity: 75,
            requires: Vec::new(),
            });
        }
    }

    // Check if bar is installed but not configured to autostart
    if has_waybar || has_polybar {
        let bar_name = if has_waybar { "waybar" } else { "polybar" };

        // Check common autostart locations
        let home = std::env::var("HOME").unwrap_or_default();
        let autostart_locations = vec![
            format!("{}/.config/sway/config", home),
            format!("{}/.config/hypr/hyprland.conf", home),
            format!("{}/.config/i3/config", home),
            format!("{}/.config/bspwm/bspwmrc", home),
        ];

        let mut bar_in_config = false;
        for location in autostart_locations {
            if let Ok(content) = std::fs::read_to_string(&location) {
                if content.contains(bar_name) {
                    bar_in_config = true;
                    break;
                }
            }
        }

        if !bar_in_config {
            result.push(Advice {
                id: format!("configure-{}-autostart", bar_name),
                title: format!("Configure {} to start automatically", bar_name),
                reason: format!("You have {} installed, but it's not configured to start automatically with your window manager! You need to add it to your WM config so it launches when you log in. Without this, you won't see your status bar unless you manually run '{}'.", bar_name, bar_name),
                action: format!("Add {} to WM autostart configuration", bar_name),
                command: Some(format!("pgrep {} || echo 'Not running'", bar_name)), // Check if bar is running
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec![
                    "https://wiki.archlinux.org/title/Sway#Autostart".to_string(),
                    "https://wiki.archlinux.org/title/I3#Autostart".to_string(),
                ],
                depends_on: Vec::new(),
                related_to: vec!["install-waybar".to_string(), "install-polybar".to_string()],
                bundle: None,
            satisfies: Vec::new(),
                popularity: 75,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_gaming_setup() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Steam is installed
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam {
        // Check if multilib repository is enabled
        if let Ok(pacman_conf) = std::fs::read_to_string("/etc/pacman.conf") {
            let multilib_enabled = pacman_conf
                .lines()
                .any(|l| l.trim() == "[multilib]" && !l.trim().starts_with("#"));

            if !multilib_enabled {
                result.push(Advice {
                    id: "multilib-repo".to_string(),
                    title: "Enable multilib repository for gaming".to_string(),
                    reason: "You have Steam installed, but the multilib repository isn't enabled. Many games need 32-bit libraries (lib32) to run properly. Without multilib, some games just won't work!".to_string(),
                    action: "Enable the multilib repository in pacman.conf".to_string(),
                    command: Some("sed -i '/\\[multilib\\]/,/Include/s/^#//' /etc/pacman.conf && pacman -Sy".to_string()),
                    risk: RiskLevel::Low,
                    priority: Priority::Recommended,
                    category: "Gaming & Entertainment".to_string(),
                    alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Official_repositories#multilib".to_string()],
                                depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
            }
        }

        // Check for gamemode
        let has_gamemode = Command::new("pacman")
            .args(&["-Q", "gamemode"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gamemode {
            result.push(Advice {
                id: "gamemode".to_string(),
                title: "Install GameMode for better gaming performance".to_string(),
                reason: "GameMode temporarily optimizes your system for gaming by adjusting CPU governor, I/O priority, and other settings. It can give you a noticeable FPS boost in games. Most modern games support it automatically!".to_string(),
                action: "Install gamemode to optimize gaming performance".to_string(),
                command: Some("pacman -S --noconfirm gamemode lib32-gamemode".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Gaming & Entertainment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamemode".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for mangohud
        let has_mangohud = Command::new("pacman")
            .args(&["-Q", "mangohud"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mangohud {
            result.push(Advice {
                id: "mangohud".to_string(),
                title: "Install MangoHud for in-game performance overlay".to_string(),
                reason: "MangoHud shows FPS, CPU/GPU usage, temperatures, and more right in your games. It's super helpful for monitoring performance and looks really cool! Works with Vulkan and OpenGL games.".to_string(),
                action: "Install mangohud for gaming metrics overlay".to_string(),
                command: Some("pacman -S --noconfirm mangohud lib32-mangohud".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Gaming & Entertainment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MangoHud".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for gamescope (Steam Deck compositor)
        let has_gamescope = Command::new("pacman")
            .args(&["-Q", "gamescope"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gamescope {
            result.push(Advice {
                id: "gamescope".to_string(),
                title: "Install Gamescope for better game compatibility".to_string(),
                reason: "Gamescope is the compositor used by Steam Deck. It can help with games that have resolution or window mode issues, and lets you run games in a contained environment with custom resolutions and upscaling.".to_string(),
                action: "Install gamescope for advanced gaming features".to_string(),
                command: Some("pacman -S --noconfirm gamescope".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Gaming & Entertainment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamescope".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for Lutris (GOG, Epic, etc.)
    let has_lutris = Command::new("pacman")
        .args(&["-Q", "lutris"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam && !has_lutris {
        result.push(Advice {
            id: "lutris".to_string(),
            title: "Install Lutris for non-Steam games".to_string(),
            reason: "You have Steam, but if you also play games from GOG, Epic Games Store, or want to run Windows games outside of Steam, Lutris makes it super easy. It handles Wine configuration and has installers for tons of games.".to_string(),
            action: "Install Lutris for managing all your games".to_string(),
            command: Some("pacman -S --noconfirm lutris wine-staging".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Gaming & Entertainment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Lutris".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_desktop_environment(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Use desktop environment from SystemFacts (more reliable than env vars when running as root)
    let desktop_env = facts
        .desktop_environment
        .as_ref()
        .map(|de| de.to_lowercase())
        .unwrap_or_default();

    // Detect display server from SystemFacts
    let display_server = facts
        .display_server
        .as_ref()
        .map(|ds| ds.to_lowercase())
        .unwrap_or_default();
    let wayland_session = display_server.contains("wayland");

    // Check for GNOME
    if desktop_env.contains("gnome") {
        // GNOME-specific recommendations
        let has_extension_manager = Command::new("pacman")
            .args(&["-Q", "gnome-shell-extensions"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_extension_manager {
            result.push(Advice {
                id: "gnome-extensions".to_string(),
                title: "Install GNOME Extensions support".to_string(),
                reason: "GNOME Extensions let you customize your desktop with features like system monitors, clipboard managers, and window tiling. They're like apps for your desktop environment - you can add the features you want!".to_string(),
                action: "Install GNOME Shell extensions support".to_string(),
                command: Some("pacman -S --noconfirm gnome-shell-extensions".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME#Extensions".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for GNOME Tweaks
        let has_tweaks = Command::new("pacman")
            .args(&["-Q", "gnome-tweaks"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_tweaks {
            result.push(Advice {
                id: "gnome-tweaks".to_string(),
                title: "Install GNOME Tweaks for more customization".to_string(),
                reason: "GNOME Tweaks gives you access to tons of settings that aren't in the default settings app. Change fonts, themes, window behavior, and more. It's essential for making GNOME truly yours!".to_string(),
                action: "Install GNOME Tweaks".to_string(),
                command: Some("pacman -S --noconfirm gnome-tweaks".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME#Customization".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for KDE Plasma
    if desktop_env.contains("kde") || desktop_env.contains("plasma") {
        // Check for KDE applications
        let has_dolphin = Command::new("pacman")
            .args(&["-Q", "dolphin"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_dolphin {
            result.push(Advice {
                id: "kde-dolphin".to_string(),
                title: "Install Dolphin file manager".to_string(),
                reason: "Dolphin is KDE's powerful file manager. It has tabs, split views, terminal integration, and tons of features. If you're using KDE, Dolphin makes file management a breeze!".to_string(),
                action: "Install Dolphin file manager".to_string(),
                command: Some("pacman -S --noconfirm dolphin".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/KDE#Dolphin".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Konsole
        let has_konsole = Command::new("pacman")
            .args(&["-Q", "konsole"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_konsole {
            result.push(Advice {
                id: "kde-konsole".to_string(),
                title: "Install Konsole terminal emulator".to_string(),
                reason: "Konsole is KDE's feature-rich terminal. It integrates beautifully with Plasma, supports tabs and splits, and has great customization options. Perfect for KDE users!".to_string(),
                action: "Install Konsole".to_string(),
                command: Some("pacman -S --noconfirm konsole".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Konsole".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for i3 window manager
    let has_i3 = Command::new("pacman")
        .args(&["-Q", "i3-wm"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || desktop_env.contains("i3");

    if has_i3 {
        // Check for i3status
        let has_i3status = Command::new("pacman")
            .args(&["-Q", "i3status"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_i3status {
            result.push(Advice {
                id: "i3-status".to_string(),
                title: "Install i3status or alternative status bar".to_string(),
                reason: "i3 doesn't show system info by default. i3status gives you a status bar with battery, network, time, and more. Or try i3blocks or polybar for even more customization!".to_string(),
                action: "Install i3status for system information".to_string(),
                command: Some("pacman -S --noconfirm i3status".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/I3#i3status".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for rofi (application launcher)
        let has_rofi = Command::new("pacman")
            .args(&["-Q", "rofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_rofi {
            result.push(Advice {
                id: "i3-rofi".to_string(),
                title: "Install Rofi application launcher".to_string(),
                reason: "Rofi is a beautiful, fast app launcher that's way better than dmenu. It can launch apps, switch windows, and even run custom scripts. It's a must-have for i3 users!".to_string(),
                action: "Install Rofi".to_string(),
                command: Some("pacman -S --noconfirm rofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Rofi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for Hyprland (Wayland compositor)
    let has_hyprland = Command::new("pacman")
        .args(&["-Q", "hyprland"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || desktop_env.contains("hyprland");

    if has_hyprland {
        // Check for waybar
        let has_waybar = Command::new("pacman")
            .args(&["-Q", "waybar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_waybar {
            result.push(Advice {
                id: "hyprland-waybar".to_string(),
                title: "Install a status bar for Hyprland".to_string(),
                reason: "A status bar shows workspaces, system info, network status, and more. You have several great options to choose from!".to_string(),
                action: "Install Waybar (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm waybar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Waybar".to_string(),
                        description: "Most popular, highly customizable with JSON/CSS config, excellent Wayland support".to_string(),
                        install_command: "pacman -S --noconfirm waybar".to_string(),
                    },
                    Alternative {
                        name: "eww".to_string(),
                        description: "Widget system with custom Lisp-like language, extremely flexible, perfect for unique setups".to_string(),
                        install_command: "yay -S --noconfirm eww".to_string(),
                    },
                    Alternative {
                        name: "yambar".to_string(),
                        description: "Lightweight, minimal, YAML-based config, great for performance-focused builds".to_string(),
                        install_command: "pacman -S --noconfirm yambar".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Waybar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for wofi (Wayland rofi alternative)
        let has_wofi = Command::new("pacman")
            .args(&["-Q", "wofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wofi {
            result.push(Advice {
                id: "hyprland-wofi".to_string(),
                title: "Install an application launcher for Wayland".to_string(),
                reason: "An app launcher lets you quickly find and launch applications with a keystroke - way faster than hunting through menus!".to_string(),
                action: "Install Wofi (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm wofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Wofi".to_string(),
                        description: "Simple, fast, native Wayland launcher with customizable CSS styling".to_string(),
                        install_command: "pacman -S --noconfirm wofi".to_string(),
                    },
                    Alternative {
                        name: "Rofi (Wayland fork)".to_string(),
                        description: "The classic, feature-rich launcher with plugins, now with Wayland support".to_string(),
                        install_command: "yay -S --noconfirm rofi-lbonn-wayland-git".to_string(),
                    },
                    Alternative {
                        name: "Fuzzel".to_string(),
                        description: "Minimal, keyboard-driven launcher inspired by dmenu, very lightweight".to_string(),
                        install_command: "pacman -S --noconfirm fuzzel".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wofi".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for mako (notification daemon)
        let has_mako = Command::new("pacman")
            .args(&["-Q", "mako"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mako {
            result.push(Advice {
                id: "hyprland-mako".to_string(),
                title: "Install a notification daemon for Wayland".to_string(),
                reason: "Wayland compositors need a notification daemon to show desktop notifications (battery warnings, app alerts, etc.).".to_string(),
                action: "Install Mako (recommended) or choose an alternative".to_string(),
                command: Some("pacman -S --noconfirm mako".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: vec![
                    Alternative {
                        name: "Mako".to_string(),
                        description: "Lightweight, minimal, perfect for Wayland, simple config".to_string(),
                        install_command: "pacman -S --noconfirm mako".to_string(),
                    },
                    Alternative {
                        name: "Dunst".to_string(),
                        description: "Highly customizable, works on both X11 and Wayland, rich configuration options".to_string(),
                        install_command: "pacman -S --noconfirm dunst".to_string(),
                    },
                    Alternative {
                        name: "SwayNC".to_string(),
                        description: "Notification center with history, control center UI, feature-rich".to_string(),
                        install_command: "yay -S --noconfirm swaync".to_string(),
                    },
                ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/Desktop_notifications#Mako".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for Sway (i3-compatible Wayland compositor)
    let has_sway = Command::new("pacman")
        .args(&["-Q", "sway"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        || desktop_env.contains("sway");

    if has_sway {
        // Waybar for sway too
        let has_waybar = Command::new("pacman")
            .args(&["-Q", "waybar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_waybar {
            result.push(Advice {
                id: "sway-waybar".to_string(),
                title: "Install Waybar for Sway".to_string(),
                reason: "Sway works great with Waybar for a status bar. It's highly customizable and shows all your system info beautifully. Way better than the default swaybar!".to_string(),
                action: "Install Waybar".to_string(),
                command: Some("pacman -S --noconfirm waybar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Sway#Waybar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Wofi for sway
        let has_wofi = Command::new("pacman")
            .args(&["-Q", "wofi"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_wofi {
            result.push(Advice {
                id: "sway-wofi".to_string(),
                title: "Install Wofi launcher for Sway".to_string(),
                reason: "Wofi is perfect for Sway - it's a Wayland-native app launcher that integrates beautifully. Much more modern than dmenu!".to_string(),
                action: "Install Wofi".to_string(),
                command: Some("pacman -S --noconfirm wofi".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Sway#Application_launchers".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Display server recommendations
    if wayland_session {
        // Check for XWayland (for running X11 apps on Wayland)
        let has_xwayland = Command::new("pacman")
            .args(&["-Q", "xorg-xwayland"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_xwayland {
            result.push(Advice {
                id: "xwayland".to_string(),
                title: "Install XWayland for X11 app compatibility".to_string(),
                reason: "You're running Wayland, but many apps still use X11. XWayland lets you run those X11 apps on your Wayland session - best of both worlds! Without it, some apps might not work at all.".to_string(),
                action: "Install XWayland".to_string(),
                command: Some("pacman -S --noconfirm xorg-xwayland".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wayland#XWayland".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for Cinnamon desktop environment
    if desktop_env.contains("cinnamon") {
        // Check for Nemo file manager (Cinnamon's default)
        let has_nemo = Command::new("pacman")
            .args(&["-Q", "nemo"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_nemo {
            result.push(Advice {
                id: "cinnamon-nemo".to_string(),
                title: "Install Nemo file manager".to_string(),
                reason: "Nemo is Cinnamon's official file manager. It's a fork of Nautilus with extra features like dual pane view, better customization, and Cinnamon-specific integrations. Essential for the full Cinnamon experience!".to_string(),
                action: "Install Nemo file manager".to_string(),
                command: Some("pacman -S --noconfirm nemo".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Cinnamon#File_manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for GNOME Terminal (commonly used with Cinnamon)
        let has_gnome_terminal = Command::new("pacman")
            .args(&["-Q", "gnome-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_gnome_terminal {
            result.push(Advice {
                id: "cinnamon-terminal".to_string(),
                title: "Install GNOME Terminal for Cinnamon".to_string(),
                reason: "GNOME Terminal is the recommended terminal for Cinnamon. It integrates well with the desktop, supports tabs, profiles, and has good keyboard shortcut support.".to_string(),
                action: "Install GNOME Terminal".to_string(),
                command: Some("pacman -S --noconfirm gnome-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/GNOME/Tips_and_tricks#Terminal".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for Cinnamon screensaver
        let has_screensaver = Command::new("pacman")
            .args(&["-Q", "cinnamon-screensaver"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_screensaver {
            result.push(Advice {
                id: "cinnamon-screensaver".to_string(),
                title: "Install Cinnamon screensaver".to_string(),
                reason: "Cinnamon's screensaver provides screen locking and power saving features. It's important for security (locks your screen when you're away) and extends your monitor's life.".to_string(),
                action: "Install Cinnamon screensaver".to_string(),
                command: Some("pacman -S --noconfirm cinnamon-screensaver".to_string()),
                risk: RiskLevel::Medium,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Cinnamon".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for XFCE desktop environment
    if desktop_env.contains("xfce") {
        // Check for Thunar file manager
        let has_thunar = Command::new("pacman")
            .args(&["-Q", "thunar"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_thunar {
            result.push(Advice {
                id: "xfce-thunar".to_string(),
                title: "Install Thunar file manager".to_string(),
                reason: "Thunar is XFCE's official file manager. It's fast, lightweight, and has great plugin support for bulk renaming, custom actions, and archive management. Perfect for XFCE's philosophy of being light but powerful!".to_string(),
                action: "Install Thunar file manager".to_string(),
                command: Some("pacman -S --noconfirm thunar".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Thunar".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for xfce4-terminal
        let has_xfce_terminal = Command::new("pacman")
            .args(&["-Q", "xfce4-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_xfce_terminal {
            result.push(Advice {
                id: "xfce-terminal".to_string(),
                title: "Install xfce4-terminal".to_string(),
                reason: "xfce4-terminal is XFCE's native terminal emulator. It's lightweight, has dropdown mode support, and integrates perfectly with XFCE. Much better than using xterm!".to_string(),
                action: "Install xfce4-terminal".to_string(),
                command: Some("pacman -S --noconfirm xfce4-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xfce#Terminal".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for xfce4-goodies (collection of plugins and extras)
        let has_goodies = Command::new("pacman")
            .args(&["-Q", "xfce4-goodies"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_goodies {
            result.push(Advice {
                id: "xfce-goodies".to_string(),
                title: "Install xfce4-goodies collection".to_string(),
                reason: "XFCE Goodies includes tons of useful plugins: panel plugins for weather, system monitoring, CPU graphs, battery indicators, and more. Think of it as the 'complete XFCE experience' package that makes your desktop actually useful!".to_string(),
                action: "Install xfce4-goodies".to_string(),
                command: Some("pacman -S --noconfirm xfce4-goodies".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Xfce#Extras".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for MATE desktop environment
    if desktop_env.contains("mate") {
        // Check for Caja file manager
        let has_caja = Command::new("pacman")
            .args(&["-Q", "caja"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_caja {
            result.push(Advice {
                id: "mate-caja".to_string(),
                title: "Install Caja file manager".to_string(),
                reason: "Caja is MATE's official file manager (a fork of the classic GNOME 2 Nautilus). It's reliable, well-tested, and has all the features you need: tabs, bookmarks, extensions, and spatial mode. It's the authentic MATE experience!".to_string(),
                action: "Install Caja file manager".to_string(),
                command: Some("pacman -S --noconfirm caja".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE#File_manager".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for MATE Terminal
        let has_mate_terminal = Command::new("pacman")
            .args(&["-Q", "mate-terminal"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mate_terminal {
            result.push(Advice {
                id: "mate-terminal".to_string(),
                title: "Install MATE Terminal".to_string(),
                reason: "MATE Terminal is the official terminal for MATE desktop. It's based on the classic GNOME 2 terminal with modern improvements. Supports tabs, profiles, transparency, and integrates perfectly with MATE.".to_string(),
                action: "Install MATE Terminal".to_string(),
                command: Some("pacman -S --noconfirm mate-terminal".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }

        // Check for MATE utilities
        let has_mate_utils = Command::new("pacman")
            .args(&["-Q", "mate-utils"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mate_utils {
            result.push(Advice {
                id: "mate-utils".to_string(),
                title: "Install MATE utilities collection".to_string(),
                reason: "MATE Utils includes essential desktop tools: screenshot utility, search tool, dictionary, system log viewer, and disk usage analyzer. These are the 'everyday tools' that make MATE actually productive!".to_string(),
                action: "Install MATE utilities".to_string(),
                command: Some("pacman -S --noconfirm mate-utils".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Recommended,
                category: "Desktop Environment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MATE".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_terminal_and_fonts() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for modern terminal emulators
    let has_alacritty = Command::new("pacman")
        .args(&["-Q", "alacritty"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_kitty = Command::new("pacman")
        .args(&["-Q", "kitty"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_wezterm = Command::new("pacman")
        .args(&["-Q", "wezterm"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Only suggest if they don't have any modern terminal
    if !has_alacritty && !has_kitty && !has_wezterm {
        result.push(Advice {
            id: "modern-terminal".to_string(),
            title: "Upgrade to a GPU-accelerated terminal emulator".to_string(),
            reason: "Modern terminals use your GPU for rendering, making them incredibly fast and smooth. They support true color, ligatures, and can handle massive outputs without lag.".to_string(),
            action: "Install Alacritty (recommended) or choose an alternative".to_string(),
            command: Some("pacman -S --noconfirm alacritty".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Desktop Customization".to_string(),
            alternatives: vec![
                Alternative {
                    name: "Alacritty".to_string(),
                    description: "Blazingly fast, minimal config (TOML), extremely lightweight, best performance".to_string(),
                    install_command: "pacman -S --noconfirm alacritty".to_string(),
                },
                Alternative {
                    name: "Kitty".to_string(),
                    description: "Feature-rich with tabs, splits, images support, Lua scripting, great for power users".to_string(),
                    install_command: "pacman -S --noconfirm kitty".to_string(),
                },
                Alternative {
                    name: "WezTerm".to_string(),
                    description: "Modern with multiplexing, cross-platform, extensive Lua config, built-in tabs".to_string(),
                    install_command: "pacman -S --noconfirm wezterm".to_string(),
                },
            ],
                wiki_refs: vec!["https://wiki.archlinux.org/title/List_of_applications#Terminal_emulators".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for nerd fonts (for powerline, starship, etc.)
    let has_nerd_fonts = Command::new("pacman")
        .args(&["-Ss", "ttf-nerd-fonts"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("[installed]"))
        .unwrap_or(false);

    if !has_nerd_fonts {
        result.push(Advice {
            id: "nerd-fonts".to_string(),
            title: "Install Nerd Fonts for beautiful terminal icons".to_string(),
            reason: "Nerd Fonts include thousands of glyphs and icons that make your terminal look amazing. They're essential for Starship prompt, file managers like lsd/eza, and many terminal apps. Without them, you'll see broken squares instead of cool icons!".to_string(),
            action: "Install Nerd Fonts".to_string(),
            command: Some("pacman -S --noconfirm ttf-nerd-fonts-symbols-mono".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Desktop Customization".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Fonts#Patched_fonts".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_audio_system(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    // Use new audio telemetry
    let audio_system = facts.audio_system.as_deref().unwrap_or("Unknown");
    let audio_running = facts.audio_server_running;
    let session_manager = facts.pipewire_session_manager.as_deref();

    // Recommend PipeWire upgrade if still on PulseAudio
    if audio_system.contains("PulseAudio") && !audio_system.contains("(not running)") {
        result.push(
            Advice::new(
                "upgrade-to-pipewire".to_string(),
                "Upgrade to PipeWire audio system".to_string(),
                "PipeWire is the modern replacement for PulseAudio with lower latency (as low as 5ms), better Bluetooth codec support (AAC, aptX, LDAC), simultaneous JACK and PulseAudio compatibility, and professional-grade audio/video routing capabilities. Most major distributions have switched to PipeWire as the default.".to_string(),
                "Install PipeWire and session manager".to_string(),
                Some("sudo pacman -S --noconfirm pipewire pipewire-pulse pipewire-alsa pipewire-jack wireplumber && systemctl --user enable --now pipewire pipewire-pulse wireplumber".to_string()),
                RiskLevel::Medium,
                Priority::Recommended,
                vec![
                    "https://wiki.archlinux.org/title/PipeWire".to_string(),
                    "https://wiki.archlinux.org/title/PipeWire#Migrating_from_PulseAudio".to_string(),
                ],
                "multimedia".to_string(),
            )
            .with_popularity(75)
        );
    }

    // Check for PipeWire without session manager
    if audio_system.contains("PipeWire") && session_manager.is_none() {
        result.push(
            Advice::new(
                "install-wireplumber".to_string(),
                "Install WirePlumber session manager for PipeWire".to_string(),
                "PipeWire requires a session manager to handle device routing and policy. WirePlumber is the modern, Lua-based session manager that replaced pipewire-media-session. Without it, your audio may not work correctly!".to_string(),
                "Install WirePlumber and enable the service".to_string(),
                Some("sudo pacman -S --noconfirm wireplumber && systemctl --user enable --now wireplumber".to_string()),
                RiskLevel::Low,
                Priority::Mandatory,
                vec![
                    "https://wiki.archlinux.org/title/PipeWire#WirePlumber".to_string(),
                ],
                "multimedia".to_string(),
            )
            .with_popularity(80)
        );
    }

    // Warn about legacy pipewire-media-session
    if let Some(manager) = session_manager {
        if manager.contains("pipewire-media-session") {
            result.push(
                Advice::new(
                    "upgrade-to-wireplumber".to_string(),
                    "Upgrade to WirePlumber from pipewire-media-session".to_string(),
                    "pipewire-media-session is deprecated and no longer maintained. WirePlumber is the official replacement with better device handling, policy management, and Bluetooth support.".to_string(),
                    "Replace pipewire-media-session with WirePlumber".to_string(),
                    Some("sudo pacman -S --noconfirm wireplumber && systemctl --user disable --now pipewire-media-session && systemctl --user enable --now wireplumber".to_string()),
                    RiskLevel::Low,
                    Priority::Recommended,
                    vec![
                        "https://wiki.archlinux.org/title/PipeWire#WirePlumber".to_string(),
                    ],
                    "multimedia".to_string(),
                )
                .with_popularity(70)
            );
        }
    }

    // Check for audio server not running
    if !audio_running && audio_system != "ALSA" {
        result.push(
            Advice::new(
                "start-audio-server".to_string(),
                format!("{} audio server is not running", audio_system),
                format!("Your {} is installed but not currently running. This means applications won't be able to play audio. The audio server should start automatically when you log in.", audio_system),
                "Start the audio server".to_string(),
                if audio_system.contains("PipeWire") {
                    Some("systemctl --user start pipewire pipewire-pulse wireplumber".to_string())
                } else {
                    Some("systemctl --user start pulseaudio".to_string())
                },
                RiskLevel::Low,
                Priority::Recommended,
                vec![
                    if audio_system.contains("PipeWire") {
                        "https://wiki.archlinux.org/title/PipeWire#Installation"
                    } else {
                        "https://wiki.archlinux.org/title/PulseAudio"
                    }.to_string(),
                ],
                "multimedia".to_string(),
            )
            .with_popularity(85)
        );
    }

    // Suggest pavucontrol for GUI volume control
    if audio_system.contains("PipeWire") || audio_system.contains("PulseAudio") {
        if !is_package_installed("pavucontrol") {
            result.push(
                Advice::new(
                    "install-pavucontrol".to_string(),
                    "Install pavucontrol for GUI audio management".to_string(),
                    "pavucontrol (PulseAudio Volume Control) provides a graphical interface to manage audio devices, application volumes, and audio routing. It works perfectly with both PipeWire and PulseAudio, making it much easier than command-line tools.".to_string(),
                    "Install pavucontrol".to_string(),
                    Some("sudo pacman -S --noconfirm pavucontrol".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/PulseAudio#Front-ends".to_string(),
                    ],
                    "multimedia".to_string(),
                )
                .with_popularity(65)
            );
        }
    }

    // Suggest Bluetooth audio codecs for PipeWire
    if audio_system.contains("PipeWire") && facts.bluetooth_status.available {
        if !is_package_installed("libldac") {
            result.push(
                Advice::new(
                    "install-bluetooth-codecs".to_string(),
                    "Install high-quality Bluetooth audio codecs".to_string(),
                    "PipeWire supports advanced Bluetooth codecs like LDAC (Sony Hi-Res), aptX, and AAC for much better wireless audio quality than standard SBC. Installing codec libraries enables these automatically.".to_string(),
                    "Install Bluetooth audio codec libraries".to_string(),
                    Some("sudo pacman -S --noconfirm libldac libfreeaptx".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/Bluetooth_headset#Bluetooth_audio_codecs".to_string(),
                        "https://wiki.archlinux.org/title/PipeWire#Bluetooth_devices".to_string(),
                    ],
                    "multimedia".to_string(),
                )
                .with_popularity(60)
            );
        }
    }

    result
}

pub(crate) fn check_gamepad_support() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for gamepad support packages
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check if any gamepad-related packages are installed
    let has_xpadneo = Command::new("pacman")
        .args(&["-Q", "xpadneo-dkms"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_xone = Command::new("pacman")
        .args(&["-Q", "xone-dkms"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // If user has Steam but no Xbox controller drivers
    if has_steam && !has_xpadneo && !has_xone {
        result.push(Advice {
            id: "gamepad-xbox".to_string(),
            title: "Install Xbox controller drivers for better support".to_string(),
            reason: "If you use Xbox controllers (especially Xbox One/Series), the default kernel drivers have limited functionality. xpadneo or xone give you full support - battery level, rumble, proper button mapping, and better wireless connectivity!".to_string(),
            action: "Install xpadneo for Xbox controller support".to_string(),
            command: Some("paru -S --noconfirm xpadneo-dkms".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Gaming & Entertainment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamepad".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for antimicrox (gamepad to keyboard/mouse mapping)
    let has_antimicrox = Command::new("pacman")
        .args(&["-Q", "antimicrox"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam && !has_antimicrox {
        result.push(Advice {
            id: "gamepad-antimicrox".to_string(),
            title: "Install AntiMicroX for gamepad mapping".to_string(),
            reason: "AntiMicroX lets you map gamepad buttons to keyboard and mouse actions. Super useful for games without native controller support, or for using your controller outside of games!".to_string(),
            action: "Install AntiMicroX".to_string(),
            command: Some("pacman -S --noconfirm antimicrox".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Gaming & Entertainment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Gamepad#Button_mapping".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_audio_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if PipeWire or PulseAudio is running
    let has_pipewire = Command::new("systemctl")
        .args(&["--user", "is-active", "pipewire"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let has_pulseaudio = Command::new("systemctl")
        .args(&["--user", "is-active", "pulseaudio"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_pipewire && !has_pulseaudio {
        return result; // No audio server detected
    }

    // Check for EasyEffects (PipeWire) or PulseEffects (PulseAudio)
    let has_easyeffects = Command::new("pacman")
        .args(&["-Q", "easyeffects"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_pipewire && !has_easyeffects {
        result.push(Advice {
            id: "audio-easyeffects".to_string(),
            title: "Install EasyEffects for audio enhancement".to_string(),
            reason: "You're using PipeWire but missing EasyEffects! It's an amazing audio processor that can add bass boost, equalizer, noise reduction, reverb, and more. Make cheap headphones sound expensive, improve microphone quality, or just make everything sound better. It's like having a professional audio engineer built in!".to_string(),
            action: "Install EasyEffects".to_string(),
            command: Some("pacman -S --noconfirm easyeffects".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PipeWire#EasyEffects".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for pavucontrol (volume control GUI)
    let has_pavucontrol = Command::new("pacman")
        .args(&["-Q", "pavucontrol"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_pavucontrol {
        result.push(Advice {
            id: "audio-pavucontrol".to_string(),
            title: "Install pavucontrol for advanced volume control".to_string(),
            reason: "You have no GUI volume mixer! pavucontrol lets you control volume per-application, switch audio devices, adjust balance, and manage recording sources. Way better than basic volume controls - you can route Discord to headphones while music plays on speakers!".to_string(),
            action: "Install pavucontrol".to_string(),
            command: Some("pacman -S --noconfirm pavucontrol".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Multimedia & Graphics".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/PulseAudio#pavucontrol".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_gaming_enhancements() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check if Steam is installed
    let has_steam = Command::new("pacman")
        .args(&["-Q", "steam"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if has_steam {
        // Check for Proton-GE (better Windows game compatibility)
        result.push(Advice {
            id: "gaming-proton-ge".to_string(),
            title: "Consider Proton-GE for better game compatibility".to_string(),
            reason: "You have Steam! Proton-GE is a community version of Proton with extra patches, better codec support, and fixes for specific games. Many Windows games run better on Proton-GE than stock Proton! Install via ProtonUp-Qt for easy management.".to_string(),
            action: "Install ProtonUp-Qt to manage Proton-GE".to_string(),
            command: Some("pacman -S --noconfirm protonup-qt".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Gaming & Entertainment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Steam#Proton".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });

        // Check for MangoHud (performance overlay)
        let has_mangohud = Command::new("pacman")
            .args(&["-Q", "mangohud"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_mangohud {
            result.push(Advice {
                id: "gaming-mangohud".to_string(),
                title: "Install MangoHud for in-game performance overlay".to_string(),
                reason: "MangoHud shows FPS, CPU/GPU usage, temps, and more while gaming! It's like MSI Afterburner for Linux - see exactly how your games perform. Launch games with 'mangohud %command%' in Steam to enable it. Essential for PC gamers!".to_string(),
                action: "Install MangoHud".to_string(),
                command: Some("pacman -S --noconfirm mangohud".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Gaming & Entertainment".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/MangoHud".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    // Check for Wine
    let has_wine = Command::new("pacman")
        .args(&["-Q", "wine"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_wine && !has_steam {
        result.push(Advice {
            id: "gaming-wine".to_string(),
            title: "Install Wine to run Windows applications".to_string(),
            reason: "Wine lets you run Windows programs on Linux! Great for games, old software, or Windows-only apps. For gaming, Steam's Proton is easier, but Wine works for non-Steam games and applications. 'wine program.exe' is all you need!".to_string(),
            action: "Install Wine".to_string(),
            command: Some("pacman -S --noconfirm wine".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Optional,
            category: "Gaming & Entertainment".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Wine".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    // Check for Discord (gaming communication)
    if !is_package_installed("discord") && has_steam {
        result.push(
            Advice::new(
                "gaming-discord".to_string(),
                "Install Discord for gaming communication".to_string(),
                "You're a gamer with Steam! Discord is THE voice chat and community platform for gamers. Screen sharing, low-latency voice chat, game integration, communities - it's essential for multiplayer gaming.".to_string(),
                "Install Discord".to_string(),
                Some("sudo pacman -S --noconfirm discord".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Discord".to_string(),
                ],
                "gaming".to_string(),
            )
            .with_popularity(75)
            .with_bundle("gaming-essentials".to_string())
        );
    }

    // Check for game controllers/input
    if has_steam && !is_package_installed("xboxdrv") && !is_package_installed("ds4drv") {
        result.push(
            Advice::new(
                "gaming-controller-support".to_string(),
                "Install controller drivers for gaming".to_string(),
                "Enhance controller support beyond Steam Input! xboxdrv handles Xbox controllers, ds4drv for PS4/PS5 controllers. Needed for non-Steam games, better compatibility, and custom button mappings. Many games expect Xbox controller layout!".to_string(),
                "Install controller drivers".to_string(),
                Some("sudo pacman -S --noconfirm xboxdrv".to_string()),
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://wiki.archlinux.org/title/Gamepad".to_string(),
                    "https://wiki.archlinux.org/title/Gamepad#PlayStation_controllers".to_string(),
                ],
                "gaming".to_string(),
            )
            .with_popularity(60)
            .with_bundle("gaming-essentials".to_string())
        );
    }

    // Check for emulators if user seems to be into gaming
    if has_steam {
        // Check for RetroArch (all-in-one emulator)
        if !is_package_installed("retroarch") {
            result.push(
                Advice::new(
                    "gaming-retroarch".to_string(),
                    "Install RetroArch for retro gaming emulation".to_string(),
                    "RetroArch is the ultimate all-in-one emulator - NES, SNES, PlayStation, N64, Game Boy, Sega, and more! Beautiful interface, shaders, netplay, achievements. Play your classic game collection on Linux!".to_string(),
                    "Install RetroArch emulator".to_string(),
                    Some("sudo pacman -S --noconfirm retroarch".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/RetroArch".to_string(),
                    ],
                    "gaming".to_string(),
                )
                .with_popularity(55)
            );
        }

        // Check for PCSX2 (PlayStation 2)
        if !is_package_installed("pcsx2") {
            result.push(
                Advice::new(
                    "gaming-pcsx2".to_string(),
                    "Install PCSX2 for PlayStation 2 emulation".to_string(),
                    "PCSX2 is the best PS2 emulator - plays thousands of PS2 games at higher resolutions than original hardware! Supports widescreen patches, texture filtering, and save states. Relive your PS2 library!".to_string(),
                    "Install PCSX2 emulator".to_string(),
                    Some("sudo pacman -S --noconfirm pcsx2".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/PCSX2".to_string(),
                    ],
                    "gaming".to_string(),
                )
                .with_popularity(50)
            );
        }

        // Check for Dolphin (GameCube/Wii)
        if !is_package_installed("dolphin-emu") {
            result.push(
                Advice::new(
                    "gaming-dolphin".to_string(),
                    "Install Dolphin for GameCube/Wii emulation".to_string(),
                    "Dolphin is THE emulator for GameCube and Wii games. Runs games at 1080p/4K, supports Wii remotes, netplay, and enhancement features. Play Mario, Zelda, Metroid, and more with better graphics than original consoles!".to_string(),
                    "Install Dolphin emulator".to_string(),
                    Some("sudo pacman -S --noconfirm dolphin-emu".to_string()),
                    RiskLevel::Low,
                    Priority::Optional,
                    vec![
                        "https://wiki.archlinux.org/title/Dolphin_emulator".to_string(),
                    ],
                    "gaming".to_string(),
                )
                .with_popularity(60)
            );
        }
    }

    // Check for Steam Tinker Launch (advanced Steam tweaking)
    if has_steam
        && !Command::new("which")
            .arg("steamtinkerlaunch")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    {
        result.push(
            Advice::new(
                "gaming-steam-tinker-launch".to_string(),
                "Install Steam Tinker Launch for advanced game tweaking".to_string(),
                "Steam Tinker Launch is a Swiss Army knife for Steam games on Linux! Easily use different Proton versions, enable GameMode/MangoHud, configure Wine settings, use ReShade, and more - all from a GUI. Power user essential!".to_string(),
                "Install Steam Tinker Launch (AUR)".to_string(),
                None, // AUR package
                RiskLevel::Low,
                Priority::Optional,
                vec![
                    "https://github.com/sonic2kk/steamtinkerlaunch".to_string(),
                ],
                "gaming".to_string(),
            )
            .with_popularity(45)
            .with_bundle("gaming-essentials".to_string())
        );
    }

    result
}

pub(crate) fn check_terminal_multiplexers() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for tmux
    let has_tmux = Command::new("pacman")
        .args(&["-Q", "tmux"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !has_tmux {
        result.push(Advice {
            id: "terminal-tmux".to_string(),
            title: "Install tmux for terminal multiplexing".to_string(),
            reason: "tmux is a terminal multiplexer! Split terminals, detach sessions, work across SSH disconnects, multiple windows. Essential for remote work and power users. 'tmux new -s mysession' creates session, Ctrl+b d detaches. Never lose your work again!".to_string(),
            action: "Install tmux".to_string(),
            command: Some("pacman -S --noconfirm tmux".to_string()),
            risk: RiskLevel::Low,
            priority: Priority::Recommended,
            category: "Utilities".to_string(),
            alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Tmux".to_string()],
                        depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
    }

    result
}

pub(crate) fn check_audio_production() -> Vec<Advice> {
    let mut result = Vec::new();

    // Check for audio files
    let has_audio_files = Command::new("find")
        .args(&[
            &format!("{}/Music", std::env::var("HOME").unwrap_or_default()),
            "-type",
            "f",
        ])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    if has_audio_files {
        // Check for Audacity
        let has_audacity = Command::new("pacman")
            .args(&["-Q", "audacity"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !has_audacity {
            result.push(Advice {
                id: "audio-audacity".to_string(),
                title: "Install Audacity for audio editing".to_string(),
                reason: "You have audio files! Audacity is the free audio editor. Record, edit, mix, add effects, export to any format. Perfect for podcasts, music editing, audio cleanup. Simple interface, professional results. The go-to audio editor!".to_string(),
                action: "Install Audacity".to_string(),
                command: Some("pacman -S --noconfirm audacity".to_string()),
                risk: RiskLevel::Low,
                priority: Priority::Optional,
                category: "Multimedia & Graphics".to_string(),
                alternatives: Vec::new(),
                wiki_refs: vec!["https://wiki.archlinux.org/title/Audacity".to_string()],
                            depends_on: Vec::new(),
                related_to: Vec::new(),
                bundle: None,
            satisfies: Vec::new(),
            popularity: 50,
            requires: Vec::new(),
            });
        }
    }

    result
}

pub(crate) fn check_window_manager_recommendations(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.window_manager.as_deref() {
        Some("i3") => {
            if !is_package_installed("rofi") && !is_package_installed("dmenu") {
                result.push(Advice {
                    id: "i3-application-launcher".to_string(),
                    title: "Install Application Launcher for i3".to_string(),
                    reason: "i3 needs an application launcher for quick application access with keyboard shortcuts.".to_string(),
                    action: "Install rofi for modern launcher or dmenu for classic".to_string(),
                    command: Some("sudo pacman -S --noconfirm rofi".to_string()),
                    priority: Priority::Recommended,
                    risk: RiskLevel::Low,
                    category: "Desktop Environment".to_string(),
                    wiki_refs: vec!["https://wiki.archlinux.org/title/I3".to_string()],
                    alternatives: vec![],
                    depends_on: vec![],
                    related_to: vec![],
                    bundle: Some("i3-setup".to_string()),
            satisfies: Vec::new(),
                    popularity: 80,
            requires: Vec::new(), // Very common for i3 users
                });
            }
        }
        _ => {}
    }

    result
}

pub(crate) fn check_desktop_environment_specific(facts: &SystemFacts) -> Vec<Advice> {
    let mut result = Vec::new();

    match facts.desktop_environment.as_deref() {
        Some("GNOME") => {
            if !is_package_installed("gnome-tweaks") {
                result.push(Advice {
                    id: "gnome-tweaks".to_string(),
                    title: "Install GNOME Tweaks for Customization".to_string(),
                    reason: "GNOME Tweaks provides advanced customization options not available in standard Settings.".to_string(),
                    action: "Install GNOME Tweaks to customize themes, fonts, startup applications, and more".to_string(),
                    command: Some("sudo pacman -S --noconfirm gnome-tweaks".to_string()),
                    priority: Priority::Optional,
                    risk: RiskLevel::Low,
                    category: "Desktop Environment".to_string(),
                    wiki_refs: vec!["https://wiki.gnome.org/Apps/Tweaks".to_string()],
                    alternatives: vec![],
                    depends_on: vec![],
                    related_to: vec![],
                    bundle: Some("gnome-enhancements".to_string()),
            satisfies: Vec::new(),
                    popularity: 85,
            requires: Vec::new(), // Very popular among GNOME users
                });
            }
        }
        _ => {}
    }

    result
}

