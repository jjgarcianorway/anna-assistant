//! LLM Protocol v0.7.0
//!
//! Defines the structured JSON schemas for LLM-A (Junior/Planner)
//! and LLM-B (Senior/Auditor) communication.
//!
//! Evidence Oracle - High reliability and performance focused.

use serde::{Deserialize, Serialize};

/// Difficulty level for question routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    /// Simple questions: knowledge-only or single probe
    #[default]
    Easy,
    /// Normal questions: 1-2 probes or probe + shell commands
    Normal,
    /// Complex questions: multiple probes, docs search, iterations
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Easy => "easy",
            Self::Normal => "normal",
            Self::Hard => "hard",
        }
    }

    /// Maximum number of probes allowed for this difficulty
    pub fn max_probes(&self) -> usize {
        match self {
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 5,
        }
    }

    /// Maximum number of shell commands allowed
    pub fn max_shell_commands(&self) -> usize {
        match self {
            Self::Easy => 1,
            Self::Normal => 2,
            Self::Hard => 5,
        }
    }

    /// Whether LLM-B review is required
    pub fn requires_senior_review(&self) -> bool {
        match self {
            Self::Easy => false,
            Self::Normal => true,
            Self::Hard => true,
        }
    }
}

/// Question intent type for classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum QuestionIntent {
    #[default]
    HardwareInfo,
    DiskLayout,
    Logs,
    Updates,
    Network,
    Dns,
    ConfigHelp,
    SelfHealth,
    PackageInfo,
    ProcessInfo,
    SystemStatus,
    Generic,
}

impl QuestionIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::HardwareInfo => "hardware_info",
            Self::DiskLayout => "disk_layout",
            Self::Logs => "logs",
            Self::Updates => "updates",
            Self::Network => "network",
            Self::Dns => "dns",
            Self::ConfigHelp => "config_help",
            Self::SelfHealth => "self_health",
            Self::PackageInfo => "package_info",
            Self::ProcessInfo => "process_info",
            Self::SystemStatus => "system_status",
            Self::Generic => "generic",
        }
    }
}

/// Latency expectation for the answer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LatencyExpectation {
    /// Fast response expected (< 2s)
    #[default]
    Low,
    /// Medium response time (2-10s)
    Medium,
    /// Slow response acceptable (> 10s)
    High,
}

// ============================================================================
// LLM-A (Junior) Output Schema
// ============================================================================

/// A request to run a specific probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeRequest {
    /// The probe ID (e.g., "cpu.info", "mem.info")
    pub probe_id: String,
    /// Why this probe is needed
    pub reason: String,
}

/// A request to run a safe shell command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeShellRequest {
    /// The command to execute (must be in whitelist)
    pub command: String,
    /// Why this command is needed
    pub reason: String,
}

/// A request to search offline documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsRequest {
    /// The search query
    pub query: String,
    /// Why this search is needed
    pub reason: String,
}

/// A request to query the knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    /// The kind of knowledge to look up
    pub kind: String,
    /// Why this lookup is needed
    pub reason: String,
}

/// Citation for evidence used in an answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Source of the evidence (e.g., "cpu.info", "lscpu")
    pub source: String,
    /// Specific detail or command
    pub detail: String,
}

/// The plan portion of LLM-A output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmAPlan {
    /// Question difficulty assessment
    pub difficulty: Difficulty,
    /// Question intent classification
    pub intent: QuestionIntent,
    /// Whether user clarification is needed
    #[serde(default)]
    pub needs_user_clarification: bool,
    /// The clarification question if needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_clarification_prompt: Option<String>,
    /// Probes to run
    #[serde(default)]
    pub probe_requests: Vec<ProbeRequest>,
    /// Safe shell commands to run
    #[serde(default)]
    pub safe_shell_requests: Vec<SafeShellRequest>,
    /// Documentation searches to perform
    #[serde(default)]
    pub docs_requests: Vec<DocsRequest>,
    /// Knowledge store queries
    #[serde(default)]
    pub knowledge_queries: Vec<KnowledgeQuery>,
    /// Whether we can answer without more evidence
    #[serde(default)]
    pub can_answer_without_more_probes: bool,
}

/// The draft answer portion of LLM-A output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmADraftAnswer {
    /// The answer text
    pub text: String,
    /// Evidence citations
    #[serde(default)]
    pub citations: Vec<Citation>,
    /// Whether generic heuristics were used
    #[serde(default)]
    pub heuristics_used: bool,
    /// Heuristics explanation if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heuristics_section: Option<String>,
    /// Self-assessed reliability (0-100)
    #[serde(default)]
    pub reliability_percent: u8,
    /// Expected latency
    #[serde(default)]
    pub latency_expectation: LatencyExpectation,
}

/// Complete LLM-A (Junior) output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmAOutput {
    /// The execution plan
    pub plan: LlmAPlan,
    /// The draft answer
    pub draft_answer: LlmADraftAnswer,
}

// ============================================================================
// LLM-B (Senior) Output Schema
// ============================================================================

/// Verdict from the senior LLM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Answer is correct and complete
    #[default]
    Approve,
    /// Answer needs fixes but is mostly correct
    FixAndAccept,
    /// Answer is wrong or too speculative
    Reject,
}

impl Verdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::FixAndAccept => "fix_and_accept",
            Self::Reject => "reject",
        }
    }

    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::Approve | Self::FixAndAccept)
    }
}

/// Reliability scores from senior review
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReliabilityScores {
    /// Evidence quality (0-100)
    pub evidence: u8,
    /// Reasoning quality (0-100)
    pub reasoning: u8,
    /// Coverage completeness (0-100)
    pub coverage: u8,
    /// Overall reliability = 0.4*evidence + 0.3*reasoning + 0.3*coverage
    pub overall_reliability_percent: u8,
}

impl ReliabilityScores {
    /// Calculate overall reliability from components
    pub fn calculate_overall(&mut self) {
        let evidence = self.evidence as f32;
        let reasoning = self.reasoning as f32;
        let coverage = self.coverage as f32;
        self.overall_reliability_percent =
            (0.4 * evidence + 0.3 * reasoning + 0.3 * coverage).round() as u8;
    }

    /// Create scores and calculate overall
    pub fn new(evidence: u8, reasoning: u8, coverage: u8) -> Self {
        let mut scores = Self {
            evidence,
            reasoning,
            coverage,
            overall_reliability_percent: 0,
        };
        scores.calculate_overall();
        scores
    }
}

/// A knowledge update to persist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeUpdate {
    /// The kind of knowledge (e.g., "cpu_model", "service_log_template")
    pub kind: String,
    /// The data to store
    pub data: serde_json::Value,
    /// Source of this knowledge
    pub source: String,
    /// Confidence in this fact (0-100)
    pub confidence_percent: u8,
    /// Why this should be stored
    pub reason: String,
}

/// Complete LLM-B (Senior) output
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmBOutput {
    /// The verdict on the draft answer
    pub verdict: Verdict,
    /// Detailed reliability scores
    pub scores: ReliabilityScores,
    /// List of problems found
    #[serde(default)]
    pub problems: Vec<String>,
    /// Additional probes needed (if rejecting)
    #[serde(default)]
    pub probe_requests: Vec<ProbeRequest>,
    /// Additional shell commands needed
    #[serde(default)]
    pub safe_shell_requests: Vec<SafeShellRequest>,
    /// Additional docs searches needed
    #[serde(default)]
    pub docs_requests: Vec<DocsRequest>,
    /// Knowledge to persist from this answer
    #[serde(default)]
    pub knowledge_updates: Vec<KnowledgeUpdate>,
    /// The final answer (fixed if needed)
    #[serde(default)]
    pub fixed_answer: String,
}

// ============================================================================
// Command Templates
// ============================================================================

/// A reusable command template for common operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandTemplate {
    /// Template kind (e.g., "service_logs_window")
    pub kind: String,
    /// The command template with placeholders
    pub template: String,
    /// Default parameter values
    #[serde(default)]
    pub defaults: std::collections::HashMap<String, String>,
    /// Confidence in this template (0-100)
    pub confidence_percent: u8,
    /// When this template was last used successfully
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success: Option<chrono::DateTime<chrono::Utc>>,
}

impl CommandTemplate {
    /// Render the template with given parameters
    pub fn render(&self, params: &std::collections::HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        // First apply provided params, then defaults
        for (key, value) in params {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        for (key, value) in &self.defaults {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_levels() {
        assert_eq!(Difficulty::Easy.max_probes(), 1);
        assert_eq!(Difficulty::Normal.max_probes(), 2);
        assert_eq!(Difficulty::Hard.max_probes(), 5);
        assert!(!Difficulty::Easy.requires_senior_review());
        assert!(Difficulty::Normal.requires_senior_review());
    }

    #[test]
    fn test_reliability_scores() {
        let scores = ReliabilityScores::new(90, 80, 70);
        // 0.4*90 + 0.3*80 + 0.3*70 = 36 + 24 + 21 = 81
        assert_eq!(scores.overall_reliability_percent, 81);
    }

    #[test]
    fn test_verdict() {
        assert!(Verdict::Approve.is_acceptable());
        assert!(Verdict::FixAndAccept.is_acceptable());
        assert!(!Verdict::Reject.is_acceptable());
    }

    #[test]
    fn test_command_template() {
        let template = CommandTemplate {
            kind: "service_logs".to_string(),
            template: "journalctl -u {service} --since \"{hours} hours ago\"".to_string(),
            defaults: [("hours".to_string(), "6".to_string())]
                .into_iter()
                .collect(),
            confidence_percent: 95,
            last_success: None,
        };

        let mut params = std::collections::HashMap::new();
        params.insert("service".to_string(), "annad".to_string());

        let rendered = template.render(&params);
        assert_eq!(
            rendered,
            "journalctl -u annad --since \"6 hours ago\""
        );
    }

    #[test]
    fn test_llm_a_output_serialization() {
        let output = LlmAOutput {
            plan: LlmAPlan {
                difficulty: Difficulty::Easy,
                intent: QuestionIntent::HardwareInfo,
                probe_requests: vec![ProbeRequest {
                    probe_id: "cpu.info".to_string(),
                    reason: "Get CPU details".to_string(),
                }],
                ..Default::default()
            },
            draft_answer: LlmADraftAnswer {
                text: "Checking CPU info...".to_string(),
                reliability_percent: 0,
                ..Default::default()
            },
        };

        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("\"difficulty\": \"easy\""));
        assert!(json.contains("\"cpu.info\""));
    }

    #[test]
    fn test_llm_b_output_serialization() {
        let output = LlmBOutput {
            verdict: Verdict::FixAndAccept,
            scores: ReliabilityScores::new(85, 90, 80),
            problems: vec!["Minor formatting issue".to_string()],
            fixed_answer: "Your CPU is Intel i9-14900HX".to_string(),
            knowledge_updates: vec![KnowledgeUpdate {
                kind: "cpu_model".to_string(),
                data: serde_json::json!({"model": "Intel i9-14900HX"}),
                source: "cpu.info".to_string(),
                confidence_percent: 97,
                reason: "Strong evidence from lscpu".to_string(),
            }],
            ..Default::default()
        };

        let json = serde_json::to_string_pretty(&output).unwrap();
        assert!(json.contains("\"verdict\": \"fix_and_accept\""));
        assert!(json.contains("\"overall_reliability_percent\": 85"));
    }
}
