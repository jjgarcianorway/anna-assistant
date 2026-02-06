//! Proactive alert display for REPL greeting.

use anna_shared::monitor::{IssueStore, Severity};

use super::colors::*;

/// Show proactive alerts from monitoring system
pub fn show_proactive_alerts() -> bool {
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return false,
    };

    // Only show CRITICAL issues - warnings are too noisy
    let critical: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
        .collect();

    if critical.is_empty() {
        return false;
    }

    println!();
    println_colored("Critical issues detected:", YELLOW);
    println!();

    for issue in &critical {
        print_colored("  [X] ", RED);
        println!("{}", issue.summary);
        if let Some(ref fix) = issue.suggested_fix {
            println_colored(&format!("      -> {}", fix), DIM);
        }
    }

    println!();
    true
}

/// Mark alerts as notified after showing them
pub fn mark_alerts_shown() {
    if let Ok(mut store) = IssueStore::load() {
        store.mark_notified();
        let _ = store.save();
    }
}
