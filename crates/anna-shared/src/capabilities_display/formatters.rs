//! Formatting functions for displaying capabilities.

use crate::team_availability::TeamAvailability;
use super::category::CapabilityCategory;

/// Format all capabilities for display
pub fn format_capabilities() -> String {
    let mut output = String::new();

    output.push_str("What Anna Can Do\n");
    output.push_str("══════════════════════════════════════\n\n");

    for category in CapabilityCategory::all() {
        output.push_str(&format!("▸ {}\n", category.name()));
        output.push_str(&format!("  {}\n", category.description()));
        output.push_str("  Examples:\n");
        for example in category.examples().iter().take(2) {
            output.push_str(&format!("    - \"{}\"\n", example));
        }
        output.push('\n');
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Just ask in natural language!\n");

    output
}

/// Format capabilities for a specific category
pub fn format_capability_category(category: CapabilityCategory) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}\n", category.name()));
    output.push_str("══════════════════════════════════════\n\n");
    output.push_str(&format!("{}\n\n", category.description()));

    output.push_str("Example queries:\n");
    for example in category.examples() {
        output.push_str(&format!("  - \"{}\"\n", example));
    }

    output
}

/// Format a compact summary of capabilities
pub fn format_capabilities_compact() -> String {
    let categories: Vec<&str> = CapabilityCategory::all()
        .iter()
        .map(|c| c.name())
        .collect();

    format!("I can help with: {}", categories.join(", "))
}

/// Format capabilities with team availability info
pub fn format_capabilities_with_teams(teams: &TeamAvailability) -> String {
    let mut output = String::new();

    output.push_str("What Anna Can Do\n");
    output.push_str("══════════════════════════════════════\n\n");

    // Show available teams count
    output.push_str(&format!(
        "Teams available: {} specialists ready to help\n\n",
        teams.available_count()
    ));

    for category in CapabilityCategory::all() {
        output.push_str(&format!("▸ {}\n", category.name()));
        output.push_str(&format!("  {}\n\n", category.description()));
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Just ask in natural language!\n");

    output
}
