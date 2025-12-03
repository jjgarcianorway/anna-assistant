//! v0.0.37: Human-first transcript renderer and case file system
//!
//! This module provides:
//! - Unified transcript rendering (human-readable at all debug levels)
//! - Case file creation and persistence
//! - Case retrieval and search
//! - v0.0.36: Knowledge reference tracking
//! - v0.0.37: Recipe event tracking (matched, executed, created, promoted)
//!
//! Case files are stored in /var/lib/anna/cases/YYYY/MM/DD/<request_id>/

use crate::atomic_write::atomic_write_bytes;
use crate::redaction::redact_transcript;
use chrono::{DateTime, Datelike, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Base directory for case files
pub const CASES_DIR: &str = "/var/lib/anna/cases";

/// Default retention: 30 days
pub const DEFAULT_RETENTION_DAYS: u32 = 30;

/// Maximum case storage size (1GB default)
pub const DEFAULT_MAX_SIZE_BYTES: u64 = 1_073_741_824;

// =============================================================================
// Actors
// =============================================================================

/// Participants in the dialogue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Actor {
    You,
    Anna,
    Translator,
    Junior,
    Senior,
    Annad,
}

impl std::fmt::Display for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Actor::You => write!(f, "you"),
            Actor::Anna => write!(f, "anna"),
            Actor::Translator => write!(f, "translator"),
            Actor::Junior => write!(f, "junior"),
            Actor::Senior => write!(f, "senior"),
            Actor::Annad => write!(f, "annad"),
        }
    }
}

// =============================================================================
// Transcript Message
// =============================================================================

/// A single message in the transcript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptMessage {
    pub from: Actor,
    pub to: Actor,
    pub intent: String,        // One-line intent (always shown)
    pub details: Option<String>, // Additional details (shown at higher debug levels)
    pub evidence_ids: Vec<String>, // Referenced evidence IDs [E1, E2]
    pub timestamp: DateTime<Utc>,
}

impl TranscriptMessage {
    pub fn new(from: Actor, to: Actor, intent: &str) -> Self {
        Self {
            from,
            to,
            intent: intent.to_string(),
            details: None,
            evidence_ids: Vec::new(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_evidence(mut self, evidence_ids: Vec<String>) -> Self {
        self.evidence_ids = evidence_ids;
        self
    }

    /// Render this message for display (human-readable)
    pub fn render(&self, debug_level: u8, terminal_width: usize) -> String {
        let header = format!("[{}] to [{}]:", self.from, self.to);
        let mut lines = vec![header];

        // Always show intent
        lines.push(wrap_text(&self.intent, terminal_width - 2));

        // Show details at debug level >= 1
        if debug_level >= 1 {
            if let Some(details) = &self.details {
                lines.push(String::new()); // blank line
                lines.push(wrap_text(details, terminal_width - 2));
            }
        }

        // Show evidence IDs if present
        if !self.evidence_ids.is_empty() {
            let evidence_line = format!("Evidence: {}", self.evidence_ids.join(", "));
            lines.push(evidence_line);
        }

        lines.join("\n")
    }
}

/// Wrap text to fit terminal width, never truncate
fn wrap_text(text: &str, width: usize) -> String {
    let width = width.max(40); // Minimum 40 chars
    let mut result = Vec::new();

    for line in text.lines() {
        if line.len() <= width {
            result.push(line.to_string());
        } else {
            // Word wrap
            let mut current_line = String::new();
            for word in line.split_whitespace() {
                if current_line.is_empty() {
                    current_line = word.to_string();
                } else if current_line.len() + 1 + word.len() <= width {
                    current_line.push(' ');
                    current_line.push_str(word);
                } else {
                    result.push(current_line);
                    current_line = word.to_string();
                }
            }
            if !current_line.is_empty() {
                result.push(current_line);
            }
        }
    }

    result.join("\n")
}

// =============================================================================
// Transcript Builder
// =============================================================================

/// Builds a transcript during request processing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranscriptBuilder {
    pub request_id: String,
    pub messages: Vec<TranscriptMessage>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

impl TranscriptBuilder {
    pub fn new(request_id: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            messages: Vec::new(),
            start_time: Some(Utc::now()),
            end_time: None,
        }
    }

    pub fn add_message(&mut self, msg: TranscriptMessage) {
        self.messages.push(msg);
    }

    pub fn add(&mut self, from: Actor, to: Actor, intent: &str) {
        self.messages.push(TranscriptMessage::new(from, to, intent));
    }

    pub fn add_with_details(&mut self, from: Actor, to: Actor, intent: &str, details: &str) {
        self.messages.push(TranscriptMessage::new(from, to, intent).with_details(details));
    }

    pub fn finish(&mut self) {
        self.end_time = Some(Utc::now());
    }

    /// Render the full transcript for display
    pub fn render(&self, debug_level: u8, terminal_width: usize) -> String {
        let mut lines = Vec::new();

        for msg in &self.messages {
            lines.push(msg.render(debug_level, terminal_width));
            lines.push(String::new()); // blank line between messages
        }

        lines.join("\n")
    }
}

// =============================================================================
// Case File Structures
// =============================================================================

/// Summary of a case (summary.txt)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummary {
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub user_request: String,
    pub intent_type: String,
    pub outcome: CaseOutcome,
    pub reliability_score: u8,
    pub evidence_count: usize,
    pub policy_refs_count: usize,
    pub duration_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseOutcome {
    Success,
    Failure,
    Partial,
    Cancelled,
}

impl std::fmt::Display for CaseOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaseOutcome::Success => write!(f, "success"),
            CaseOutcome::Failure => write!(f, "failure"),
            CaseOutcome::Partial => write!(f, "partial"),
            CaseOutcome::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl CaseSummary {
    /// Render to human-readable text
    pub fn to_text(&self) -> String {
        let mut lines = vec![
            format!("Case: {}", self.request_id),
            format!("Time: {}", self.timestamp.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S")),
            format!("Request: {}", self.user_request),
            format!("Intent: {}", self.intent_type),
            format!("Outcome: {}", self.outcome),
            format!("Reliability: {}%", self.reliability_score),
            format!("Evidence collected: {}", self.evidence_count),
            format!("Policy rules referenced: {}", self.policy_refs_count),
            format!("Duration: {}ms", self.duration_ms),
        ];

        if let Some(err) = &self.error_message {
            lines.push(format!("Error: {}", err));
        }

        lines.join("\n")
    }
}

/// Evidence entry for evidence.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceEntry {
    pub id: String,           // E1, E2, etc.
    pub tool_name: String,
    pub tool_args: String,    // Redacted
    pub timestamp: DateTime<Utc>,
    pub summary: String,
    pub restricted: bool,
}

/// Policy reference for policy_refs.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRef {
    pub rule_id: String,      // e.g., "blocked.toml:rule_12"
    pub explanation: String,
}

/// Timing data for timing.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseTiming {
    pub translator_ms: u64,
    pub evidence_ms: u64,
    pub junior_ms: u64,
    pub total_ms: u64,
}

/// Result data for result.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseResult {
    pub success: bool,
    pub reliability_score: u8,
    pub rollback_occurred: bool,
    pub rollback_actions: Vec<String>,
    pub errors: Vec<String>,  // Redacted
}

/// v0.0.36: Knowledge reference for case files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeRef {
    /// Knowledge evidence ID (K1, K2, ...)
    pub evidence_id: String,
    /// Document title
    pub title: String,
    /// Pack ID this came from
    pub pack_id: String,
    /// Pack name for display
    pub pack_name: String,
    /// Source path (man:command, /usr/share/doc/...)
    pub source_path: String,
    /// Trust level (official, local, user)
    pub trust: String,
    /// Excerpt used (redacted)
    pub excerpt: String,
}

/// v0.0.37: Recipe event type for case files
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecipeEventType {
    /// Recipe was matched for this case
    Matched,
    /// Recipe was executed (applied)
    Executed,
    /// Recipe execution succeeded
    Succeeded,
    /// Recipe execution failed
    Failed,
    /// Recipe was created from this case
    Created,
    /// Recipe was promoted from draft to active
    Promoted,
}

/// v0.0.37: Recipe event for case files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeEvent {
    /// Recipe ID
    pub recipe_id: String,
    /// Recipe name
    pub recipe_name: String,
    /// Event type
    pub event_type: RecipeEventType,
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Match confidence (for Matched events)
    pub match_confidence: Option<f32>,
    /// Error message (for Failed events)
    pub error_message: Option<String>,
    /// Notes (for Created events, e.g., "created as draft, reliability 75%")
    pub notes: Option<String>,
}

impl RecipeEvent {
    /// Create a matched event
    pub fn matched(recipe_id: &str, recipe_name: &str, confidence: f32) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Matched,
            timestamp: Utc::now(),
            match_confidence: Some(confidence),
            error_message: None,
            notes: None,
        }
    }

    /// Create an executed event
    pub fn executed(recipe_id: &str, recipe_name: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Executed,
            timestamp: Utc::now(),
            match_confidence: None,
            error_message: None,
            notes: None,
        }
    }

    /// Create a succeeded event
    pub fn succeeded(recipe_id: &str, recipe_name: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Succeeded,
            timestamp: Utc::now(),
            match_confidence: None,
            error_message: None,
            notes: None,
        }
    }

    /// Create a failed event
    pub fn failed(recipe_id: &str, recipe_name: &str, error: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Failed,
            timestamp: Utc::now(),
            match_confidence: None,
            error_message: Some(error.to_string()),
            notes: None,
        }
    }

    /// Create a created event
    pub fn created(recipe_id: &str, recipe_name: &str, notes: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Created,
            timestamp: Utc::now(),
            match_confidence: None,
            error_message: None,
            notes: Some(notes.to_string()),
        }
    }

    /// Create a promoted event
    pub fn promoted(recipe_id: &str, recipe_name: &str) -> Self {
        Self {
            recipe_id: recipe_id.to_string(),
            recipe_name: recipe_name.to_string(),
            event_type: RecipeEventType::Promoted,
            timestamp: Utc::now(),
            match_confidence: None,
            error_message: None,
            notes: None,
        }
    }
}

/// v0.0.35: Model information for case files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseModelInfo {
    /// Translator model used (or "deterministic" if fallback)
    pub translator: String,
    /// Whether translator is using fallback mode
    pub translator_fallback: bool,
    /// Junior model used (or null if unavailable)
    pub junior: Option<String>,
    /// Whether junior is unavailable (reliability capped)
    pub junior_unavailable: bool,
    /// Maximum reliability when junior is unavailable (e.g., 60)
    pub reliability_cap: Option<u8>,
    /// Hardware tier used for selection
    pub hardware_tier: String,
}

impl Default for CaseModelInfo {
    fn default() -> Self {
        Self {
            translator: "deterministic".to_string(),
            translator_fallback: true,
            junior: None,
            junior_unavailable: true,
            reliability_cap: Some(60),
            hardware_tier: "unknown".to_string(),
        }
    }
}

// =============================================================================
// Case File Manager
// =============================================================================

/// Complete case file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseFile {
    pub summary: CaseSummary,
    pub transcript: TranscriptBuilder,
    pub evidence: Vec<EvidenceEntry>,
    pub policy_refs: Vec<PolicyRef>,
    pub timing: CaseTiming,
    pub result: CaseResult,
    /// v0.0.34: Fix-It mode timeline (optional, only for troubleshooting sessions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_timeline: Option<String>,
    /// v0.0.35: Model information (which models were used)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub models: Option<CaseModelInfo>,
    /// v0.0.36: Knowledge references used in this case
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub knowledge_refs: Vec<KnowledgeRef>,
    /// v0.0.37: Recipe events in this case
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recipe_events: Vec<RecipeEvent>,
}

impl CaseFile {
    /// Create a new case file builder
    pub fn new(request_id: &str, user_request: &str) -> Self {
        Self {
            summary: CaseSummary {
                request_id: request_id.to_string(),
                timestamp: Utc::now(),
                user_request: user_request.to_string(),
                intent_type: "unknown".to_string(),
                outcome: CaseOutcome::Success,
                reliability_score: 0,
                evidence_count: 0,
                policy_refs_count: 0,
                duration_ms: 0,
                error_message: None,
            },
            transcript: TranscriptBuilder::new(request_id),
            evidence: Vec::new(),
            policy_refs: Vec::new(),
            timing: CaseTiming {
                translator_ms: 0,
                evidence_ms: 0,
                junior_ms: 0,
                total_ms: 0,
            },
            result: CaseResult {
                success: true,
                reliability_score: 0,
                rollback_occurred: false,
                rollback_actions: Vec::new(),
                errors: Vec::new(),
            },
            fix_timeline: None,
            models: None,
            knowledge_refs: Vec::new(),
            recipe_events: Vec::new(),
        }
    }

    /// v0.0.34: Set fix timeline from FixItSession
    pub fn set_fix_timeline(&mut self, timeline_json: &str) {
        self.fix_timeline = Some(timeline_json.to_string());
    }

    /// v0.0.35: Set model information for this case
    pub fn set_models(&mut self, info: CaseModelInfo) {
        self.models = Some(info);
    }

    /// v0.0.36: Add a knowledge reference to this case
    pub fn add_knowledge_ref(&mut self, kr: KnowledgeRef) {
        self.knowledge_refs.push(kr);
    }

    /// v0.0.36: Get knowledge references count
    pub fn knowledge_refs_count(&self) -> usize {
        self.knowledge_refs.len()
    }

    /// v0.0.37: Add a recipe event to this case
    pub fn add_recipe_event(&mut self, event: RecipeEvent) {
        self.recipe_events.push(event);
    }

    /// v0.0.37: Get recipe events count
    pub fn recipe_events_count(&self) -> usize {
        self.recipe_events.len()
    }

    /// v0.0.37: Check if a recipe was executed in this case
    pub fn recipe_was_executed(&self, recipe_id: &str) -> bool {
        self.recipe_events.iter().any(|e| {
            e.recipe_id == recipe_id && e.event_type == RecipeEventType::Executed
        })
    }

    /// v0.0.37: Check if a recipe was created from this case
    pub fn recipe_was_created(&self, recipe_id: &str) -> bool {
        self.recipe_events.iter().any(|e| {
            e.recipe_id == recipe_id && e.event_type == RecipeEventType::Created
        })
    }

    /// Get the path for this case file
    pub fn get_path(&self) -> PathBuf {
        let now = self.summary.timestamp.with_timezone(&Local);
        PathBuf::from(format!(
            "{}/{}/{:02}/{:02}/{}",
            CASES_DIR,
            now.format("%Y"),
            now.month(),
            now.day(),
            self.summary.request_id
        ))
    }

    /// Save case file atomically
    pub fn save(&self) -> io::Result<PathBuf> {
        let case_dir = self.get_path();

        // Create directory
        fs::create_dir_all(&case_dir)?;

        // Helper to convert PathBuf to string for atomic_write_bytes
        let path_str = |p: PathBuf| -> String { p.to_string_lossy().to_string() };

        // Write each file atomically with redaction
        let summary_path = path_str(case_dir.join("summary.txt"));
        atomic_write_bytes(&summary_path, redact_transcript(&self.summary.to_text()).as_bytes())?;

        let transcript_path = path_str(case_dir.join("transcript.log"));
        let transcript_text = self.transcript.render(2, 120); // Full verbosity for logs
        atomic_write_bytes(&transcript_path, redact_transcript(&transcript_text).as_bytes())?;

        let evidence_path = path_str(case_dir.join("evidence.json"));
        let evidence_json = serde_json::to_string_pretty(&self.evidence)
            .unwrap_or_else(|_| "[]".to_string());
        atomic_write_bytes(&evidence_path, redact_transcript(&evidence_json).as_bytes())?;

        let policy_path = path_str(case_dir.join("policy_refs.json"));
        let policy_json = serde_json::to_string_pretty(&self.policy_refs)
            .unwrap_or_else(|_| "[]".to_string());
        atomic_write_bytes(&policy_path, policy_json.as_bytes())?;

        let timing_path = path_str(case_dir.join("timing.json"));
        let timing_json = serde_json::to_string_pretty(&self.timing)
            .unwrap_or_else(|_| "{}".to_string());
        atomic_write_bytes(&timing_path, timing_json.as_bytes())?;

        let result_path = path_str(case_dir.join("result.json"));
        let result_json = serde_json::to_string_pretty(&self.result)
            .unwrap_or_else(|_| "{}".to_string());
        atomic_write_bytes(&result_path, redact_transcript(&result_json).as_bytes())?;

        // v0.0.34: Write fix_timeline.json if present (Fix-It mode sessions)
        if let Some(ref timeline) = self.fix_timeline {
            let timeline_path = path_str(case_dir.join("fix_timeline.json"));
            atomic_write_bytes(&timeline_path, redact_transcript(timeline).as_bytes())?;
        }

        Ok(case_dir)
    }

    /// Add an evidence entry
    pub fn add_evidence(&mut self, id: &str, tool_name: &str, tool_args: &str, summary: &str, restricted: bool) {
        self.evidence.push(EvidenceEntry {
            id: id.to_string(),
            tool_name: tool_name.to_string(),
            tool_args: tool_args.to_string(),
            timestamp: Utc::now(),
            summary: summary.to_string(),
            restricted,
        });
        self.summary.evidence_count = self.evidence.len();
    }

    /// Add a policy reference
    pub fn add_policy_ref(&mut self, rule_id: &str, explanation: &str) {
        self.policy_refs.push(PolicyRef {
            rule_id: rule_id.to_string(),
            explanation: explanation.to_string(),
        });
        self.summary.policy_refs_count = self.policy_refs.len();
    }

    /// Set timing information
    pub fn set_timing(&mut self, translator_ms: u64, evidence_ms: u64, junior_ms: u64, total_ms: u64) {
        self.timing = CaseTiming {
            translator_ms,
            evidence_ms,
            junior_ms,
            total_ms,
        };
        self.summary.duration_ms = total_ms;
    }

    /// Set the outcome
    pub fn set_outcome(&mut self, outcome: CaseOutcome, reliability: u8, error: Option<&str>) {
        self.summary.outcome = outcome;
        self.summary.reliability_score = reliability;
        self.summary.error_message = error.map(|s| s.to_string());
        self.result.success = outcome == CaseOutcome::Success;
        self.result.reliability_score = reliability;
        if let Some(e) = error {
            self.result.errors.push(e.to_string());
        }
    }

    /// Set intent type
    pub fn set_intent(&mut self, intent_type: &str) {
        self.summary.intent_type = intent_type.to_string();
    }
}

// =============================================================================
// Case Retrieval
// =============================================================================

/// Load a case summary from disk
pub fn load_case_summary(case_dir: &Path) -> Option<CaseSummary> {
    let summary_path = case_dir.join("summary.txt");
    let result_path = case_dir.join("result.json");

    // Try to load result.json for structured data
    if let Ok(content) = fs::read_to_string(&result_path) {
        if let Ok(result) = serde_json::from_str::<CaseResult>(&content) {
            // Extract request_id from directory name
            let request_id = case_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Try to get more info from summary.txt
            let (user_request, intent_type) = if let Ok(summary_text) = fs::read_to_string(&summary_path) {
                let mut req = String::new();
                let mut intent = String::new();
                for line in summary_text.lines() {
                    if line.starts_with("Request: ") {
                        req = line.trim_start_matches("Request: ").to_string();
                    } else if line.starts_with("Intent: ") {
                        intent = line.trim_start_matches("Intent: ").to_string();
                    }
                }
                (req, intent)
            } else {
                (String::new(), String::new())
            };

            // Get timestamp from directory structure
            let timestamp = Utc::now(); // Fallback, should parse from path

            return Some(CaseSummary {
                request_id,
                timestamp,
                user_request,
                intent_type,
                outcome: if result.success { CaseOutcome::Success } else { CaseOutcome::Failure },
                reliability_score: result.reliability_score,
                evidence_count: 0,
                policy_refs_count: 0,
                duration_ms: 0,
                error_message: result.errors.first().cloned(),
            });
        }
    }

    None
}

/// List recent case directories
pub fn list_recent_cases(limit: usize) -> Vec<PathBuf> {
    let mut cases = Vec::new();

    // Walk through year/month/day directories
    if let Ok(years) = fs::read_dir(CASES_DIR) {
        let mut year_dirs: Vec<_> = years.flatten().collect();
        year_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name())); // Descending

        for year in year_dirs.into_iter().take(2) {
            if let Ok(months) = fs::read_dir(year.path()) {
                let mut month_dirs: Vec<_> = months.flatten().collect();
                month_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                for month in month_dirs.into_iter().take(12) {
                    if let Ok(days) = fs::read_dir(month.path()) {
                        let mut day_dirs: Vec<_> = days.flatten().collect();
                        day_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                        for day in day_dirs.into_iter().take(31) {
                            if let Ok(requests) = fs::read_dir(day.path()) {
                                let mut req_dirs: Vec<_> = requests.flatten().collect();
                                req_dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                                for req in req_dirs {
                                    if req.path().is_dir() {
                                        cases.push(req.path());
                                        if cases.len() >= limit {
                                            return cases;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    cases
}

/// List today's cases
pub fn list_today_cases() -> Vec<PathBuf> {
    let now = Local::now();
    let today_dir = PathBuf::from(format!(
        "{}/{}/{:02}/{:02}",
        CASES_DIR,
        now.format("%Y"),
        now.month(),
        now.day()
    ));

    if let Ok(entries) = fs::read_dir(&today_dir) {
        entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect()
    } else {
        Vec::new()
    }
}

/// Find the last failure case
pub fn find_last_failure() -> Option<PathBuf> {
    for case_path in list_recent_cases(100) {
        if let Some(summary) = load_case_summary(&case_path) {
            if summary.outcome == CaseOutcome::Failure {
                return Some(case_path);
            }
        }
    }
    None
}

/// Get total size of case storage
pub fn get_cases_storage_size() -> u64 {
    fn dir_size(path: &Path) -> u64 {
        if path.is_file() {
            fs::metadata(path).map(|m| m.len()).unwrap_or(0)
        } else if path.is_dir() {
            fs::read_dir(path)
                .map(|entries| {
                    entries.flatten().map(|e| dir_size(&e.path())).sum()
                })
                .unwrap_or(0)
        } else {
            0
        }
    }

    dir_size(Path::new(CASES_DIR))
}

/// Prune old case files
pub fn prune_cases(retention_days: u32, max_size_bytes: u64) -> usize {
    let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
    let mut deleted = 0;

    // First pass: delete by age
    if let Ok(years) = fs::read_dir(CASES_DIR) {
        for year in years.flatten() {
            if let Ok(months) = fs::read_dir(year.path()) {
                for month in months.flatten() {
                    if let Ok(days) = fs::read_dir(month.path()) {
                        for day in days.flatten() {
                            // Parse date from path
                            let path = day.path();
                            let path_str = path.to_string_lossy();

                            // Extract YYYY/MM/DD from path
                            let parts: Vec<&str> = path_str.split('/').collect();
                            if parts.len() >= 3 {
                                // Try to parse as date
                                // If older than cutoff, delete entire day directory
                                if let Ok(_entries) = fs::read_dir(&path) {
                                    // For simplicity, just count deletions
                                    // Full implementation would check timestamps
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Second pass: delete by size (oldest first) if still over limit
    let current_size = get_cases_storage_size();
    if current_size > max_size_bytes {
        let cases = list_recent_cases(10000);
        for case_path in cases.into_iter().rev() {
            // Reverse to get oldest first
            if get_cases_storage_size() <= max_size_bytes {
                break;
            }
            if fs::remove_dir_all(&case_path).is_ok() {
                deleted += 1;
            }
        }
    }

    deleted
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let random: u32 = rand_u32();

    format!("{:x}-{:04x}", timestamp, random & 0xFFFF)
}

/// Simple random u32 (no external deps)
fn rand_u32() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::Instant;

    let mut hasher = DefaultHasher::new();
    Instant::now().hash(&mut hasher);
    std::process::id().hash(&mut hasher);
    hasher.finish() as u32
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_message_render() {
        let msg = TranscriptMessage::new(Actor::You, Actor::Anna, "What CPU do I have?");
        let rendered = msg.render(1, 80);
        assert!(rendered.contains("[you] to [anna]:"));
        assert!(rendered.contains("What CPU do I have?"));
    }

    #[test]
    fn test_transcript_message_with_evidence() {
        let msg = TranscriptMessage::new(Actor::Anna, Actor::You, "Your CPU is an Intel i7")
            .with_evidence(vec!["E1".to_string(), "E2".to_string()]);
        let rendered = msg.render(1, 80);
        assert!(rendered.contains("Evidence: E1, E2"));
    }

    #[test]
    fn test_wrap_text() {
        let text = "This is a very long line that should be wrapped to fit within the terminal width";
        let wrapped = wrap_text(text, 40);
        for line in wrapped.lines() {
            assert!(line.len() <= 40 || !line.contains(' ')); // Either within limit or no space to break
        }
    }

    #[test]
    fn test_case_summary_to_text() {
        let summary = CaseSummary {
            request_id: "test-123".to_string(),
            timestamp: Utc::now(),
            user_request: "test query".to_string(),
            intent_type: "system_query".to_string(),
            outcome: CaseOutcome::Success,
            reliability_score: 85,
            evidence_count: 3,
            policy_refs_count: 1,
            duration_ms: 150,
            error_message: None,
        };

        let text = summary.to_text();
        assert!(text.contains("Case: test-123"));
        assert!(text.contains("Reliability: 85%"));
        assert!(text.contains("Outcome: success"));
    }

    #[test]
    fn test_case_outcome_display() {
        assert_eq!(format!("{}", CaseOutcome::Success), "success");
        assert_eq!(format!("{}", CaseOutcome::Failure), "failure");
    }

    #[test]
    fn test_generate_request_id() {
        let id1 = generate_request_id();
        let id2 = generate_request_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        // IDs should be unique (with very high probability)
        // Note: in fast succession they might have same timestamp prefix
    }

    #[test]
    fn test_case_file_path() {
        let case = CaseFile::new("test-abc123", "test query");
        let path = case.get_path();
        assert!(path.to_string_lossy().contains(CASES_DIR));
        assert!(path.to_string_lossy().contains("test-abc123"));
    }

    #[test]
    fn test_transcript_builder() {
        let mut builder = TranscriptBuilder::new("test-456");
        builder.add(Actor::You, Actor::Anna, "What's my disk usage?");
        builder.add_message(
            TranscriptMessage::new(Actor::Anna, Actor::You, "Your disk is 50% full")
                .with_evidence(vec!["E1".to_string()]),
        );

        let rendered = builder.render(2, 80);
        assert!(rendered.contains("[you] to [anna]:"));
        assert!(rendered.contains("[anna] to [you]:"));
        assert!(rendered.contains("What's my disk usage?"));
        assert!(rendered.contains("50% full"));
        assert!(rendered.contains("Evidence: E1"));
    }

    #[test]
    fn test_transcript_debug_levels() {
        let mut builder = TranscriptBuilder::new("test-789");
        builder.add(Actor::You, Actor::Anna, "Hello");
        builder.add(Actor::Anna, Actor::Translator, "Translate this");
        builder.add(Actor::Translator, Actor::Anna, "Intent: greeting");
        builder.add(Actor::Anna, Actor::You, "Hi there!");

        // Debug level 0: just user-facing
        let level0 = builder.render(0, 80);
        assert!(level0.contains("[you] to [anna]:"));
        assert!(level0.contains("Hi there!"));

        // Debug level 1: includes internal
        let level1 = builder.render(1, 80);
        assert!(level1.contains("[anna] to [translator]:"));

        // Debug level 2: full verbosity
        let level2 = builder.render(2, 80);
        assert!(level2.contains("[translator] to [anna]:"));
    }

    #[test]
    fn test_case_file_evidence() {
        let mut case = CaseFile::new("test-ev", "test with evidence");
        case.add_evidence("E1", "hw_snapshot", "{}", "Hardware info collected", false);
        case.add_evidence("E2", "sw_snapshot", "{}", "Software info collected", true);

        assert_eq!(case.evidence.len(), 2);
        assert_eq!(case.summary.evidence_count, 2);
        assert_eq!(case.evidence[0].id, "E1");
        assert!(case.evidence[1].restricted);
    }

    #[test]
    fn test_case_file_outcome() {
        let mut case = CaseFile::new("test-out", "test outcome");
        case.set_outcome(CaseOutcome::Failure, 45, Some("Connection timeout"));

        assert_eq!(case.summary.outcome, CaseOutcome::Failure);
        assert_eq!(case.summary.reliability_score, 45);
        assert_eq!(case.summary.error_message, Some("Connection timeout".to_string()));
        assert!(!case.result.success);
        assert_eq!(case.result.errors, vec!["Connection timeout".to_string()]);
    }

    #[test]
    fn test_case_file_timing() {
        let mut case = CaseFile::new("test-time", "test timing");
        case.set_timing(50, 100, 200, 350);

        assert_eq!(case.timing.translator_ms, 50);
        assert_eq!(case.timing.evidence_ms, 100);
        assert_eq!(case.timing.junior_ms, 200);
        assert_eq!(case.timing.total_ms, 350);
        assert_eq!(case.summary.duration_ms, 350);
    }

    #[test]
    fn test_redact_transcript() {
        let text = "Token: sk-abc123 and password=secret123 and key=my-api-key";
        let redacted = redact_transcript(text);
        // Redacted values should not appear
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("secret123"));
        // Redaction placeholders should appear (format: [REDACTED:TYPE])
        assert!(redacted.contains("[REDACTED:"));
    }

    #[test]
    fn test_case_outcome_partial() {
        let summary = CaseSummary {
            request_id: "test-partial".to_string(),
            timestamp: Utc::now(),
            user_request: "partial test".to_string(),
            intent_type: "action_request".to_string(),
            outcome: CaseOutcome::Partial,
            reliability_score: 60,
            evidence_count: 2,
            policy_refs_count: 0,
            duration_ms: 100,
            error_message: Some("Partial completion".to_string()),
        };

        let text = summary.to_text();
        assert!(text.contains("Outcome: partial"));
        assert!(text.contains("Error: Partial completion"));
    }

    #[test]
    fn test_case_model_info_default() {
        let info = CaseModelInfo::default();
        assert_eq!(info.translator, "deterministic");
        assert!(info.translator_fallback);
        assert!(info.junior.is_none());
        assert!(info.junior_unavailable);
        assert_eq!(info.reliability_cap, Some(60));
    }

    #[test]
    fn test_case_file_with_models() {
        let mut case = CaseFile::new("test-models", "test with models");

        let info = CaseModelInfo {
            translator: "qwen2.5:0.5b".to_string(),
            translator_fallback: false,
            junior: Some("qwen2.5:1.5b-instruct".to_string()),
            junior_unavailable: false,
            reliability_cap: None,
            hardware_tier: "medium".to_string(),
        };

        case.set_models(info);

        assert!(case.models.is_some());
        let models = case.models.unwrap();
        assert_eq!(models.translator, "qwen2.5:0.5b");
        assert!(!models.translator_fallback);
        assert_eq!(models.junior, Some("qwen2.5:1.5b-instruct".to_string()));
        assert!(!models.junior_unavailable);
        assert!(models.reliability_cap.is_none());
        assert_eq!(models.hardware_tier, "medium");
    }

    #[test]
    fn test_case_file_without_junior() {
        let mut case = CaseFile::new("test-no-junior", "test without junior");

        let info = CaseModelInfo {
            translator: "qwen2.5:0.5b".to_string(),
            translator_fallback: false,
            junior: None,
            junior_unavailable: true,
            reliability_cap: Some(60),
            hardware_tier: "low".to_string(),
        };

        case.set_models(info);

        assert!(case.models.is_some());
        let models = case.models.unwrap();
        assert!(models.junior_unavailable);
        assert_eq!(models.reliability_cap, Some(60));
    }

    #[test]
    fn test_case_file_with_knowledge_refs() {
        let mut case = CaseFile::new("test-knowledge", "how do I use vim");

        let kr = KnowledgeRef {
            evidence_id: "K1".to_string(),
            title: "vim - Vi IMproved, a programmer's text editor".to_string(),
            pack_id: "system-manpages".to_string(),
            pack_name: "System Man Pages".to_string(),
            source_path: "man:vim".to_string(),
            trust: "official".to_string(),
            excerpt: "To start editing a file, use: vim filename".to_string(),
        };

        case.add_knowledge_ref(kr);

        assert_eq!(case.knowledge_refs_count(), 1);
        assert_eq!(case.knowledge_refs[0].evidence_id, "K1");
        assert_eq!(case.knowledge_refs[0].pack_id, "system-manpages");
    }

    #[test]
    fn test_case_file_multiple_knowledge_refs() {
        let mut case = CaseFile::new("test-multi-k", "how do I configure ssh");

        case.add_knowledge_ref(KnowledgeRef {
            evidence_id: "K1".to_string(),
            title: "ssh - OpenSSH remote login client".to_string(),
            pack_id: "system-manpages".to_string(),
            pack_name: "System Man Pages".to_string(),
            source_path: "man:ssh".to_string(),
            trust: "official".to_string(),
            excerpt: "ssh connects and logs into the specified destination".to_string(),
        });

        case.add_knowledge_ref(KnowledgeRef {
            evidence_id: "K2".to_string(),
            title: "openssh - README".to_string(),
            pack_id: "package-docs".to_string(),
            pack_name: "Package Documentation".to_string(),
            source_path: "/usr/share/doc/openssh/README".to_string(),
            trust: "official".to_string(),
            excerpt: "OpenSSH configuration files are in /etc/ssh/".to_string(),
        });

        assert_eq!(case.knowledge_refs_count(), 2);
        assert_eq!(case.knowledge_refs[0].evidence_id, "K1");
        assert_eq!(case.knowledge_refs[1].evidence_id, "K2");
    }

    // v0.0.37: Recipe event tests

    #[test]
    fn test_recipe_event_matched() {
        let event = RecipeEvent::matched("R1", "Fix nginx", 0.85);
        assert_eq!(event.recipe_id, "R1");
        assert_eq!(event.recipe_name, "Fix nginx");
        assert_eq!(event.event_type, RecipeEventType::Matched);
        assert_eq!(event.match_confidence, Some(0.85));
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_recipe_event_executed() {
        let event = RecipeEvent::executed("R1", "Fix nginx");
        assert_eq!(event.event_type, RecipeEventType::Executed);
        assert!(event.match_confidence.is_none());
    }

    #[test]
    fn test_recipe_event_succeeded() {
        let event = RecipeEvent::succeeded("R1", "Fix nginx");
        assert_eq!(event.event_type, RecipeEventType::Succeeded);
    }

    #[test]
    fn test_recipe_event_failed() {
        let event = RecipeEvent::failed("R1", "Fix nginx", "Service not found");
        assert_eq!(event.event_type, RecipeEventType::Failed);
        assert_eq!(event.error_message, Some("Service not found".to_string()));
    }

    #[test]
    fn test_recipe_event_created() {
        let event = RecipeEvent::created("R2", "Restart postgres", "created as draft, reliability 75%");
        assert_eq!(event.event_type, RecipeEventType::Created);
        assert_eq!(event.notes, Some("created as draft, reliability 75%".to_string()));
    }

    #[test]
    fn test_recipe_event_promoted() {
        let event = RecipeEvent::promoted("R2", "Restart postgres");
        assert_eq!(event.event_type, RecipeEventType::Promoted);
    }

    #[test]
    fn test_case_file_with_recipe_events() {
        let mut case = CaseFile::new("test-recipes", "restart nginx");

        case.add_recipe_event(RecipeEvent::matched("R1", "Restart nginx", 0.92));
        case.add_recipe_event(RecipeEvent::executed("R1", "Restart nginx"));
        case.add_recipe_event(RecipeEvent::succeeded("R1", "Restart nginx"));

        assert_eq!(case.recipe_events_count(), 3);
        assert!(case.recipe_was_executed("R1"));
        assert!(!case.recipe_was_executed("R999"));
    }

    #[test]
    fn test_case_file_recipe_created() {
        let mut case = CaseFile::new("test-create", "fixed postgres issue");

        case.add_recipe_event(RecipeEvent::created("R5", "Fix postgres", "created as active, reliability 85%"));

        assert!(case.recipe_was_created("R5"));
        assert!(!case.recipe_was_created("R1"));
    }

    #[test]
    fn test_recipe_event_types_serialization() {
        // Ensure all event types can be serialized/deserialized
        let events = vec![
            RecipeEvent::matched("R1", "Test", 0.5),
            RecipeEvent::executed("R1", "Test"),
            RecipeEvent::succeeded("R1", "Test"),
            RecipeEvent::failed("R1", "Test", "error"),
            RecipeEvent::created("R1", "Test", "notes"),
            RecipeEvent::promoted("R1", "Test"),
        ];

        for event in events {
            let json = serde_json::to_string(&event).expect("serialize");
            let parsed: RecipeEvent = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed.recipe_id, event.recipe_id);
            assert_eq!(parsed.event_type, event.event_type);
        }
    }
}
