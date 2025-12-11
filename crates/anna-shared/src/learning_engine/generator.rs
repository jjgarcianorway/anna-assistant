//! Recipe generator from tickets (v0.0.427).
//!
//! Creates new recipes from successful specialist responses:
//! - Extracts pattern from ticket
//! - Captures probes used
//! - Generates answer templates
//! - Tracks citations

use super::{
    AnswerKind, AnswerTemplate, LearnedRecipe, LogicType, RecipeInputs, RecipeLogic,
    RecipeOrigin, RecipePattern, RecipeProbe, RecipeSafety, RiskLevel,
};
use crate::specialist_v3::SpecialistResponse;
use crate::ticket_lifecycle::TicketRecord;
use std::collections::HashMap;

/// Generated recipe from a ticket
#[derive(Debug, Clone)]
pub struct GeneratedRecipe {
    /// The recipe
    pub recipe: LearnedRecipe,
    /// Confidence in the generation
    pub confidence: f32,
    /// Warnings about the generated recipe
    pub warnings: Vec<String>,
}

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
    let intent = super::extract_intent(&ticket.user_question);
    let keywords = extract_keywords(&ticket.user_question);

    // Infer domain from specialist or question
    let domain = infer_domain(&response.specialist.department, &ticket.user_question);

    // Generate recipe ID
    let recipe_id = generate_id(&intent, &ticket.ticket_id);

    // Extract parameters
    let params = super::extract_params(&ticket.user_question);
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
    let stopwords = ["the", "a", "an", "is", "are", "was", "were", "how", "what",
                     "why", "when", "where", "my", "your", "this", "that", "do",
                     "does", "can", "could", "would", "should", "be", "been"];

    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2 && !stopwords.contains(w))
        .take(10)
        .map(|s| s.to_string())
        .collect()
}

/// Infer domain from specialist department or question
fn infer_domain(department: &str, question: &str) -> String {
    // First try department
    let domain = match department.to_lowercase().as_str() {
        "desktop" => "desktop",
        "server" => "server",
        "network" => "network",
        "security" => "security",
        _ => "",
    };

    if !domain.is_empty() {
        // Add sub-domain from question
        let sub = infer_subdomain(question);
        if sub.is_empty() {
            domain.to_string()
        } else {
            format!("{}.{}", domain, sub)
        }
    } else {
        // Infer from question only
        let sub = infer_subdomain(question);
        if sub.is_empty() {
            "general".to_string()
        } else {
            sub
        }
    }
}

/// Infer sub-domain from question content
fn infer_subdomain(question: &str) -> String {
    let q = question.to_lowercase();

    if q.contains("systemd") || q.contains("service") || q.contains("systemctl") {
        "services.systemd"
    } else if q.contains("memory") || q.contains("ram") || q.contains("swap") {
        "performance.memory"
    } else if q.contains("disk") || q.contains("storage") || q.contains("mount") {
        "storage.disk"
    } else if q.contains("network") || q.contains("wifi") || q.contains("ethernet") {
        "network"
    } else if q.contains("pacman") || q.contains("package") || q.contains("install") {
        "packages"
    } else if q.contains("boot") || q.contains("startup") {
        "boot"
    } else if q.contains("process") || q.contains("cpu") {
        "performance.cpu"
    } else {
        ""
    }
    .to_string()
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

/// Build recipe inputs from extracted parameters
fn build_inputs(params: &[(String, String)]) -> RecipeInputs {
    let mut inputs = RecipeInputs::default();

    for (name, _value) in params {
        inputs.params.insert(
            name.clone(),
            format!("Extracted {}", name.replace('_', " ")),
        );
    }

    inputs
}

/// Build pattern from intent, keywords, and probes
fn build_pattern(
    intent: &str,
    keywords: &[String],
    probes: &[crate::specialist_v3::ProbeUsed],
) -> RecipePattern {
    let required_signals: Vec<String> = probes
        .iter()
        .filter(|p| p.status == crate::specialist_v3::ProbeStatus::Ok)
        .map(|p| p.id.clone())
        .collect();

    RecipePattern {
        intent: intent.to_string(),
        keywords: keywords.to_vec(),
        required_signals,
        optional_signals: vec![],
    }
}

/// Build probes from response
fn build_probes(probes_used: &[crate::specialist_v3::ProbeUsed]) -> Vec<RecipeProbe> {
    probes_used
        .iter()
        .map(|p| {
            let tool = p.raw_key.as_deref().unwrap_or(&p.id);
            RecipeProbe {
                id: p.id.clone(),
                tool: format!("probe.{}", tool.trim_start_matches("probe:")),
                params: vec![],
                optional: p.status != crate::specialist_v3::ProbeStatus::Ok,
                timeout_ms: 5000,
            }
        })
        .collect()
}

/// Build logic from analysis and recommendations
fn build_logic(
    analysis: &[String],
    recommendations: &[crate::specialist_v3::Recommendation],
) -> RecipeLogic {
    let mut steps = vec![];

    // Add analysis as steps
    for bullet in analysis {
        steps.push(bullet.clone());
    }

    // Add recommendations as steps
    for rec in recommendations {
        steps.push(format!("Recommendation: {}", rec.title));
    }

    RecipeLogic {
        logic_type: if steps.len() > 1 {
            LogicType::Sequential
        } else {
            LogicType::Template
        },
        answer_kind: AnswerKind::Diagnostic,
        steps,
        conditionals: HashMap::new(),
    }
}

/// Build answer template from response
fn build_answer_template(
    summary: &str,
    analysis: &[String],
    params: &[(String, String)],
) -> AnswerTemplate {
    // Replace specific values with placeholders
    let mut short = summary.to_string();
    let mut detailed = summary.to_string();

    if !analysis.is_empty() {
        detailed.push_str("\n\nAnalysis:\n");
        for bullet in analysis {
            detailed.push_str(&format!("• {}\n", bullet));
        }
    }

    // Extract variables from params
    let variables: Vec<String> = params.iter().map(|(k, _)| k.clone()).collect();

    // Try to generalize the template by replacing known values with placeholders
    for (name, value) in params {
        if !value.is_empty() && value.len() < 50 {
            short = short.replace(value, &format!("{{{{{}}}}}", name));
            detailed = detailed.replace(value, &format!("{{{{{}}}}}", name));
        }
    }

    AnswerTemplate {
        short,
        detailed,
        variables,
    }
}

/// Infer safety level from actions
fn infer_safety(actions: &[crate::specialist_v3::Action]) -> RecipeSafety {
    let mut max_risk = RiskLevel::Low;
    let mut requires_sudo = false;

    for action in actions {
        if action.run_as == crate::specialist_v3::RunAs::Root {
            requires_sudo = true;
        }

        let action_risk = match action.risk_level {
            crate::specialist_v3::RiskLevel::High => RiskLevel::High,
            crate::specialist_v3::RiskLevel::Medium => RiskLevel::Medium,
            crate::specialist_v3::RiskLevel::Low => RiskLevel::Low,
        };

        if action_risk > max_risk {
            max_risk = action_risk;
        }
    }

    RecipeSafety {
        risk: max_risk,
        needs_backup: max_risk >= RiskLevel::Medium,
        requires_sudo,
        warning: if max_risk >= RiskLevel::High {
            Some("This recipe may make significant system changes".to_string())
        } else {
            None
        },
    }
}

/// Build origin with citations
fn build_origin(
    ticket_id: &str,
    specialist: &str,
    citations: &[crate::specialist_v3::KnowledgeCitation],
    probes: &[crate::specialist_v3::ProbeUsed],
) -> RecipeOrigin {
    let mut sources = vec![];

    // Add knowledge citations
    for citation in citations {
        sources.push(format!("{}: {}", citation.source, citation.topic));
    }

    // Add probe sources
    for probe in probes {
        if probe.status == crate::specialist_v3::ProbeStatus::Ok {
            sources.push(probe.id.clone());
        }
    }

    RecipeOrigin {
        created_from_ticket: Some(ticket_id.to_string()),
        created_by: specialist.to_string(),
        created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        sources,
        is_seed: false,
    }
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
    fn test_domain_inference() {
        assert!(infer_domain("desktop", "check memory").contains("memory"));
        assert!(infer_domain("server", "systemd service").contains("systemd"));
        assert!(infer_domain("", "check disk space").contains("disk"));
    }

    #[test]
    fn test_keyword_extraction() {
        let keywords = extract_keywords("How much free memory is available?");
        assert!(keywords.contains(&"free".to_string()));
        assert!(keywords.contains(&"memory".to_string()));
        assert!(!keywords.contains(&"how".to_string())); // Stopword
    }

    #[test]
    fn test_safety_inference() {
        let actions = vec![crate::specialist_v3::Action {
            id: "act-1".to_string(),
            title: "Restart".to_string(),
            command: "sudo systemctl restart nginx".to_string(),
            run_as: crate::specialist_v3::RunAs::Root,
            risk_level: crate::specialist_v3::RiskLevel::Medium,
            auto_run: false,
        }];

        let safety = infer_safety(&actions);
        assert!(safety.requires_sudo);
        assert_eq!(safety.risk, RiskLevel::Medium);
    }
}
