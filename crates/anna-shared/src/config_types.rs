//! Config types for editor configuration (v0.0.264).
//!
//! Core types for representing config edit targets and actions.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_target_expand() {
        let target = ConfigTarget::vim();
        let path = target.expand_path();
        assert!(path.to_string_lossy().contains(".vimrc"));
        assert!(!path.to_string_lossy().contains("$HOME"));
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
    fn test_neovim_config_path() {
        let target = ConfigTarget::neovim();
        assert!(target.config_path.contains("nvim"));
    }

    #[test]
    fn test_helix_config_path() {
        let target = ConfigTarget::helix();
        assert!(target.config_path.contains("helix"));
        assert!(target.config_path.contains("toml"));
    }
}
