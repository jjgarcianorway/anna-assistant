//! Answer template rendering (v0.0.427).
//!
//! Provides template substitution for recipe answers with:
//! - Variable substitution using {{variable}} syntax
//! - Short and detailed answer variants
//! - Variable tracking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Answer templates for a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnswerTemplate {
    /// Short one-line answer
    pub short: String,
    /// Detailed answer with explanation
    pub detailed: String,
    /// Variables available for substitution
    #[serde(default)]
    pub variables: Vec<String>,
}

impl AnswerTemplate {
    /// Create a new template
    pub fn new(short: &str, detailed: &str) -> Self {
        Self {
            short: short.to_string(),
            detailed: detailed.to_string(),
            variables: vec![],
        }
    }

    /// Add available variable
    pub fn with_variable(mut self, var: &str) -> Self {
        self.variables.push(var.to_string());
        self
    }

    /// Render short template with values
    pub fn render_short(&self, values: &HashMap<String, String>) -> String {
        substitute_template(&self.short, values)
    }

    /// Render detailed template with values
    pub fn render_detailed(&self, values: &HashMap<String, String>) -> String {
        substitute_template(&self.detailed, values)
    }
}

/// Substitute {{variable}} placeholders in template
fn substitute_template(template: &str, values: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in values {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_substitution() {
        let template = AnswerTemplate::new(
            "Service {{service_name}} is {{state}}",
            "Details: {{details}}",
        );

        let mut values = HashMap::new();
        values.insert("service_name".to_string(), "nginx".to_string());
        values.insert("state".to_string(), "running".to_string());

        let short = template.render_short(&values);
        assert_eq!(short, "Service nginx is running");
    }
}
