//! Cron and scheduled task patterns for crontab, at, anacron, systemd timers.
//! v0.0.964: Initial implementation.
//! v0.0.989: Added systemd timers, cron environment, mail, permissions

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a cron-related DeepUnderstanding
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

type CronPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match cron-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_crontab(q)
        .or_else(|| match_system_cron(q))
        .or_else(|| match_systemd_timers(q))
        .or_else(|| match_at_jobs(q))
        .or_else(|| match_anacron(q))
        .or_else(|| match_cron_troubleshoot(q))
}

/// Crontab patterns
fn match_crontab(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // User crontab
        (&["my", "crontab"], "show my crontab", "cron",
         &["crontab -l"]),
        (&["user", "crontab"], "show user crontab", "cron",
         &["crontab -l"]),
        (&["crontab", "list"], "list crontab entries", "cron",
         &["crontab -l"]),
        (&["show", "crontab"], "show crontab", "cron",
         &["crontab -l"]),
        // All user crontabs
        (&["all", "crontabs"], "show all crontabs", "cron",
         &["for user in $(cut -f1 -d: /etc/passwd); do echo \"=== $user ===\"; crontab -l -u $user 2>/dev/null; done"]),
        (&["list", "crontabs"], "list all crontabs", "cron",
         &["ls /var/spool/cron/crontabs/ 2>/dev/null || ls /var/spool/cron/ 2>/dev/null"]),
        // Crontab syntax
        (&["crontab", "syntax"], "show crontab syntax help", "cron",
         &["echo '# MIN HOUR DAY MONTH DOW COMMAND'; echo '# 0-59 0-23 1-31 1-12 0-6 (0=Sunday)'; echo '# Examples:'; echo '# 0 * * * * every hour'; echo '# 0 0 * * * daily at midnight'; echo '# 0 0 * * 0 weekly on Sunday'"]),
        (&["cron", "syntax"], "show cron syntax", "cron",
         &["echo 'MIN HOUR DAY MONTH DOW COMMAND'; echo '*/5 = every 5'; echo '0,30 = at 0 and 30'"]),
        // Root crontab
        (&["root", "crontab"], "show root crontab", "cron",
         &["sudo crontab -l 2>/dev/null || echo 'Need sudo'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// System cron patterns
fn match_system_cron(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // System cron directories
        (&["system", "cron"], "show system cron jobs", "cron",
         &["ls -la /etc/cron.d/", "ls -la /etc/cron.daily/", "ls -la /etc/cron.hourly/"]),
        (&["cron", "directories"], "list cron directories", "cron",
         &["ls -la /etc/cron.d/ /etc/cron.daily/ /etc/cron.hourly/ /etc/cron.weekly/ /etc/cron.monthly/ 2>/dev/null"]),
        // Specific directories
        (&["cron", "daily"], "show daily cron jobs", "cron",
         &["ls -la /etc/cron.daily/"]),
        (&["cron", "hourly"], "show hourly cron jobs", "cron",
         &["ls -la /etc/cron.hourly/"]),
        (&["cron", "weekly"], "show weekly cron jobs", "cron",
         &["ls -la /etc/cron.weekly/"]),
        (&["cron", "monthly"], "show monthly cron jobs", "cron",
         &["ls -la /etc/cron.monthly/"]),
        // /etc/crontab
        (&["etc", "crontab"], "show /etc/crontab", "cron",
         &["cat /etc/crontab"]),
        // Cron service
        (&["cron", "service"], "show cron service status", "cron",
         &["systemctl status cronie 2>/dev/null || systemctl status cron 2>/dev/null"]),
        (&["cron", "status"], "check cron status", "cron",
         &["systemctl is-active cronie 2>/dev/null || systemctl is-active cron 2>/dev/null"]),
        // Cron logs
        (&["cron", "logs"], "show cron logs", "cron",
         &["journalctl -u cronie -n 30 2>/dev/null || journalctl -u cron -n 30 2>/dev/null || grep -i cron /var/log/syslog 2>/dev/null | tail -30"]),
        (&["cron", "history"], "show cron job history", "cron",
         &["journalctl -u cronie -n 50 2>/dev/null || grep CRON /var/log/syslog 2>/dev/null | tail -50"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// At jobs patterns
/// Note: Avoid short "at" keyword as it matches substring in "what", "that", etc.
fn match_at_jobs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // List at jobs - use "atq" or "atd" to avoid "what/that" substring matches
        (&["atq"], "list at jobs", "cron",
         &["atq"]),
        (&["atd", "jobs"], "show at daemon jobs", "cron",
         &["atq"]),
        (&["list", "atjobs"], "list at jobs", "cron",
         &["atq"]),
        (&["scheduled", "jobs"], "show scheduled jobs", "cron",
         &["atq", "systemctl list-timers"]),
        (&["pending", "jobs"], "show pending jobs", "cron",
         &["atq"]),
        // At service
        (&["atd", "service"], "show atd service status", "cron",
         &["systemctl status atd"]),
        (&["atd", "status"], "show atd status", "cron",
         &["systemctl status atd"]),
        // Batch jobs
        (&["batch", "jobs"], "show batch jobs", "cron",
         &["atq -q b"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Anacron patterns
fn match_anacron(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // Anacron config
        (&["anacron", "config"], "show anacron config", "cron",
         &["cat /etc/anacrontab"]),
        (&["anacrontab"], "show anacrontab", "cron",
         &["cat /etc/anacrontab"]),
        // Anacron status
        (&["anacron", "status"], "show anacron status", "cron",
         &["cat /var/spool/anacron/*"]),
        // Anacron timestamps
        (&["anacron", "timestamps"], "show anacron timestamps", "cron",
         &["ls -la /var/spool/anacron/"]),
        (&["anacron", "last"], "show last anacron runs", "cron",
         &["cat /var/spool/anacron/*"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Systemd timer patterns (modern cron alternative)
fn match_systemd_timers(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // Timer units
        (&["timer", "units"], "list systemd timer units", "cron",
         &["systemctl list-timers --all"]),
        (&["systemd", "timers"], "show systemd timers", "cron",
         &["systemctl list-timers"]),
        (&["list", "timers"], "list all timers", "cron",
         &["systemctl list-timers --all"]),
        // Active timers
        (&["active", "timers"], "show active timers", "cron",
         &["systemctl list-timers"]),
        // Timer status
        (&["timer", "status"], "show timer status", "cron",
         &["systemctl list-timers", "echo 'For specific: systemctl status <timer>.timer'"]),
        // Scheduled tasks (generic)
        (&["scheduled", "tasks"], "show scheduled tasks", "cron",
         &["systemctl list-timers", "crontab -l 2>/dev/null"]),
        // Task scheduler
        (&["task", "scheduler"], "show task scheduler status", "cron",
         &["systemctl list-timers", "systemctl status cronie 2>/dev/null || systemctl status cron"]),
        // Cron alternatives
        (&["cron", "alternatives"], "show cron alternatives", "cron",
         &["echo 'Systemd timers: systemctl list-timers'",
           "echo 'Anacron: cat /etc/anacrontab'",
           "echo 'At daemon: atq'"]),
        // Create timer
        (&["create", "timer"], "how to create systemd timer", "cron",
         &["echo 'Create /etc/systemd/system/mytask.timer and mytask.service'",
           "echo 'Timer: [Timer] OnCalendar=daily'",
           "echo 'Enable: systemctl enable --now mytask.timer'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Cron troubleshooting patterns
fn match_cron_troubleshoot(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[CronPattern] = &[
        // Cron environment
        (&["cron", "environment"], "show cron environment info", "cron",
         &["echo 'Cron runs with minimal PATH'",
           "echo 'Add PATH=/usr/local/bin:/usr/bin:/bin at top of crontab'",
           "echo 'Or use full paths in commands'"]),
        // Cron mail
        (&["cron", "mail"], "configure cron mail", "cron",
         &["echo 'Set MAILTO=user@example.com in crontab'",
           "echo 'MAILTO=\"\" disables mail'",
           "cat /var/mail/$USER 2>/dev/null | tail -30"]),
        // Cron permissions
        (&["cron", "permissions"], "show cron access permissions", "cron",
         &["cat /etc/cron.allow 2>/dev/null || echo 'No cron.allow (all users allowed unless denied)'",
           "cat /etc/cron.deny 2>/dev/null || echo 'No cron.deny'"]),
        // Cron not running
        (&["cron", "not", "running"], "troubleshoot cron not running", "cron",
         &["systemctl status cronie 2>/dev/null || systemctl status cron",
           "echo 'Start: sudo systemctl start cronie'",
           "echo 'Enable: sudo systemctl enable cronie'"]),
        // Cron job not executing
        (&["cron", "job", "not"], "troubleshoot cron job not executing", "cron",
         &["echo 'Check logs: journalctl -u cronie -n 50'",
           "echo 'Check PATH, use full paths'",
           "echo 'Check permissions on script'"]),
        // Debug cron
        (&["debug", "cron"], "debug cron jobs", "cron",
         &["journalctl -u cronie -f",
           "echo 'Add output redirection: command >> /tmp/cron.log 2>&1'"]),
        // Cron examples
        (&["cron", "examples"], "show cron schedule examples", "cron",
         &["echo '0 * * * *     every hour'",
           "echo '0 0 * * *     daily at midnight'",
           "echo '0 0 * * 0     weekly on Sunday'",
           "echo '*/5 * * * *   every 5 minutes'",
           "echo '0 9-17 * * 1-5   hourly 9-5 weekdays'"]),
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
    fn test_crontab() {
        assert!(match_patterns("my crontab").is_some());
        assert!(match_patterns("crontab list").is_some());
        assert!(match_patterns("crontab syntax").is_some());
    }

    #[test]
    fn test_system_cron() {
        assert!(match_patterns("system cron").is_some());
        assert!(match_patterns("cron daily").is_some());
        assert!(match_patterns("cron logs").is_some());
        assert!(match_patterns("cron service").is_some());
    }

    #[test]
    fn test_at_jobs() {
        assert!(match_patterns("atq").is_some());
        assert!(match_patterns("atd jobs").is_some());
        assert!(match_patterns("scheduled jobs").is_some());
    }

    #[test]
    fn test_anacron() {
        assert!(match_patterns("anacrontab").is_some());
        assert!(match_patterns("anacron status").is_some());
    }

    #[test]
    fn test_systemd_timers() {
        assert!(match_patterns("timer units").is_some());
        assert!(match_patterns("scheduled tasks").is_some());
        assert!(match_patterns("task scheduler").is_some());
        assert!(match_patterns("cron alternatives").is_some());
    }

    #[test]
    fn test_cron_troubleshoot() {
        assert!(match_patterns("cron environment").is_some());
        assert!(match_patterns("cron mail").is_some());
        assert!(match_patterns("cron permissions").is_some());
    }
}
