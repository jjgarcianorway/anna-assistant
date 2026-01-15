//! Systemd patterns for services, units, timers, targets, journal, coredump.

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
        .or_else(|| match_journal(q))
        .or_else(|| match_systemd_advanced(q))
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
        // Enable/disable/mask
        (&["enable", "disable", "service"], "enable/disable service info", "systemd",
         &["echo 'Enable: systemctl enable <service>; Disable: systemctl disable <service>'"]),
        (&["mask", "service"], "mask service", "systemd",
         &["echo 'Mask: systemctl mask <service>; Unmask: systemctl unmask <service>'"]),
        // Service config
        (&["service", "config"], "show service configuration", "systemd",
         &["echo 'Use: systemctl cat <service> or systemctl show <service>'"]),
        // Service environment
        (&["service", "environment"], "show service environment variables", "systemd",
         &["echo 'Use: systemctl show <service> --property=Environment'"]),
        // Restart policy
        (&["restart", "policy"], "service restart configuration", "systemd",
         &["echo 'Restart=on-failure in [Service] section; RestartSec=5 for delay'"]),
        // Scope units
        (&["scope", "unit"], "systemd scope units", "systemd",
         &["systemctl list-units --type=scope"]),
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

/// Journal/logging patterns
fn match_journal(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Journal logs
        (&["journal", "logs"], "show journal logs", "systemd",
         &["journalctl -n 50 --no-pager"]),
        (&["system", "logs"], "show system logs", "systemd",
         &["journalctl -b --no-pager -n 50"]),
        (&["boot", "logs"], "show boot logs", "systemd",
         &["journalctl -b --no-pager | head -100"]),
        (&["previous", "boot", "logs"], "show previous boot logs", "systemd",
         &["journalctl -b -1 --no-pager | head -100"]),
        // Journal errors
        (&["journal", "errors"], "show journal errors", "systemd",
         &["journalctl -p err -b --no-pager -n 50"]),
        (&["system", "errors"], "show system errors", "systemd",
         &["journalctl -p err -b --no-pager -n 30"]),
        (&["kernel", "errors"], "show kernel errors", "systemd",
         &["journalctl -k -p err -b --no-pager -n 30"]),
        // Journal for unit
        (&["service", "logs"], "show service logs", "systemd",
         &["echo 'Use: journalctl -u <service> -n 50'"]),
        (&["unit", "logs"], "show unit logs", "systemd",
         &["echo 'Use: journalctl -u <unit> -n 50'"]),
        // Journal disk usage
        (&["journal", "size"], "show journal disk usage", "systemd",
         &["journalctl --disk-usage"]),
        // Journal follow
        (&["follow", "logs"], "follow system logs", "systemd",
         &["journalctl -f -n 20"]),
        // Journal cleanup
        (&["journal", "cleanup"], "cleanup journal logs", "systemd",
         &["echo 'Vacuum: journalctl --vacuum-size=500M or --vacuum-time=7d'"]),
        // Journal boots
        (&["list", "boots"], "list recorded boots", "systemd",
         &["journalctl --list-boots"]),
        // Kernel messages
        (&["kernel", "messages"], "show kernel messages", "systemd",
         &["journalctl -k -b --no-pager | tail -50"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Advanced systemd patterns (coredump, resolved, resource control)
fn match_systemd_advanced(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[SystemdPattern] = &[
        // Coredump
        (&["coredump", "list"], "list coredumps", "systemd",
         &["coredumpctl list"]),
        (&["core", "dumps"], "show core dumps", "systemd",
         &["coredumpctl list"]),
        (&["crash", "dumps"], "show crash dumps", "systemd",
         &["coredumpctl list"]),
        (&["coredump", "info"], "show coredump info", "systemd",
         &["coredumpctl info", "echo 'For specific: coredumpctl info <PID>'"]),
        // Resolved (DNS)
        (&["resolved", "status"], "show systemd-resolved status", "systemd",
         &["resolvectl status"]),
        (&["dns", "status"], "show DNS resolver status", "systemd",
         &["resolvectl status", "resolvectl dns"]),
        (&["dns", "servers"], "show configured DNS servers", "systemd",
         &["resolvectl dns"]),
        (&["dns", "cache"], "show DNS cache statistics", "systemd",
         &["resolvectl statistics"]),
        (&["flush", "dns"], "flush DNS cache", "systemd",
         &["echo 'Run: sudo resolvectl flush-caches'"]),
        // Networkd
        (&["networkd", "status"], "show systemd-networkd status", "systemd",
         &["networkctl status"]),
        (&["network", "interfaces"], "list network interfaces", "systemd",
         &["networkctl list"]),
        // Resource control / cgroups
        (&["cgroup", "tree"], "show cgroup tree", "systemd",
         &["systemd-cgls"]),
        (&["resource", "usage"], "show resource usage by service", "systemd",
         &["systemd-cgtop -n 1"]),
        (&["service", "resources"], "show service resource usage", "systemd",
         &["systemd-cgtop -n 1"]),
        (&["memory", "usage", "service"], "show memory usage by service", "systemd",
         &["systemd-cgtop -n 1 --order=memory"]),
        // Machine/container
        (&["machines", "list"], "list systemd machines", "systemd",
         &["machinectl list"]),
        (&["container", "list"], "list systemd-nspawn containers", "systemd",
         &["machinectl list"]),
        // Inhibitors
        (&["inhibitor", "list"], "list shutdown inhibitors", "systemd",
         &["systemd-inhibit --list"]),
        (&["what", "blocking", "shutdown"], "what blocks shutdown", "systemd",
         &["systemd-inhibit --list"]),
        // Systemctl show
        (&["service", "properties"], "show service properties", "systemd",
         &["echo 'Use: systemctl show <service>'"]),
        (&["unit", "properties"], "show unit properties", "systemd",
         &["echo 'Use: systemctl show <unit>'"]),
        // Service analysis
        (&["service", "security"], "analyze service security", "systemd",
         &["systemd-analyze security | head -30"]),
        (&["security", "score"], "show service security scores", "systemd",
         &["systemd-analyze security | head -30"]),
        // Portablectl
        (&["portable", "services"], "list portable services", "systemd",
         &["portablectl list 2>/dev/null || echo 'No portable services'"]),
        // Systemd version
        (&["systemd", "version"], "show systemd version", "systemd",
         &["systemctl --version"]),
        // Environment
        (&["systemd", "environment"], "show systemd environment", "systemd",
         &["systemctl show-environment"]),
        // Reloads
        (&["reload", "daemon"], "reload systemd daemon", "systemd",
         &["echo 'Run: sudo systemctl daemon-reload'"]),
        (&["daemon", "reexec"], "re-execute systemd daemon", "systemd",
         &["echo 'Run: sudo systemctl daemon-reexec'"]),
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
    fn test_systemd_patterns() {
        // Services
        for q in ["failed services", "running services", "list services", "user services"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
        // Units
        for q in ["list units", "unit files", "mount units"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
        // Timers and targets
        for q in ["list timers", "active timers", "list targets", "default target"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
        // System
        for q in ["boot time", "boot blame", "slow boot", "hostname info", "time date"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
        // Journal
        for q in ["journal logs", "system logs", "boot logs", "journal errors",
                  "kernel errors", "journal size", "list boots", "kernel messages"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
        // Advanced
        for q in ["coredump list", "core dumps", "resolved status", "dns status",
                  "dns servers", "flush dns", "cgroup tree", "resource usage",
                  "inhibitor list", "service security", "systemd version"] {
            assert!(match_patterns(q).is_some(), "Failed: {}", q);
        }
    }
}
