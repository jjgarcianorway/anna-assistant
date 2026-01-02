//! Core recipe generation logic (v0.0.427).
//!
//! Creates new recipes from successful specialist responses:
//! - Extracts pattern from ticket
//! - Captures probes used
//! - Generates answer templates
//! - Tracks citations

use super::builder::*;
use super::inference::*;
use super::types::GeneratedRecipe;
use crate::learning_engine::LearnedRecipe;
use crate::specialist_v3::SpecialistResponse;
use crate::ticket_lifecycle::TicketRecord;

/// Generate a recipe from a successful ticket
pub fn generate_from_ticket(
    ticket: &TicketRecord,
    response: &SpecialistResponse,
) -> Result<GeneratedRecipe, String> {
    // Validate inputs
    if ticket.ticket_id.is_empty() {
        return Err("Ticket ID is required".to_string());
    }
    if response.summary.is_empty() {
        return Err("Response summary is required".to_string());
    }

    let mut warnings = vec![];

    // Extract intent and keywords
    let intent = crate::learning_engine::extract_intent(&ticket.user_question);
    let keywords = extract_keywords(&ticket.user_question);

    // Infer domain from specialist or question
    let domain = infer_domain(&response.specialist.department, &ticket.user_question);

    // Generate recipe ID
    let recipe_id = generate_id(&intent, &ticket.ticket_id);

    // Extract parameters
    let params = crate::learning_engine::extract_params(&ticket.user_question);
    let inputs = build_inputs(&params);

    // Build pattern
    let pattern = build_pattern(&intent, &keywords, &response.probes_used);

    // Build probes from response
    let probes = build_probes(&response.probes_used);

    if probes.is_empty() {
        warnings.push("No probes captured - recipe may not be reproducible".to_string());
    }

    // Build logic
    let logic = build_logic(&response.analysis, &response.recommendations);

    // Build answer template
    let answer_template = build_answer_template(&response.summary, &response.analysis, &params);

    // Determine safety
    let safety = infer_safety(&response.actions);

    // Build origin with citations
    let origin = build_origin(
        &ticket.ticket_id,
        ticket.lead_specialist().unwrap_or("unknown"),
        &response.knowledge_citations,
        &response.probes_used,
    );

    // Build the recipe
    let recipe = LearnedRecipe {
        id: recipe_id,
        domain,
        pattern,
        inputs,
        probes,
        logic,
        answer_template,
        safety,
        origin,
        stats: Default::default(),
        version: 1,
        enabled: true,
    };

    Ok(GeneratedRecipe {
        recipe,
        confidence: response.confidence,
        warnings,
    })
}

/// Extract keywords from question
fn extract_keywords(text: &str) -> Vec<String> {
    let stopwords = [
        "the", "a", "an", "is", "are", "was", "were", "how", "what", "why", "when", "where", "my",
        "your", "this", "that", "do", "does", "can", "could", "would", "should", "be", "been",
    ];

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .take(10)
        .map(|s| s.to_string())
        .collect()
}

/// Generate a unique recipe ID
fn generate_id(intent: &str, ticket_id: &str) -> String {
    let prefix = ticket_id
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(4)
        .collect::<String>()
        .to_lowercase();

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() % 10000)
        .unwrap_or(0);

    format!("{}-{}-{}", intent, prefix, timestamp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_v3::{ProbeStatus, ProbeUsed, SpecialistIdentity};

    fn make_ticket(question: &str) -> TicketRecord {
        let mut ticket = TicketRecord::new("TEST-001", question);
        ticket.escalation_chain.push("desktop.junior".to_string());
        ticket
    }

    fn make_response(summary: &str) -> SpecialistResponse {
        SpecialistResponse {
            ticket_id: "TEST-001".to_string(),
            specialist: SpecialistIdentity {
                name: "Sofia".to_string(),
                role: "System Admin".to_string(),
                department: "desktop".to_string(),
                ..Default::default()
            },
            status: crate::specialist_v3::ResponseStatus::Success,
            summary: summary.to_string(),
            confidence: 0.9,
            probes_used: vec![ProbeUsed {
                id: "probe:free".to_string(),
                status: ProbeStatus::Ok,
                description: "Memory check".to_string(),
                raw_key: Some("free".to_string()),
            }],
            analysis: vec!["Memory usage is healthy".to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_from_ticket() {
        let ticket = make_ticket("How much RAM is available?");
        let response = make_response("Available RAM: 16GB");

        let result = generate_from_ticket(&ticket, &response);
        assert!(result.is_ok());

        let generated = result.unwrap();
        assert!(generated.recipe.id.contains("check_free_ram"));
        assert!(!generated.recipe.probes.is_empty());
    }

    #[test]
    fn test_keyword_extraction() {
        let keywords = extract_keywords("How much free memory is available?");
        assert!(keywords.contains(&"free".to_string()));
        assert!(keywords.contains(&"memory".to_string()));
        assert!(!keywords.contains(&"how".to_string())); // Stopword
    }
}
