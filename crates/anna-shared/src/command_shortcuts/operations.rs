//! Command shortcut operations (expansion, querying, mapping).

use std::collections::HashMap;

use super::shortcuts::builtin_shortcuts;

/// Build a lookup map for quick expansion
pub fn build_shortcut_map() -> HashMap<&'static str, &'static str> {
    builtin_shortcuts()
        .iter()
        .map(|s| (s.short, s.expanded))
        .collect()
}

/// Expand a shortcut if it matches
pub fn expand_shortcut(input: &str) -> Option<String> {
    let trimmed = input.trim().to_lowercase();

    for shortcut in builtin_shortcuts() {
        if trimmed == shortcut.short {
            return Some(shortcut.expanded.to_string());
        }
    }

    None
}

/// Check if input is a shortcut
pub fn is_shortcut(input: &str) -> bool {
    let trimmed = input.trim().to_lowercase();
    builtin_shortcuts().iter().any(|s| s.short == trimmed)
}

/// Detect if query is asking about shortcuts
pub fn is_shortcuts_query(query: &str) -> bool {
    let lower = query.to_lowercase();

    let patterns = [
        "shortcut",
        "short command",
        "quick command",
        "alias",
        "abbreviation",
        "what shortcuts",
        "list shortcuts",
        "show shortcuts",
    ];

    for pattern in patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    false
}
