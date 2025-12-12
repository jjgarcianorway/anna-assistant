//! Ticket-to-recipe builder (v0.0.423).
//!
//! Converts successful ticket resolutions into reusable recipes.
//! Strict learning rules prevent learning garbage.

use super::{
    ConfirmationPolicy, RecipeAuthor, RecipeCondition, RecipeDomain, RecipeMatcher, RecipeOrigin,
    RecipeRiskLevel, RecipeStep, RecipeV3,
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
        self.recipe.origin = RecipeOrigin::LearnedFromTicket;
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

/// Ticket data for recipe extraction
#[derive(Debug, Clone, Default)]
pub struct TicketData {
    pub id: String,
    pub question: String,
    pub intent: Option<String>,
    pub domain: Option<String>,
    pub entities: Vec<String>,
    pub keywords: Vec<String>,
    pub commands_run: Vec<CommandRecord>,
    pub answer: Option<String>,
    pub citations: Vec<String>,
    pub success: bool,
}

/// Record of a command that was run
#[derive(Debug, Clone)]
pub struct CommandRecord {
    pub command: String,
    pub output: Option<String>,
    pub success: bool,
    pub is_probe: bool,
}

/// Extract a recipe from ticket data
pub fn extract_recipe_from_ticket(ticket: &TicketData) -> Option<RecipeV3> {
    // Don't learn from failed tickets
    if !ticket.success {
        return None;
    }

    // Must have meaningful commands
    if ticket.commands_run.is_empty() {
        return None;
    }

    // Must have intent
    let intent = ticket.intent.as_ref()?;

    // Build recipe
    let id = format!("learned-{}", ticket.id);
    let title = generate_title(intent, &ticket.entities);

    let mut builder = RecipeBuilder::new(&id)
        .title(&title)
        .learned_from(&ticket.id)
        .author(RecipeAuthor::System)
        .intent(intent);

    // Set domain if detected
    if let Some(ref domain) = ticket.domain {
        builder = builder.domain(RecipeDomain::from_str(domain));
    }

    // Add keywords from question
    for kw in &ticket.keywords {
        builder = builder.keyword(kw);
    }

    // Build similarity key
    let sim_key = format!("{} {}", intent, ticket.entities.join(" "));
    builder = builder.similarity_key(&sim_key);

    // Add commands as steps
    for cmd in &ticket.commands_run {
        let step = if cmd.is_probe {
            RecipeStep::RunProbe {
                probe: cmd.command.clone(),
                output_var: "probe_result".to_string(),
                description: format!("Probe: {}", truncate(&cmd.command, 30)),
            }
        } else {
            RecipeStep::RunCommand {
                command: cmd.command.clone(),
                description: format!("Execute: {}", truncate(&cmd.command, 30)),
                risk: None,
                capture_output: true,
                output_var: None,
            }
        };
        builder = builder.step(step);
    }

    // Add answer as explanation if present
    if let Some(ref answer) = ticket.answer {
        if !answer.is_empty() {
            builder = builder.step(RecipeStep::Explain {
                text: answer.clone(),
                citation: ticket.citations.first().cloned(),
            });
        }
    }

    // Add citations
    for citation in &ticket.citations {
        builder = builder.citation(citation);
    }

    // Try to build
    builder.build().ok()
}

/// Generate a recipe title from intent and entities
fn generate_title(intent: &str, entities: &[String]) -> String {
    let intent_cap = capitalize(intent);
    if entities.is_empty() {
        intent_cap
    } else {
        format!("{} {}", intent_cap, entities.join(" "))
    }
}

/// Capitalize first letter
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Truncate string for display
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}

/// Seed recipes for common operations
pub fn seed_recipes() -> Vec<RecipeV3> {
    vec![
        // Service restart
        RecipeBuilder::new("restart-service")
            .title("Restart Service")
            .description("Restart a systemd service")
            .domain(RecipeDomain::Systemd)
            .intent("restart")
            .keyword("restart")
            .keyword("service")
            .keyword("systemctl")
            .similarity_key("restart service")
            .parameter("service", "Name of the service to restart")
            .precondition(RecipeCondition::CommandExists {
                command: "systemctl".to_string(),
            })
            .step(RecipeStep::RunCommand {
                command: "sudo systemctl restart ${service}".to_string(),
                description: "Restart the service".to_string(),
                risk: Some(RecipeRiskLevel::Medium),
                capture_output: true,
                output_var: None,
            })
            .step(RecipeStep::RunProbe {
                probe: "systemctl is-active ${service}".to_string(),
                output_var: "status".to_string(),
                description: "Check service status".to_string(),
            })
            .postcondition(RecipeCondition::ServiceState {
                service: "${service}".to_string(),
                state: "running".to_string(),
            })
            .risk(RecipeRiskLevel::Medium)
            .confirmation(ConfirmationPolicy::Once)
            .citation("man systemctl")
            .tag("systemd")
            .build()
            .unwrap(),
        // Service status
        RecipeBuilder::new("check-service-status")
            .title("Check Service Status")
            .description("Check if a service is running")
            .domain(RecipeDomain::Systemd)
            .intent("status")
            .intent("check")
            .keyword("status")
            .keyword("running")
            .keyword("service")
            .similarity_key("check service status")
            .parameter("service", "Name of the service to check")
            .step(RecipeStep::RunProbe {
                probe: "systemctl status ${service} --no-pager".to_string(),
                output_var: "status".to_string(),
                description: "Get service status".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man systemctl")
            .tag("systemd")
            .build()
            .unwrap(),
        // Package install
        RecipeBuilder::new("install-package")
            .title("Install Package")
            .description("Install a package with pacman")
            .domain(RecipeDomain::Package)
            .intent("install")
            .keyword("install")
            .keyword("package")
            .keyword("pacman")
            .similarity_key("install package")
            .parameter("package", "Name of the package to install")
            .precondition(RecipeCondition::CommandExists {
                command: "pacman".to_string(),
            })
            .step(RecipeStep::RunCommand {
                command: "sudo pacman -S --noconfirm ${package}".to_string(),
                description: "Install the package".to_string(),
                risk: Some(RecipeRiskLevel::Low),
                capture_output: true,
                output_var: None,
            })
            .postcondition(RecipeCondition::PackageInstalled {
                package: "${package}".to_string(),
            })
            .risk(RecipeRiskLevel::Low)
            .confirmation(ConfirmationPolicy::Once)
            .citation("man pacman")
            .tag("pacman")
            .build()
            .unwrap(),
        // Disk usage
        RecipeBuilder::new("check-disk-usage")
            .title("Check Disk Usage")
            .description("Show disk space usage")
            .domain(RecipeDomain::Disk)
            .intent("check")
            .intent("status")
            .keyword("disk")
            .keyword("space")
            .keyword("usage")
            .keyword("storage")
            .similarity_key("check disk usage")
            .step(RecipeStep::RunProbe {
                probe: "df -h".to_string(),
                output_var: "disk_usage".to_string(),
                description: "Get disk usage".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man df")
            .tag("disk")
            .build()
            .unwrap(),
        // Memory usage
        RecipeBuilder::new("check-memory-usage")
            .title("Check Memory Usage")
            .description("Show memory and swap usage")
            .domain(RecipeDomain::Memory)
            .intent("check")
            .intent("status")
            .keyword("memory")
            .keyword("ram")
            .keyword("swap")
            .keyword("usage")
            .similarity_key("check memory usage")
            .step(RecipeStep::RunProbe {
                probe: "free -h".to_string(),
                output_var: "memory".to_string(),
                description: "Get memory usage".to_string(),
            })
            .risk(RecipeRiskLevel::None)
            .confirmation(ConfirmationPolicy::Never)
            .citation("man free")
            .tag("memory")
            .build()
            .unwrap(),
    ]
}

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

    #[test]
    fn test_extract_from_ticket() {
        let ticket = TicketData {
            id: "ticket-123".to_string(),
            question: "How do I restart nginx?".to_string(),
            intent: Some("restart".to_string()),
            domain: Some("systemd".to_string()),
            entities: vec!["nginx".to_string()],
            keywords: vec!["restart".to_string(), "nginx".to_string()],
            commands_run: vec![CommandRecord {
                command: "sudo systemctl restart nginx".to_string(),
                output: Some("".to_string()),
                success: true,
                is_probe: false,
            }],
            answer: Some("Service restarted".to_string()),
            citations: vec!["man systemctl".to_string()],
            success: true,
        };

        let recipe = extract_recipe_from_ticket(&ticket);
        assert!(recipe.is_some());

        let r = recipe.unwrap();
        assert!(r.id.contains("ticket-123"));
        assert!(r.matcher.intents.contains(&"restart".to_string()));
    }

    #[test]
    fn test_seed_recipes() {
        let seeds = seed_recipes();
        assert!(!seeds.is_empty());

        // All should be valid
        for recipe in &seeds {
            assert!(!recipe.id.is_empty());
            assert!(!recipe.title.is_empty());
            assert!(!recipe.steps.is_empty());
        }
    }
}
