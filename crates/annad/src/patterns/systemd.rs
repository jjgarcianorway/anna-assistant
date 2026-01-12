//! Systemd patterns for services, units, timers, targets.
//! v0.0.961: Initial implementation.
//! v0.0.989: Added journal, coredump, resolved, resource control patterns

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
        // Enable/disable
        (&["enable", "disable", "service"], "enable/disable service info", "systemd",
         &["echo 'Enable: systemctl enable <service>'",
           "echo 'Disable: systemctl disable <service>'",
           "echo 'Start: systemctl start <service>'"]),
        // Mask/unmask
        (&["mask", "unmask", "service"], "mask/unmask service info", "systemd",
         &["echo 'Mask: systemctl mask <service>'",
           "echo 'Unmask: systemctl unmask <service>'",
           "systemctl list-unit-files --state=masked"]),
        (&["mask", "service"], "mask service", "systemd",
         &["echo 'Run: sudo systemctl mask <service>'",
           "echo 'This prevents the service from being started'"]),
        // Service config
        (&["service", "config"], "show service configuration", "systemd",
         &["echo 'Use: systemctl cat <service>'",
           "echo 'Or: systemctl show <service>'"]),
        (&["show", "service", "config"], "display service config", "systemd",
         &["echo 'Run: systemctl cat <service>'"]),
        // Service environment
        (&["service", "environment"], "show service environment variables", "systemd",
         &["echo 'Use: systemctl show <service> --property=Environment'",
           "echo 'Edit: systemctl edit <service> to add Environment='"]),
        // Service resource limits
        (&["service", "resource", "limit"], "show service resource limits", "systemd",
         &["echo 'Use: systemctl show <service> | grep -E \"Limit|Memory|CPU\"'",
           "echo 'Edit: systemctl edit <service> to add MemoryMax=, CPUQuota='"]),
        // Restart policy
        (&["service", "restart", "policy"], "show service restart policy", "systemd",
         &["echo 'Use: systemctl show <service> --property=Restart'",
           "echo 'Options: no, on-success, on-failure, always'"]),
        (&["restart", "policy"], "service restart configuration", "systemd",
         &["echo 'Restart=on-failure in [Service] section'",
           "echo 'RestartSec=5 for delay between restarts'"]),
        // Transient services
        (&["transient", "service"], "transient systemd services", "systemd",
         &["echo 'Create: systemd-run --unit=myservice <command>'",
           "echo 'With scope: systemd-run --scope <command>'"]),
        // Service ordering
        (&["service", "ordering"], "service start ordering", "systemd",
         &["echo 'Use After=, Before=, Requires=, Wants= in unit file'",
           "echo 'Check: systemctl list-dependencies <service>'"]),
        // Scope units
        (&["scope", "unit"], "systemd scope units", "systemd",
         &["systemctl list-units --type=scope",
           "echo 'Scopes group processes started externally'"]),
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
        (&["journal", "disk"], "show journal disk space", "systemd",
         &["journalctl --disk-usage"]),
        // Journal follow
        (&["follow", "logs"], "follow system logs", "systemd",
         &["echo 'Use: journalctl -f'"]),
        (&["tail", "logs"], "tail system logs", "systemd",
         &["journalctl -f -n 20"]),
        // Journal cleanup
        (&["journal", "cleanup"], "cleanup journal logs", "systemd",
         &["echo 'Vacuum: sudo journalctl --vacuum-size=500M'",
           "echo 'Or by time: sudo journalctl --vacuum-time=7d'"]),
        (&["journal", "vacuum"], "vacuum journal", "systemd",
         &["journalctl --disk-usage",
           "echo 'Run: sudo journalctl --vacuum-size=500M'"]),
        // Journal boots
        (&["list", "boots"], "list recorded boots", "systemd",
         &["journalctl --list-boots"]),
        (&["boot", "history"], "show boot history", "systemd",
         &["journalctl --list-boots"]),
        // Kernel messages
        (&["kernel", "messages"], "show kernel messages", "systemd",
         &["journalctl -k -b --no-pager | tail -50"]),
        (&["dmesg", "errors"], "show dmesg errors", "systemd",
         &["journalctl -k -p err -b --no-pager"]),
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

    #[test]
    fn test_journal() {
        assert!(match_patterns("journal logs").is_some());
        assert!(match_patterns("system logs").is_some());
        assert!(match_patterns("boot logs").is_some());
        assert!(match_patterns("journal errors").is_some());
        assert!(match_patterns("system errors").is_some());
        assert!(match_patterns("kernel errors").is_some());
        assert!(match_patterns("journal size").is_some());
        assert!(match_patterns("journal cleanup").is_some());
        assert!(match_patterns("list boots").is_some());
        assert!(match_patterns("kernel messages").is_some());
    }

    #[test]
    fn test_systemd_advanced() {
        assert!(match_patterns("coredump list").is_some());
        assert!(match_patterns("core dumps").is_some());
        assert!(match_patterns("resolved status").is_some());
        assert!(match_patterns("dns status").is_some());
        assert!(match_patterns("dns servers").is_some());
        assert!(match_patterns("flush dns").is_some());
        assert!(match_patterns("cgroup tree").is_some());
        assert!(match_patterns("resource usage").is_some());
        assert!(match_patterns("machines list").is_some());
        assert!(match_patterns("inhibitor list").is_some());
        assert!(match_patterns("service security").is_some());
        assert!(match_patterns("systemd version").is_some());
    }
}
