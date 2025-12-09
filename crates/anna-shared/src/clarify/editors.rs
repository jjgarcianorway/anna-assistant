//! Editor clarification functions (v0.0.191).

use crate::facts::{FactKey, FactsStore};
use crate::inventory::{load_or_create_inventory, InventoryCache};

use super::legacy::{ClarifyKind, ClarifyOption, ClarifyQuestion};
use super::menu::{ClarifyPrompt, MenuOption};

/// Known text editors
pub const KNOWN_EDITORS: &[&str] = &["vim", "vi", "nvim", "nano", "emacs", "code", "micro", "hx"];

/// Cancel key constant (legacy)
pub const CLARIFY_CANCEL_KEY: &str = "__cancel__";

/// Other key constant (legacy)
pub const CLARIFY_OTHER_KEY: &str = "__other__";

/// Generate editor menu prompt from inventory (v0.0.42)
pub fn editor_menu_prompt(cache: &InventoryCache) -> ClarifyPrompt {
    let editors = [
        ("vim", "Vim"),
        ("nvim", "Neovim"),
        ("nano", "Nano"),
        ("emacs", "Emacs"),
        ("code", "VS Code"),
        ("micro", "Micro"),
    ];

    let mut opts = Vec::new();
    let mut key: u8 = 1;

    for (cmd, label) in &editors {
        if cache.is_installed(cmd).unwrap_or(false) && key < crate::clarify_v2::KEY_OTHER {
            opts.push(
                MenuOption::new(key, *label)
                    .with_fact("preferred_editor", *cmd)
                    .with_verify(format!("command -v {}", cmd)),
            );
            key += 1;
        }
    }

    ClarifyPrompt::new(
        "editor_select",
        "Editor Selection",
        "Which editor do you prefer?",
    )
    .with_options(opts)
    .with_reason("I need to know your editor to configure it")
}

/// Find installed alternative when verification fails (v0.0.42)
pub fn find_installed_alternative(tool: &str, cache: &InventoryCache) -> Option<String> {
    let alts: &[(&str, &[&str])] = &[
        ("vim", &["nvim", "vi", "nano"]),
        ("nvim", &["vim", "vi", "nano"]),
        ("emacs", &["vim", "nano", "code"]),
        ("code", &["vim", "nano", "nvim"]),
        ("nano", &["vim", "micro", "vi"]),
    ];

    for (t, alternatives) in alts {
        if *t == tool {
            for alt in *alternatives {
                if cache.is_installed(alt).unwrap_or(false) {
                    return Some(alt.to_string());
                }
            }
        }
    }
    None
}

/// Generate editor options (sync version)
pub fn generate_editor_options_sync() -> Vec<ClarifyOption> {
    generate_editor_options_with_cache(&load_or_create_inventory())
}

/// Generate editor options with cache
pub fn generate_editor_options_with_cache(cache: &InventoryCache) -> Vec<ClarifyOption> {
    let mut options = Vec::new();
    let installed_editors = cache.installed_editors();

    for editor in KNOWN_EDITORS {
        if installed_editors.contains(editor) {
            options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
        } else if let Some(true) = cache.is_installed(editor) {
            options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
        }
    }

    if options.is_empty() {
        for editor in KNOWN_EDITORS {
            if verify_editor_installed(editor) {
                options.push(ClarifyOption::new(*editor, *editor).with_evidence("installed: true"));
                break;
            }
        }
    }

    options.push(ClarifyOption::new(CLARIFY_OTHER_KEY, "Other").with_evidence("custom input"));
    options.push(ClarifyOption::new(CLARIFY_CANCEL_KEY, "Cancel").with_evidence("skip"));
    options
}

/// Check if selection is cancel
pub fn is_cancel_selection(key: &str) -> bool {
    key == CLARIFY_CANCEL_KEY
}

/// Check if selection is other
pub fn is_other_selection(key: &str) -> bool {
    key == CLARIFY_OTHER_KEY
}

/// Verify if editor is installed
pub fn verify_editor_installed(editor: &str) -> bool {
    std::process::Command::new("which")
        .arg(editor)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Generate editor clarification (legacy)
pub fn generate_editor_clarification(facts: &FactsStore) -> (ClarifyQuestion, Vec<ClarifyOption>) {
    let default = facts
        .get_verified(&FactKey::PreferredEditor)
        .map(|s| s.to_string());
    let question = ClarifyQuestion::new(
        ClarifyKind::PreferredEditor,
        "Which text editor do you prefer?",
    )
    .with_verify("which {}")
    .with_hint("Select from installed editors")
    .with_default(default.unwrap_or_default());
    let options = generate_editor_options_sync();
    (question, options)
}
