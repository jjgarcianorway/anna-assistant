//! Strict Specialist Contract (v0.0.415).
//!
//! THE single source of truth for specialist JSON responses.
//! All specialists MUST return exactly this schema. No exceptions.
//!
//! Design principles:
//! - JSON only, no prose outside JSON
//! - Every field has a clear purpose
//! - Validation catches all known failure modes
//! - Metrics are domain-specific and structured
//! - Citations are mandatory for grounded answers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The STRICT response schema for all specialists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrictSpecialistResponse {
    /// Ticket ID (echoed back)
    pub ticket_id: String,

    /// Intent classification
    pub intent: String,

    /// Response status
    pub status: StrictStatus,

    /// Confidence (0.0-1.0)
    pub confidence: f32,

    /// Short one-line answer (MAX 100 chars)
    pub summary: String,

    /// Optional bullet details (0-5 items, each MAX 200 chars)
    #[serde(default)]
    pub details: Vec<String>,

    /// Domain-specific structured metrics
    #[serde(default)]
    pub metrics: Option<serde_json::Value>,

    /// Suggested actions (0-3 items)
    #[serde(default)]
    pub actions: Vec<SuggestedAction>,

    /// Evidence from probes (REQUIRED if status=ok)
    #[serde(default)]
    pub evidence: Vec<EvidenceItem>,

    /// Citations from docs (if docs were consulted)
    #[serde(default)]
    pub citations: Vec<Citation>,
}

/// Response status - STRICT enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrictStatus {
    /// Answer is complete and grounded
    Ok,
    /// Answer is partial - some data missing
    Partial,
    /// Cannot answer - insufficient data or error
    Failed,
}

/// A suggested action for the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action kind
    pub kind: ActionKind,
    /// One-sentence description
    pub description: String,
    /// Optional shell command (safe, generic)
    #[serde(default)]
    pub command: Option<String>,
    /// Risk level
    #[serde(default)]
    pub risk: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    #[default]
    Suggestion,
    Fix,
    Investigate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

/// Evidence from a probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Probe ID
    pub probe: String,
    /// Short summary of what this probe shows
    pub summary: String,
}

/// Citation from documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Document ID (e.g., "man:systemctl")
    pub doc_id: String,
    /// Source kind
    pub kind: CitationKind,
    /// Display string for user
    pub display: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    ManPage,
    ArchWiki,
    HelpOutput,
    BuiltIn,
    LearnedRecipe,
}

impl StrictSpecialistResponse {
    /// Create a successful response
    pub fn ok(ticket_id: &str, intent: &str, summary: &str, confidence: f32) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Ok,
            confidence: confidence.clamp(0.0, 1.0),
            summary: summary.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create a partial response
    pub fn partial(ticket_id: &str, intent: &str, summary: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Partial,
            confidence: 0.0,
            summary: summary.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create a failed response
    pub fn failed(ticket_id: &str, intent: &str, reason: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            intent: intent.to_string(),
            status: StrictStatus::Failed,
            confidence: 0.0,
            summary: reason.to_string(),
            details: vec![],
            metrics: None,
            actions: vec![],
            evidence: vec![],
            citations: vec![],
        }
    }

    /// Create from timeout
    pub fn timeout(ticket_id: &str, intent: &str, elapsed_secs: u64) -> Self {
        Self::failed(
            ticket_id,
            intent,
            &format!("Specialist timed out after {}s.", elapsed_secs),
        )
    }

    /// Create from parse error
    pub fn parse_error(ticket_id: &str, intent: &str, error: &str) -> Self {
        Self::failed(
            ticket_id,
            intent,
            &format!("Invalid specialist response: {}", truncate(error, 100)),
        )
    }

    /// Builder: add evidence
    pub fn with_evidence(mut self, probe: &str, summary: &str) -> Self {
        self.evidence.push(EvidenceItem {
            probe: probe.to_string(),
            summary: summary.to_string(),
        });
        self
    }

    /// Builder: add metrics
    pub fn with_metrics(mut self, metrics: serde_json::Value) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Builder: add action
    pub fn with_action(mut self, kind: ActionKind, desc: &str, cmd: Option<&str>, risk: RiskLevel) -> Self {
        self.actions.push(SuggestedAction {
            kind,
            description: desc.to_string(),
            command: cmd.map(|s| s.to_string()),
            risk,
        });
        self
    }

    /// Validate response - returns list of issues
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Summary must not be empty
        if self.summary.trim().is_empty() {
            issues.push("summary is empty".to_string());
        }

        // Summary length check
        if self.summary.len() > 200 {
            issues.push(format!("summary too long ({} > 200 chars)", self.summary.len()));
        }

        // Confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            issues.push(format!("confidence {} out of range [0.0, 1.0]", self.confidence));
        }

        // Ok status requires evidence
        if self.status == StrictStatus::Ok && self.confidence >= 0.8 && self.evidence.is_empty() {
            issues.push("status=ok with high confidence but no evidence".to_string());
        }

        // Check for forbidden patterns
        let forbidden = [
            "unknown is installed",
            "unknown is not installed",
            "**unknown**",
            "2 is installed",
            "1 is installed",
            "installed package is not installed",
        ];

        let summary_lower = self.summary.to_lowercase();
        for f in forbidden {
            if summary_lower.contains(f) {
                issues.push(format!("contains forbidden pattern: '{}'", f));
            }
        }

        // Check details
        for (i, detail) in self.details.iter().enumerate() {
            if detail.len() > 300 {
                issues.push(format!("details[{}] too long ({} > 300 chars)", i, detail.len()));
            }
            let detail_lower = detail.to_lowercase();
            for f in forbidden {
                if detail_lower.contains(f) {
                    issues.push(format!("details[{}] contains forbidden pattern: '{}'", i, f));
                }
            }
        }

        // Actions limit
        if self.actions.len() > 5 {
            issues.push(format!("too many actions ({} > 5)", self.actions.len()));
        }

        issues
    }

    /// Check if this is a valid, meaningful response
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Check if this should count as "resolved" for stats
    pub fn is_resolved(&self) -> bool {
        self.status == StrictStatus::Ok
            && self.confidence >= 0.8
            && !self.summary.trim().is_empty()
            && self.is_valid()
    }
}

/// Parse result with error classification
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Successfully parsed and validated
    Success(StrictSpecialistResponse),
    /// No JSON found in output
    NoJson { raw: String },
    /// JSON found but invalid structure
    InvalidJson { raw: String, error: String },
    /// Schema validation failed
    ValidationFailed { response: StrictSpecialistResponse, issues: Vec<String> },
    /// LLM timed out
    Timeout { elapsed_secs: u64 },
}

impl ParseResult {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Convert to StrictSpecialistResponse (creates error response if not success)
    pub fn to_response(self, ticket_id: &str, intent: &str) -> StrictSpecialistResponse {
        match self {
            Self::Success(r) => r,
            Self::NoJson { raw } => {
                StrictSpecialistResponse::parse_error(ticket_id, intent, &format!("No JSON found: {}", truncate(&raw, 100)))
            }
            Self::InvalidJson { error, .. } => {
                StrictSpecialistResponse::parse_error(ticket_id, intent, &error)
            }
            Self::ValidationFailed { mut response, issues } => {
                // Downgrade to failed with issues noted
                response.status = StrictStatus::Failed;
                response.summary = format!("Invalid response: {}", issues.join(", "));
                response.confidence = 0.0;
                response
            }
            Self::Timeout { elapsed_secs } => {
                StrictSpecialistResponse::timeout(ticket_id, intent, elapsed_secs)
            }
        }
    }
}

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
            let json = json.lines()
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
fn parse_lenient(json_str: &str, ticket_id: &str, intent: &str) -> Result<StrictSpecialistResponse, String> {
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
    let summary = l.summary
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

/// Time budgets for the pipeline (in milliseconds)
#[derive(Debug, Clone, Copy)]
pub struct TimeBudgets {
    /// Translator LLM call max time
    pub translator_ms: u64,
    /// Specialist LLM call max time
    pub specialist_ms: u64,
    /// Individual probe max time
    pub probe_ms: u64,
    /// Total probes combined max time
    pub probes_total_ms: u64,
    /// Knowledge query max time
    pub knowledge_ms: u64,
}

impl Default for TimeBudgets {
    fn default() -> Self {
        Self {
            translator_ms: 1500,  // 1.5s
            specialist_ms: 4000,  // 4s
            probe_ms: 3000,       // 3s per probe
            probes_total_ms: 5000, // 5s total for all probes
            knowledge_ms: 500,    // 500ms for knowledge queries
        }
    }
}

impl TimeBudgets {
    /// Aggressive budgets for fast responses
    pub fn fast() -> Self {
        Self {
            translator_ms: 1000,
            specialist_ms: 3000,
            probe_ms: 2000,
            probes_total_ms: 4000,
            knowledge_ms: 300,
        }
    }

    /// Relaxed budgets for complex queries
    pub fn thorough() -> Self {
        Self {
            translator_ms: 2000,
            specialist_ms: 6000,
            probe_ms: 4000,
            probes_total_ms: 8000,
            knowledge_ms: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_response_ok() {
        let response = StrictSpecialistResponse::ok("DSK-001", "query_metric", "Available memory: 17.0 GiB", 0.95)
            .with_evidence("memory_info", "MemAvailable: 17892232 kB");

        assert!(response.is_valid());
        assert!(response.is_resolved());
    }

    #[test]
    fn test_strict_response_validates_forbidden() {
        let response = StrictSpecialistResponse::ok("DSK-001", "check_package", "unknown is installed", 0.9);
        let issues = response.validate();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("forbidden")));
    }

    #[test]
    fn test_strict_response_validates_evidence() {
        let response = StrictSpecialistResponse::ok("DSK-001", "query_metric", "Your RAM is 16GB", 0.95);
        // No evidence but high confidence + ok status
        let issues = response.validate();
        assert!(issues.iter().any(|i| i.contains("no evidence")));
    }

    #[test]
    fn test_parse_clean_json() {
        let json = r#"{"ticket_id":"DSK-001","intent":"query_metric","status":"ok","confidence":0.9,"summary":"You have 16GB RAM","evidence":[{"probe":"memory_info","summary":"MemTotal: 16384000 kB"}]}"#;
        let result = parse_specialist_output(json, "DSK-001", "query_metric");
        assert!(result.is_success());
    }

    #[test]
    fn test_parse_markdown_json() {
        let raw = r#"Here's my analysis:
```json
{"ticket_id":"DSK-001","intent":"query_metric","status":"ok","confidence":0.9,"summary":"16GB RAM available","evidence":[]}
```"#;
        let result = parse_specialist_output(raw, "DSK-001", "query_metric");
        // Should parse but fail validation (no evidence with high confidence)
        match result {
            ParseResult::ValidationFailed { .. } => (), // Expected
            ParseResult::Success(_) => panic!("Should fail validation"),
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[test]
    fn test_lenient_parsing() {
        // Old schema format
        let json = r#"{"ticket_id":"DSK-001","status":"ok","answer":{"short":"You have 0 failed services"},"confidence":0.85}"#;
        let result = parse_lenient(json, "DSK-001", "check_status");
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.summary, "You have 0 failed services");
    }

    #[test]
    fn test_is_resolved() {
        let good = StrictSpecialistResponse::ok("DSK-001", "query", "Answer", 0.9)
            .with_evidence("probe", "data");
        assert!(good.is_resolved());

        let low_conf = StrictSpecialistResponse::ok("DSK-001", "query", "Answer", 0.5);
        assert!(!low_conf.is_resolved());

        let partial = StrictSpecialistResponse::partial("DSK-001", "query", "Partial answer");
        assert!(!partial.is_resolved());
    }
}
