//! Git recipe struct (v0.0.224).

use serde::{Deserialize, Serialize};

use super::types::{GitFeature, GitScope};

/// A git configuration recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRecipe {
    pub feature: GitFeature,
    pub scope: GitScope,
    pub description: String,
    /// Commands to run (git config ...)
    pub commands: Vec<String>,
    /// Parameters that need user input
    pub parameters: Vec<GitParameter>,
    pub rollback_hint: Option<String>,
}

/// A parameter that needs user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitParameter {
    pub name: String,
    pub prompt: String,
    pub default: Option<String>,
}

impl GitRecipe {
    pub fn new(feature: GitFeature, scope: GitScope, desc: &str, commands: Vec<&str>) -> Self {
        Self {
            feature,
            scope,
            description: desc.to_string(),
            commands: commands.into_iter().map(|s| s.to_string()).collect(),
            parameters: vec![],
            rollback_hint: None,
        }
    }

    pub fn with_param(mut self, name: &str, prompt: &str, default: Option<&str>) -> Self {
        self.parameters.push(GitParameter {
            name: name.to_string(),
            prompt: prompt.to_string(),
            default: default.map(|s| s.to_string()),
        });
        self
    }

    pub fn with_rollback(mut self, hint: &str) -> Self {
        self.rollback_hint = Some(hint.to_string());
        self
    }

    /// Check if recipe needs parameters
    pub fn needs_parameters(&self) -> bool {
        !self.parameters.is_empty()
    }

    /// Apply parameters to commands
    pub fn apply_params(&self, values: &[(String, String)]) -> Vec<String> {
        self.commands
            .iter()
            .map(|cmd| {
                let mut result = cmd.clone();
                for (name, value) in values {
                    result = result.replace(&format!("{{{}}}", name), value);
                }
                result
            })
            .collect()
    }
}
