//! Printing and CUPS patterns.
//! v0.0.967: Initial implementation.

use anna_shared::rpc::{DeepUnderstanding, IntentCategory};

/// Helper to create a printing-related DeepUnderstanding
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

type PrintPattern<'a> = (&'a [&'a str], &'a str, &'a str, &'a [&'a str]);

/// Match printing-related patterns
pub fn match_patterns(q: &str) -> Option<DeepUnderstanding> {
    match_printers(q)
        .or_else(|| match_print_jobs(q))
        .or_else(|| match_cups_service(q))
        .or_else(|| match_printer_config(q))
}

/// Printer patterns
fn match_printers(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PrintPattern] = &[
        // List printers
        (&["list", "printers"], "list printers", "printing",
         &["lpstat -p", "lpstat -a"]),
        (&["available", "printers"], "show available printers", "printing",
         &["lpstat -p -d"]),
        (&["my", "printers"], "show my printers", "printing",
         &["lpstat -p"]),
        (&["installed", "printers"], "show installed printers", "printing",
         &["lpstat -p", "cat /etc/cups/printers.conf 2>/dev/null | grep '<Printer'"]),
        // Default printer
        (&["default", "printer"], "show default printer", "printing",
         &["lpstat -d"]),
        (&["which", "printer"], "show which printer is default", "printing",
         &["lpstat -d"]),
        // Printer status
        (&["printer", "status"], "show printer status", "printing",
         &["lpstat -p", "lpstat -t"]),
        (&["printer", "info"], "show printer info", "printing",
         &["lpstat -p -l"]),
        // Printer drivers
        (&["printer", "drivers"], "list printer drivers", "printing",
         &["lpinfo -m | head -50"]),
        (&["available", "drivers"], "show available printer drivers", "printing",
         &["lpinfo -m | head -50"]),
        // Network printers
        (&["network", "printers"], "find network printers", "printing",
         &["lpinfo -v | grep -E 'network|socket|ipp'", "avahi-browse -t _ipp._tcp 2>/dev/null"]),
        (&["discover", "printers"], "discover printers", "printing",
         &["lpinfo -v", "avahi-browse -t _ipp._tcp 2>/dev/null"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Print job patterns
fn match_print_jobs(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PrintPattern] = &[
        // Print queue
        (&["print", "queue"], "show print queue", "printing",
         &["lpq", "lpstat -o"]),
        (&["print", "jobs"], "show print jobs", "printing",
         &["lpq -a", "lpstat -o"]),
        (&["pending", "prints"], "show pending print jobs", "printing",
         &["lpstat -o"]),
        // My print jobs
        (&["my", "print", "jobs"], "show my print jobs", "printing",
         &["lpq"]),
        // All print jobs
        (&["all", "print", "jobs"], "show all print jobs", "printing",
         &["lpq -a", "lpstat -o"]),
        // Completed jobs
        (&["completed", "prints"], "show completed print jobs", "printing",
         &["lpstat -W completed 2>/dev/null | tail -20"]),
        (&["print", "history"], "show print history", "printing",
         &["lpstat -W completed 2>/dev/null | tail -20"]),
        // Job status
        (&["print", "job", "status"], "show print job status", "printing",
         &["lpstat -o"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// CUPS service patterns
fn match_cups_service(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PrintPattern] = &[
        // CUPS status
        (&["cups", "status"], "show CUPS status", "printing",
         &["systemctl status cups"]),
        (&["cups", "service"], "show CUPS service status", "printing",
         &["systemctl status cups"]),
        (&["cups", "running"], "check if CUPS is running", "printing",
         &["systemctl is-active cups"]),
        // CUPS version
        (&["cups", "version"], "show CUPS version", "printing",
         &["cupsd -v 2>/dev/null || dpkg -l cups 2>/dev/null | tail -1 || pacman -Q cups 2>/dev/null"]),
        // CUPS logs
        (&["cups", "logs"], "show CUPS logs", "printing",
         &["journalctl -u cups -n 30", "tail -30 /var/log/cups/error_log 2>/dev/null"]),
        (&["print", "errors"], "show print errors", "printing",
         &["tail -50 /var/log/cups/error_log 2>/dev/null"]),
        // CUPS web interface
        (&["cups", "web"], "show CUPS web interface URL", "printing",
         &["echo 'CUPS web interface: http://localhost:631'"]),
        (&["cups", "interface"], "show CUPS interface", "printing",
         &["echo 'CUPS web interface: http://localhost:631'"]),
    ];

    for (keywords, desc, topic, commands) in patterns {
        if keywords.iter().all(|k| q.contains(k)) {
            return Some(make_understanding(desc, topic, commands));
        }
    }
    None
}

/// Printer configuration patterns
fn match_printer_config(q: &str) -> Option<DeepUnderstanding> {
    let patterns: &[PrintPattern] = &[
        // Printer options
        (&["printer", "options"], "show printer options", "printing",
         &["lpoptions -l"]),
        (&["print", "options"], "show print options", "printing",
         &["lpoptions -l"]),
        // PPD files
        (&["ppd", "files"], "list PPD files", "printing",
         &["ls /etc/cups/ppd/"]),
        (&["printer", "ppd"], "show printer PPD", "printing",
         &["ls -la /etc/cups/ppd/"]),
        // CUPS config
        (&["cups", "config"], "show CUPS config", "printing",
         &["cat /etc/cups/cupsd.conf | grep -v '^#' | grep -v '^$' | head -50"]),
        (&["cups", "configuration"], "show CUPS configuration", "printing",
         &["cat /etc/cups/cupsd.conf | grep -v '^#' | grep -v '^$' | head -50"]),
        // Printers.conf
        (&["printers", "config"], "show printers config", "printing",
         &["cat /etc/cups/printers.conf 2>/dev/null | head -50"]),
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
    fn test_printers() {
        assert!(match_patterns("list printers").is_some());
        assert!(match_patterns("default printer").is_some());
        assert!(match_patterns("printer status").is_some());
        assert!(match_patterns("network printers").is_some());
    }

    #[test]
    fn test_print_jobs() {
        assert!(match_patterns("print queue").is_some());
        assert!(match_patterns("print jobs").is_some());
        assert!(match_patterns("my print jobs").is_some());
    }

    #[test]
    fn test_cups_service() {
        assert!(match_patterns("cups status").is_some());
        assert!(match_patterns("cups logs").is_some());
        assert!(match_patterns("cups web").is_some());
    }

    #[test]
    fn test_printer_config() {
        assert!(match_patterns("printer options").is_some());
        assert!(match_patterns("cups config").is_some());
    }
}
