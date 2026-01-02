//! Self-healing logic for fixing invalid answers.

use anna_shared::grounding::ParsedEvidence;
use tracing::debug;

use crate::ollama;

use super::types::ValidationIssue;

/// Attempt to heal an answer by regenerating with constraints
pub async fn heal_answer(
    original_answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    issues: &[ValidationIssue],
    model: &str,
    timeout_secs: u64,
) -> anyhow::Result<String> {
    // Build correction prompt based on issues
    let correction_prompt = build_correction_prompt(original_answer, query, evidence, issues);

    debug!("Sending correction prompt to LLM");
    let response = ollama::chat_with_timeout(model, &correction_prompt, timeout_secs).await?;

    // Extract just the answer part (remove any thinking)
    let cleaned = clean_llm_response(&response);

    Ok(cleaned)
}

/// Build a prompt that instructs the LLM to fix specific issues
pub fn build_correction_prompt(
    original_answer: &str,
    query: &str,
    evidence: &ParsedEvidence,
    issues: &[ValidationIssue],
) -> String {
    let mut constraints: Vec<String> = Vec::new();

    for issue in issues {
        match issue {
            ValidationIssue::UngroundedClaims { .. } => {
                constraints.push(
                    "- Only make claims that are directly supported by the evidence below"
                        .to_string(),
                );
            }
            ValidationIssue::InventionDetected { ref claim } => {
                constraints.push(format!("- Do NOT claim: {}", claim));
            }
            ValidationIssue::MissingEvidence { ref kind } => {
                constraints.push(format!("- Include {} information from the evidence", kind));
            }
            ValidationIssue::TooVague => {
                constraints
                    .push("- Be specific with numbers and values from the evidence".to_string());
            }
            ValidationIssue::LowConfidence { .. } => {
                constraints.push("- Focus on answering exactly what was asked".to_string());
            }
        }
    }

    let evidence_text = evidence.summary();

    format!(
        r#"The user asked: "{}"

Your previous answer had issues. Please write a corrected answer.

EVIDENCE (use ONLY this data):
{}

CONSTRAINTS (you MUST follow these):
{}

Previous answer (has errors):
{}

Write a corrected answer that:
1. Only uses facts from the evidence
2. Directly answers the question
3. Is concise and specific

Corrected answer:"#,
        query,
        evidence_text,
        constraints.join("\n"),
        original_answer
    )
}

/// Clean LLM response (remove thinking markers, etc.)
pub fn clean_llm_response(response: &str) -> String {
    let mut result = response.to_string();

    // Remove <think>...</think> blocks
    while let (Some(start), Some(end)) = (result.find("<think>"), result.find("</think>")) {
        if end > start {
            result = format!("{}{}", &result[..start], &result[end + 8..]);
        } else {
            break;
        }
    }

    // Remove /no_think and similar markers
    result = result.replace("/no_think", "");
    result = result.replace("<|endofthink|>", "");

    result.trim().to_string()
}
