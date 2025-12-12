//! Specialist JSON contract (v0.0.419).
//!
//! This defines the STRICT schema that all specialists must output.
//! Specialists ONLY output JSON - no prose, no roleplay, no excuses.
//! The personality layer (Sofia, Tomas, etc.) is handled by the renderer.
//!
//! Key principles:
//! - answer.short MUST directly answer the user's question
//! - evidence[] MUST back up every claim
//! - evidence_references[] MUST list IDs of knowledge items used
//! - citations[] MUST provide provenance for all knowledge used
//! - can_answer MUST be false if insufficient evidence
//! - discovery.new_probes/recipes is how Anna learns new capabilities
//! - Specialists NEVER speak to the user, only return structured data
//!
//! v0.0.419: Added KnowledgeCitation for provenance tracking

use serde::{Deserialize, Serialize};

/// Input to a specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistInput {
    pub ticket_id: String,
    pub domain: String,
    pub intent: SpecialistIntent,
    pub question: String,
    pub probes: std::collections::HashMap<String, String>,
}

/// What kind of request this is
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecialistIntent {
    Question,
    Investigate,
    Request,
}

/// The complete response from a specialist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistResponse {
    pub ticket_id: String,
    pub status: ResponseStatus,
    pub answer: Answer,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub confidence: f32,
    #[serde(default)]
    pub staff_view: Option<StaffView>,
    #[serde(default)]
    pub next_steps: Option<NextSteps>,
    #[serde(default)]
    pub discovery: Option<Discovery>,
    /// If status is needs_more_data, which probes are missing
    #[serde(default)]
    pub missing_probes: Vec<String>,
    /// v0.0.408: Explicit "I cannot answer this" flag
    #[serde(default = "default_can_answer")]
    pub can_answer: bool,
    /// v0.0.408: IDs of knowledge items referenced by the answer
    #[serde(default)]
    pub evidence_references: Vec<String>,
    /// v0.0.408: Short titles of knowledge items used (for display)
    #[serde(default)]
    pub knowledge_used: Vec<String>,
    /// v0.0.419: Citations with full provenance from knowledge sources
    #[serde(default)]
    pub citations: Vec<KnowledgeCitation>,
}

fn default_can_answer() -> bool {
    true
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Ok,
    NeedsMoreData,
    CannotAnswer,
    Error,
    /// v0.0.408: Probes ran but no relevant documentation found
    NoEvidence,
}

/// The actual answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer {
    /// Direct one-sentence answer to the question
    pub short: String,
    /// Optional longer explanation
    #[serde(default)]
    pub detail: Option<String>,
    /// Domain-specific structured data
    #[serde(default)]
    pub domain_summary: Option<serde_json::Value>,
}

/// Evidence backing up a claim
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    /// Which probe this evidence comes from
    pub probe: String,
    /// Short relevant excerpt from the probe output
    pub snippet: String,
    /// What this snippet means for the answer
    pub interpretation: String,
}

/// Citation from a knowledge source (v0.0.419)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCitation {
    /// Citation ID for provenance (e.g., "man:systemctl:line42-50")
    pub citation_id: String,
    /// Kind of source (man, help, wiki, doc)
    pub kind: CitationKind,
    /// Human-readable title
    pub title: String,
    /// Relevant excerpt that was used
    pub excerpt: String,
    /// Relevance score (0-100)
    #[serde(default)]
    pub relevance: u8,
}

impl KnowledgeCitation {
    /// Create a new citation
    pub fn new(citation_id: &str, kind: CitationKind, title: &str, excerpt: &str) -> Self {
        Self {
            citation_id: citation_id.to_string(),
            kind,
            title: title.to_string(),
            excerpt: excerpt.to_string(),
            relevance: 80,
        }
    }

    /// Format as inline reference (e.g., "[man systemctl(1)]")
    pub fn inline_ref(&self) -> String {
        match self.kind {
            CitationKind::ManPage => format!("[man {}]", self.title),
            CitationKind::CliHelp => format!("[{} --help]", self.title),
            CitationKind::ArchWiki => format!("[wiki:{}]", self.title),
            CitationKind::LocalDoc => format!("[doc:{}]", self.title),
            CitationKind::Internal => format!("[{}]", self.title),
        }
    }

    /// Format for citation footer
    pub fn footer_display(&self) -> String {
        let kind_str = match self.kind {
            CitationKind::ManPage => "man page",
            CitationKind::CliHelp => "command help",
            CitationKind::ArchWiki => "Arch Wiki",
            CitationKind::LocalDoc => "local doc",
            CitationKind::Internal => "internal",
        };
        format!(
            "{} ({}): \"{}\"",
            self.title,
            kind_str,
            truncate_str(&self.excerpt, 100)
        )
    }
}

/// Kind of citation source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationKind {
    ManPage,
    CliHelp,
    ArchWiki,
    LocalDoc,
    Internal,
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Internal staff view (for personality rendering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffView {
    #[serde(default = "default_role")]
    pub assignee_role: String,
    #[serde(default)]
    pub severity: Severity,
    #[serde(default)]
    pub mood: Mood,
    /// Short internal note (NOT for user)
    #[serde(default)]
    pub short_note: Option<String>,
    #[serde(default = "default_complexity")]
    pub complexity: u8,
}

fn default_role() -> String {
    "System Specialist".to_string()
}

fn default_complexity() -> u8 {
    1
}

/// Severity of the finding
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    #[default]
    Info,
    Warning,
    Critical,
    Unknown,
}

/// Specialist's confidence/mood
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Mood {
    #[default]
    Confident,
    Uncertain,
    Blocked,
}

/// Next steps (user actions and internal actions)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NextSteps {
    #[serde(default)]
    pub user_actions: Vec<UserAction>,
    #[serde(default)]
    pub internal_actions: Vec<InternalAction>,
}

/// Action the user can take
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAction {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub recipe_id: Option<String>,
}

/// Internal action (run more probes, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAction {
    pub id: String,
    #[serde(default)]
    pub probes: Vec<String>,
}

/// Discovery: how specialists propose new probes and recipes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Discovery {
    #[serde(default)]
    pub new_probes: Vec<ProbeProposal>,
    #[serde(default)]
    pub new_recipes: Vec<RecipeProposal>,
}

/// A proposed new probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub command: String,
    #[serde(default)]
    pub parse_hint: Option<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

/// A proposed new recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeProposal {
    pub id: String,
    pub intent: String,
    pub domain: String,
    pub summary: String,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default = "default_risk")]
    pub risk_level: RiskLevel,
    #[serde(default)]
    pub steps_high_level: Vec<String>,
    #[serde(default)]
    pub reusable_for: Vec<String>,
}

fn default_risk() -> RiskLevel {
    RiskLevel::Low
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
}

impl SpecialistResponse {
    /// v0.0.409: Validate response for forbidden patterns and invalid data
    pub fn validate(&self) -> Vec<String> {
        let mut errors = vec![];

        // Check for forbidden patterns in answer
        let forbidden = [
            "unknown is installed",
            "unknown is not installed",
            "**unknown**",
            "2 is installed", // Common parse bug
            "1 is installed",
        ];

        let answer_lower = self.answer.short.to_lowercase();
        let detail_lower = self
            .answer
            .detail
            .as_ref()
            .map(|d| d.to_lowercase())
            .unwrap_or_default();

        for f in forbidden {
            if answer_lower.contains(f) || detail_lower.contains(f) {
                errors.push(format!("Answer contains forbidden pattern: '{}'", f));
            }
        }

        // Check confidence vs evidence consistency
        if self.status == ResponseStatus::Ok {
            if self.confidence > 0.8 && self.evidence.is_empty() {
                errors.push("High confidence (>0.8) but no evidence provided".to_string());
            }
            if self.can_answer && self.answer.short.is_empty() {
                errors.push("can_answer=true but answer.short is empty".to_string());
            }
        }

        // Check for can_answer consistency
        if !self.can_answer && self.status == ResponseStatus::Ok && self.confidence > 0.7 {
            errors.push("can_answer=false but status=ok with high confidence".to_string());
        }

        // Check confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            errors.push(format!(
                "Confidence {} out of range [0.0, 1.0]",
                self.confidence
            ));
        }

        errors
    }

    /// v0.0.409: Check if this is a valid, meaningful response
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// Create a fallback response when JSON parsing fails
    pub fn parse_error(ticket_id: &str, error: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Error,
            answer: Answer {
                short: "Failed to parse specialist response.".to_string(),
                detail: Some(format!("Parse error: {}", error)),
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.0,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Unknown,
                mood: Mood::Blocked,
                short_note: Some("JSON parse failed".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: false,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        }
    }

    /// Create a response from deterministic/direct answer
    pub fn from_direct_answer(
        ticket_id: &str,
        answer: &str,
        evidence_probe: &str,
        evidence_snippet: &str,
    ) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: answer.to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: evidence_probe.to_string(),
                snippet: evidence_snippet.to_string(),
                interpretation: "Directly extracted from probe output.".to_string(),
            }],
            confidence: 0.95,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("Direct answer from probe data.".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![evidence_probe.to_string()],
            knowledge_used: vec![evidence_probe.to_string()],
            citations: vec![],
        }
    }

    /// v0.0.408: Create a "cannot answer" response with suggestions
    pub fn no_evidence(ticket_id: &str, reason: &str, suggestions: Vec<String>) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::NoEvidence,
            answer: Answer {
                short: "I cannot safely answer this from local data.".to_string(),
                detail: Some(reason.to_string()),
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.0,
            staff_view: Some(StaffView {
                assignee_role: "System".to_string(),
                severity: Severity::Info,
                mood: Mood::Uncertain,
                short_note: Some("No evidence found".to_string()),
                complexity: 1,
            }),
            next_steps: Some(NextSteps {
                user_actions: suggestions
                    .into_iter()
                    .enumerate()
                    .map(|(i, s)| UserAction {
                        id: format!("suggest_{}", i),
                        description: s,
                        recipe_id: None,
                    })
                    .collect(),
                internal_actions: vec![],
            }),
            discovery: None,
            missing_probes: vec![],
            can_answer: false,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        }
    }

    /// v0.0.410: Create response from instant answer (knowledge index hit)
    pub fn instant_answer(ticket_id: &str, answer: &str) -> Self {
        Self {
            ticket_id: ticket_id.to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: answer.to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: "knowledge_index".to_string(),
                snippet: "Learned from previous successful answer".to_string(),
                interpretation: "This pattern was previously verified and stored.".to_string(),
            }],
            confidence: 0.85,
            staff_view: Some(StaffView {
                assignee_role: "Knowledge Index".to_string(),
                severity: Severity::Info,
                mood: Mood::Confident,
                short_note: Some("Instant answer from learned pattern".to_string()),
                complexity: 1,
            }),
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec!["knowledge_index".to_string()],
            knowledge_used: vec!["learned_pattern".to_string()],
            citations: vec![],
        }
    }

    /// v0.0.419: Create response with citations
    pub fn with_citations(mut self, citations: Vec<KnowledgeCitation>) -> Self {
        self.citations = citations;
        self
    }

    /// v0.0.419: Add a single citation
    pub fn add_citation(&mut self, citation: KnowledgeCitation) {
        self.citations.push(citation);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_specialist_response() {
        let json = r#"{
            "ticket_id": "DSK-0101",
            "status": "ok",
            "answer": {
                "short": "No, there is no active swap configured.",
                "detail": "Both free -h and /proc/swaps show 0B swap."
            },
            "evidence": [
                {
                    "probe": "swap_files",
                    "snippet": "Filename Type Size Used Priority",
                    "interpretation": "No entries listed."
                }
            ],
            "confidence": 0.95,
            "staff_view": {
                "assignee_role": "System Specialist",
                "severity": "info",
                "mood": "confident",
                "short_note": "No swap configured.",
                "complexity": 1
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::Ok);
        assert!(response.answer.short.contains("swap"));
        assert_eq!(response.evidence.len(), 1);
    }

    #[test]
    fn test_parse_needs_more_data() {
        let json = r#"{
            "ticket_id": "DSK-0102",
            "status": "needs_more_data",
            "answer": {
                "short": "I cannot determine if zram is enabled."
            },
            "missing_probes": ["zram_devices"],
            "confidence": 0.3
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, ResponseStatus::NeedsMoreData);
        assert_eq!(response.missing_probes, vec!["zram_devices"]);
    }

    #[test]
    fn test_parse_with_discovery() {
        let json = r#"{
            "ticket_id": "DSK-0103",
            "status": "ok",
            "answer": {
                "short": "Test answer"
            },
            "evidence": [],
            "confidence": 0.8,
            "discovery": {
                "new_probes": [
                    {
                        "id": "zram_devices",
                        "intent": "Detect zram configuration",
                        "domain": "system",
                        "command": "lsblk | grep zram",
                        "reusable_for": ["is zram enabled", "compressed memory"]
                    }
                ],
                "new_recipes": []
            }
        }"#;

        let response: SpecialistResponse = serde_json::from_str(json).unwrap();
        assert!(response.discovery.is_some());
        let discovery = response.discovery.unwrap();
        assert_eq!(discovery.new_probes.len(), 1);
        assert_eq!(discovery.new_probes[0].id, "zram_devices");
    }

    #[test]
    fn test_validate_forbidden_patterns() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0104".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "unknown is installed on your system".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![],
            confidence: 0.9,
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| e.contains("forbidden pattern")));
    }

    #[test]
    fn test_validate_high_confidence_no_evidence() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0105".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "vim is installed".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![], // No evidence!
            confidence: 0.95, // High confidence!
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec![],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(errors.iter().any(|e| e.contains("no evidence")));
    }

    #[test]
    fn test_validate_valid_response() {
        let response = SpecialistResponse {
            ticket_id: "DSK-0106".to_string(),
            status: ResponseStatus::Ok,
            answer: Answer {
                short: "vim is installed at /usr/bin/vim".to_string(),
                detail: None,
                domain_summary: None,
            },
            evidence: vec![Evidence {
                probe: "command_v".to_string(),
                snippet: "/usr/bin/vim".to_string(),
                interpretation: "vim binary found".to_string(),
            }],
            confidence: 0.9,
            staff_view: None,
            next_steps: None,
            discovery: None,
            missing_probes: vec![],
            can_answer: true,
            evidence_references: vec!["command_v".to_string()],
            knowledge_used: vec![],
            citations: vec![],
        };

        let errors = response.validate();
        assert!(errors.is_empty());
        assert!(response.is_valid());
    }
}
