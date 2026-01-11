//! Systemd patterns for services, units, timers, targets.
//! v0.0.961: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a systemd-related DeepUnderstanding
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

type SystemdPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match systemd-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_services(q)
        .or_else(|| match_units(q))
        .or_else(|| match_timers(q))
        .or_else(|| match_targets(q))
        .or_else(|| match_system(q))
}

/// Service patterns
fn match_services(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Service status
        (&["failed", "services"], "list failed services", "systemd",
         &["systemctl --failed"]),
        (&["running", "services"], "list running services", "systemd",
         &["systemctl list-units --type=service --state=running"]),
        (&["active", "services"], "list active services", "systemd",
         &["systemctl list-units --type=service --state=active"]),
        (&["enabled", "services"], "list enabled services", "systemd",
         &["systemctl list-unit-files --type=service --state=enabled"]),
        (&["disabled", "services"], "list disabled services", "systemd",
         &["systemctl list-unit-files --type=service --state=disabled"]),
        // All services
        (&["list", "services"], "list all services", "systemd",
         &["systemctl list-units --type=service"]),
        (&["all", "services"], "show all services", "systemd",
         &["systemctl list-unit-files --type=service"]),
        // Service dependencies
        (&["service", "dependencies"], "show service dependencies", "systemd",
         &["echo 'Use: systemctl list-dependencies <service>'"]),
        // User services
        (&["user", "services"], "list user services", "systemd",
         &["systemctl --user list-units --type=service"]),
        (&["user", "failed"], "list failed user services", "systemd",
         &["systemctl --user --failed"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Unit patterns
fn match_units(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Unit listing
        (&["list", "units"], "list all units", "systemd",
         &["systemctl list-units"]),
        (&["loaded", "units"], "list loaded units", "systemd",
         &["systemctl list-units --state=loaded"]),
        (&["failed", "units"], "list failed units", "systemd",
         &["systemctl --failed"]),
        // Unit files
        (&["unit", "files"], "list unit files", "systemd",
         &["systemctl list-unit-files"]),
        (&["unit", "location"], "find unit file location", "systemd",
         &["systemctl show -p FragmentPath <unit_name>"]),
        // Mount units
        (&["mount", "units"], "list mount units", "systemd",
         &["systemctl list-units --type=mount"]),
        (&["failed", "mounts"], "list failed mounts", "systemd",
         &["systemctl list-units --type=mount --state=failed"]),
        // Socket units
        (&["socket", "units"], "list socket units", "systemd",
         &["systemctl list-units --type=socket"]),
        (&["sockets", "listening"], "list listening sockets", "systemd",
         &["systemctl list-sockets"]),
        // Path units
        (&["path", "units"], "list path units", "systemd",
         &["systemctl list-units --type=path"]),
        // Slice units
        (&["slice", "units"], "list slice units", "systemd",
         &["systemctl list-units --type=slice"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Timer patterns
fn match_timers(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Timer listing
        (&["list", "timers"], "list all timers", "systemd",
         &["systemctl list-timers --all"]),
        (&["active", "timers"], "list active timers", "systemd",
         &["systemctl list-timers"]),
        (&["systemd", "timers"], "show systemd timers", "systemd",
         &["systemctl list-timers --all"]),
        // Timer status
        (&["timer", "status"], "check timer status", "systemd",
         &["systemctl list-timers --all"]),
        (&["next", "timer"], "show next timer run", "systemd",
         &["systemctl list-timers"]),
        // User timers
        (&["user", "timers"], "list user timers", "systemd",
         &["systemctl --user list-timers --all"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Target patterns
fn match_targets(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Target listing
        (&["list", "targets"], "list all targets", "systemd",
         &["systemctl list-units --type=target"]),
        (&["active", "targets"], "list active targets", "systemd",
         &["systemctl list-units --type=target --state=active"]),
        // Default target
        (&["default", "target"], "show default target", "systemd",
         &["systemctl get-default"]),
        (&["boot", "target"], "show boot target", "systemd",
         &["systemctl get-default"]),
        // Target info
        (&["graphical", "target"], "check graphical target", "systemd",
         &["systemctl is-active graphical.target"]),
        (&["multi-user", "target"], "check multi-user target", "systemd",
         &["systemctl is-active multi-user.target"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// System-wide systemd patterns
fn match_system(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Boot analysis
        (&["boot", "time"], "show boot time analysis", "systemd",
         &["systemd-analyze"]),
        (&["boot", "blame"], "show slow boot services", "systemd",
         &["systemd-analyze blame | head -20"]),
        (&["boot", "critical"], "show critical boot chain", "systemd",
         &["systemd-analyze critical-chain"]),
        (&["slow", "boot"], "diagnose slow boot", "systemd",
         &["systemd-analyze blame | head -20", "systemd-analyze critical-chain"]),
        // System state
        (&["systemd", "status"], "show systemd status", "systemd",
         &["systemctl status"]),
        (&["system", "state"], "show system state", "systemd",
         &["systemctl is-system-running"]),
        // Reload
        (&["daemon", "reload"], "daemon-reload info", "systemd",
         &["echo 'Run: sudo systemctl daemon-reload'"]),
        // Logs
        (&["systemd", "errors"], "show systemd errors", "systemd",
         &["journalctl -p err -b --no-pager -n 30"]),
        // Hostnamectl
        (&["hostname", "info"], "show hostname info", "systemd",
         &["hostnamectl"]),
        // Timedatectl
        (&["time", "date"], "show time/date info", "systemd",
         &["timedatectl"]),
        (&["timezone"], "show timezone", "systemd",
         &["timedatectl | grep 'Time zone'"]),
        (&["ntp", "status"], "show NTP status", "systemd",
         &["timedatectl | grep -i ntp"]),
        // Localectl
        (&["locale", "settings"], "show locale settings", "systemd",
         &["localectl"]),
        (&["keyboard", "layout"], "show keyboard layout", "systemd",
         &["localectl | grep 'Keymap\\|Layout'"]),
        // Loginctl
        (&["logged", "users"], "show logged in users", "systemd",
         &["loginctl list-users"]),
        (&["user", "sessions"], "list user sessions", "systemd",
         &["loginctl list-sessions"]),
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
    fn test_services() {
        assert!(match_patterns("failed services").is_some());
        assert!(match_patterns("running services").is_some());
        assert!(match_patterns("list services").is_some());
        assert!(match_patterns("user services").is_some());
    }

    #[test]
    fn test_units() {
        assert!(match_patterns("list units").is_some());
        assert!(match_patterns("unit files").is_some());
        assert!(match_patterns("mount units").is_some());
    }

    #[test]
    fn test_timers() {
        assert!(match_patterns("list timers").is_some());
        assert!(match_patterns("active timers").is_some());
    }

    #[test]
    fn test_targets() {
        assert!(match_patterns("list targets").is_some());
        assert!(match_patterns("default target").is_some());
    }

    #[test]
    fn test_system() {
        assert!(match_patterns("boot time").is_some());
        assert!(match_patterns("boot blame").is_some());
        assert!(match_patterns("slow boot").is_some());
        assert!(match_patterns("hostname info").is_some());
        assert!(match_patterns("time date").is_some());
    }
}
