//! Formatting utilities for quick status display.

use super::types::{HealthLevel, QuickStatus};

/// Format quick status for one-line display
pub fn format_quick_status_oneline(status: &QuickStatus) -> String {
    let symbol = status.overall.symbol();
    format!("{} {}", symbol, status.summary)
}

/// Format quick status for compact display
pub fn format_quick_status_compact(status: &QuickStatus) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "{} {}\n",
        status.overall.symbol(),
        status.summary
    ));

    // Only show non-good items in compact mode
    for item in &status.items {
        if item.health != HealthLevel::Good {
            output.push_str(&format!("  {}\n", item.format()));
        }
    }

    output
}

/// Format quick status for full display
pub fn format_quick_status_full(status: &QuickStatus) -> String {
    let mut output = String::new();

    output.push_str("Quick Status\n");
    output.push_str("══════════════════════════════════════\n\n");

    output.push_str(&format!(
        "{} {}\n\n",
        status.overall.symbol(),
        status.summary
    ));

    for item in &status.items {
        output.push_str(&format!("  {}\n", item.format()));
    }

    output
}
