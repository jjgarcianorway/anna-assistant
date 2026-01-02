//! Specialist consultation - asks IT specialists to solve problems.
//!
//! This module handles:
//! - Asking specialists (LLM models) to solve problems
//! - Parsing specialist responses and confidence levels
//! - Citing knowledge sources in answers

use std::collections::HashMap;

use crate::ollama;
use super::types::{ParsedQuery, SpecialistSolution};

/// Ask specialist to solve the problem
pub async fn ask_specialist(
    model: &str,
    query: &str,
    parsed: &ParsedQuery,
    evidence: &HashMap<String, String>,
    knowledge: &[anna_shared::evidence_engine::DocSnippet],
    specialist_name: &str,
) -> SpecialistSolution {
    let evidence_str = evidence
        .iter()
        .map(|(k, v)| format!("## {}\n```\n{}\n```", k, v))
        .collect::<Vec<_>>()
        .join("\n\n");

    // Format knowledge sources
    let knowledge_str = if knowledge.is_empty() {
        String::new()
    } else {
        let sections: Vec<String> = knowledge
            .iter()
            .map(|k| format!("## {} ({})\n{}", k.title, k.source, k.snippet))
            .collect();
        format!("\n\nKnowledge Sources:\n{}", sections.join("\n\n"))
    };

    let prompt = format!(
        r#"You are {}, a Linux {} specialist. Answer this query using the evidence and knowledge provided.

Query: "{}"

Evidence:
{}{}

IMPORTANT: If you use information from Knowledge Sources, cite them in your answer.
Example: "According to the Arch Wiki, ..." or "The man page states..."

Provide a clear, direct answer to the user's question.
Then respond with JSON containing your answer and confidence:
{{"answer": "your answer here", "confidence": 0.9, "explanation": "brief reasoning"}}"#,
        specialist_name, parsed.domain, query, evidence_str, knowledge_str
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
                "answer": response,
                "confidence": 0.6,
                "explanation": "Raw response (JSON parsing failed)"
            })
        });

    SpecialistSolution {
        answer: json["answer"].as_str().unwrap_or(&response).to_string(),
        confidence: json["confidence"].as_f64().unwrap_or(0.6) as f32,
        explanation: json["explanation"].as_str().unwrap_or("").to_string(),
    }
}
