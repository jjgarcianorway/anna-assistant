// v0.0.541: Tips System Formatters (Phase 117)

use crate::tips_system::manager::TipsSystem;
use crate::tips_system::types::Tip;

/// Format tip for display
pub fn format_tip(tip: &Tip) -> String {
    format!("Tip: {}\n{}", tip.title, tip.content)
}

/// Format tip compact (for greeting)
pub fn format_tip_compact(tip: &Tip) -> String {
    format!("Tip: {}", tip.content)
}

/// Format tips summary
pub fn format_tips_summary(system: &TipsSystem) -> String {
    let mut output = String::new();
    output.push_str("=== Tips System ===\n\n");

    output.push_str(&format!("Total Tips: {}\n", system.total()));
    output.push_str(&format!("Tips Enabled: {}\n", system.show_tips));
    output.push_str(&format!("Max Daily: {}\n", system.max_daily_tips));
    output.push_str(&format!("Remaining Today: {}\n", system.remaining_today()));

    output.push_str("\nBy Category:\n");
    for (cat, count) in system.category_stats() {
        output.push_str(&format!("  {}: {}\n", cat, count));
    }

    output
}
