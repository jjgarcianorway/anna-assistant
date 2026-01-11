//! Cron and scheduled task patterns for crontab, at, anacron.
//! v0.0.964: Initial implementation.

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
        .or_else(|| match_at_jobs(q))
        .or_else(|| match_anacron(q))
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
}
