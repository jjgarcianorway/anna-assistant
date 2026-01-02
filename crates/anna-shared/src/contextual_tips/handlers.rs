//! Public API handlers for contextual tips.

use super::tips::*;
use super::types::{ContextualTip, TipContext};

/// Get contextual tips based on current context
pub fn get_contextual_tips(context: &TipContext) -> Vec<ContextualTip> {
    let mut tips = Vec::new();

    // Add topic-specific tips
    for topic in &context.topics {
        match topic.as_str() {
            "editor" => tips.extend(editor_tips()),
            "containers" => tips.extend(container_tips()),
            "git" => tips.extend(git_tips()),
            "services" => tips.extend(service_tips()),
            "network" => tips.extend(network_tips()),
            "storage" => tips.extend(storage_tips()),
            "packages" => tips.extend(package_tips()),
            "scheduling" => tips.extend(scheduling_tips()),
            "security" => tips.extend(security_tips()),
            _ => {}
        }
    }

    // Add learning tips if learning mode is on
    if context.learning_mode {
        tips.extend(learning_tips());
    }

    // If no specific tips, add general tips
    if tips.is_empty() {
        tips = general_tips();
    }

    tips
}

/// Select a single tip from available tips
pub fn select_tip(tips: &[ContextualTip], seed: u64) -> Option<&ContextualTip> {
    if tips.is_empty() {
        return None;
    }
    let idx = (seed as usize) % tips.len();
    tips.get(idx)
}

/// Format a tip for display
pub fn format_tip(tip: &ContextualTip) -> String {
    if let Some(action) = tip.related_action {
        format!("Tip: {} (try: \"{}\")", tip.message, action)
    } else {
        format!("Tip: {}", tip.message)
    }
}

/// Get a single contextual tip for display
pub fn get_tip_for_query(query: &str, learning_mode: bool) -> Option<String> {
    let context = TipContext::from_query(query).with_learning_mode(learning_mode);

    let tips = get_contextual_tips(&context);

    // Use timestamp-based seed for variety
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    select_tip(&tips, seed).map(format_tip)
}

/// Check if we should show a tip (probability-based)
/// Shows tip roughly 1 in 4 times
pub fn should_show_tip() -> bool {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);

    seed % 4 == 0
}
