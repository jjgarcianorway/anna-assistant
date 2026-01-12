//! Desktop environment patterns
//! v0.0.918: GNOME, KDE, Wayland, X11, and display server queries
//! v0.0.989: Added input, screensaver, clipboard, fonts, notifications patterns

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
    if let Some(u) = match_input_devices(q) {
        return Some(u);
    }
    if let Some(u) = match_desktop_utils(q) {
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

fn match_input_devices(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // Input devices listing
        (&["input", "devices"], "list input devices", "input",
            &["xinput list 2>/dev/null || libinput list-devices 2>/dev/null | grep Device"]),
        (&["list", "keyboard"], "list keyboards", "input",
            &["xinput list 2>/dev/null | grep -i keyboard || libinput list-devices 2>/dev/null | grep -A1 Keyboard"]),
        // Mouse/touchpad
        (&["mouse", "setting"], "mouse settings", "input",
            &["xinput list-props $(xinput list | grep -i mouse | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | head -20"]),
        (&["touchpad", "setting"], "touchpad settings", "input",
            &["xinput list-props $(xinput list | grep -i touchpad | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | head -20"]),
        (&["disable", "touchpad"], "disable touchpad", "input",
            &["echo 'X11: xinput disable <touchpad-id>'",
              "echo 'Find ID: xinput list | grep -i touchpad'"]),
        (&["enable", "touchpad"], "enable touchpad", "input",
            &["echo 'X11: xinput enable <touchpad-id>'",
              "echo 'Find ID: xinput list | grep -i touchpad'"]),
        // Keyboard layout
        (&["keyboard", "layout"], "keyboard layout", "input",
            &["setxkbmap -query 2>/dev/null || localectl status | grep -i keymap"]),
        (&["change", "keyboard", "layout"], "change keyboard layout", "input",
            &["echo 'X11: setxkbmap <layout> (e.g., setxkbmap us)'",
              "echo 'Permanent: localectl set-x11-keymap <layout>'"]),
        (&["keyboard", "map"], "keyboard mapping", "input",
            &["xmodmap -pke 2>/dev/null | head -20", "setxkbmap -query 2>/dev/null"]),
        // Mouse speed/accel
        (&["mouse", "speed"], "mouse speed settings", "input",
            &["xinput list-props $(xinput list | grep -i mouse | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | grep -i accel"]),
        (&["mouse", "acceleration"], "mouse acceleration", "input",
            &["xinput list-props $(xinput list | grep -i mouse | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | grep -i accel"]),
        // Scroll direction
        (&["natural", "scrolling"], "natural scrolling setting", "input",
            &["xinput list-props $(xinput list | grep -i touchpad | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | grep -i natural"]),
        // Tap to click
        (&["tap", "click"], "tap to click setting", "input",
            &["xinput list-props $(xinput list | grep -i touchpad | head -1 | sed 's/.*id=\\([0-9]*\\).*/\\1/') 2>/dev/null | grep -i tap"]),
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

fn match_desktop_utils(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DesktopPattern] = &[
        // Screensaver/screen lock
        (&["screensaver", "setting"], "screensaver settings", "desktop",
            &["gsettings get org.gnome.desktop.screensaver lock-enabled 2>/dev/null",
              "gsettings get org.gnome.desktop.session idle-delay 2>/dev/null"]),
        (&["screen", "lock"], "screen lock settings", "desktop",
            &["gsettings get org.gnome.desktop.screensaver lock-enabled 2>/dev/null",
              "echo 'Lock now: loginctl lock-session'"]),
        (&["disable", "screen", "lock"], "disable screen lock", "desktop",
            &["echo 'GNOME: gsettings set org.gnome.desktop.screensaver lock-enabled false'",
              "echo 'KDE: Check System Settings > Screen Locking'"]),
        (&["lock", "timeout"], "screen lock timeout", "desktop",
            &["gsettings get org.gnome.desktop.session idle-delay 2>/dev/null",
              "gsettings get org.gnome.desktop.screensaver lock-delay 2>/dev/null"]),
        // Clipboard
        (&["clipboard", "content"], "clipboard contents", "desktop",
            &["xclip -o -selection clipboard 2>/dev/null || wl-paste 2>/dev/null"]),
        (&["clipboard", "history"], "clipboard history", "desktop",
            &["echo 'Install clipboard manager: parcellite, copyq, or gpaste'"]),
        (&["copy", "clipboard"], "copy to clipboard", "desktop",
            &["echo 'X11: echo text | xclip -selection clipboard'",
              "echo 'Wayland: echo text | wl-copy'"]),
        // Fonts
        (&["installed", "fonts"], "list installed fonts", "desktop",
            &["fc-list | head -30"]),
        (&["system", "fonts"], "list system fonts", "desktop",
            &["fc-list : family | sort -u | head -30"]),
        (&["font", "cache"], "rebuild font cache", "desktop",
            &["echo 'Run: fc-cache -fv'"]),
        (&["font", "config"], "font configuration", "desktop",
            &["cat /etc/fonts/local.conf 2>/dev/null || ls /etc/fonts/conf.d/"]),
        // Notifications
        (&["notification", "setting"], "notification settings", "desktop",
            &["gsettings get org.gnome.desktop.notifications show-banners 2>/dev/null",
              "echo 'Check: notify-send \"test\" for testing'"]),
        (&["do", "not", "disturb"], "do not disturb mode", "desktop",
            &["gsettings get org.gnome.desktop.notifications show-banners 2>/dev/null",
              "echo 'GNOME: Toggle in notification panel'"]),
        (&["test", "notification"], "test notification", "desktop",
            &["notify-send 'Test' 'This is a test notification'"]),
        // Screenshots
        (&["screenshot", "tool"], "screenshot tools", "desktop",
            &["echo 'gnome-screenshot, spectacle (KDE), flameshot, grim (Wayland)'",
              "echo 'Quick: gnome-screenshot -i or flameshot gui'"]),
        (&["take", "screenshot"], "take screenshot", "desktop",
            &["echo 'X11: gnome-screenshot or scrot or flameshot gui'",
              "echo 'Wayland: grim or spectacle'"]),
        // Autostart
        (&["autostart", "apps"], "autostart applications", "desktop",
            &["ls ~/.config/autostart/ 2>/dev/null",
              "ls /etc/xdg/autostart/ 2>/dev/null | head -20"]),
        (&["startup", "applications"], "startup applications", "desktop",
            &["ls ~/.config/autostart/ 2>/dev/null"]),
        (&["add", "autostart"], "add to autostart", "desktop",
            &["echo 'Create .desktop file in ~/.config/autostart/'"]),
        // Default apps
        (&["default", "browser"], "default browser", "desktop",
            &["xdg-settings get default-web-browser"]),
        (&["default", "application"], "default applications", "desktop",
            &["xdg-mime query default text/html",
              "xdg-mime query default application/pdf"]),
        (&["set", "default", "app"], "set default application", "desktop",
            &["echo 'Use: xdg-mime default <app.desktop> <mimetype>'",
              "echo 'Example: xdg-mime default firefox.desktop text/html'"]),
        // Cursors
        (&["cursor", "theme"], "cursor theme", "desktop",
            &["gsettings get org.gnome.desktop.interface cursor-theme 2>/dev/null",
              "echo $XCURSOR_THEME"]),
        (&["cursor", "size"], "cursor size", "desktop",
            &["gsettings get org.gnome.desktop.interface cursor-size 2>/dev/null",
              "echo $XCURSOR_SIZE"]),
        // Display manager
        (&["display", "manager"], "show display manager", "desktop",
            &["cat /etc/systemd/system/display-manager.service 2>/dev/null | grep ExecStart",
              "systemctl status display-manager"]),
        (&["show", "display", "manager"], "display manager info", "desktop",
            &["systemctl status display-manager",
              "echo 'Common: gdm, sddm, lightdm'"]),
        // Installed themes
        (&["installed", "themes"], "list installed themes", "desktop",
            &["ls /usr/share/themes/ 2>/dev/null",
              "ls ~/.themes/ 2>/dev/null",
              "ls ~/.local/share/themes/ 2>/dev/null"]),
        (&["list", "themes"], "list available themes", "desktop",
            &["ls /usr/share/themes/", "ls ~/.themes/ 2>/dev/null"]),
        // Plasma settings
        (&["plasma", "setting"], "KDE Plasma settings", "kde",
            &["echo 'Open: systemsettings5'",
              "cat ~/.config/kdeglobals 2>/dev/null | head -30"]),
        // Desktop shortcuts
        (&["desktop", "shortcut"], "desktop keyboard shortcuts", "desktop",
            &["gsettings list-recursively org.gnome.desktop.wm.keybindings 2>/dev/null | head -20",
              "echo 'KDE: Check System Settings > Shortcuts'"]),
        // Taskbar
        (&["taskbar", "config"], "taskbar configuration", "desktop",
            &["echo 'GNOME: Use gnome-extensions or dconf-editor'",
              "echo 'KDE: Right-click panel > Edit Panel'"]),
        // Desktop icons
        (&["desktop", "icons"], "desktop icons settings", "desktop",
            &["gsettings get org.gnome.desktop.background show-desktop-icons 2>/dev/null",
              "echo 'GNOME 40+: Install desktop-icons-ng extension'"]),
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

    #[test]
    fn test_input_devices() {
        assert!(match_patterns("input devices").is_some());
        assert!(match_patterns("mouse settings").is_some());
        assert!(match_patterns("touchpad settings").is_some());
        assert!(match_patterns("keyboard layout").is_some());
        assert!(match_patterns("disable touchpad").is_some());
        assert!(match_patterns("natural scrolling").is_some());
    }

    #[test]
    fn test_desktop_utils() {
        assert!(match_patterns("screensaver settings").is_some());
        assert!(match_patterns("screen lock").is_some());
        assert!(match_patterns("clipboard contents").is_some());
        assert!(match_patterns("installed fonts").is_some());
        assert!(match_patterns("notification settings").is_some());
        assert!(match_patterns("screenshot tool").is_some());
        assert!(match_patterns("autostart apps").is_some());
        assert!(match_patterns("default browser").is_some());
        assert!(match_patterns("cursor theme").is_some());
    }
}
