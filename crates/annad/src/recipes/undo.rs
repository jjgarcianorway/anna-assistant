//! Undo operations for reversing changes.
//! v0.0.998: Initial implementation

use crate::changes::{undo_change, get_undo_summary, ChangeHistory};
use tracing::info;

use super::RecipeResult;

/// Handle undo request via natural language
pub fn handle_undo_request(query: &str) -> Option<RecipeResult> {
    try_undo(query)
}

/// Try to match an undo-related request
pub fn try_undo(q: &str) -> Option<RecipeResult> {
    if !q.contains("undo") && !q.contains("revert") && !q.contains("restore") {
        return None;
    }

    // "show what I can undo" or "what changes" or "undo what"
    if q.contains("what") || q.contains("show") || q.contains("list") {
        return Some(show_undoable());
    }

    // "undo last change" or "undo last"
    if q.contains("last") {
        return Some(undo_last());
    }

    // Try to find a specific change to undo
    let history = ChangeHistory::load();
    let undoable = history.undoable();

    // Check if query mentions a specific change
    for change in &undoable {
        if q.contains(&change.name) || q.contains(&change.category) {
            return Some(do_undo(change));
        }
    }

    // Generic undo - show options
    Some(show_undoable())
}

fn show_undoable() -> RecipeResult {
    let summary = get_undo_summary();

    RecipeResult {
        success: true,
        message: summary,
        needs_confirmation: false,
        confirmation_prompt: None,
    }
}

fn undo_last() -> RecipeResult {
    let history = ChangeHistory::load();
    let undoable = history.undoable();

    if undoable.is_empty() {
        return RecipeResult {
            success: true,
            message: "Nothing to undo. I haven't made any changes yet.".to_string(),
            needs_confirmation: false,
            confirmation_prompt: None,
        };
    }

    let last = undoable[0];
    do_undo(last)
}

fn do_undo(change: &crate::changes::ChangeRecord) -> RecipeResult {
    info!("Undoing change: {} ({})", change.name, change.id);

    match undo_change(change) {
        Ok(msg) => RecipeResult {
            success: true,
            message: format!("Undone! {}\n\n{} has been restored to its previous state.", msg, change.description),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
        Err(e) => RecipeResult {
            success: false,
            message: format!("Failed to undo: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}
