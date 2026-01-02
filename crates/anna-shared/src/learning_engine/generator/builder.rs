//! Recipe component builders (v0.0.427).

use crate::learning_engine::{
    AnswerKind, AnswerTemplate, LogicType, RecipeInputs, RecipeLogic, RecipeOrigin, RecipePattern,
    RecipeProbe,
};
use std::collections::HashMap;

/// Build recipe inputs from extracted parameters
pub fn build_inputs(params: &[(String, String)]) -> RecipeInputs {
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
pub fn build_pattern(
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
pub fn build_probes(probes_used: &[crate::specialist_v3::ProbeUsed]) -> Vec<RecipeProbe> {
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
pub fn build_logic(
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
pub fn build_answer_template(
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

/// Build origin with citations
pub fn build_origin(
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
