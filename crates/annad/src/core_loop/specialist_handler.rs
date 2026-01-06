//! Specialist consultation - asks IT specialists to solve problems (v0.0.830).
//!
//! This module handles:
//! - Asking specialists (LLM models) to solve problems
//! - Parsing specialist responses and confidence levels
//! - Citing knowledge sources in answers
//!
//! v0.0.830: Updated to use better prompts from specialist_prompt module

use std::collections::HashMap;

use anna_shared::rpc::SpecialistDomain;
use crate::ollama;
use crate::specialist_prompt::build_specialist_prompt;
use super::types::{ParsedQuery, SpecialistSolution};

/// Map domain string to SpecialistDomain enum
fn domain_to_specialist(domain: &str) -> SpecialistDomain {
    match domain.to_lowercase().as_str() {
        "system" => SpecialistDomain::System,
        "boot" => SpecialistDomain::Boot,
        "services" => SpecialistDomain::Services,
        "network" => SpecialistDomain::Network,
        "storage" => SpecialistDomain::Storage,
        "packages" => SpecialistDomain::Packages,
        "audio" => SpecialistDomain::Audio,
        "display" => SpecialistDomain::Display,
        "desktop" => SpecialistDomain::Desktop,
        "security" => SpecialistDomain::Security,
        _ => SpecialistDomain::System, // Default fallback
    }
}

/// Ask specialist to solve the problem
/// v0.0.830: Now uses the better prompts from specialist_prompt.rs
pub async fn ask_specialist(
    model: &str,
    query: &str,
    parsed: &ParsedQuery,
    evidence: &HashMap<String, String>,
    knowledge: &[anna_shared::evidence_engine::DocSnippet],
    specialist_name: &str,
) -> SpecialistSolution {
    // v0.0.830: Build structured input for specialist
    let probes_json: serde_json::Map<String, serde_json::Value> = evidence
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();

    let docs_json: Vec<serde_json::Value> = knowledge
        .iter()
        .map(|k| serde_json::json!({
            "source": k.source,
            "title": k.title,
            "snippet": k.snippet
        }))
        .collect();

    let input_json = serde_json::json!({
        "ticket_id": format!("core-{}", chrono::Utc::now().format("%H%M%S")),
        "domain": parsed.domain,
        "intent": "question",
        "question": query,
        "probes": probes_json,
        "docs": docs_json,
        "metadata": {
            "probes_run": evidence.keys().collect::<Vec<_>>(),
            "docs_searched": if knowledge.is_empty() { vec![] } else { vec!["arch_wiki", "man"] },
        }
    });

    // Get domain-specific system prompt
    let domain_enum = domain_to_specialist(&parsed.domain);
    let system_prompt = build_specialist_prompt(domain_enum);

    // v0.0.830: Use improved prompt format
    let prompt = format!(
        "{}\n\nInput:\n{}",
        system_prompt,
        serde_json::to_string_pretty(&input_json).unwrap_or_default()
    );

    let response = match ollama::chat_with_timeout(model, &prompt, 30).await {
        Ok(r) => r,
        Err(e) => {
            return SpecialistSolution {
                answer: format!("Specialist error: {}", e),
                confidence: 0.0,
                explanation: "Failed to get specialist response".to_string(),
            };
        }
    };

    // Try to parse JSON from response
    let json: serde_json::Value = serde_json::from_str(&response)
        .or_else(|_| {
            if let Some(start) = response.find('{') {
                if let Some(end) = response.rfind('}') {
                    return serde_json::from_str(&response[start..=end]);
                }
            }
            Err(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No JSON found",
            )))
        })
        .unwrap_or_else(|_| {
            // If JSON parsing fails, use the response as-is
            serde_json::json!({
                "answer": { "short": response },
                "confidence": 0.6,
                "staff_view": { "short_note": "Raw response (JSON parsing failed)" }
            })
        });

    // v0.0.830: Handle both old format (answer: string) and new format (answer: {short, detail})
    let answer = if let Some(answer_obj) = json.get("answer") {
        if let Some(short) = answer_obj.get("short").and_then(|v| v.as_str()) {
            // New format: combine short and detail
            let detail = answer_obj.get("detail").and_then(|v| v.as_str()).unwrap_or("");
            if detail.is_empty() {
                short.to_string()
            } else {
                format!("{}\n\n{}", short, detail)
            }
        } else if let Some(s) = answer_obj.as_str() {
            // Old format: answer is just a string
            s.to_string()
        } else {
            response.clone()
        }
    } else {
        response.clone()
    };

    // Get confidence (0.0-1.0)
    let confidence = json.get("confidence")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6) as f32;

    // Get explanation from staff_view.short_note or fallback to old format
    let explanation = json.get("staff_view")
        .and_then(|sv| sv.get("short_note"))
        .and_then(|v| v.as_str())
        .or_else(|| json.get("explanation").and_then(|v| v.as_str()))
        .unwrap_or("")
        .to_string();

    SpecialistSolution {
        answer,
        confidence,
        explanation,
    }
}
