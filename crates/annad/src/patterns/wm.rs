//! Window manager patterns for Hyprland, Sway, i3, and others.
//! v0.0.983: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a WM-related DeepUnderstanding
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

type WmPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match window manager patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_hyprland(q)
        .or_else(|| match_sway(q))
        .or_else(|| match_i3(q))
        .or_else(|| match_wm_general(q))
        .or_else(|| match_compositor(q))
}

/// Hyprland patterns
fn match_hyprland(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[WmPattern] = &[
        // Hyprland config
        (&["hyprland", "config"], "show Hyprland config", "wm",
         &["cat ~/.config/hypr/hyprland.conf 2>/dev/null | head -50"]),
        (&["hyprland", "conf"], "show Hyprland config", "wm",
         &["cat ~/.config/hypr/hyprland.conf 2>/dev/null | head -50"]),
        // Hyprland reload
        (&["hyprland", "reload"], "reload Hyprland", "wm",
         &["hyprctl reload"]),
        // Hyprland version
        (&["hyprland", "version"], "show Hyprland version", "wm",
         &["hyprctl version"]),
        // Hyprland monitors
        (&["hyprland", "monitors"], "show Hyprland monitors", "wm",
         &["hyprctl monitors"]),
        (&["hyprland", "displays"], "show Hyprland displays", "wm",
         &["hyprctl monitors"]),
        // Hyprland workspaces
        (&["hyprland", "workspaces"], "show Hyprland workspaces", "wm",
         &["hyprctl workspaces"]),
        (&["hyprland", "workspace"], "show active workspace", "wm",
         &["hyprctl activeworkspace"]),
        // Hyprland windows
        (&["hyprland", "windows"], "show Hyprland windows", "wm",
         &["hyprctl clients"]),
        (&["hyprland", "clients"], "list Hyprland clients", "wm",
         &["hyprctl clients"]),
        // Hyprland binds
        (&["hyprland", "binds"], "show Hyprland keybinds", "wm",
         &["hyprctl binds"]),
        (&["hyprland", "keybinds"], "show Hyprland keybinds", "wm",
         &["hyprctl binds"]),
        // Hyprland devices
        (&["hyprland", "devices"], "show Hyprland devices", "wm",
         &["hyprctl devices"]),
        // Hyprland layers
        (&["hyprland", "layers"], "show Hyprland layers", "wm",
         &["hyprctl layers"]),
        // Hyprland info
        (&["hyprland", "info"], "show Hyprland info", "wm",
         &["hyprctl version", "hyprctl monitors", "hyprctl workspaces"]),
        // Hyprctl
        (&["hyprctl"], "show hyprctl commands", "wm",
         &["hyprctl --help 2>&1 | head -30"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Sway patterns
fn match_sway(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[WmPattern] = &[
        // Sway config
        (&["sway", "config"], "show Sway config", "wm",
         &["cat ~/.config/sway/config 2>/dev/null | head -50"]),
        // Sway reload
        (&["sway", "reload"], "reload Sway config", "wm",
         &["swaymsg reload"]),
        // Sway version
        (&["sway", "version"], "show Sway version", "wm",
         &["sway --version"]),
        // Sway outputs
        (&["sway", "outputs"], "show Sway outputs", "wm",
         &["swaymsg -t get_outputs"]),
        (&["sway", "monitors"], "show Sway monitors", "wm",
         &["swaymsg -t get_outputs"]),
        // Sway workspaces
        (&["sway", "workspaces"], "show Sway workspaces", "wm",
         &["swaymsg -t get_workspaces"]),
        // Sway inputs
        (&["sway", "inputs"], "show Sway inputs", "wm",
         &["swaymsg -t get_inputs"]),
        // Sway windows
        (&["sway", "windows"], "show Sway windows", "wm",
         &["swaymsg -t get_tree | jq '.nodes[].nodes[].nodes[] | select(.name != null) | .name' 2>/dev/null"]),
        // Sway bindings
        (&["sway", "binds"], "show Sway keybinds", "wm",
         &["grep -E '^bindsym|^bindcode' ~/.config/sway/config 2>/dev/null | head -30"]),
        // Swaymsg
        (&["swaymsg"], "show swaymsg help", "wm",
         &["swaymsg --help 2>&1 | head -20"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// i3 patterns
fn match_i3(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[WmPattern] = &[
        // i3 config
        (&["i3", "config"], "show i3 config", "wm",
         &["cat ~/.config/i3/config 2>/dev/null | head -50 || cat ~/.i3/config 2>/dev/null | head -50"]),
        // i3 reload
        (&["i3", "reload"], "reload i3 config", "wm",
         &["i3-msg reload"]),
        (&["i3", "restart"], "restart i3", "wm",
         &["i3-msg restart"]),
        // i3 version
        (&["i3", "version"], "show i3 version", "wm",
         &["i3 --version"]),
        // i3 workspaces
        (&["i3", "workspaces"], "show i3 workspaces", "wm",
         &["i3-msg -t get_workspaces"]),
        // i3 outputs
        (&["i3", "outputs"], "show i3 outputs", "wm",
         &["i3-msg -t get_outputs"]),
        // i3 tree
        (&["i3", "tree"], "show i3 window tree", "wm",
         &["i3-msg -t get_tree | head -100"]),
        // i3 bindings
        (&["i3", "binds"], "show i3 keybinds", "wm",
         &["grep -E '^bindsym|^bindcode' ~/.config/i3/config 2>/dev/null | head -30"]),
        // i3-msg
        (&["i3-msg"], "show i3-msg help", "wm",
         &["i3-msg --help 2>&1 | head -20"]),
        // i3bar
        (&["i3bar"], "show i3bar config", "wm",
         &["grep -A20 'bar {' ~/.config/i3/config 2>/dev/null"]),
        // i3status
        (&["i3status"], "show i3status config", "wm",
         &["cat ~/.config/i3status/config 2>/dev/null | head -30"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// General WM patterns
fn match_wm_general(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[WmPattern] = &[
        // Which WM
        (&["which", "window", "manager"], "show current window manager", "wm",
         &["echo $XDG_CURRENT_DESKTOP", "pgrep -l 'hyprland|sway|i3|openbox|bspwm|dwm|awesome' 2>/dev/null"]),
        (&["current", "wm"], "show current WM", "wm",
         &["echo $XDG_CURRENT_DESKTOP", "wmctrl -m 2>/dev/null"]),
        (&["running", "wm"], "show running WM", "wm",
         &["pgrep -l 'hyprland|sway|i3|openbox|bspwm|dwm|awesome|kwin|mutter'"]),
        // WM logs
        (&["wm", "logs"], "show window manager logs", "wm",
         &["journalctl --user -u 'hyprland|sway' -n 30 2>/dev/null || cat ~/.local/share/hyprland/hyprland.log 2>/dev/null | tail -30"]),
        // Openbox
        (&["openbox", "config"], "show Openbox config", "wm",
         &["cat ~/.config/openbox/rc.xml 2>/dev/null | head -50"]),
        (&["openbox", "menu"], "show Openbox menu", "wm",
         &["cat ~/.config/openbox/menu.xml 2>/dev/null | head -50"]),
        // bspwm
        (&["bspwm", "config"], "show bspwm config", "wm",
         &["cat ~/.config/bspwm/bspwmrc 2>/dev/null | head -50"]),
        (&["bspwm", "rules"], "show bspwm rules", "wm",
         &["bspc rule -l"]),
        // awesome
        (&["awesome", "config"], "show awesome config", "wm",
         &["cat ~/.config/awesome/rc.lua 2>/dev/null | head -50"]),
        // dwm
        (&["dwm", "config"], "show dwm config", "wm",
         &["cat ~/dwm/config.h 2>/dev/null | head -50 || echo 'dwm uses config.h, recompile to change'"]),
        // Waybar
        (&["waybar", "config"], "show Waybar config", "wm",
         &["cat ~/.config/waybar/config 2>/dev/null | head -50"]),
        (&["waybar", "style"], "show Waybar style", "wm",
         &["cat ~/.config/waybar/style.css 2>/dev/null | head -50"]),
        // Polybar
        (&["polybar", "config"], "show Polybar config", "wm",
         &["cat ~/.config/polybar/config.ini 2>/dev/null | head -50"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Compositor patterns
fn match_compositor(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[WmPattern] = &[
        // Picom
        (&["picom", "config"], "show Picom config", "wm",
         &["cat ~/.config/picom/picom.conf 2>/dev/null | head -50 || cat ~/.config/picom.conf 2>/dev/null | head -50"]),
        (&["picom", "running"], "check if Picom is running", "wm",
         &["pgrep -a picom"]),
        // Compton (old)
        (&["compton", "config"], "show Compton config", "wm",
         &["cat ~/.config/compton.conf 2>/dev/null | head -50"]),
        // Screen tearing
        (&["screen", "tearing"], "fix screen tearing", "wm",
         &["echo 'For X11: use picom with vsync'", "echo 'For Wayland: check compositor settings'"]),
        // Compositor status
        (&["compositor", "status"], "show compositor status", "wm",
         &["pgrep -l 'picom|compton|kwin|mutter|weston|sway|hyprland'"]),
        // Transparency
        (&["window", "transparency"], "configure window transparency", "wm",
         &["echo 'X11: Use picom with opacity rules'", "echo 'Hyprland: decoration:active_opacity'"]),
        // Blur
        (&["window", "blur"], "configure window blur", "wm",
         &["echo 'Picom: blur-method in config'", "echo 'Hyprland: decoration:blur'"]),
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
    fn test_hyprland() {
        assert!(match_patterns("hyprland config").is_some());
        assert!(match_patterns("hyprland monitors").is_some());
        assert!(match_patterns("hyprland workspaces").is_some());
        assert!(match_patterns("hyprctl").is_some());
    }

    #[test]
    fn test_sway() {
        assert!(match_patterns("sway config").is_some());
        assert!(match_patterns("sway workspaces").is_some());
        assert!(match_patterns("swaymsg").is_some());
    }

    #[test]
    fn test_i3() {
        assert!(match_patterns("i3 config").is_some());
        assert!(match_patterns("i3 workspaces").is_some());
        assert!(match_patterns("i3-msg").is_some());
    }

    #[test]
    fn test_wm_general() {
        assert!(match_patterns("which window manager").is_some());
        assert!(match_patterns("waybar config").is_some());
        assert!(match_patterns("polybar config").is_some());
    }

    #[test]
    fn test_compositor() {
        assert!(match_patterns("picom config").is_some());
        assert!(match_patterns("screen tearing").is_some());
        assert!(match_patterns("compositor status").is_some());
    }
}
