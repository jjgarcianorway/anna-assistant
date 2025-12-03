//! Source Labels v0.0.20 - Ask Me Anything Mode
//!
//! Source labeling for answers with clear provenance tracking.
//! Every claim must be labeled with its source type:
//! - Evidence IDs [E#] for system measurements
//! - Knowledge IDs [K#] for documentation/manpages
//! - (Reasoning) for general inference

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// Directory for Q&A statistics
pub const QA_STATS_DIR: &str = "/var/lib/anna/internal/qa_stats";
/// Path to daily statistics file
pub const QA_STATS_FILE: &str = "/var/lib/anna/internal/qa_stats/daily.json";

/// Source types for answer claims
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    /// System evidence from tools/snapshots [E#]
    SystemEvidence,
    /// Knowledge pack documentation [K#]
    KnowledgeDocs,
    /// General reasoning (no direct source)
    Reasoning,
    /// Mixed - contains multiple source types
    Mixed,
    /// Unknown/uncited - should be flagged
    Uncited,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::SystemEvidence => "system_evidence",
            SourceType::KnowledgeDocs => "knowledge_docs",
            SourceType::Reasoning => "reasoning",
            SourceType::Mixed => "mixed",
            SourceType::Uncited => "uncited",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SourceType::SystemEvidence => "System Evidence",
            SourceType::KnowledgeDocs => "Knowledge Docs",
            SourceType::Reasoning => "Reasoning",
            SourceType::Mixed => "Mixed Sources",
            SourceType::Uncited => "Uncited",
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            SourceType::SystemEvidence => "[E",
            SourceType::KnowledgeDocs => "[K",
            SourceType::Reasoning => "(Reasoning)",
            SourceType::Mixed => "",
            SourceType::Uncited => "",
        }
    }
}

/// Question type classification for source planning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuestionType {
    /// "How do I...?" - knowledge first
    HowTo,
    /// "What is happening on my machine?" - system evidence first
    SystemStatus,
    /// Mixed question - both sources
    Mixed,
    /// General question - reasoning with optional sources
    General,
}

impl QuestionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            QuestionType::HowTo => "how_to",
            QuestionType::SystemStatus => "system_status",
            QuestionType::Mixed => "mixed",
            QuestionType::General => "general",
        }
    }

    /// Returns priority order for sources based on question type
    pub fn source_priority(&self) -> Vec<SourceType> {
        match self {
            QuestionType::HowTo => vec![SourceType::KnowledgeDocs, SourceType::SystemEvidence],
            QuestionType::SystemStatus => vec![SourceType::SystemEvidence, SourceType::KnowledgeDocs],
            QuestionType::Mixed => vec![SourceType::SystemEvidence, SourceType::KnowledgeDocs],
            QuestionType::General => vec![SourceType::KnowledgeDocs, SourceType::Reasoning],
        }
    }
}

/// Classify question type from request text
pub fn classify_question_type(request: &str) -> QuestionType {
    let lower = request.to_lowercase();

    // How-to patterns
    let how_to_patterns = [
        "how do i", "how can i", "how to", "how should i",
        "what's the best way to", "what is the best way to",
        "steps to", "guide to", "tutorial",
        "enable", "disable", "configure", "set up", "setup",
        "where can i find", "where is",
    ];

    // System status patterns
    let system_patterns = [
        "what is happening", "what's happening",
        "why is my", "why does my", "why am i",
        "what is using", "what's using",
        "what did i install", "what was installed",
        "what changed", "what's changed",
        "is my", "is the", "are there",
        "check my", "show my", "list my",
        "performance", "slow", "high cpu", "high memory",
        "disk space", "running out of",
        "crash", "error", "fail", "warning",
    ];

    // Check patterns
    for pattern in &how_to_patterns {
        if lower.contains(pattern) {
            return QuestionType::HowTo;
        }
    }

    for pattern in &system_patterns {
        if lower.contains(pattern) {
            return QuestionType::SystemStatus;
        }
    }

    // Check for mixed indicators
    let has_how = lower.contains("how") || lower.contains("what");
    let has_my_machine = lower.contains("my ") || lower.contains("system") || lower.contains("machine");

    if has_how && has_my_machine {
        return QuestionType::Mixed;
    }

    QuestionType::General
}

/// Source plan describing which sources will be queried
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePlan {
    pub question_type: QuestionType,
    pub primary_sources: Vec<SourceType>,
    pub knowledge_query: Option<String>,
    pub system_tools: Vec<String>,
    pub rationale: String,
}

impl SourcePlan {
    /// Create a source plan for a given question
    pub fn create(request: &str, question_type: QuestionType) -> Self {
        let primary_sources = question_type.source_priority();

        let (knowledge_query, system_tools, rationale) = match question_type {
            QuestionType::HowTo => {
                let keywords = extract_keywords(request);
                (
                    Some(keywords.join(" ")),
                    vec![],
                    format!("How-to question: searching knowledge packs for documentation on '{}'",
                        keywords.join(", "))
                )
            }
            QuestionType::SystemStatus => {
                let tools = suggest_tools_for_query(request);
                (
                    None,
                    tools.clone(),
                    format!("System status question: using {} to gather evidence",
                        if tools.is_empty() { "status_snapshot".to_string() } else { tools.join(", ") })
                )
            }
            QuestionType::Mixed => {
                let keywords = extract_keywords(request);
                let tools = suggest_tools_for_query(request);
                (
                    Some(keywords.join(" ")),
                    tools.clone(),
                    format!("Mixed question: searching knowledge for '{}' and using {} for system evidence",
                        keywords.join(", "),
                        if tools.is_empty() { "status_snapshot".to_string() } else { tools.join(", ") })
                )
            }
            QuestionType::General => {
                let keywords = extract_keywords(request);
                (
                    if keywords.is_empty() { None } else { Some(keywords.join(" ")) },
                    vec![],
                    "General question: using knowledge packs if relevant, otherwise reasoning".to_string()
                )
            }
        };

        Self {
            question_type,
            primary_sources,
            knowledge_query,
            system_tools,
            rationale,
        }
    }

    /// Format as natural language for transcript
    pub fn to_natural_language(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Question type: {}", self.question_type.as_str()));

        if let Some(ref query) = self.knowledge_query {
            parts.push(format!("Knowledge search: '{}'", query));
        }

        if !self.system_tools.is_empty() {
            parts.push(format!("System tools: {}", self.system_tools.join(", ")));
        }

        parts.push(format!("Rationale: {}", self.rationale));

        parts.join("\n")
    }
}

/// Answer context with environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerContext {
    pub target_user: Option<String>,
    pub distro: String,
    pub kernel_version: Option<String>,
    pub relevant_packages: Vec<String>,
    pub relevant_services: Vec<String>,
    pub knowledge_packs_available: usize,
    pub knowledge_docs_count: usize,
}

impl AnswerContext {
    /// Build answer context from system state
    pub fn build() -> Self {
        // Detect distro
        let distro = detect_distro();
        let kernel_version = detect_kernel_version();
        let target_user = detect_target_user();

        // Knowledge pack info (will be populated if available)
        let (packs, docs) = get_knowledge_counts();

        Self {
            target_user,
            distro,
            kernel_version,
            relevant_packages: Vec::new(),
            relevant_services: Vec::new(),
            knowledge_packs_available: packs,
            knowledge_docs_count: docs,
        }
    }

    /// Add relevant packages based on request context
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.relevant_packages = packages;
        self
    }

    /// Add relevant services based on request context
    pub fn with_services(mut self, services: Vec<String>) -> Self {
        self.relevant_services = services;
        self
    }

    /// Format as natural language for transcript
    pub fn to_natural_language(&self) -> String {
        let mut parts = Vec::new();

        if let Some(ref user) = self.target_user {
            parts.push(format!("Target user: {}", user));
        }

        parts.push(format!("Distro: {}", self.distro));

        if let Some(ref kernel) = self.kernel_version {
            parts.push(format!("Kernel: {}", kernel));
        }

        if !self.relevant_packages.is_empty() {
            parts.push(format!("Relevant packages: {}", self.relevant_packages.join(", ")));
        }

        if !self.relevant_services.is_empty() {
            parts.push(format!("Relevant services: {}", self.relevant_services.join(", ")));
        }

        parts.push(format!("Knowledge packs: {} ({} documents indexed)",
            self.knowledge_packs_available, self.knowledge_docs_count));

        parts.join("\n")
    }
}

/// Q&A statistics for today
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QaStats {
    pub date: String,
    pub answers_count: usize,
    pub total_reliability: u64,
    pub source_type_counts: HashMap<String, usize>,
    pub knowledge_citations: usize,
    pub evidence_citations: usize,
    pub reasoning_labels: usize,
}

impl QaStats {
    /// Load today's stats or create new
    pub fn load_today() -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();

        if let Ok(content) = fs::read_to_string(QA_STATS_FILE) {
            if let Ok(stats) = serde_json::from_str::<QaStats>(&content) {
                if stats.date == today {
                    return stats;
                }
            }
        }

        QaStats {
            date: today,
            ..Default::default()
        }
    }

    /// Record a new answer
    pub fn record_answer(&mut self, reliability: u8, source_types: &[SourceType]) {
        self.answers_count += 1;
        self.total_reliability += reliability as u64;

        for source_type in source_types {
            let count = self.source_type_counts
                .entry(source_type.as_str().to_string())
                .or_insert(0);
            *count += 1;

            match source_type {
                SourceType::KnowledgeDocs => self.knowledge_citations += 1,
                SourceType::SystemEvidence => self.evidence_citations += 1,
                SourceType::Reasoning => self.reasoning_labels += 1,
                _ => {}
            }
        }
    }

    /// Get average reliability
    pub fn avg_reliability(&self) -> u8 {
        if self.answers_count == 0 {
            0
        } else {
            (self.total_reliability / self.answers_count as u64) as u8
        }
    }

    /// Get top source types
    pub fn top_source_types(&self, n: usize) -> Vec<(String, usize)> {
        let mut sorted: Vec<_> = self.source_type_counts.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.into_iter().take(n).collect()
    }

    /// Save stats to file
    pub fn save(&self) -> anyhow::Result<()> {
        // Ensure directory exists
        fs::create_dir_all(QA_STATS_DIR)?;

        let content = serde_json::to_string_pretty(self)?;
        fs::write(QA_STATS_FILE, content)?;
        Ok(())
    }
}

/// Missing evidence report for "I don't know" responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingEvidenceReport {
    pub what_is_missing: Vec<String>,
    pub suggested_tools: Vec<String>,
    pub suggested_knowledge_queries: Vec<String>,
    pub can_partially_answer: bool,
    pub partial_answer_sources: Vec<SourceType>,
}

impl MissingEvidenceReport {
    /// Create a report when evidence is missing
    pub fn new() -> Self {
        Self {
            what_is_missing: Vec::new(),
            suggested_tools: Vec::new(),
            suggested_knowledge_queries: Vec::new(),
            can_partially_answer: false,
            partial_answer_sources: Vec::new(),
        }
    }

    /// Add a missing piece of evidence
    pub fn add_missing(&mut self, description: &str) {
        self.what_is_missing.push(description.to_string());
    }

    /// Suggest a tool to gather missing evidence
    pub fn suggest_tool(&mut self, tool: &str) {
        if !self.suggested_tools.contains(&tool.to_string()) {
            self.suggested_tools.push(tool.to_string());
        }
    }

    /// Suggest a knowledge query
    pub fn suggest_query(&mut self, query: &str) {
        if !self.suggested_knowledge_queries.contains(&query.to_string()) {
            self.suggested_knowledge_queries.push(query.to_string());
        }
    }

    /// Format as natural language for transcript
    pub fn to_natural_language(&self) -> String {
        let mut parts = Vec::new();

        if !self.what_is_missing.is_empty() {
            parts.push(format!("Missing information:\n  - {}",
                self.what_is_missing.join("\n  - ")));
        }

        if !self.suggested_tools.is_empty() {
            parts.push(format!("Could check with tools: {}",
                self.suggested_tools.join(", ")));
        }

        if !self.suggested_knowledge_queries.is_empty() {
            parts.push(format!("Could search knowledge for: {}",
                self.suggested_knowledge_queries.join(", ")));
        }

        if self.can_partially_answer {
            parts.push("Can provide partial answer with available sources.".to_string());
        }

        parts.join("\n")
    }
}

impl Default for MissingEvidenceReport {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

fn extract_keywords(request: &str) -> Vec<String> {
    let stop_words = [
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being",
        "have", "has", "had", "do", "does", "did", "will", "would", "could", "should",
        "may", "might", "must", "shall", "can", "need", "to", "of", "in", "for",
        "on", "with", "at", "by", "from", "up", "about", "into", "through",
        "how", "what", "when", "where", "which", "who", "why",
        "i", "my", "me", "we", "our", "you", "your", "it", "its",
        "this", "that", "these", "those",
    ];

    request
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| {
            !word.is_empty() &&
            word.len() > 2 &&
            !stop_words.contains(&word.as_ref())
        })
        .map(String::from)
        .take(5)
        .collect()
}

fn suggest_tools_for_query(request: &str) -> Vec<String> {
    let lower = request.to_lowercase();
    let mut tools = Vec::new();

    // Package-related
    if lower.contains("install") || lower.contains("package") || lower.contains("software") {
        tools.push("recent_installs".to_string());
        tools.push("sw_snapshot_summary".to_string());
    }

    // Performance-related
    if lower.contains("slow") || lower.contains("performance") || lower.contains("cpu")
       || lower.contains("memory") || lower.contains("ram") {
        tools.push("top_resource_processes".to_string());
        tools.push("slowness_hypotheses".to_string());
    }

    // Disk-related
    if lower.contains("disk") || lower.contains("space") || lower.contains("storage") {
        tools.push("disk_usage".to_string());
    }

    // Service-related
    if lower.contains("service") || lower.contains("systemd") || lower.contains("running") {
        tools.push("status_snapshot".to_string());
    }

    // Error/warning related
    if lower.contains("error") || lower.contains("warning") || lower.contains("fail")
       || lower.contains("crash") {
        tools.push("journal_warnings".to_string());
        tools.push("active_alerts".to_string());
    }

    // Changes
    if lower.contains("changed") || lower.contains("different") || lower.contains("recent") {
        tools.push("what_changed".to_string());
    }

    // Hardware
    if lower.contains("hardware") || lower.contains("cpu") || lower.contains("gpu")
       || lower.contains("memory") {
        tools.push("hw_snapshot_summary".to_string());
    }

    // Boot
    if lower.contains("boot") || lower.contains("startup") {
        tools.push("boot_time_trend".to_string());
    }

    // Default: at least get status
    if tools.is_empty() {
        tools.push("status_snapshot".to_string());
    }

    tools
}

fn detect_distro() -> String {
    // Try /etc/os-release
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .strip_prefix("PRETTY_NAME=")
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string();
            }
        }
        for line in content.lines() {
            if line.starts_with("NAME=") {
                return line
                    .strip_prefix("NAME=")
                    .unwrap_or("")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }

    // Fallback
    "Linux".to_string()
}

fn detect_kernel_version() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/proc/version") {
        // Extract kernel version
        let parts: Vec<&str> = content.split_whitespace().collect();
        if parts.len() >= 3 {
            return Some(parts[2].to_string());
        }
    }
    None
}

fn detect_target_user() -> Option<String> {
    // Check SUDO_USER first
    if let Ok(user) = std::env::var("SUDO_USER") {
        if !user.is_empty() && user != "root" {
            return Some(user);
        }
    }

    // Check USER
    if let Ok(user) = std::env::var("USER") {
        if !user.is_empty() && user != "root" {
            return Some(user);
        }
    }

    None
}

fn get_knowledge_counts() -> (usize, usize) {
    // Try to read knowledge stats
    use crate::knowledge_packs::KnowledgeIndex;

    if let Ok(index) = KnowledgeIndex::open() {
        if let Ok(stats) = index.get_stats() {
            return (stats.pack_count, stats.document_count);
        }
    }

    (0, 0)
}

/// Detect source types in text by looking for citation patterns
pub fn detect_source_types(text: &str) -> Vec<SourceType> {
    let mut types = Vec::new();

    // Check for evidence citations [E#]
    if text.contains("[E") && text.chars().any(|c| c.is_ascii_digit()) {
        types.push(SourceType::SystemEvidence);
    }

    // Check for knowledge citations [K#]
    if text.contains("[K") && text.chars().any(|c| c.is_ascii_digit()) {
        types.push(SourceType::KnowledgeDocs);
    }

    // Check for reasoning labels
    if text.contains("(Reasoning)") || text.contains("(reasoning)") {
        types.push(SourceType::Reasoning);
    }

    if types.is_empty() {
        types.push(SourceType::Uncited);
    }

    types
}

/// Check if text has proper source labels
pub fn has_proper_source_labels(text: &str) -> bool {
    let types = detect_source_types(text);
    !types.contains(&SourceType::Uncited)
}

/// Count citations in text
pub fn count_citations(text: &str) -> (usize, usize, usize) {
    let mut evidence = 0;
    let mut knowledge = 0;
    let mut reasoning = 0;

    // Count [E#] citations
    for i in 1..100 {
        if text.contains(&format!("[E{}]", i)) {
            evidence += 1;
        }
    }

    // Count [K#] citations
    for i in 1..100 {
        if text.contains(&format!("[K{}]", i)) {
            knowledge += 1;
        }
    }

    // Count (Reasoning) labels
    reasoning = text.matches("(Reasoning)").count() + text.matches("(reasoning)").count();

    (evidence, knowledge, reasoning)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_question_how_to() {
        assert_eq!(classify_question_type("How do I enable syntax highlighting in vim?"), QuestionType::HowTo);
        assert_eq!(classify_question_type("how to configure nginx"), QuestionType::HowTo);
        assert_eq!(classify_question_type("What's the best way to set up ssh keys?"), QuestionType::HowTo);
    }

    #[test]
    fn test_classify_question_system_status() {
        assert_eq!(classify_question_type("What is happening on my machine?"), QuestionType::SystemStatus);
        assert_eq!(classify_question_type("Why is my system slow?"), QuestionType::SystemStatus);
        assert_eq!(classify_question_type("What did I install recently?"), QuestionType::SystemStatus);
    }

    #[test]
    fn test_classify_question_mixed() {
        // Questions with both "what/how" and "my system" indicators get Mixed
        assert_eq!(classify_question_type("What services are running on my system and why?"), QuestionType::Mixed);
        // Pure how-to with "how can I" pattern
        assert_eq!(classify_question_type("How can I fix this error?"), QuestionType::HowTo);
    }

    #[test]
    fn test_source_plan_creation() {
        let plan = SourcePlan::create("How do I configure vim?", QuestionType::HowTo);
        assert!(plan.knowledge_query.is_some());
        assert!(plan.system_tools.is_empty());

        let plan = SourcePlan::create("Why is my CPU at 100%?", QuestionType::SystemStatus);
        assert!(plan.knowledge_query.is_none());
        assert!(!plan.system_tools.is_empty());
    }

    #[test]
    fn test_detect_source_types() {
        let text = "Based on the configuration [E1], you should enable syntax highlighting.";
        let types = detect_source_types(text);
        assert!(types.contains(&SourceType::SystemEvidence));

        let text = "According to the documentation [K1], vim supports syntax highlighting.";
        let types = detect_source_types(text);
        assert!(types.contains(&SourceType::KnowledgeDocs));

        let text = "(Reasoning) This is a common configuration.";
        let types = detect_source_types(text);
        assert!(types.contains(&SourceType::Reasoning));

        let text = "This is uncited text.";
        let types = detect_source_types(text);
        assert!(types.contains(&SourceType::Uncited));
    }

    #[test]
    fn test_count_citations() {
        let text = "Based on [E1] and [E2], with knowledge from [K1], (Reasoning) this is my answer.";
        let (e, k, r) = count_citations(text);
        assert_eq!(e, 2);
        assert_eq!(k, 1);
        assert_eq!(r, 1);
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("How do I enable syntax highlighting in vim?");
        assert!(keywords.contains(&"syntax".to_string()) || keywords.contains(&"vim".to_string()));
    }

    #[test]
    fn test_source_type_display() {
        assert_eq!(SourceType::SystemEvidence.display_name(), "System Evidence");
        assert_eq!(SourceType::KnowledgeDocs.display_name(), "Knowledge Docs");
        assert_eq!(SourceType::Reasoning.display_name(), "Reasoning");
    }

    #[test]
    fn test_qa_stats() {
        let mut stats = QaStats::default();
        stats.date = "2025-12-03".to_string();
        stats.record_answer(85, &[SourceType::KnowledgeDocs, SourceType::SystemEvidence]);

        assert_eq!(stats.answers_count, 1);
        assert_eq!(stats.avg_reliability(), 85);
        assert_eq!(stats.knowledge_citations, 1);
        assert_eq!(stats.evidence_citations, 1);
    }

    #[test]
    fn test_missing_evidence_report() {
        let mut report = MissingEvidenceReport::new();
        report.add_missing("No vim configuration found");
        report.suggest_tool("package_info");
        report.suggest_query("vim configuration");

        let nl = report.to_natural_language();
        assert!(nl.contains("Missing information"));
        assert!(nl.contains("package_info"));
    }
}
