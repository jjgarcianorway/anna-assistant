//! Core recipe builder (v0.0.423).
//!
//! Provides the main RecipeBuilder for creating recipes with validation.

use super::{
    ConfirmationPolicy, RecipeAuthor, RecipeCondition, RecipeDomain, RecipeRiskLevel, RecipeStep,
    RecipeV3,
};

/// Builder for creating recipes from tickets
pub struct RecipeBuilder {
    /// Recipe being built
    recipe: RecipeV3,
    /// Validation errors
    errors: Vec<String>,
}

impl RecipeBuilder {
    /// Start building a new recipe
    pub fn new(id: &str) -> Self {
        Self {
            recipe: RecipeV3::new(id, ""),
            errors: vec![],
        }
    }

    /// Set title
    pub fn title(mut self, title: &str) -> Self {
        self.recipe.title = title.to_string();
        self
    }

    /// Set description
    pub fn description(mut self, desc: &str) -> Self {
        self.recipe.description = desc.to_string();
        self
    }

    /// Set as learned from ticket
    pub fn learned_from(mut self, ticket_id: &str) -> Self {
        self.recipe.origin = super::RecipeOrigin::LearnedFromTicket;
        self.recipe.source_ticket_id = Some(ticket_id.to_string());
        self
    }

    /// Set author
    pub fn author(mut self, author: RecipeAuthor) -> Self {
        self.recipe.author = author;
        self
    }

    /// Set domain
    pub fn domain(mut self, domain: RecipeDomain) -> Self {
        self.recipe.matcher.domain = domain;
        self
    }

    /// Add intent
    pub fn intent(mut self, intent: &str) -> Self {
        self.recipe.matcher.intents.push(intent.to_string());
        self
    }

    /// Add keyword
    pub fn keyword(mut self, keyword: &str) -> Self {
        self.recipe.matcher.keywords.push(keyword.to_string());
        self
    }

    /// Set similarity key
    pub fn similarity_key(mut self, key: &str) -> Self {
        self.recipe.matcher.similarity_key = key.to_string();
        self
    }

    /// Add precondition
    pub fn precondition(mut self, cond: RecipeCondition) -> Self {
        self.recipe.preconditions.push(cond);
        self
    }

    /// Add step
    pub fn step(mut self, step: RecipeStep) -> Self {
        self.recipe.steps.push(step);
        self
    }

    /// Add postcondition
    pub fn postcondition(mut self, cond: RecipeCondition) -> Self {
        self.recipe.postconditions.push(cond);
        self
    }

    /// Set risk level
    pub fn risk(mut self, risk: RecipeRiskLevel) -> Self {
        self.recipe.risk_level = risk;
        self
    }

    /// Set confirmation policy
    pub fn confirmation(mut self, policy: ConfirmationPolicy) -> Self {
        self.recipe.confirmation = policy;
        self
    }

    /// Add citation
    pub fn citation(mut self, citation: &str) -> Self {
        self.recipe.citations.push(citation.to_string());
        self
    }

    /// Add tag
    pub fn tag(mut self, tag: &str) -> Self {
        self.recipe.tags.push(tag.to_string());
        self
    }

    /// Add parameter
    pub fn parameter(mut self, name: &str, description: &str) -> Self {
        self.recipe
            .parameters
            .insert(name.to_string(), description.to_string());
        self
    }

    /// Validate and build the recipe
    pub fn build(mut self) -> Result<RecipeV3, BuildError> {
        self.validate();

        if !self.errors.is_empty() {
            return Err(BuildError::ValidationFailed(self.errors));
        }

        Ok(self.recipe)
    }

    /// Validate the recipe
    fn validate(&mut self) {
        // Must have ID
        if self.recipe.id.is_empty() {
            self.errors.push("Recipe must have an ID".to_string());
        }

        // Must have title
        if self.recipe.title.is_empty() {
            self.errors.push("Recipe must have a title".to_string());
        }

        // Must have at least one step
        if self.recipe.steps.is_empty() {
            self.errors
                .push("Recipe must have at least one step".to_string());
        }

        // Must have at least one intent
        if self.recipe.matcher.intents.is_empty() {
            self.errors
                .push("Recipe must have at least one intent".to_string());
        }

        // Limit number of steps
        if self.recipe.steps.len() > super::MAX_RECIPE_STEPS {
            self.errors.push(format!(
                "Recipe has too many steps (max {})",
                super::MAX_RECIPE_STEPS
            ));
        }

        // Check for dangerous commands without high risk level
        let max_risk = self
            .recipe
            .steps
            .iter()
            .map(|s| s.risk_level())
            .max()
            .unwrap_or(RecipeRiskLevel::None);

        if max_risk > self.recipe.risk_level {
            self.recipe.risk_level = max_risk;
        }

        // High risk recipes must have confirmation
        if self.recipe.risk_level == RecipeRiskLevel::High
            && self.recipe.confirmation == ConfirmationPolicy::Never
        {
            self.errors
                .push("High-risk recipes must require confirmation".to_string());
        }
    }
}

/// Build error
#[derive(Debug, Clone)]
pub enum BuildError {
    ValidationFailed(Vec<String>),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationFailed(errors) => {
                write!(f, "Recipe validation failed: {}", errors.join(", "))
            }
        }
    }
}

impl std::error::Error for BuildError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let recipe = RecipeBuilder::new("test-1")
            .title("Test Recipe")
            .intent("test")
            .step(RecipeStep::Explain {
                text: "Hello".to_string(),
                citation: None,
            })
            .build();

        assert!(recipe.is_ok());
        let r = recipe.unwrap();
        assert_eq!(r.id, "test-1");
        assert_eq!(r.title, "Test Recipe");
    }

    #[test]
    fn test_builder_validation() {
        // Missing title
        let r1 = RecipeBuilder::new("test")
            .intent("test")
            .step(RecipeStep::Explain {
                text: "Hi".to_string(),
                citation: None,
            })
            .build();
        assert!(r1.is_err());

        // Missing intent
        let r2 = RecipeBuilder::new("test")
            .title("Test")
            .step(RecipeStep::Explain {
                text: "Hi".to_string(),
                citation: None,
            })
            .build();
        assert!(r2.is_err());

        // Missing steps
        let r3 = RecipeBuilder::new("test")
            .title("Test")
            .intent("test")
            .build();
        assert!(r3.is_err());
    }
}
