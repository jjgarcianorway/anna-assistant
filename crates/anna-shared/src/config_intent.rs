//! Config edit intent detection (v0.0.27).
//!
//! Detects when user requests are config edits and extracts actionable information.
//! Maps user requests to change plans and recipes.

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

/// Known vim config patterns (query pattern -> line to add)
const VIM_SYNTAX_PATTERNS: &[(&str, &str)] = &[
    ("syntax highlighting", "syntax on"),
    ("enable syntax", "syntax on"),
    ("syntax on", "syntax on"),
    ("line numbers", "set number"),
    ("show line numbers", "set number"),
    ("enable line numbers", "set number"),
    ("relative numbers", "set relativenumber"),
    ("auto indent", "set autoindent"),
    ("enable autoindent", "set autoindent"),
    ("tab spaces", "set expandtab"),
    ("spaces instead of tabs", "set expandtab"),
    ("mouse support", "set mouse=a"),
    ("enable mouse", "set mouse=a"),
];

/// Detect if a query is requesting a vim config change
pub fn detect_vim_config_intent(query: &str) -> Option<ConfigIntent> {
    let query_lower = query.to_lowercase();

    // Must mention vim
    if !query_lower.contains("vim") {
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

/// Detect config intent from query and entities
pub fn detect_config_intent(query: &str, entities: &[String]) -> Option<ConfigIntent> {
    // Check vim first
    if let Some(intent) = detect_vim_config_intent(query) {
        return Some(intent);
    }

    // Check entities for editor mentions
    let has_vim = entities.iter().any(|e| e.to_lowercase() == "vim");
    if has_vim {
        // Try to detect intent from query alone
        return detect_vim_config_intent(&format!("vim {}", query));
    }

    // Future: add nano, bash, etc.
    None
}

/// Check if a query is a config edit request (quick check)
pub fn is_config_edit_request(query: &str) -> bool {
    let q = query.to_lowercase();
    // Must have both an editor and an action
    let has_editor = q.contains("vim") || q.contains("nano") || q.contains("emacs");
    let has_action = q.contains("enable")
        || q.contains("add")
        || q.contains("set")
        || q.contains("turn on")
        || q.contains("configure");
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
}
