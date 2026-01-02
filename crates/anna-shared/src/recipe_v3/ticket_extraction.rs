//! Ticket-to-recipe conversion (v0.0.423).
//!
//! Extracts reusable recipes from successful ticket resolutions.

use super::{RecipeAuthor, RecipeDomain, RecipeStep, RecipeV3, RecipeBuilder};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
