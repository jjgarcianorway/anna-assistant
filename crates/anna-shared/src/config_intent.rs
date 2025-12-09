//! Config edit intent detection (v0.0.263).
//!
//! Detects when user requests are config edits and extracts actionable information.
//! Maps user requests to change plans and recipes.
//!
//! v0.0.263: Added neovim, nano, helix, and colorscheme support.

use crate::change::{plan_ensure_line, ChangePlan};
use crate::recipe::{RecipeAction, RecipeKind, RecipeTarget};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Known config targets with their canonical paths
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigTarget {
    /// Application identifier (vim, nano, bash, etc.)
    pub app_id: String,
    /// Config file path template (uses $HOME)
    pub config_path: String,
}

impl ConfigTarget {
    pub fn vim() -> Self {
        Self {
            app_id: "vim".to_string(),
            config_path: "$HOME/.vimrc".to_string(),
        }
    }

    pub fn nano() -> Self {
        Self {
            app_id: "nano".to_string(),
            config_path: "$HOME/.nanorc".to_string(),
        }
    }

    pub fn bash() -> Self {
        Self {
            app_id: "bash".to_string(),
            config_path: "$HOME/.bashrc".to_string(),
        }
    }

    /// v0.0.263: Neovim config target
    pub fn neovim() -> Self {
        Self {
            app_id: "neovim".to_string(),
            config_path: "$HOME/.config/nvim/init.vim".to_string(),
        }
    }

    /// v0.0.263: Helix config target
    pub fn helix() -> Self {
        Self {
            app_id: "helix".to_string(),
            config_path: "$HOME/.config/helix/config.toml".to_string(),
        }
    }

    /// v0.0.263: Emacs config target
    pub fn emacs() -> Self {
        Self {
            app_id: "emacs".to_string(),
            config_path: "$HOME/.emacs".to_string(),
        }
    }

    /// Expand path template to actual path
    pub fn expand_path(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(self.config_path.replace("$HOME", &home).replace("~", &home))
    }

    /// Convert to RecipeTarget
    pub fn to_recipe_target(&self) -> RecipeTarget {
        RecipeTarget::new(&self.app_id, &self.config_path)
    }
}

/// Detected config edit action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigEditAction {
    /// Line to add/ensure
    pub line: String,
    /// Whether this is idempotent (ensure vs append)
    pub idempotent: bool,
}

impl ConfigEditAction {
    pub fn ensure_line(line: impl Into<String>) -> Self {
        Self {
            line: line.into(),
            idempotent: true,
        }
    }

    /// Convert to RecipeAction
    pub fn to_recipe_action(&self) -> RecipeAction {
        if self.idempotent {
            RecipeAction::EnsureLine {
                line: self.line.clone(),
            }
        } else {
            RecipeAction::AppendLine {
                line: self.line.clone(),
            }
        }
    }
}

/// Result of config intent detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigIntent {
    /// Target application and config file
    pub target: ConfigTarget,
    /// Action to perform
    pub action: ConfigEditAction,
    /// Recipe kind for persistence
    pub recipe_kind: RecipeKind,
    /// Confidence in detection (0.0-1.0)
    pub confidence: f32,
}

impl ConfigIntent {
    /// Create a change plan from this intent
    pub fn to_change_plan(&self) -> std::io::Result<ChangePlan> {
        let path = self.target.expand_path();
        plan_ensure_line(&path, &self.action.line)
    }
}

/// Known vim/neovim config patterns (query pattern -> line to add)
const VIM_SYNTAX_PATTERNS: &[(&str, &str)] = &[
    // Syntax highlighting
    ("syntax highlighting", "syntax on"),
    ("enable syntax", "syntax on"),
    ("syntax on", "syntax on"),
    ("turn on syntax", "syntax on"),
    ("disable syntax", "syntax off"),
    ("turn off syntax", "syntax off"),
    ("no syntax highlighting", "syntax off"),
    // Colorscheme
    ("colorscheme", "colorscheme desert"),
    ("color scheme", "colorscheme desert"),
    ("theme", "colorscheme desert"),
    // Line numbers
    ("line numbers", "set number"),
    ("show line numbers", "set number"),
    ("enable line numbers", "set number"),
    ("relative numbers", "set relativenumber"),
    ("relative line numbers", "set relativenumber"),
    ("no line numbers", "set nonumber"),
    ("hide line numbers", "set nonumber"),
    // Indentation
    ("auto indent", "set autoindent"),
    ("enable autoindent", "set autoindent"),
    ("smart indent", "set smartindent"),
    ("tab spaces", "set expandtab"),
    ("spaces instead of tabs", "set expandtab"),
    ("tabs to spaces", "set expandtab"),
    ("tab width", "set tabstop=4"),
    ("4 spaces", "set tabstop=4 shiftwidth=4 expandtab"),
    ("2 spaces", "set tabstop=2 shiftwidth=2 expandtab"),
    // Mouse
    ("mouse support", "set mouse=a"),
    ("enable mouse", "set mouse=a"),
    // Search
    ("highlight search", "set hlsearch"),
    ("incremental search", "set incsearch"),
    ("case insensitive", "set ignorecase"),
    // UI
    ("cursor line", "set cursorline"),
    ("show cursor line", "set cursorline"),
    ("status line", "set laststatus=2"),
    ("show status", "set laststatus=2"),
];

/// v0.0.263: Nano config patterns
const NANO_PATTERNS: &[(&str, &str)] = &[
    ("syntax highlighting", "include /usr/share/nano/*.nanorc"),
    ("enable syntax", "include /usr/share/nano/*.nanorc"),
    ("line numbers", "set linenumbers"),
    ("show line numbers", "set linenumbers"),
    ("auto indent", "set autoindent"),
    ("word wrap", "set softwrap"),
    ("smooth scroll", "set smooth"),
    ("mouse support", "set mouse"),
    ("enable mouse", "set mouse"),
    ("no line numbers", "unset linenumbers"),
    ("hide line numbers", "unset linenumbers"),
];

/// v0.0.263: Helix config patterns (TOML)
const HELIX_PATTERNS: &[(&str, &str)] = &[
    ("line numbers", "[editor]\nline-number = \"absolute\""),
    ("relative numbers", "[editor]\nline-number = \"relative\""),
    ("no line numbers", "[editor]\nline-number = \"off\""),
    ("word wrap", "[editor.soft-wrap]\nenable = true"),
    ("cursor line", "[editor]\ncursorline = true"),
    ("auto pairs", "[editor]\nauto-pairs = true"),
    ("theme", "theme = \"onedark\""),
    ("colorscheme", "theme = \"onedark\""),
    ("color scheme", "theme = \"onedark\""),
];

/// v0.0.263: Emacs config patterns
const EMACS_PATTERNS: &[(&str, &str)] = &[
    ("syntax highlighting", "(global-font-lock-mode t)"),
    ("enable syntax", "(global-font-lock-mode t)"),
    ("line numbers", "(global-display-line-numbers-mode t)"),
    ("show line numbers", "(global-display-line-numbers-mode t)"),
    ("no line numbers", "(global-display-line-numbers-mode -1)"),
    ("word wrap", "(global-visual-line-mode t)"),
    ("auto indent", "(electric-indent-mode t)"),
    ("theme", "(load-theme 'wombat t)"),
    ("colorscheme", "(load-theme 'wombat t)"),
];

/// Detect if a query is requesting a vim config change
pub fn detect_vim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    // Must mention vim (but not neovim)
    if !query_lower.contains("vim") || query_lower.contains("neovim") || query_lower.contains("nvim")
    {
        return None;
    }

    // Check for known patterns
    for (pattern, line) in VIM_SYNTAX_PATTERNS {
        if query_lower.contains(pattern) {
            return Some(ConfigIntent {
                target: ConfigTarget::vim(),
                action: ConfigEditAction::ensure_line(*line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: 0.9,
            });
        }
    }

    None
}

/// v0.0.263: Detect neovim config intent
pub fn detect_neovim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    // Must mention neovim or nvim
    if !query_lower.contains("neovim") && !query_lower.contains("nvim") {
        return None;
    }

    // Use same patterns as vim
    for (pattern, line) in VIM_SYNTAX_PATTERNS {
        if query_lower.contains(pattern) {
            return Some(ConfigIntent {
                target: ConfigTarget::neovim(),
                action: ConfigEditAction::ensure_line(*line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: 0.9,
            });
        }
    }

    None
}

/// v0.0.263: Detect nano config intent
pub fn detect_nano_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("nano") {
        return None;
    }

    for (pattern, line) in NANO_PATTERNS {
        if query_lower.contains(pattern) {
            return Some(ConfigIntent {
                target: ConfigTarget::nano(),
                action: ConfigEditAction::ensure_line(*line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: 0.9,
            });
        }
    }

    None
}

/// v0.0.263: Detect helix config intent
pub fn detect_helix_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("helix") && !query_lower.contains("hx ") {
        return None;
    }

    for (pattern, line) in HELIX_PATTERNS {
        if query_lower.contains(pattern) {
            return Some(ConfigIntent {
                target: ConfigTarget::helix(),
                action: ConfigEditAction::ensure_line(*line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: 0.85, // Slightly lower - TOML config is more complex
            });
        }
    }

    None
}

/// v0.0.263: Detect emacs config intent
pub fn detect_emacs_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    if !query_lower.contains("emacs") {
        return None;
    }

    for (pattern, line) in EMACS_PATTERNS {
        if query_lower.contains(pattern) {
            return Some(ConfigIntent {
                target: ConfigTarget::emacs(),
                action: ConfigEditAction::ensure_line(*line),
                recipe_kind: RecipeKind::ConfigEnsureLine,
                confidence: 0.85,
            });
        }
    }

    None
}

/// Detect config intent from query and entities (v0.0.263: expanded)
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
    let query_lower = query.to_lowercase();
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

    // Fall back to checking query for generic editor references
    if query_lower.contains("editor") && !entities.is_empty() {
        // Try to match based on entities
        return None;
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

    #[test]
    fn test_detect_vim_syntax() {
        let intent = detect_vim_config_intent("enable syntax highlighting in vim").unwrap();
        assert_eq!(intent.target.app_id, "vim");
        assert_eq!(intent.action.line, "syntax on");
        assert!(intent.action.idempotent);
        assert_eq!(intent.recipe_kind, RecipeKind::ConfigEnsureLine);
    }

    #[test]
    fn test_detect_vim_line_numbers() {
        let intent = detect_vim_config_intent("show line numbers in vim").unwrap();
        assert_eq!(intent.action.line, "set number");
    }

    #[test]
    fn test_no_vim_no_detection() {
        assert!(detect_vim_config_intent("enable syntax highlighting").is_none());
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

    #[test]
    fn test_detect_from_entities() {
        let intent = detect_config_intent("enable syntax highlighting", &["vim".to_string()]);
        assert!(intent.is_some());
        assert_eq!(intent.unwrap().action.line, "syntax on");
    }

    #[test]
    fn test_to_recipe_action() {
        let action = ConfigEditAction::ensure_line("syntax on");
        let recipe_action = action.to_recipe_action();
        match recipe_action {
            RecipeAction::EnsureLine { line } => assert_eq!(line, "syntax on"),
            _ => panic!("Expected EnsureLine"),
        }
    }

    #[test]
    fn test_to_recipe_target() {
        let target = ConfigTarget::vim();
        let recipe_target = target.to_recipe_target();
        assert_eq!(recipe_target.app_id, "vim");
        assert_eq!(recipe_target.config_path_template, "$HOME/.vimrc");
    }

    #[test]
    fn test_to_change_plan() {
        let intent = ConfigIntent {
            target: ConfigTarget::vim(),
            action: ConfigEditAction::ensure_line("syntax on"),
            recipe_kind: RecipeKind::ConfigEnsureLine,
            confidence: 0.9,
        };

        let plan = intent.to_change_plan().unwrap();
        assert!(plan.target_path.to_string_lossy().contains(".vimrc"));
        assert!(!plan.is_noop); // File doesn't exist, so it's not a noop
    }

    // v0.0.263: New editor tests
    #[test]
    fn test_detect_neovim_syntax() {
        let intent = detect_neovim_config_intent("enable syntax highlighting in neovim").unwrap();
        assert_eq!(intent.target.app_id, "neovim");
        assert_eq!(intent.action.line, "syntax on");
    }

    #[test]
    fn test_detect_nvim_alias() {
        let intent = detect_neovim_config_intent("nvim line numbers").unwrap();
        assert_eq!(intent.target.app_id, "neovim");
        assert_eq!(intent.action.line, "set number");
    }

    #[test]
    fn test_detect_nano_syntax() {
        let intent = detect_nano_config_intent("enable syntax highlighting in nano").unwrap();
        assert_eq!(intent.target.app_id, "nano");
        assert!(intent.action.line.contains("nano"));
    }

    #[test]
    fn test_detect_helix_line_numbers() {
        let intent = detect_helix_config_intent("helix line numbers").unwrap();
        assert_eq!(intent.target.app_id, "helix");
        assert!(intent.action.line.contains("line-number"));
    }

    #[test]
    fn test_detect_emacs_syntax() {
        let intent = detect_emacs_config_intent("emacs syntax highlighting").unwrap();
        assert_eq!(intent.target.app_id, "emacs");
        assert!(intent.action.line.contains("font-lock"));
    }

    #[test]
    fn test_vim_colorscheme() {
        let intent = detect_vim_config_intent("vim colorscheme").unwrap();
        assert!(intent.action.line.contains("colorscheme"));
    }

    #[test]
    fn test_neovim_not_detected_as_vim() {
        // "neovim" should not trigger vim detection
        assert!(detect_vim_config_intent("neovim syntax highlighting").is_none());
    }

    #[test]
    fn test_is_config_edit_request_expanded() {
        assert!(is_config_edit_request("neovim syntax highlighting"));
        assert!(is_config_edit_request("nvim theme"));
        assert!(is_config_edit_request("helix colorscheme"));
        assert!(is_config_edit_request("disable syntax in nano"));
    }
}
