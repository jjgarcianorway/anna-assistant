//! Log and journalctl patterns for system log analysis.
//! v0.0.958: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a log-related DeepUnderstanding
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

type LogPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match log-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_journalctl(q)
        .or_else(|| match_dmesg(q))
        .or_else(|| match_log_files(q))
        .or_else(|| match_log_analysis(q))
}

/// Journalctl patterns
fn match_journalctl(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LogPattern] = &[
        // Recent logs
        (&["recent", "logs"], "show recent system logs", "logs",
         &["journalctl -n 50 --no-pager"]),
        (&["latest", "logs"], "show latest system logs", "logs",
         &["journalctl -n 50 --no-pager"]),
        (&["last", "logs"], "show last system logs", "logs",
         &["journalctl -n 50 --no-pager"]),
        // Boot logs
        (&["boot", "logs"], "show boot logs", "logs",
         &["journalctl -b --no-pager | head -100"]),
        (&["last", "boot", "log"], "show last boot log", "logs",
         &["journalctl -b -1 --no-pager | head -100"]),
        (&["previous", "boot"], "show previous boot log", "logs",
         &["journalctl -b -1 --no-pager | head -100"]),
        // Error logs
        (&["error", "logs"], "show error logs", "logs",
         &["journalctl -p err --no-pager -n 50"]),
        (&["errors", "journalctl"], "show journalctl errors", "logs",
         &["journalctl -p err --no-pager -n 50"]),
        (&["warning", "logs"], "show warning logs", "logs",
         &["journalctl -p warning --no-pager -n 50"]),
        (&["critical", "logs"], "show critical logs", "logs",
         &["journalctl -p crit --no-pager -n 30"]),
        // Service logs
        (&["service", "logs"], "show service logs", "logs",
         &["echo 'Use: journalctl -u <service_name>'"]),
        (&["systemd", "logs"], "show systemd logs", "logs",
         &["journalctl -n 50 --no-pager"]),
        // Kernel logs
        (&["kernel", "logs"], "show kernel logs", "logs",
         &["journalctl -k --no-pager -n 50"]),
        (&["kernel", "messages"], "show kernel messages", "logs",
         &["journalctl -k --no-pager -n 50", "dmesg | tail -50"]),
        // Time-based logs
        (&["logs", "today"], "show today's logs", "logs",
         &["journalctl --since today --no-pager | head -100"]),
        (&["logs", "hour"], "show last hour's logs", "logs",
         &["journalctl --since '1 hour ago' --no-pager | head -100"]),
        (&["logs", "yesterday"], "show yesterday's logs", "logs",
         &["journalctl --since yesterday --until today --no-pager | head -100"]),
        // Follow logs
        (&["follow", "logs"], "follow live logs", "logs",
         &["echo 'Use: journalctl -f'"]),
        (&["tail", "logs"], "tail logs", "logs",
         &["journalctl -f -n 20"]),
        (&["live", "logs"], "show live logs", "logs",
         &["echo 'Use: journalctl -f'"]),
        // Disk usage
        (&["journal", "size"], "show journal disk usage", "logs",
         &["journalctl --disk-usage"]),
        (&["logs", "disk"], "show log disk usage", "logs",
         &["journalctl --disk-usage", "du -sh /var/log/"]),
        // List boots
        (&["list", "boots"], "list available boots", "logs",
         &["journalctl --list-boots"]),
        (&["boot", "history"], "show boot history", "logs",
         &["journalctl --list-boots"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Dmesg patterns
fn match_dmesg(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LogPattern] = &[
        // General dmesg
        (&["dmesg"], "show kernel ring buffer", "logs",
         &["dmesg | tail -50"]),
        (&["dmesg", "errors"], "show dmesg errors", "logs",
         &["dmesg --level=err,crit,alert,emerg"]),
        (&["dmesg", "usb"], "show USB-related dmesg", "logs",
         &["dmesg | grep -i usb | tail -20"]),
        (&["dmesg", "disk"], "show disk-related dmesg", "logs",
         &["dmesg | grep -iE 'sd[a-z]|nvme|ata' | tail -20"]),
        (&["dmesg", "network"], "show network-related dmesg", "logs",
         &["dmesg | grep -iE 'eth|wlan|wifi|enp|wlp' | tail -20"]),
        (&["hardware", "errors"], "show hardware errors", "logs",
         &["dmesg --level=err,crit | tail -30"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Log file patterns
fn match_log_files(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LogPattern] = &[
        // Common log files
        (&["syslog"], "show syslog", "logs",
         &["tail -50 /var/log/syslog 2>/dev/null || journalctl -n 50"]),
        (&["auth", "log"], "show authentication log", "logs",
         &["tail -50 /var/log/auth.log 2>/dev/null || journalctl -u sshd -n 50"]),
        (&["login", "attempts"], "show login attempts", "logs",
         &["last -20", "journalctl -u sshd -n 30"]),
        (&["failed", "logins"], "show failed login attempts", "logs",
         &["lastb 2>/dev/null | head -20", "journalctl -u sshd | grep -i failed | tail -20"]),
        // Package manager logs
        (&["pacman", "log"], "show pacman log", "logs",
         &["tail -50 /var/log/pacman.log"]),
        (&["package", "history"], "show package install history", "logs",
         &["tail -100 /var/log/pacman.log | grep -E 'installed|upgraded|removed'"]),
        // X/Wayland logs
        (&["xorg", "log"], "show Xorg log", "logs",
         &["cat ~/.local/share/xorg/Xorg.0.log 2>/dev/null | tail -50 || cat /var/log/Xorg.0.log 2>/dev/null | tail -50"]),
        (&["display", "log"], "show display server logs", "logs",
         &["journalctl -b | grep -iE 'sddm|gdm|lightdm|wayland' | tail -30"]),
        // List log files
        (&["log", "files"], "list log files", "logs",
         &["ls -lh /var/log/"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Log analysis patterns
fn match_log_analysis(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[LogPattern] = &[
        // Crash/panic
        (&["crash", "logs"], "show crash logs", "logs",
         &["journalctl -p crit -b --no-pager", "dmesg | grep -i crash"]),
        (&["panic", "logs"], "show kernel panic logs", "logs",
         &["journalctl -k | grep -i panic", "dmesg | grep -i panic"]),
        (&["segfault"], "show segfault logs", "logs",
         &["journalctl | grep -i segfault | tail -20", "dmesg | grep -i segfault"]),
        (&["oom", "killer"], "show OOM killer logs", "logs",
         &["journalctl | grep -i 'out of memory' | tail -10", "dmesg | grep -i 'oom-killer'"]),
        // What happened
        (&["what", "happened"], "analyze recent issues", "logs",
         &["journalctl -p err -b --no-pager -n 30", "dmesg --level=err,crit | tail -20"]),
        (&["why", "crash"], "investigate crash", "logs",
         &["journalctl -p crit -b --no-pager", "coredumpctl list 2>/dev/null | tail -5"]),
        // Coredumps
        (&["coredumps"], "list coredumps", "logs",
         &["coredumpctl list | tail -10"]),
        (&["core", "dumps"], "show core dumps", "logs",
         &["coredumpctl list | tail -10"]),
        // Audit
        (&["audit", "logs"], "show audit logs", "logs",
         &["journalctl -u auditd | tail -30", "cat /var/log/audit/audit.log 2>/dev/null | tail -30"]),
        // Sudo logs
        (&["sudo", "logs"], "show sudo usage", "logs",
         &["journalctl | grep -i sudo | tail -30"]),
        (&["sudo", "history"], "show sudo history", "logs",
         &["journalctl | grep -i sudo | tail -50"]),
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
    fn test_journalctl() {
        assert!(match_patterns("recent logs").is_some());
        assert!(match_patterns("boot logs").is_some());
        assert!(match_patterns("error logs").is_some());
        assert!(match_patterns("kernel logs").is_some());
    }

    #[test]
    fn test_dmesg() {
        assert!(match_patterns("dmesg").is_some());
        assert!(match_patterns("dmesg errors").is_some());
        assert!(match_patterns("dmesg usb").is_some());
    }

    #[test]
    fn test_log_files() {
        assert!(match_patterns("auth log").is_some());
        assert!(match_patterns("pacman log").is_some());
        assert!(match_patterns("log files").is_some());
    }

    #[test]
    fn test_log_analysis() {
        assert!(match_patterns("crash logs").is_some());
        assert!(match_patterns("what happened").is_some());
        assert!(match_patterns("sudo logs").is_some());
    }
}
