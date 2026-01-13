//! Proactive alert display for REPL greeting.

use anna_shared::monitor::{IssueStore, Severity};

use super::colors::*;

/// Show proactive alerts from monitoring system
pub fn show_proactive_alerts() -> bool {
    let store = match IssueStore::load() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let critical: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Critical && !i.acknowledged)
        .collect();
    let warnings: Vec<_> = store.active_issues.iter()
        .filter(|i| i.severity == Severity::Warning && !i.acknowledged)
        .collect();

    if critical.is_empty() && warnings.is_empty() {
        return false;
    }

    println!();

    if !critical.is_empty() {
        println_colored("Issues detected:", YELLOW);
        println!();
        for issue in &critical {
            print_colored("  [X] ", RED);
            println!("{}", issue.summary);
            if let Some(ref fix) = issue.suggested_fix {
                println_colored(&format!("      -> {}", fix), DIM);
            }
        }
    }

    if !warnings.is_empty() {
        if critical.is_empty() {
            println_colored("Heads up:", YELLOW);
            println!();
        }
        for issue in warnings.iter().take(3) {
            print_colored("  [!] ", YELLOW);
            println!("{}", issue.summary);
        }
        if warnings.len() > 3 {
            println_colored(&format!("      ... and {} more", warnings.len() - 3), DIM);
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
