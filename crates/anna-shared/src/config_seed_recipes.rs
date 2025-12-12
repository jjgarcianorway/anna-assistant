//! Seed recipes for editor configuration (v0.0.264).
//!
//! These are BOOTSTRAP recipes used when Anna hasn't learned from specialists yet.
//! Once Anna learns the correct answer from a specialist (like Sofia), the learned
//! recipe takes precedence over these seed recipes.
//!
//! v0.0.264: Extracted from config_intent.rs to support recipe learning model.

use crate::config_intent::{ConfigEditAction, ConfigIntent, ConfigTarget};
use crate::recipe::RecipeKind;

/// Seed recipe: known patterns for a specific editor
pub struct SeedRecipe {
    /// Query pattern to match (lowercase)
    pub pattern: &'static str,
    /// Config line to add
    pub line: &'static str,
    /// Confidence score (lower than learned recipes)
    pub confidence: f32,
}

/// Get seed recipes for vim
pub const VIM_SEED_RECIPES: &[SeedRecipe] = &[
    // Syntax highlighting
    SeedRecipe {
        pattern: "syntax highlighting",
        line: "syntax on",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "enable syntax",
        line: "syntax on",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "syntax on",
        line: "syntax on",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "turn on syntax",
        line: "syntax on",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "disable syntax",
        line: "syntax off",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "turn off syntax",
        line: "syntax off",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "no syntax highlighting",
        line: "syntax off",
        confidence: 0.7,
    },
    // Colorscheme
    SeedRecipe {
        pattern: "colorscheme",
        line: "colorscheme desert",
        confidence: 0.6,
    },
    SeedRecipe {
        pattern: "color scheme",
        line: "colorscheme desert",
        confidence: 0.6,
    },
    SeedRecipe {
        pattern: "theme",
        line: "colorscheme desert",
        confidence: 0.5,
    },
    // Line numbers
    SeedRecipe {
        pattern: "line numbers",
        line: "set number",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "show line numbers",
        line: "set number",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "enable line numbers",
        line: "set number",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "relative numbers",
        line: "set relativenumber",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "no line numbers",
        line: "set nonumber",
        confidence: 0.7,
    },
    // Indentation
    SeedRecipe {
        pattern: "auto indent",
        line: "set autoindent",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "smart indent",
        line: "set smartindent",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "tab spaces",
        line: "set expandtab",
        confidence: 0.6,
    },
    SeedRecipe {
        pattern: "tabs to spaces",
        line: "set expandtab",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "4 spaces",
        line: "set tabstop=4 shiftwidth=4 expandtab",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "2 spaces",
        line: "set tabstop=2 shiftwidth=2 expandtab",
        confidence: 0.7,
    },
    // Mouse
    SeedRecipe {
        pattern: "mouse support",
        line: "set mouse=a",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "enable mouse",
        line: "set mouse=a",
        confidence: 0.7,
    },
    // Search
    SeedRecipe {
        pattern: "highlight search",
        line: "set hlsearch",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "incremental search",
        line: "set incsearch",
        confidence: 0.7,
    },
    // UI
    SeedRecipe {
        pattern: "cursor line",
        line: "set cursorline",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "status line",
        line: "set laststatus=2",
        confidence: 0.7,
    },
];

/// Get seed recipes for nano
pub const NANO_SEED_RECIPES: &[SeedRecipe] = &[
    SeedRecipe {
        pattern: "syntax highlighting",
        line: "include /usr/share/nano/*.nanorc",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "enable syntax",
        line: "include /usr/share/nano/*.nanorc",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "line numbers",
        line: "set linenumbers",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "show line numbers",
        line: "set linenumbers",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "auto indent",
        line: "set autoindent",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "word wrap",
        line: "set softwrap",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "smooth scroll",
        line: "set smooth",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "mouse support",
        line: "set mouse",
        confidence: 0.7,
    },
    SeedRecipe {
        pattern: "no line numbers",
        line: "unset linenumbers",
        confidence: 0.7,
    },
];

/// Get seed recipes for helix
pub const HELIX_SEED_RECIPES: &[SeedRecipe] = &[
    SeedRecipe {
        pattern: "line numbers",
        line: "[editor]\nline-number = \"absolute\"",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "relative numbers",
        line: "[editor]\nline-number = \"relative\"",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "no line numbers",
        line: "[editor]\nline-number = \"off\"",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "word wrap",
        line: "[editor.soft-wrap]\nenable = true",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "cursor line",
        line: "[editor]\ncursorline = true",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "auto pairs",
        line: "[editor]\nauto-pairs = true",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "theme",
        line: "theme = \"onedark\"",
        confidence: 0.5,
    },
    SeedRecipe {
        pattern: "colorscheme",
        line: "theme = \"onedark\"",
        confidence: 0.5,
    },
];

/// Get seed recipes for emacs
pub const EMACS_SEED_RECIPES: &[SeedRecipe] = &[
    SeedRecipe {
        pattern: "syntax highlighting",
        line: "(global-font-lock-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "enable syntax",
        line: "(global-font-lock-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "line numbers",
        line: "(global-display-line-numbers-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "show line numbers",
        line: "(global-display-line-numbers-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "no line numbers",
        line: "(global-display-line-numbers-mode -1)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "word wrap",
        line: "(global-visual-line-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "auto indent",
        line: "(electric-indent-mode t)",
        confidence: 0.65,
    },
    SeedRecipe {
        pattern: "theme",
        line: "(load-theme 'wombat t)",
        confidence: 0.5,
    },
];

/// Try to find a seed recipe for a query.
/// Returns None if no seed recipe matches.
/// Seed recipes have lower confidence than learned recipes.
pub fn find_seed_recipe(query: &str, editor: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    let (recipes, target) = match editor {
        "vim" | "vi" => (VIM_SEED_RECIPES, ConfigTarget::vim()),
        "neovim" | "nvim" => (VIM_SEED_RECIPES, ConfigTarget::neovim()),
        "nano" => (NANO_SEED_RECIPES, ConfigTarget::nano()),
        "helix" | "hx" => (HELIX_SEED_RECIPES, ConfigTarget::helix()),
        "emacs" => (EMACS_SEED_RECIPES, ConfigTarget::emacs()),
        _ => return None,
    };

    for recipe in recipes {
        if query_lower.contains(recipe.pattern) {
            return Some(ConfigIntent {
                target,
                action: ConfigEditAction::ensure_line(recipe.line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: recipe.confidence,
            });
        }
    }

    None
}

/// Check if there's a seed recipe for this editor/feature combo.
/// Used to decide whether to use seed recipe OR ask specialist.
pub fn has_seed_recipe(query: &str, editor: &str) -> bool {
    find_seed_recipe(query, editor).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_vim_seed_recipe() {
        let intent = find_seed_recipe("enable syntax highlighting", "vim").unwrap();
        assert_eq!(intent.target.app_id, "vim");
        assert_eq!(intent.action.line, "syntax on");
        assert!(intent.confidence < 0.9); // Seed recipes have lower confidence
    }

    #[test]
    fn test_find_nano_seed_recipe() {
        let intent = find_seed_recipe("line numbers", "nano").unwrap();
        assert_eq!(intent.target.app_id, "nano");
        assert_eq!(intent.action.line, "set linenumbers");
    }

    #[test]
    fn test_no_seed_recipe_for_unknown() {
        assert!(find_seed_recipe("enable syntax", "unknown_editor").is_none());
    }

    #[test]
    fn test_seed_confidence_lower_than_learned() {
        // Seed recipes should have confidence < 0.8
        // Learned recipes have confidence >= 0.8
        let intent = find_seed_recipe("syntax highlighting", "vim").unwrap();
        assert!(intent.confidence < 0.8);
    }
}
