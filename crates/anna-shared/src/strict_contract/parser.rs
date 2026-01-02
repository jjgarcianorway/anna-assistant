//! Parsing logic for specialist responses
//!
//! This module handles extracting and parsing JSON from LLM output,
//! including lenient parsing for backwards compatibility.

use crate::strict_contract::types::{
    ActionKind, Citation, CitationKind, EvidenceItem, ParseResult, RiskLevel, StrictSpecialistResponse,
    StrictStatus, SuggestedAction,
};
use serde::Deserialize;

/// Extract JSON from raw LLM output
pub fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Try clean JSON object first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed.to_string());
    }

    // Try markdown code block with json
    if let Some(start) = trimmed.find("```json") {
        if let Some(end) = trimmed[start + 7..].find("```") {
            let json = trimmed[start + 7..start + 7 + end].trim();
            if json.starts_with('{') {
                return Some(json.to_string());
            }
        }
    }

    // Try bare code block
    if let Some(start) = trimmed.find("```") {
        if let Some(end) = trimmed[start + 3..].find("```") {
            let json = trimmed[start + 3..start + 3 + end].trim();
            // Skip language identifier if present
            let json = json
                .lines()
                .skip_while(|l| !l.trim().starts_with('{'))
                .collect::<Vec<_>>()
                .join("\n");
            if json.starts_with('{') {
                return Some(json);
            }
        }
    }

    // Find first { and last }
    let first_brace = trimmed.find('{')?;
    let last_brace = trimmed.rfind('}')?;
    if last_brace > first_brace {
        return Some(trimmed[first_brace..=last_brace].to_string());
    }

    None
}

/// Parse raw LLM output into validated response
pub fn parse_specialist_output(raw: &str, ticket_id: &str, intent: &str) -> ParseResult {
    // Step 1: Extract JSON
    let json_str = match extract_json(raw) {
        Some(j) => j,
        None => {
            return ParseResult::NoJson {
                raw: truncate(raw, 500),
            }
        }
    };

    // Step 2: Parse JSON
    let response: StrictSpecialistResponse = match serde_json::from_str(&json_str) {
        Ok(r) => r,
        Err(e) => {
            // Try lenient parsing
            if let Ok(partial) = parse_lenient(&json_str, ticket_id, intent) {
                partial
            } else {
                return ParseResult::InvalidJson {
                    raw: truncate(&json_str, 500),
                    error: e.to_string(),
                };
            }
        }
    };

    // Step 3: Validate
    let issues = response.validate();
    if !issues.is_empty() {
        return ParseResult::ValidationFailed { response, issues };
    }

    ParseResult::Success(response)
}

/// Lenient parsing - fill in missing fields with defaults
pub fn parse_lenient(
    json_str: &str,
    ticket_id: &str,
    intent: &str,
) -> Result<StrictSpecialistResponse, String> {
    #[derive(Deserialize, Default)]
    struct Lenient {
        #[serde(default)]
        ticket_id: Option<String>,
        #[serde(default)]
        intent: Option<String>,
        #[serde(default)]
        status: Option<String>,
        #[serde(default)]
        confidence: Option<f32>,
        #[serde(default)]
        summary: Option<String>,
        // Also accept "answer.short" pattern
        #[serde(default)]
        answer: Option<LenientAnswer>,
        #[serde(default)]
        details: Option<Vec<String>>,
        #[serde(default)]
        metrics: Option<serde_json::Value>,
        #[serde(default)]
        actions: Option<Vec<SuggestedAction>>,
        #[serde(default)]
        evidence: Option<Vec<EvidenceItem>>,
        #[serde(default)]
        citations: Option<Vec<Citation>>,
    }

    #[derive(Deserialize, Default)]
    struct LenientAnswer {
        #[serde(default)]
        short: Option<String>,
        #[serde(default)]
        detail: Option<String>,
    }

    let l: Lenient = serde_json::from_str(json_str).map_err(|e| e.to_string())?;

    // Extract summary from either field
    let summary = l
        .summary
        .or_else(|| l.answer.as_ref().and_then(|a| a.short.clone()))
        .unwrap_or_else(|| "No summary provided".to_string());

    // Extract details
    let mut details = l.details.unwrap_or_default();
    if let Some(detail) = l.answer.as_ref().and_then(|a| a.detail.clone()) {
        if !detail.is_empty() && details.is_empty() {
            details.push(detail);
        }
    }

    // Parse status
    let status = match l.status.as_deref().unwrap_or("ok").to_lowercase().as_str() {
        "ok" => StrictStatus::Ok,
        "partial" | "needs_more_data" => StrictStatus::Partial,
        "failed" | "error" | "cannot_answer" | "no_evidence" => StrictStatus::Failed,
        _ => StrictStatus::Partial,
    };

    Ok(StrictSpecialistResponse {
        ticket_id: l.ticket_id.unwrap_or_else(|| ticket_id.to_string()),
        intent: l.intent.unwrap_or_else(|| intent.to_string()),
        status,
        confidence: l.confidence.unwrap_or(0.0).clamp(0.0, 1.0),
        summary,
        details,
        metrics: l.metrics,
        actions: l.actions.unwrap_or_default(),
        evidence: l.evidence.unwrap_or_default(),
        citations: l.citations.unwrap_or_default(),
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
