//! Command shortcut formatting and display.

use super::shortcuts::shortcuts_by_category;
use super::types::ShortcutCategory;

/// Format all shortcuts for display
pub fn format_shortcuts() -> String {
    let mut output = String::new();

    output.push_str("Command Shortcuts\n");
    output.push_str("══════════════════════════════════════\n\n");

    for category in ShortcutCategory::all() {
        let shortcuts = shortcuts_by_category(*category);
        if shortcuts.is_empty() {
            continue;
        }

        output.push_str(&format!("▸ {}\n", category.name()));
        for shortcut in shortcuts {
            output.push_str(&format!(
                "  {:8} → {}\n",
                shortcut.short, shortcut.expanded
            ));
        }
        output.push('\n');
    }

    output.push_str("──────────────────────────────────────\n");
    output.push_str("Type any shortcut to expand it.\n");

    output
}

/// Format shortcuts for a specific category
pub fn format_category_shortcuts(category: ShortcutCategory) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} Shortcuts\n", category.name()));
    output.push_str("══════════════════════════════════════\n\n");

    for shortcut in shortcuts_by_category(category) {
        output.push_str(&format!(
            "  {:8} → {}\n           {}\n\n",
            shortcut.short, shortcut.expanded, shortcut.description
        ));
    }

    output
}
