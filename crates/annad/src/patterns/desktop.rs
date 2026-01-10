//! Desktop environment patterns
//! v0.0.918: GNOME, KDE, Wayland, X11, and display server queries

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Match desktop environment queries
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    if let Some(u) = match_display_server(q) {
        return Some(u);
    }
    if let Some(u) = match_gnome(q) {
        return Some(u);
    }
    if let Some(u) = match_kde(q) {
        return Some(u);
    }
    if let Some(u) = match_window_manager(q) {
        return Some(u);
    }
    if let Some(u) = match_display(q) {
        return Some(u);
    }
    None
}

/// Pattern with keywords, description, topic, and commands
type DesktopPattern = (&'static [&'static str], &'static str, &'static str, &'static [&'static str]);

fn match_display_server(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // Wayland vs X11
        (&["wayland", "x11"], "display server check", "display",
            &["echo $XDG_SESSION_TYPE", "loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type 2>/dev/null"]),
        (&["which", "display", "server"], "display server check", "display",
            &["echo $XDG_SESSION_TYPE"]),
        (&["running", "wayland"], "Wayland check", "display",
            &["echo $XDG_SESSION_TYPE", "echo $WAYLAND_DISPLAY"]),
        (&["running", "x11"], "X11 check", "display",
            &["echo $XDG_SESSION_TYPE", "echo $DISPLAY"]),
        // Compositor
        (&["compositor"], "compositor info", "display",
            &["echo $XDG_SESSION_TYPE", "pgrep -l 'mutter|kwin|sway|hyprland|weston' 2>/dev/null"]),
        // Session info
        (&["desktop", "session"], "desktop session info", "display",
            &["echo $XDG_CURRENT_DESKTOP", "echo $XDG_SESSION_TYPE", "echo $DESKTOP_SESSION"]),
        (&["which", "desktop"], "desktop environment", "display",
            &["echo $XDG_CURRENT_DESKTOP", "echo $DESKTOP_SESSION"]),
        (&["what", "desktop"], "desktop environment", "display",
            &["echo $XDG_CURRENT_DESKTOP", "echo $DESKTOP_SESSION"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_gnome(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // GNOME version
        (&["gnome", "version"], "GNOME version", "gnome",
            &["gnome-shell --version"]),
        // GNOME settings
        (&["gnome", "setting"], "GNOME settings", "gnome",
            &["gsettings list-schemas | head -20"]),
        (&["gsetting"], "gsettings", "gnome",
            &["echo 'Usage: gsettings get <schema> <key>'",
              "echo 'List: gsettings list-recursively <schema>'"]),
        // GNOME extensions
        (&["gnome", "extension"], "GNOME extensions", "gnome",
            &["gnome-extensions list 2>/dev/null || ls ~/.local/share/gnome-shell/extensions/ 2>/dev/null"]),
        (&["enable", "extension"], "enable GNOME extension", "gnome",
            &["echo 'Usage: gnome-extensions enable <uuid>'"]),
        (&["disable", "extension"], "disable GNOME extension", "gnome",
            &["echo 'Usage: gnome-extensions disable <uuid>'"]),
        // GNOME troubleshooting
        (&["gnome", "restart"], "restart GNOME shell", "gnome",
            &["echo 'X11: Alt+F2 then type r'",
              "echo 'Wayland: Log out and back in (no restart on Wayland)'"]),
        (&["gnome", "crash"], "GNOME crash logs", "gnome",
            &["journalctl -b /usr/bin/gnome-shell | tail -30"]),
        // GTK
        (&["gtk", "theme"], "GTK theme", "gnome",
            &["gsettings get org.gnome.desktop.interface gtk-theme",
              "gsettings get org.gnome.desktop.interface color-scheme"]),
        (&["icon", "theme"], "icon theme", "gnome",
            &["gsettings get org.gnome.desktop.interface icon-theme"]),
        (&["dark", "mode"], "dark mode setting", "gnome",
            &["gsettings get org.gnome.desktop.interface color-scheme",
              "echo 'Set: gsettings set org.gnome.desktop.interface color-scheme prefer-dark'"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_kde(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // KDE/Plasma version
        (&["kde", "version"], "KDE Plasma version", "kde",
            &["plasmashell --version"]),
        (&["plasma", "version"], "Plasma version", "kde",
            &["plasmashell --version"]),
        // KDE settings
        (&["kde", "setting"], "KDE settings", "kde",
            &["echo 'Open: systemsettings5 or kcmshell5'",
              "cat ~/.config/kdeglobals | head -30 2>/dev/null"]),
        (&["kwin"], "KWin compositor", "kde",
            &["qdbus org.kde.KWin /Compositor org.kde.kwin.Compositing.active 2>/dev/null"]),
        // KDE troubleshooting
        (&["plasma", "restart"], "restart Plasma", "kde",
            &["echo 'Restart: kquitapp5 plasmashell && kstart5 plasmashell'"]),
        (&["plasma", "crash"], "Plasma crash logs", "kde",
            &["journalctl -b /usr/bin/plasmashell | tail -30"]),
        (&["kwin", "crash"], "KWin crash", "kde",
            &["journalctl -b /usr/bin/kwin_x11 /usr/bin/kwin_wayland | tail -30"]),
        // Qt theme
        (&["qt", "theme"], "Qt theme", "kde",
            &["echo $QT_QPA_PLATFORMTHEME",
              "cat ~/.config/kdeglobals | grep -i theme | head -10 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_window_manager(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // i3/Sway
        (&["i3", "config"], "i3 configuration", "wm",
            &["cat ~/.config/i3/config 2>/dev/null | head -30"]),
        (&["sway", "config"], "Sway configuration", "wm",
            &["cat ~/.config/sway/config 2>/dev/null | head -30"]),
        (&["i3", "reload"], "reload i3 config", "wm",
            &["echo 'Reload: i3-msg reload'", "echo 'Restart: i3-msg restart'"]),
        (&["sway", "reload"], "reload Sway config", "wm",
            &["echo 'Reload: swaymsg reload'"]),
        // Hyprland
        (&["hyprland", "config"], "Hyprland configuration", "wm",
            &["cat ~/.config/hypr/hyprland.conf 2>/dev/null | head -30"]),
        (&["hyprland", "reload"], "reload Hyprland", "wm",
            &["echo 'Reload: hyprctl reload'"]),
        // Window manager info
        (&["window", "manager"], "window manager info", "wm",
            &["echo $XDG_CURRENT_DESKTOP", "pgrep -l 'i3|sway|hyprland|openbox|bspwm|dwm' 2>/dev/null"]),
        (&["which", "wm"], "which window manager", "wm",
            &["pgrep -l 'i3|sway|hyprland|openbox|bspwm|dwm|kwin|mutter' 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

fn match_display(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // Monitors/displays
        (&["list", "monitor"], "list monitors", "display",
            &["xrandr --query 2>/dev/null | grep -w connected || wlr-randr 2>/dev/null"]),
        (&["connected", "monitor"], "connected monitors", "display",
            &["xrandr --query 2>/dev/null | grep -w connected || wlr-randr 2>/dev/null"]),
        (&["xrandr"], "xrandr display info", "display",
            &["xrandr --query"]),
        // Resolution
        (&["screen", "resolution"], "screen resolution", "display",
            &["xrandr --query 2>/dev/null | grep '*' || wlr-randr 2>/dev/null | grep current"]),
        (&["change", "resolution"], "change resolution", "display",
            &["echo 'X11: xrandr --output <display> --mode <resolution>'",
              "echo 'Wayland: Use Settings or wlr-randr'"]),
        // Refresh rate
        (&["refresh", "rate"], "refresh rate", "display",
            &["xrandr --query 2>/dev/null | grep '*' || wlr-randr 2>/dev/null"]),
        // Multi-monitor
        (&["dual", "monitor"], "dual monitor setup", "display",
            &["xrandr --query 2>/dev/null | grep -w connected",
              "echo 'X11: xrandr --output <display> --right-of <other>'"]),
        (&["extend", "display"], "extend display", "display",
            &["echo 'X11: xrandr --output <display> --right-of <primary>'",
              "echo 'Wayland: Use Settings'"]),
        // DPI/Scaling
        (&["dpi", "setting"], "DPI settings", "display",
            &["xrdb -query 2>/dev/null | grep dpi",
              "gsettings get org.gnome.desktop.interface scaling-factor 2>/dev/null"]),
        (&["hidpi"], "HiDPI settings", "display",
            &["echo $GDK_SCALE", "echo $QT_SCALE_FACTOR",
              "gsettings get org.gnome.desktop.interface scaling-factor 2>/dev/null"]),
    ];

    for (keywords, interpreted, topic, commands) in patterns {
        if keywords.iter().all(|kw| q.contains(kw)) {
            return Some(DeepUnderstanding {
                interpreted_as: interpreted.to_string(),
                category: IntentCategory::Factual,
                confidence: 0.9,
                topic: Some(topic.to_string()),
                needs_confirmation: false,
                suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_server() {
        let result = match_patterns("am i running wayland or x11");
        assert!(result.is_some());
    }

    #[test]
    fn test_gnome_version() {
        let result = match_patterns("gnome version");
        assert!(result.is_some());
    }

    #[test]
    fn test_kde_plasma() {
        let result = match_patterns("plasma version");
        assert!(result.is_some());
    }

    #[test]
    fn test_monitors() {
        let result = match_patterns("list connected monitors");
        assert!(result.is_some());
    }
}
