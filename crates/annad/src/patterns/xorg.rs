//! Xorg patterns for X server, xrandr, and input configuration.
//! v0.0.988: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create an Xorg-related DeepUnderstanding
fn make_understanding(interpreted: &str, topic: &str, commands: &[&str]) -> DeepUnderstanding {
    DeepUnderstanding {
        interpreted_as: interpreted.to_string(),
        category: IntentCategory::Factual,
        confidence: 0.9,
        topic: Some(topic.to_string()),
        needs_confirmation: false,
        suggested_commands: commands.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

type XorgPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match Xorg patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_xrandr(q)
        .or_else(|| match_xorg_server(q))
        .or_else(|| match_xorg_input(q))
        .or_else(|| match_xorg_config(q))
        .or_else(|| match_xorg_tools(q))
}

/// Xrandr patterns
fn match_xrandr(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[XorgPattern] = &[
        // Xrandr status
        (&["xrandr"], "show xrandr output", "xorg",
         &["xrandr"]),
        (&["xrandr", "info"], "show display info", "xorg",
         &["xrandr --verbose | head -50"]),
        // Monitors
        (&["xrandr", "monitor"], "list monitors", "xorg",
         &["xrandr --listmonitors"]),
        (&["connected", "display"], "show connected displays", "xorg",
         &["xrandr | grep ' connected'"]),
        (&["connected", "monitor"], "show connected monitors", "xorg",
         &["xrandr | grep ' connected'"]),
        // Active outputs
        (&["active", "output"], "show active outputs", "xorg",
         &["xrandr --listactivemonitors"]),
        // Resolution
        (&["xrandr", "resolution"], "show current resolution", "xorg",
         &["xrandr | grep '*'"]),
        (&["screen", "resolution"], "show screen resolution", "xorg",
         &["xrandr | grep '*'"]),
        // Refresh rate
        (&["xrandr", "refresh"], "show refresh rates", "xorg",
         &["xrandr | grep -E '[0-9]+\\.[0-9]+\\*'"]),
        (&["refresh", "rate"], "show monitor refresh rate", "xorg",
         &["xrandr | grep '*'"]),
        // Available modes
        (&["xrandr", "mode"], "show available modes", "xorg",
         &["xrandr"]),
        (&["display", "mode"], "show display modes", "xorg",
         &["xrandr | head -30"]),
        // Providers
        (&["xrandr", "provider"], "show xrandr providers", "xorg",
         &["xrandr --listproviders"]),
        // Screen size
        (&["screen", "size"], "show screen size", "xorg",
         &["xrandr | head -5"]),
        // DPI
        (&["xrandr", "dpi"], "show X DPI", "xorg",
         &["xdpyinfo | grep -E 'dimensions|resolution'"]),
        (&["x11", "dpi"], "show X11 DPI", "xorg",
         &["xdpyinfo | grep resolution"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// X server patterns
fn match_xorg_server(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[XorgPattern] = &[
        // X server info
        (&["xorg", "version"], "show Xorg version", "xorg",
         &["Xorg -version 2>&1 | head -5"]),
        (&["x11", "version"], "show X11 version", "xorg",
         &["Xorg -version 2>&1 | head -5"]),
        (&["xserver", "version"], "show X server version", "xorg",
         &["Xorg -version 2>&1"]),
        // Display info
        (&["xdpyinfo"], "show X display info", "xorg",
         &["xdpyinfo | head -30"]),
        (&["display", "info"], "show display information", "xorg",
         &["xdpyinfo | head -20"]),
        // X session
        (&["x11", "session"], "show X11 session", "xorg",
         &["echo $DISPLAY", "echo $XDG_SESSION_TYPE"]),
        (&["display", "session"], "show display session", "xorg",
         &["echo $DISPLAY", "loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') -p Type 2>/dev/null"]),
        // Is X running
        (&["x11", "running"], "check if X11 is running", "xorg",
         &["pgrep -a Xorg", "echo $DISPLAY"]),
        (&["xorg", "running"], "check if Xorg is running", "xorg",
         &["pgrep -a Xorg"]),
        // X logs
        (&["xorg", "log"], "show Xorg logs", "xorg",
         &["cat /var/log/Xorg.0.log 2>/dev/null | tail -50 || journalctl | grep -i xorg | tail -30"]),
        (&["x11", "log"], "show X11 logs", "xorg",
         &["cat /var/log/Xorg.0.log 2>/dev/null | tail -50"]),
        (&["xorg", "error"], "show Xorg errors", "xorg",
         &["grep -iE 'error|fail|warn' /var/log/Xorg.0.log 2>/dev/null | tail -30"]),
        // X extensions
        (&["x11", "extension"], "list X11 extensions", "xorg",
         &["xdpyinfo | grep -A100 'number of extensions'"]),
        // GLX info
        (&["glxinfo"], "show GLX info", "xorg",
         &["glxinfo | head -20"]),
        (&["opengl", "info"], "show OpenGL info", "xorg",
         &["glxinfo | grep -iE 'vendor|renderer|version' | head -10"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// X input patterns
fn match_xorg_input(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[XorgPattern] = &[
        // Input devices
        (&["xinput"], "list X input devices", "xorg",
         &["xinput list"]),
        (&["xinput", "list"], "list input devices", "xorg",
         &["xinput list"]),
        (&["x11", "input"], "show X11 input devices", "xorg",
         &["xinput list"]),
        // Input device properties
        (&["xinput", "prop"], "show input device properties", "xorg",
         &["xinput list-props <device_id>"]),
        // Mouse
        (&["xinput", "mouse"], "show mouse settings", "xorg",
         &["xinput list | grep -i mouse", "xinput list-props 'pointer' 2>/dev/null | head -20"]),
        (&["mouse", "setting"], "show mouse settings", "xorg",
         &["xinput list | grep -iE 'mouse|pointer'"]),
        // Touchpad
        (&["xinput", "touchpad"], "show touchpad settings", "xorg",
         &["xinput list | grep -i touchpad", "libinput list-devices 2>/dev/null | grep -A5 Touchpad"]),
        (&["touchpad", "setting"], "show touchpad settings", "xorg",
         &["xinput list | grep -i touchpad"]),
        // Keyboard
        (&["xinput", "keyboard"], "show keyboard info", "xorg",
         &["xinput list | grep -i keyboard"]),
        (&["xkb", "layout"], "show keyboard layout", "xorg",
         &["setxkbmap -query"]),
        (&["x11", "keyboard"], "show X11 keyboard settings", "xorg",
         &["setxkbmap -query", "xkbcomp $DISPLAY - 2>/dev/null | head -30"]),
        // Pointer devices
        (&["pointer", "device"], "show pointer devices", "xorg",
         &["xinput list | grep -i pointer"]),
        // Libinput
        (&["libinput", "device"], "list libinput devices", "xorg",
         &["libinput list-devices | head -50"]),
        (&["libinput", "debug"], "debug libinput events", "xorg",
         &["echo 'Run: sudo libinput debug-events'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Xorg config patterns
fn match_xorg_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[XorgPattern] = &[
        // Xorg config
        (&["xorg", "config"], "show Xorg configuration", "xorg",
         &["cat /etc/X11/xorg.conf 2>/dev/null || ls /etc/X11/xorg.conf.d/ 2>/dev/null"]),
        (&["xorg", "conf"], "show Xorg conf files", "xorg",
         &["ls -la /etc/X11/xorg.conf.d/ 2>/dev/null", "cat /etc/X11/xorg.conf.d/*.conf 2>/dev/null | head -50"]),
        // X11 config dir
        (&["x11", "config"], "show X11 configuration", "xorg",
         &["ls -la /etc/X11/", "cat /etc/X11/xorg.conf.d/*.conf 2>/dev/null | head -50"]),
        // Xresources
        (&["xresource"], "show Xresources", "xorg",
         &["cat ~/.Xresources 2>/dev/null | head -30", "xrdb -query | head -20"]),
        (&["xrdb"], "show X resources database", "xorg",
         &["xrdb -query | head -30"]),
        // Xdefaults
        (&["xdefault"], "show Xdefaults", "xorg",
         &["cat ~/.Xdefaults 2>/dev/null | head -30"]),
        // Xmodmap
        (&["xmodmap"], "show keyboard mappings", "xorg",
         &["xmodmap -pke | head -30", "cat ~/.Xmodmap 2>/dev/null"]),
        // Xinit
        (&["xinitrc"], "show xinitrc", "xorg",
         &["cat ~/.xinitrc 2>/dev/null", "cat /etc/X11/xinit/xinitrc 2>/dev/null | head -30"]),
        (&["xprofile"], "show xprofile", "xorg",
         &["cat ~/.xprofile 2>/dev/null"]),
        // Xauthority
        (&["xauthority"], "show X authority", "xorg",
         &["xauth list"]),
        (&["xauth"], "show X auth info", "xorg",
         &["xauth list", "echo $XAUTHORITY"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// X tools patterns
fn match_xorg_tools(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[XorgPattern] = &[
        // Xev
        (&["xev"], "info on xev", "xorg",
         &["echo 'Run: xev to see X events (key presses, mouse clicks)'"]),
        // Xprop
        (&["xprop"], "info on xprop", "xorg",
         &["echo 'Run: xprop to see window properties (click on window)'"]),
        (&["window", "class"], "get window class", "xorg",
         &["echo 'Run: xprop WM_CLASS and click on the window'"]),
        // Xwininfo
        (&["xwininfo"], "info on xwininfo", "xorg",
         &["echo 'Run: xwininfo to get window information'"]),
        (&["window", "info"], "get window info", "xorg",
         &["echo 'Run: xwininfo and click on a window'"]),
        // Xkill
        (&["xkill"], "info on xkill", "xorg",
         &["echo 'Run: xkill and click on a window to kill it'"]),
        // Xclip/xsel
        (&["xclip"], "show clipboard", "xorg",
         &["xclip -selection clipboard -o 2>/dev/null | head -5"]),
        (&["x11", "clipboard"], "show X11 clipboard", "xorg",
         &["xclip -selection clipboard -o 2>/dev/null | head -10"]),
        // Xset
        (&["xset"], "show X settings", "xorg",
         &["xset q"]),
        (&["screensaver", "setting"], "show screensaver settings", "xorg",
         &["xset q | grep -A5 'Screen Saver'"]),
        (&["dpms"], "show DPMS settings", "xorg",
         &["xset q | grep -A5 DPMS"]),
        // Font path
        (&["font", "path"], "show X font path", "xorg",
         &["xset q | grep -A5 'Font Path'"]),
        // Compositing
        (&["xcompmgr"], "check X compositor", "xorg",
         &["pgrep -a 'xcompmgr\\|picom\\|compton'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xrandr() {
        assert!(match_patterns("xrandr").is_some());
        assert!(match_patterns("xrandr monitors").is_some());
        assert!(match_patterns("screen resolution").is_some());
    }

    #[test]
    fn test_xorg_server() {
        assert!(match_patterns("xorg version").is_some());
        assert!(match_patterns("xdpyinfo").is_some());
        assert!(match_patterns("xorg log").is_some());
    }

    #[test]
    fn test_xorg_input() {
        assert!(match_patterns("xinput").is_some());
        assert!(match_patterns("xinput list").is_some());
        assert!(match_patterns("touchpad settings").is_some());
    }

    #[test]
    fn test_xorg_config() {
        assert!(match_patterns("xorg config").is_some());
        assert!(match_patterns("xresources").is_some());
        assert!(match_patterns("xmodmap").is_some());
    }

    #[test]
    fn test_xorg_tools() {
        assert!(match_patterns("xev").is_some());
        assert!(match_patterns("xprop").is_some());
        assert!(match_patterns("xset").is_some());
    }
}
