//! Config edit intent detection (v0.0.264).
//!
//! Detects when user requests are config edits and provides ROUTING HINTS to specialists.
//! Anna learns actual solutions from specialists, not from hardcoded patterns here.
//!
//! v0.0.263: Added neovim, nano, helix, and colorscheme support.
//! v0.0.264: Refactored to provide routing hints instead of hardcoded answers.
//!           Anna now learns recipes from specialists (Sofia for Desktop team).

use serde::{Deserialize, Serialize};

// Re-export types from config_types for backward compatibility
pub use crate::config_types::{ConfigEditAction, ConfigIntent, ConfigTarget};

/// v0.0.264: Hint for specialists about what config change the user wants.
/// This is NOT the answer - it's context to help the specialist understand the request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigHint {
    /// The editor/app being configured (vim, nano, helix, etc.)
    pub app_id: String,
    /// What feature the user wants (syntax, line_numbers, theme, etc.)
    pub feature: ConfigFeatureHint,
    /// Whether they want to enable or disable it
    pub enable: bool,
    /// Optional parameter value (e.g., theme name, tab width)
    pub param: Option<String>,
}

/// v0.0.264: Feature categories for config hints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigFeatureHint {
    /// Syntax highlighting
    Syntax,
    /// Line numbers (absolute, relative, or off)
    LineNumbers,
    /// Color theme/colorscheme
    Theme,
    /// Indentation settings
    Indent,
    /// Mouse support
    Mouse,
    /// Word wrap
    WordWrap,
    /// Search highlighting
    SearchHighlight,
    /// Cursor line
    CursorLine,
    /// Status line
    StatusLine,
    /// Unknown feature - needs specialist interpretation
    Unknown,
}

impl ConfigFeatureHint {
    /// Parse from query keywords
    pub fn from_query(query: &str) -> Self {
        let q = query.to_lowercase();
        if q.contains("syntax") || q.contains("highlight") && !q.contains("search") {
            Self::Syntax
        } else if q.contains("line number") || q.contains("linenumber") {
            Self::LineNumbers
        } else if q.contains("theme") || q.contains("colorscheme") || q.contains("color scheme") {
            Self::Theme
        } else if q.contains("indent") || q.contains("tab") || q.contains("spaces") {
            Self::Indent
        } else if q.contains("mouse") {
            Self::Mouse
        } else if q.contains("wrap") {
            Self::WordWrap
        } else if q.contains("search") && q.contains("highlight") {
            Self::SearchHighlight
        } else if q.contains("cursor line") || q.contains("cursorline") {
            Self::CursorLine
        } else if q.contains("status") {
            Self::StatusLine
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for ConfigFeatureHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Syntax => "syntax highlighting",
            Self::LineNumbers => "line numbers",
            Self::Theme => "color theme",
            Self::Indent => "indentation",
            Self::Mouse => "mouse support",
            Self::WordWrap => "word wrap",
            Self::SearchHighlight => "search highlighting",
            Self::CursorLine => "cursor line",
            Self::StatusLine => "status line",
            Self::Unknown => "configuration",
        };
        write!(f, "{}", s)
    }
}

impl ConfigHint {
    /// v0.0.264: Create a config hint from a user query.
    /// Returns None if query doesn't look like a config request.
    pub fn from_query(query: &str) -> Option<Self> {
        let q = query.to_lowercase();

        // Detect app/editor
        let app_id = if q.contains("neovim") || q.contains("nvim") {
            "neovim"
        } else if q.contains("vim") {
            "vim"
        } else if q.contains("nano") {
            "nano"
        } else if q.contains("helix") || q.contains(" hx ") {
            "helix"
        } else if q.contains("emacs") {
            "emacs"
        } else if q.contains("micro") {
            "micro"
        } else if q.contains("code") || q.contains("vscode") {
            "vscode"
        } else {
            return None; // No editor detected
        };

        // Detect feature
        let feature = ConfigFeatureHint::from_query(query);

        // Detect enable/disable
        let enable = !q.contains("disable")
            && !q.contains("turn off")
            && !q.contains("no ")
            && !q.contains("hide ")
            && !q.contains("remove ");

        // Extract optional parameter (theme name, tab width, etc.)
        let param = extract_param(&q);

        Some(ConfigHint {
            app_id: app_id.to_string(),
            feature,
            enable,
            param,
        })
    }

    /// Build specialist hint text for the query.
    /// This helps the specialist (Sofia) understand what the user wants.
    pub fn to_specialist_context(&self) -> String {
        let action = if self.enable { "enable" } else { "disable" };
        let param_text = self.param.as_ref()
            .map(|p| format!(" (value: {})", p))
            .unwrap_or_default();

        format!(
            "User wants to {} {} in {}{}. Config file: {}",
            action,
            self.feature,
            self.app_id,
            param_text,
            config_path_for_app(&self.app_id)
        )
    }
}

/// Extract parameter value from query (theme name, tab width, etc.)
fn extract_param(query: &str) -> Option<String> {
    // Look for patterns like "theme gruvbox", "4 spaces", "tab width 2"
    let words: Vec<&str> = query.split_whitespace().collect();

    // Theme names after "theme" or "colorscheme"
    for (i, word) in words.iter().enumerate() {
        if (*word == "theme" || *word == "colorscheme") && i + 1 < words.len() {
            let next = words[i + 1];
            if !["in", "for", "on", "to"].contains(&next) {
                return Some(next.to_string());
            }
        }
    }

    // Tab width patterns
    if query.contains("2 spaces") || query.contains("2-space") {
        return Some("2".to_string());
    }
    if query.contains("4 spaces") || query.contains("4-space") {
        return Some("4".to_string());
    }

    None
}

/// Get config file path for an app
fn config_path_for_app(app_id: &str) -> &'static str {
    match app_id {
        "vim" => "~/.vimrc",
        "neovim" | "nvim" => "~/.config/nvim/init.vim",
        "nano" => "~/.nanorc",
        "helix" => "~/.config/helix/config.toml",
        "emacs" => "~/.emacs",
        "micro" => "~/.config/micro/settings.json",
        "vscode" | "code" => "settings.json (via GUI)",
        _ => "unknown",
    }
}

// v0.0.264: ConfigTarget, ConfigEditAction, ConfigIntent moved to config_types.rs
// These functions now delegate to seed recipes (used as bootstrap when no learned recipe exists)

use crate::config_seed_recipes::find_seed_recipe;

/// Detect if a query is requesting a vim config change.
/// v0.0.264: Now uses seed recipes (bootstrap, lower confidence).
pub fn detect_vim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    // Must mention vim (but not neovim)
    if !query_lower.contains("vim") || query_lower.contains("neovim") || query_lower.contains("nvim")
    {
        return None;
    }

    find_seed_recipe(query, "vim")
}

/// v0.0.264: Detect neovim config intent using seed recipes
pub fn detect_neovim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("neovim") && !query_lower.contains("nvim") {
        return None;
    }

    find_seed_recipe(query, "neovim")
}

/// v0.0.264: Detect nano config intent using seed recipes
pub fn detect_nano_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("nano") {
        return None;
    }

    find_seed_recipe(query, "nano")
}

/// v0.0.264: Detect helix config intent using seed recipes
pub fn detect_helix_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("helix") && !query_lower.contains("hx ") {
        return None;
    }

    find_seed_recipe(query, "helix")
}

/// v0.0.264: Detect emacs config intent using seed recipes
pub fn detect_emacs_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("emacs") {
        return None;
    }

    find_seed_recipe(query, "emacs")
}

/// Detect config intent from query and entities.
/// v0.0.264: Uses seed recipes as bootstrap, prefers learned recipes.
pub fn detect_config_intent(query: &str, entities: &[String]) -> Option<ConfigIntent> {
    // Check specific editors in order of specificity
    if let Some(intent) = detect_neovim_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_vim_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_nano_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_helix_config_intent(query) {
        return Some(intent);
    }
    if let Some(intent) = detect_emacs_config_intent(query) {
        return Some(intent);
    }

    // Check entities for editor mentions
    for entity in entities {
        let e = entity.to_lowercase();
        if e == "neovim" || e == "nvim" {
            return detect_neovim_config_intent(&format!("neovim {}", query));
        }
        if e == "vim" {
            return detect_vim_config_intent(&format!("vim {}", query));
        }
        if e == "nano" {
            return detect_nano_config_intent(&format!("nano {}", query));
        }
        if e == "helix" || e == "hx" {
            return detect_helix_config_intent(&format!("helix {}", query));
        }
        if e == "emacs" {
            return detect_emacs_config_intent(&format!("emacs {}", query));
        }
    }

    None
}

/// Check if a query is a config edit request (quick check) (v0.0.263: expanded)
pub fn is_config_edit_request(query: &str) -> bool {
    let q = query.to_lowercase();
    // Must have both an editor and an action
    let has_editor = q.contains("vim")
        || q.contains("neovim")
        || q.contains("nvim")
        || q.contains("nano")
        || q.contains("emacs")
        || q.contains("helix")
        || q.contains("micro");
    let has_action = q.contains("enable")
        || q.contains("disable")
        || q.contains("add")
        || q.contains("set")
        || q.contains("turn on")
        || q.contains("turn off")
        || q.contains("configure")
        || q.contains("theme")
        || q.contains("colorscheme")
        || q.contains("color scheme")
        || q.contains("syntax");
    has_editor && has_action
}

#[cfg(test)]
mod tests {
    use super::*;

    // v0.0.264: Core functionality tests

    #[test]
    fn test_config_hint_from_query_vim() {
        let hint = ConfigHint::from_query("enable syntax highlighting in vim").unwrap();
        assert_eq!(hint.app_id, "vim");
        assert_eq!(hint.feature, ConfigFeatureHint::Syntax);
        assert!(hint.enable);
    }

    #[test]
    fn test_config_hint_disable() {
        let hint = ConfigHint::from_query("disable line numbers in nano").unwrap();
        assert_eq!(hint.app_id, "nano");
        assert_eq!(hint.feature, ConfigFeatureHint::LineNumbers);
        assert!(!hint.enable);
    }

    #[test]
    fn test_config_hint_no_editor() {
        assert!(ConfigHint::from_query("enable syntax highlighting").is_none());
    }

    #[test]
    fn test_config_hint_specialist_context() {
        let hint = ConfigHint::from_query("enable syntax in vim").unwrap();
        let ctx = hint.to_specialist_context();
        assert!(ctx.contains("enable"));
        assert!(ctx.contains("vim"));
        assert!(ctx.contains(".vimrc"));
    }

    #[test]
    fn test_config_target_expand() {
        let target = ConfigTarget::vim();
        let path = target.expand_path();
        assert!(path.to_string_lossy().contains(".vimrc"));
        assert!(!path.to_string_lossy().contains("$HOME"));
    }

    #[test]
    fn test_is_config_edit_request() {
        assert!(is_config_edit_request("enable syntax in vim"));
        assert!(is_config_edit_request("add line numbers to vim"));
        assert!(!is_config_edit_request("what is vim"));
        assert!(!is_config_edit_request("enable something"));
    }

    // v0.0.264: Tests for ConfigEditAction and ConfigTarget moved to config_types.rs

    // v0.0.264: Seed recipe tests (lower confidence)

    #[test]
    fn test_detect_vim_uses_seed_recipe() {
        let intent = detect_vim_config_intent("enable syntax highlighting in vim").unwrap();
        assert_eq!(intent.target.app_id, "vim");
        // Seed recipes have lower confidence than learned recipes
        assert!(intent.confidence < 0.8);
    }

    #[test]
    fn test_detect_neovim_uses_seed_recipe() {
        let intent = detect_neovim_config_intent("nvim syntax highlighting").unwrap();
        assert_eq!(intent.target.app_id, "neovim");
        assert!(intent.confidence < 0.8);
    }

    #[test]
    fn test_neovim_not_detected_as_vim() {
        assert!(detect_vim_config_intent("neovim syntax highlighting").is_none());
    }

    #[test]
    fn test_is_config_edit_request_expanded() {
        assert!(is_config_edit_request("neovim syntax highlighting"));
        assert!(is_config_edit_request("nvim theme"));
        assert!(is_config_edit_request("helix colorscheme"));
        assert!(is_config_edit_request("disable syntax in nano"));
    }

    #[test]
    fn test_config_feature_hint_display() {
        assert_eq!(ConfigFeatureHint::Syntax.to_string(), "syntax highlighting");
        assert_eq!(ConfigFeatureHint::LineNumbers.to_string(), "line numbers");
        assert_eq!(ConfigFeatureHint::Theme.to_string(), "color theme");
    }
}
