//! Utility functions for learning command display.

use anna_shared::ui::colors;

/// Create a visual score bar
pub fn score_bar(score: f32, width: usize) -> String {
    let filled = (score * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "{}{}{}{}{}",
        colors::OK,
        "█".repeat(filled),
        colors::DIM,
        "░".repeat(empty),
        colors::RESET
    )
}

/// Truncate string with ellipsis
pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
