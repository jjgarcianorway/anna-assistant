//! Memory System v0.0.13
//!
//! Local conversation memory for Anna:
//! - Session summaries (compact records of interactions)
//! - Privacy-preserving by default (no raw transcripts unless configured)
//! - Evidence ID generation for memory entries
//! - Query support for introspection
//!
//! Storage: /var/lib/anna/memory/
//! - sessions.jsonl - append-only session records
//! - memory_index.json - searchable index

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Memory storage directory
pub const MEMORY_DIR: &str = "/var/lib/anna/memory";
/// Sessions file (JSONL format)
pub const SESSIONS_FILE: &str = "/var/lib/anna/memory/sessions.jsonl";
/// Memory index file
pub const MEMORY_INDEX_FILE: &str = "/var/lib/anna/memory/memory_index.json";
/// Archive directory for deleted memories
pub const MEMORY_ARCHIVE_DIR: &str = "/var/lib/anna/memory/archive";

/// Schema version for memory records
pub const MEMORY_SCHEMA_VERSION: u32 = 1;

/// Evidence ID prefix for memory entries
pub const MEMORY_EVIDENCE_PREFIX: &str = "MEM";

/// Generate a unique memory evidence ID
pub fn generate_memory_evidence_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{}{:05}", MEMORY_EVIDENCE_PREFIX, ts % 100000)
}

/// A session record - compact summary of an interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Unique evidence ID for this memory
    pub evidence_id: String,
    /// Request ID (if available)
    pub request_id: Option<String>,
    /// Timestamp of the interaction
    pub timestamp: DateTime<Utc>,
    /// User's original request text
    pub request_text: String,
    /// Translator's plan summary (intent, targets, risk)
    pub translator_summary: TranslatorSummary,
    /// Tools used during execution
    pub tools_used: Vec<ToolUsage>,
    /// Evidence IDs referenced
    pub evidence_ids_referenced: Vec<String>,
    /// Final answer summary (brief)
    pub answer_summary: String,
    /// Junior's reliability score (0-100)
    pub reliability_score: u32,
    /// Junior's critique (brief)
    pub junior_critique: Option<String>,
    /// Whether a recipe was created or updated
    pub recipe_action: Option<RecipeAction>,
    /// Whether this was a REPL session or one-shot
    pub session_type: SessionType,
    /// Success flag
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Schema version
    pub schema_version: u32,
}

/// Translator's plan summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatorSummary {
    /// Detected intent
    pub intent: String,
    /// Targets (cpu, memory, docker, etc.)
    pub targets: Vec<String>,
    /// Risk level
    pub risk: String,
    /// Was clarification needed?
    pub clarification_needed: bool,
}

/// Tool usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsage {
    /// Tool name
    pub tool_name: String,
    /// Whether it's a mutation tool
    pub is_mutation: bool,
    /// Whether it succeeded
    pub success: bool,
}

/// Recipe action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipeAction {
    /// New recipe created
    Created { recipe_id: String },
    /// Existing recipe updated
    Updated { recipe_id: String },
    /// Recipe reused
    Reused { recipe_id: String },
    /// Experimental draft created (low reliability)
    DraftCreated { recipe_id: String },
}

/// Session type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Interactive REPL session
    Repl,
    /// One-shot command
    OneShot,
}

impl SessionRecord {
    /// Create a new session record
    pub fn new(request_text: &str, session_type: SessionType) -> Self {
        Self {
            evidence_id: generate_memory_evidence_id(),
            request_id: None,
            timestamp: Utc::now(),
            request_text: request_text.to_string(),
            translator_summary: TranslatorSummary {
                intent: String::new(),
                targets: Vec::new(),
                risk: "unknown".to_string(),
                clarification_needed: false,
            },
            tools_used: Vec::new(),
            evidence_ids_referenced: Vec::new(),
            answer_summary: String::new(),
            reliability_score: 0,
            junior_critique: None,
            recipe_action: None,
            session_type,
            success: false,
            duration_ms: 0,
            schema_version: MEMORY_SCHEMA_VERSION,
        }
    }

    /// Format as a one-line summary
    pub fn format_summary(&self) -> String {
        let recipe_info = match &self.recipe_action {
            Some(RecipeAction::Created { recipe_id }) => format!(" [recipe created: {}]", recipe_id),
            Some(RecipeAction::Updated { recipe_id }) => format!(" [recipe updated: {}]", recipe_id),
            Some(RecipeAction::Reused { recipe_id }) => format!(" [used recipe: {}]", recipe_id),
            Some(RecipeAction::DraftCreated { recipe_id }) => format!(" [draft: {}]", recipe_id),
            None => String::new(),
        };

        format!(
            "[{}] {} - {} ({}% reliability){}",
            self.evidence_id,
            self.timestamp.format("%Y-%m-%d %H:%M"),
            truncate_string(&self.request_text, 50),
            self.reliability_score,
            recipe_info
        )
    }

    /// Format detailed view
    pub fn format_detail(&self) -> String {
        let mut lines = vec![
            format!("[{}] Session Record", self.evidence_id),
            format!("  Timestamp:   {}", self.timestamp.format("%Y-%m-%d %H:%M:%S UTC")),
            format!("  Type:        {:?}", self.session_type),
            format!("  Request:     {}", self.request_text),
            format!("  Intent:      {}", self.translator_summary.intent),
            format!("  Targets:     {}", self.translator_summary.targets.join(", ")),
            format!("  Risk:        {}", self.translator_summary.risk),
            format!("  Reliability: {}%", self.reliability_score),
            format!("  Success:     {}", self.success),
        ];

        if !self.tools_used.is_empty() {
            let tools: Vec<String> = self.tools_used.iter()
                .map(|t| if t.is_mutation { format!("{}*", t.tool_name) } else { t.tool_name.clone() })
                .collect();
            lines.push(format!("  Tools:       {}", tools.join(", ")));
        }

        if let Some(ref critique) = self.junior_critique {
            lines.push(format!("  Critique:    {}", critique));
        }

        if let Some(ref action) = self.recipe_action {
            lines.push(format!("  Recipe:      {:?}", action));
        }

        lines.push(format!("  Answer:      {}", truncate_string(&self.answer_summary, 100)));

        lines.join("\n")
    }
}

/// Memory manager for storing and querying session records
#[derive(Debug)]
pub struct MemoryManager {
    /// Whether to store raw transcripts
    pub store_raw: bool,
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self { store_raw: false }
    }
}

impl MemoryManager {
    /// Create with configuration
    pub fn with_config(store_raw: bool) -> Self {
        Self { store_raw }
    }

    /// Ensure memory directory exists
    pub fn ensure_dirs() -> std::io::Result<()> {
        fs::create_dir_all(MEMORY_DIR)?;
        fs::create_dir_all(MEMORY_ARCHIVE_DIR)?;
        Ok(())
    }

    /// Store a session record
    pub fn store_session(&self, record: &SessionRecord) -> std::io::Result<()> {
        Self::ensure_dirs()?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(SESSIONS_FILE)?;

        let json = serde_json::to_string(record)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        writeln!(file, "{}", json)?;

        // Update index
        self.update_index(record)?;

        Ok(())
    }

    /// Update the memory index with a new record
    fn update_index(&self, record: &SessionRecord) -> std::io::Result<()> {
        let mut index = MemoryIndex::load();

        index.total_sessions += 1;
        index.last_session_at = Some(record.timestamp);

        if record.success {
            index.successful_sessions += 1;
        }

        if record.recipe_action.is_some() {
            index.sessions_with_recipes += 1;
        }

        // Add to recent sessions (keep last 100)
        index.recent_session_ids.insert(0, record.evidence_id.clone());
        index.recent_session_ids.truncate(100);

        // Update keyword index (simple word extraction)
        let words = extract_keywords(&record.request_text);
        for word in words {
            index.keyword_index
                .entry(word)
                .or_insert_with(Vec::new)
                .push(record.evidence_id.clone());
        }

        index.save()
    }

    /// Get recent sessions
    pub fn get_recent_sessions(&self, limit: usize) -> Vec<SessionRecord> {
        let index = MemoryIndex::load();
        let mut sessions = Vec::new();

        for evidence_id in index.recent_session_ids.iter().take(limit) {
            if let Some(record) = self.get_session_by_id(evidence_id) {
                sessions.push(record);
            }
        }

        sessions
    }

    /// Get session by evidence ID
    pub fn get_session_by_id(&self, evidence_id: &str) -> Option<SessionRecord> {
        let path = Path::new(SESSIONS_FILE);
        if !path.exists() {
            return None;
        }

        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
                    if record.evidence_id == evidence_id {
                        return Some(record);
                    }
                }
            }
        }

        None
    }

    /// Search sessions by keyword
    pub fn search_sessions(&self, query: &str, limit: usize) -> Vec<SessionRecord> {
        let index = MemoryIndex::load();
        let query_words = extract_keywords(query);

        // Find matching evidence IDs
        let mut matches: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for word in &query_words {
            if let Some(ids) = index.keyword_index.get(word) {
                for id in ids {
                    *matches.entry(id.clone()).or_insert(0) += 1;
                }
            }
        }

        // Sort by match count
        let mut sorted: Vec<_> = matches.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // Fetch records
        let mut sessions = Vec::new();
        for (evidence_id, _) in sorted.into_iter().take(limit) {
            if let Some(record) = self.get_session_by_id(&evidence_id) {
                sessions.push(record);
            }
        }

        sessions
    }

    /// Get sessions that created or updated recipes
    pub fn get_learning_sessions(&self, limit: usize) -> Vec<SessionRecord> {
        let path = Path::new(SESSIONS_FILE);
        if !path.exists() {
            return Vec::new();
        }

        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Vec::new(),
        };

        let reader = BufReader::new(file);
        let mut learning_sessions = Vec::new();

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
                    if matches!(
                        record.recipe_action,
                        Some(RecipeAction::Created { .. }) | Some(RecipeAction::Updated { .. })
                    ) {
                        learning_sessions.push(record);
                    }
                }
            }
        }

        // Return most recent first
        learning_sessions.reverse();
        learning_sessions.truncate(limit);
        learning_sessions
    }

    /// Archive a session (for "forget" functionality)
    pub fn archive_session(&self, evidence_id: &str) -> std::io::Result<bool> {
        Self::ensure_dirs()?;

        // Find and archive the session
        let path = Path::new(SESSIONS_FILE);
        if !path.exists() {
            return Ok(false);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut found = false;
        let mut remaining_lines = Vec::new();
        let mut archived_record: Option<SessionRecord> = None;

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(record) = serde_json::from_str::<SessionRecord>(&line) {
                    if record.evidence_id == evidence_id {
                        found = true;
                        archived_record = Some(record);
                    } else {
                        remaining_lines.push(line);
                    }
                } else {
                    remaining_lines.push(line);
                }
            }
        }

        if !found {
            return Ok(false);
        }

        // Write archive record
        if let Some(record) = archived_record {
            let archive_path = format!("{}/{}.json", MEMORY_ARCHIVE_DIR, evidence_id);
            let archive_data = ArchivedMemory {
                record,
                archived_at: Utc::now(),
                reason: "user_requested_forget".to_string(),
            };
            let json = serde_json::to_string_pretty(&archive_data)?;
            fs::write(archive_path, json)?;
        }

        // Rewrite sessions file without the archived record
        let mut file = File::create(path)?;
        for line in remaining_lines {
            writeln!(file, "{}", line)?;
        }

        // Update index
        let mut index = MemoryIndex::load();
        index.total_sessions = index.total_sessions.saturating_sub(1);
        index.recent_session_ids.retain(|id| id != evidence_id);
        index.archived_count += 1;
        index.save()?;

        Ok(true)
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let index = MemoryIndex::load();

        MemoryStats {
            total_sessions: index.total_sessions,
            successful_sessions: index.successful_sessions,
            sessions_with_recipes: index.sessions_with_recipes,
            archived_count: index.archived_count,
            last_session_at: index.last_session_at,
            store_raw: self.store_raw,
        }
    }
}

/// Archived memory record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchivedMemory {
    pub record: SessionRecord,
    pub archived_at: DateTime<Utc>,
    pub reason: String,
}

/// Memory index for fast lookups
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryIndex {
    /// Schema version
    pub schema_version: u32,
    /// Total sessions stored
    pub total_sessions: u64,
    /// Successful sessions
    pub successful_sessions: u64,
    /// Sessions that created/updated recipes
    pub sessions_with_recipes: u64,
    /// Archived (forgotten) count
    pub archived_count: u64,
    /// Last session timestamp
    pub last_session_at: Option<DateTime<Utc>>,
    /// Recent session IDs (most recent first)
    pub recent_session_ids: Vec<String>,
    /// Keyword to evidence ID mapping (simple search index)
    #[serde(default)]
    pub keyword_index: std::collections::HashMap<String, Vec<String>>,
}

impl MemoryIndex {
    /// Load from file or create default
    pub fn load() -> Self {
        let path = Path::new(MEMORY_INDEX_FILE);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(index) = serde_json::from_str(&content) {
                    return index;
                }
            }
        }
        Self {
            schema_version: MEMORY_SCHEMA_VERSION,
            ..Default::default()
        }
    }

    /// Save to file
    pub fn save(&self) -> std::io::Result<()> {
        MemoryManager::ensure_dirs()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(MEMORY_INDEX_FILE, json)
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub total_sessions: u64,
    pub successful_sessions: u64,
    pub sessions_with_recipes: u64,
    pub archived_count: u64,
    pub last_session_at: Option<DateTime<Utc>>,
    pub store_raw: bool,
}

impl MemoryStats {
    /// Format for status display
    pub fn format_summary(&self) -> String {
        let last = self.last_session_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "never".to_string());

        format!(
            "{} sessions ({} with recipes), last: {}",
            self.total_sessions,
            self.sessions_with_recipes,
            last
        )
    }
}

/// Extract keywords from text for indexing
fn extract_keywords(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() >= 3)
        .map(|s| s.to_string())
        .collect()
}

/// Truncate string with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_memory_evidence_id() {
        let id = generate_memory_evidence_id();
        assert!(id.starts_with(MEMORY_EVIDENCE_PREFIX));
        assert!(id.len() > MEMORY_EVIDENCE_PREFIX.len());
    }

    #[test]
    fn test_session_record_new() {
        let record = SessionRecord::new("test request", SessionType::OneShot);
        assert!(record.evidence_id.starts_with(MEMORY_EVIDENCE_PREFIX));
        assert_eq!(record.request_text, "test request");
        assert_eq!(record.session_type, SessionType::OneShot);
        assert!(!record.success);
    }

    #[test]
    fn test_session_record_format_summary() {
        let mut record = SessionRecord::new("what CPU do I have?", SessionType::OneShot);
        record.reliability_score = 85;
        let summary = record.format_summary();
        assert!(summary.contains("what CPU"));
        assert!(summary.contains("85%"));
    }

    #[test]
    fn test_extract_keywords() {
        let keywords = extract_keywords("What CPU do I have?");
        assert!(keywords.contains(&"what".to_string()));
        assert!(keywords.contains(&"cpu".to_string()));
        assert!(keywords.contains(&"have".to_string()));
        // "do" and "I" are too short
        assert!(!keywords.contains(&"do".to_string()));
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(truncate_string("this is a long string", 10), "this is...");
    }

    #[test]
    fn test_memory_index_default() {
        let index = MemoryIndex::default();
        assert_eq!(index.total_sessions, 0);
        assert!(index.recent_session_ids.is_empty());
    }

    #[test]
    fn test_memory_stats_format() {
        let stats = MemoryStats {
            total_sessions: 42,
            successful_sessions: 40,
            sessions_with_recipes: 5,
            archived_count: 2,
            last_session_at: None,
            store_raw: false,
        };
        let summary = stats.format_summary();
        assert!(summary.contains("42 sessions"));
        assert!(summary.contains("5 with recipes"));
    }
}
