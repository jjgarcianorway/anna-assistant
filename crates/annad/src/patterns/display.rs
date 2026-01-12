//! Display and monitor patterns for xrandr, resolution, scaling.
//! v0.0.975: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a display-related DeepUnderstanding
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

type DisplayPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match display-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    // Check GDM/login screen patterns first (more specific)
    match_login_display(q)
        .or_else(|| match_resolution(q))
        .or_else(|| match_monitors(q))
        .or_else(|| match_scaling(q))
        .or_else(|| match_refresh_rate(q))
        .or_else(|| match_display_config(q))
}

/// v0.1.0: GDM and login screen display patterns
fn match_login_display(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // GDM resolution
        (&["gdm", "resolution"], "change GDM resolution", "display",
         &["echo 'GDM Resolution: Create /etc/gdm/custom.conf or /etc/gdm3/custom.conf'",
           "echo 'For X11: sudo -u gdm dbus-launch gsettings set org.gnome.desktop.interface scaling-factor 1'",
           "echo 'For custom resolution, create /usr/share/gdm/greeter-dconf-defaults'"]),
        (&["gdm3", "resolution"], "change GDM3 resolution", "display",
         &["echo 'Method 1: Edit /etc/gdm3/custom.conf to add: [daemon]\\nWaylandEnable=false'",
           "echo 'Method 2: Create /usr/share/gdm/greeter-dconf-defaults for scale settings'",
           "echo 'Method 3: Use xrandr in /etc/gdm3/PostSession/Default or custom script'"]),
        (&["change", "resolution", "gdm"], "change GDM resolution", "display",
         &["echo 'Option 1: Switch GDM to X11 by editing /etc/gdm3/custom.conf'",
           "echo 'Option 2: Create a monitor config in /var/lib/gdm3/.config/monitors.xml'",
           "echo 'Option 3: Use xrandr in /etc/gdm3/Init/Default script'"]),
        // Login screen resolution
        (&["login", "screen", "resolution"], "change login screen resolution", "display",
         &["echo 'For GDM: Edit /etc/gdm3/custom.conf or create monitors.xml'",
           "echo 'For SDDM: Edit /etc/sddm.conf.d/ or use DisplayCommand'",
           "echo 'For LightDM: Edit /etc/lightdm/lightdm.conf display-setup-script'"]),
        (&["login", "resolution"], "change login resolution", "display",
         &["cat /etc/systemd/system/display-manager.service 2>/dev/null | grep ExecStart",
           "echo 'Identify your DM, then configure resolution in its config'"]),
        // GDM scaling
        (&["gdm", "scaling"], "GDM HiDPI/scaling", "display",
         &["echo 'For GDM scaling: sudo -u gdm dbus-launch gsettings set org.gnome.desktop.interface scaling-factor 2'",
           "cat /var/lib/gdm3/.config/monitors.xml 2>/dev/null | head -20"]),
        // GDM Wayland vs X11
        (&["gdm", "wayland"], "GDM Wayland config", "display",
         &["cat /etc/gdm3/custom.conf 2>/dev/null || cat /etc/gdm/custom.conf 2>/dev/null",
           "echo 'To disable Wayland: Add WaylandEnable=false under [daemon] section'"]),
        (&["gdm", "x11"], "GDM X11 config", "display",
         &["echo 'To force X11: Edit /etc/gdm3/custom.conf and add WaylandEnable=false'",
           "cat /etc/gdm3/custom.conf 2>/dev/null || cat /etc/gdm/custom.conf 2>/dev/null"]),
        // SDDM resolution
        (&["sddm", "resolution"], "change SDDM resolution", "display",
         &["echo 'Edit /etc/sddm.conf.d/resolution.conf or use DisplayCommand'",
           "echo 'Example: [X11]\\nDisplayCommand=/usr/share/sddm/scripts/Xsetup'"]),
        // LightDM resolution
        (&["lightdm", "resolution"], "change LightDM resolution", "display",
         &["echo 'Edit /etc/lightdm/lightdm.conf:'",
           "echo '[Seat:*]\\ndisplay-setup-script=xrandr --output HDMI-1 --mode 1920x1080'"]),
        // Display manager info
        (&["display", "manager", "resolution"], "display manager resolution", "display",
         &["systemctl status display-manager | grep -E 'Loaded|Active|Main'",
           "echo 'Check your DM docs (GDM/SDDM/LightDM) for resolution config'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Resolution patterns
fn match_resolution(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // Current resolution
        (&["current", "resolution"], "show current resolution", "display",
         &["xrandr | grep '*'", "xdpyinfo 2>/dev/null | grep dimensions"]),
        (&["screen", "resolution"], "show screen resolution", "display",
         &["xrandr | grep '*'"]),
        (&["display", "resolution"], "show display resolution", "display",
         &["xrandr | grep '*'"]),
        (&["my", "resolution"], "show my resolution", "display",
         &["xrandr | grep '*'"]),
        // Available resolutions
        (&["available", "resolutions"], "list available resolutions", "display",
         &["xrandr"]),
        (&["supported", "resolutions"], "show supported resolutions", "display",
         &["xrandr"]),
        // Native resolution
        (&["native", "resolution"], "show native resolution", "display",
         &["xrandr | grep -E '\\+|\\*'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Monitor patterns
fn match_monitors(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // Connected monitors
        (&["connected", "monitors"], "show connected monitors", "display",
         &["xrandr | grep ' connected'"]),
        (&["connected", "displays"], "show connected displays", "display",
         &["xrandr | grep ' connected'"]),
        // List monitors
        (&["list", "monitors"], "list monitors", "display",
         &["xrandr --listmonitors", "xrandr | grep -E 'connected|disconnected'"]),
        // Monitor info
        (&["monitor", "info"], "show monitor info", "display",
         &["xrandr --verbose | head -50"]),
        (&["display", "info"], "show display info", "display",
         &["xrandr --verbose | head -50"]),
        // Primary monitor
        (&["primary", "monitor"], "show primary monitor", "display",
         &["xrandr | grep primary"]),
        (&["primary", "display"], "show primary display", "display",
         &["xrandr | grep primary"]),
        // External monitor
        (&["external", "monitor"], "show external monitors", "display",
         &["xrandr | grep -E 'HDMI|DP|VGA|DVI' | grep connected"]),
        (&["external", "display"], "show external displays", "display",
         &["xrandr | grep -E 'HDMI|DP|VGA|DVI' | grep connected"]),
        // Multi-monitor
        (&["multi", "monitor"], "show multi-monitor setup", "display",
         &["xrandr --listmonitors"]),
        (&["dual", "monitor"], "show dual monitor setup", "display",
         &["xrandr --listmonitors"]),
        // Monitor names
        (&["monitor", "names"], "show monitor names", "display",
         &["xrandr | grep connected | cut -d' ' -f1"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Scaling patterns
fn match_scaling(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // Display scaling
        (&["display", "scaling"], "show display scaling", "display",
         &["xrandr | grep -A1 connected | grep -E 'x|\\*'", "echo $GDK_SCALE $QT_SCALE_FACTOR"]),
        (&["screen", "scaling"], "show screen scaling", "display",
         &["echo GDK_SCALE=$GDK_SCALE QT_SCALE_FACTOR=$QT_SCALE_FACTOR"]),
        // DPI
        (&["display", "dpi"], "show display DPI", "display",
         &["xdpyinfo 2>/dev/null | grep -i dpi", "xrdb -query | grep dpi"]),
        (&["screen", "dpi"], "show screen DPI", "display",
         &["xdpyinfo 2>/dev/null | grep -i dpi"]),
        // HiDPI
        (&["hidpi"], "show HiDPI settings", "display",
         &["echo GDK_SCALE=$GDK_SCALE QT_SCALE_FACTOR=$QT_SCALE_FACTOR", "xrdb -query | grep -i dpi"]),
        (&["high", "dpi"], "show high DPI settings", "display",
         &["echo GDK_SCALE=$GDK_SCALE QT_SCALE_FACTOR=$QT_SCALE_FACTOR"]),
        // Fractional scaling
        (&["fractional", "scaling"], "show fractional scaling", "display",
         &["gsettings get org.gnome.mutter experimental-features 2>/dev/null", "echo 'Check DE settings for fractional scaling'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Refresh rate patterns
fn match_refresh_rate(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // Current refresh rate
        (&["refresh", "rate"], "show refresh rate", "display",
         &["xrandr | grep '*'"]),
        (&["current", "refresh"], "show current refresh rate", "display",
         &["xrandr | grep '*'"]),
        // Available refresh rates
        (&["available", "refresh"], "show available refresh rates", "display",
         &["xrandr"]),
        // Hz
        (&["monitor", "hz"], "show monitor Hz", "display",
         &["xrandr | grep '*'"]),
        (&["screen", "hz"], "show screen Hz", "display",
         &["xrandr | grep '*'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Display configuration patterns
fn match_display_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[DisplayPattern] = &[
        // Xrandr
        (&["xrandr", "status"], "show xrandr status", "display",
         &["xrandr"]),
        (&["xrandr", "info"], "show xrandr info", "display",
         &["xrandr --verbose | head -80"]),
        // Display server
        (&["display", "server"], "show display server info", "display",
         &["echo $XDG_SESSION_TYPE", "loginctl show-session $(loginctl | grep $USER | awk '{print $1}') -p Type 2>/dev/null"]),
        // Wayland
        (&["wayland", "display"], "show Wayland display info", "display",
         &["echo $WAYLAND_DISPLAY", "echo $XDG_SESSION_TYPE"]),
        // X11
        (&["x11", "display"], "show X11 display info", "display",
         &["echo $DISPLAY", "xdpyinfo 2>/dev/null | head -20"]),
        // Screen info
        (&["screen", "info"], "show screen info", "display",
         &["xrandr", "xdpyinfo 2>/dev/null | head -30"]),
        // Display outputs
        (&["display", "outputs"], "show display outputs", "display",
         &["xrandr | grep -E 'connected|disconnected'"]),
        // Brightness
        (&["screen", "brightness"], "show screen brightness", "display",
         &["brightnessctl 2>/dev/null || cat /sys/class/backlight/*/brightness 2>/dev/null"]),
        (&["display", "brightness"], "show display brightness", "display",
         &["brightnessctl 2>/dev/null || cat /sys/class/backlight/*/brightness"]),
        // Night mode / blue light
        (&["night", "mode"], "show night mode status", "display",
         &["gsettings get org.gnome.settings-daemon.plugins.color night-light-enabled 2>/dev/null || echo 'Check DE settings'"]),
        (&["blue", "light"], "show blue light filter", "display",
         &["gsettings get org.gnome.settings-daemon.plugins.color night-light-enabled 2>/dev/null"]),
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
    fn test_resolution() {
        assert!(match_patterns("current resolution").is_some());
        assert!(match_patterns("screen resolution").is_some());
        assert!(match_patterns("available resolutions").is_some());
    }

    #[test]
    fn test_monitors() {
        assert!(match_patterns("connected monitors").is_some());
        assert!(match_patterns("list monitors").is_some());
        assert!(match_patterns("primary monitor").is_some());
        assert!(match_patterns("external monitor").is_some());
    }

    #[test]
    fn test_scaling() {
        assert!(match_patterns("display scaling").is_some());
        assert!(match_patterns("display dpi").is_some());
        assert!(match_patterns("hidpi").is_some());
    }

    #[test]
    fn test_refresh_rate() {
        assert!(match_patterns("refresh rate").is_some());
        assert!(match_patterns("monitor hz").is_some());
    }

    #[test]
    fn test_display_config() {
        assert!(match_patterns("xrandr status").is_some());
        assert!(match_patterns("display server").is_some());
        assert!(match_patterns("screen brightness").is_some());
    }

    #[test]
    fn test_login_display() {
        assert!(match_patterns("gdm resolution").is_some());
        assert!(match_patterns("gdm3 resolution").is_some());
        assert!(match_patterns("change resolution gdm").is_some());
        assert!(match_patterns("login screen resolution").is_some());
        assert!(match_patterns("sddm resolution").is_some());
        assert!(match_patterns("lightdm resolution").is_some());
        assert!(match_patterns("gdm scaling").is_some());
    }
}
