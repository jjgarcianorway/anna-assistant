//! Vim/Neovim configuration recipes.
//! v0.0.998: Initial implementation

use crate::changes::{append_to_file, apply_change};
use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

use super::RecipeResult;

/// Pending vim recipes awaiting confirmation
static PENDING_VIM: RwLock<Option<HashMap<String, PendingVimChange>>> = RwLock::new(None);

#[derive(Clone)]
struct PendingVimChange {
    config_lines: Vec<String>,
    description: String,
}

/// Get the vimrc path
fn vimrc_path() -> String {
    // Check for neovim first
    let nvim_config = dirs::config_dir()
        .map(|d| d.join("nvim/init.vim"))
        .filter(|p| p.exists());

    if nvim_config.is_some() {
        return nvim_config.unwrap().to_string_lossy().to_string();
    }

    // Fall back to regular vimrc
    dirs::home_dir()
        .map(|h| h.join(".vimrc"))
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "~/.vimrc".to_string())
}

/// Try to match a vim-related recipe
pub fn try_recipe(q: &str) -> Option<RecipeResult> {
    // Dark mode
    if (q.contains("dark") && q.contains("mode") && (q.contains("vim") || q.contains("neovim")))
        || (q.contains("dark") && q.contains("vim"))
        || (q.contains("vim") && q.contains("dark") && q.contains("theme"))
    {
        return Some(offer_dark_mode());
    }

    // Syntax highlighting
    if (q.contains("syntax") && q.contains("highlight"))
        || (q.contains("enable") && q.contains("syntax") && q.contains("vim"))
        || (q.contains("vim") && q.contains("color") && q.contains("syntax"))
    {
        return Some(offer_syntax_highlighting());
    }

    // Line numbers
    if (q.contains("line") && q.contains("number") && q.contains("vim"))
        || (q.contains("vim") && q.contains("show") && q.contains("number"))
    {
        return Some(offer_line_numbers());
    }

    // Mouse support
    if q.contains("vim") && q.contains("mouse") {
        return Some(offer_mouse_support());
    }

    // Tab settings
    if q.contains("vim") && (q.contains("tab") || q.contains("indent")) && q.contains("space") {
        return Some(offer_tab_settings());
    }

    None
}

fn offer_dark_mode() -> RecipeResult {
    let config = vec![
        "set background=dark".to_string(),
        "colorscheme slate".to_string(),
    ];

    store_pending("vim-dark-mode", PendingVimChange {
        config_lines: config.clone(),
        description: "Enable dark mode".to_string(),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I can add these settings to your vimrc:\n  {}\n\nThis will enable dark mode with the slate colorscheme.",
            config.join("\n  ")
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply these changes?".to_string()),
    }
}

fn offer_syntax_highlighting() -> RecipeResult {
    let config = vec![
        "syntax on".to_string(),
        "filetype plugin indent on".to_string(),
    ];

    store_pending("vim-syntax", PendingVimChange {
        config_lines: config.clone(),
        description: "Enable syntax highlighting".to_string(),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I can add these settings to your vimrc:\n  {}\n\nThis will enable syntax highlighting and filetype detection.",
            config.join("\n  ")
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply these changes?".to_string()),
    }
}

fn offer_line_numbers() -> RecipeResult {
    let config = vec![
        "set number".to_string(),
        "set relativenumber".to_string(),
    ];

    store_pending("vim-line-numbers", PendingVimChange {
        config_lines: config.clone(),
        description: "Enable line numbers".to_string(),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I can add these settings to your vimrc:\n  {}\n\nThis shows absolute line numbers with relative numbers for easier navigation.",
            config.join("\n  ")
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply these changes?".to_string()),
    }
}

fn offer_mouse_support() -> RecipeResult {
    let config = vec!["set mouse=a".to_string()];

    store_pending("vim-mouse", PendingVimChange {
        config_lines: config.clone(),
        description: "Enable mouse support".to_string(),
    });

    RecipeResult {
        success: true,
        message: "I can add this to your vimrc:\n  set mouse=a\n\nThis enables mouse support for clicking, scrolling, and selecting.".to_string(),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply this change?".to_string()),
    }
}

fn offer_tab_settings() -> RecipeResult {
    let config = vec![
        "set tabstop=4".to_string(),
        "set shiftwidth=4".to_string(),
        "set expandtab".to_string(),
        "set smartindent".to_string(),
    ];

    store_pending("vim-tabs", PendingVimChange {
        config_lines: config.clone(),
        description: "Configure tabs as 4 spaces".to_string(),
    });

    RecipeResult {
        success: true,
        message: format!(
            "I can add these settings to your vimrc:\n  {}\n\nThis converts tabs to 4 spaces and enables smart indentation.",
            config.join("\n  ")
        ),
        needs_confirmation: true,
        confirmation_prompt: Some("Apply these changes?".to_string()),
    }
}

fn store_pending(id: &str, change: PendingVimChange) {
    if let Ok(mut guard) = PENDING_VIM.write() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(id.to_string(), change);
    }
}

fn take_pending(id: &str) -> Option<PendingVimChange> {
    if let Ok(mut guard) = PENDING_VIM.write() {
        if let Some(map) = guard.as_mut() {
            return map.remove(id);
        }
    }
    None
}

/// Execute a confirmed vim recipe
pub fn execute_confirmed(recipe_id: &str) -> RecipeResult {
    let pending = match take_pending(recipe_id) {
        Some(p) => p,
        None => {
            return RecipeResult {
                success: false,
                message: "Recipe expired or not found. Please try again.".to_string(),
                needs_confirmation: false,
                confirmation_prompt: None,
            };
        }
    };

    let vimrc = vimrc_path();
    let content = pending.config_lines.join("\n");

    match append_to_file(&vimrc, &content, recipe_id, &pending.description, "vim") {
        Ok(record) => {
            info!("Applied vim recipe: {}", recipe_id);
            RecipeResult {
                success: true,
                message: format!(
                    "Done! Added to {}:\n  {}\n\nBackup created. Say 'undo {}' to revert.",
                    vimrc,
                    pending.config_lines.join("\n  "),
                    recipe_id
                ),
                needs_confirmation: false,
                confirmation_prompt: None,
            }
        }
        Err(e) => RecipeResult {
            success: false,
            message: format!("Failed to apply changes: {}", e),
            needs_confirmation: false,
            confirmation_prompt: None,
        },
    }
}
