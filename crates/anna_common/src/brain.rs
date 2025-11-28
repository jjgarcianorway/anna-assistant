//! Anna Core Brain - v0.85.0
//!
//! The Brain layer sits between annactl and the LLMs.
//! It attempts to answer questions without LLM calls by using:
//! - Stored command patterns (COMMAND_LIBRARY)
//! - Learned output parsers (OUTPUT_PARSERS)
//! - Failure memory to avoid repeating mistakes (FAILURE_MEMORY)
//!
//! Only when Brain cannot answer does it escalate to Junior/Senior.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Brain configuration directory
pub const BRAIN_DIR: &str = "/var/lib/anna/brain";

/// Minimum reliability score to use a cached pattern
pub const MIN_PATTERN_RELIABILITY: f64 = 0.85;

/// Maximum age for failure memory entries (24 hours)
pub const FAILURE_MEMORY_TTL_SECS: u64 = 86400;

// ============================================================================
// Part 1: Command Library
// ============================================================================

/// A stored command pattern with reliability tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    /// The command string (e.g., "lscpu -J")
    pub command: String,
    /// Number of successful uses
    pub success_count: u32,
    /// Number of failed uses
    pub failure_count: u32,
    /// Reliability score (0.0-1.0)
    pub reliability: f64,
    /// Last successful use timestamp
    pub last_success: u64,
    /// Output parser ID to use
    pub parser_id: Option<String>,
}

impl CommandPattern {
    pub fn new(command: &str) -> Self {
        Self {
            command: command.to_string(),
            success_count: 0,
            failure_count: 0,
            reliability: 0.5, // Start neutral
            last_success: 0,
            parser_id: None,
        }
    }

    /// Record a successful use
    pub fn record_success(&mut self) {
        self.success_count += 1;
        self.last_success = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.update_reliability();
    }

    /// Record a failed use
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.update_reliability();
    }

    fn update_reliability(&mut self) {
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.reliability = self.success_count as f64 / total as f64;
        }
    }
}

/// Command library: maps question patterns to commands
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandLibrary {
    /// Pattern -> list of commands (ordered by reliability)
    patterns: HashMap<String, Vec<CommandPattern>>,
}

impl CommandLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(BRAIN_DIR).join("command_library.json");
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(BRAIN_DIR).join("command_library.json");
        fs::create_dir_all(BRAIN_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    /// Normalize a question to a pattern key
    pub fn normalize_question(question: &str) -> String {
        let q = question.to_lowercase();
        // Remove punctuation and extra whitespace
        let q: String = q.chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();
        q.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Get the best command for a question pattern
    pub fn get_best_command(&self, question: &str) -> Option<&CommandPattern> {
        let normalized = Self::normalize_question(question);

        // Try exact match first
        if let Some(commands) = self.patterns.get(&normalized) {
            return commands.iter()
                .filter(|c| c.reliability >= MIN_PATTERN_RELIABILITY)
                .max_by(|a, b| a.reliability.partial_cmp(&b.reliability).unwrap());
        }

        // Try fuzzy match - check if key contains all words from question
        let question_words: Vec<&str> = normalized.split_whitespace().collect();
        for (key, commands) in &self.patterns {
            let key_words: Vec<&str> = key.split_whitespace().collect();
            let matches = question_words.iter()
                .filter(|w| key_words.contains(w))
                .count();
            // If 80% of words match, consider it a match
            if matches * 100 / question_words.len().max(1) >= 80 {
                return commands.iter()
                    .filter(|c| c.reliability >= MIN_PATTERN_RELIABILITY)
                    .max_by(|a, b| a.reliability.partial_cmp(&b.reliability).unwrap());
            }
        }

        None
    }

    /// Register or update a command for a pattern
    pub fn register_command(&mut self, question: &str, command: &str, success: bool) {
        let key = Self::normalize_question(question);
        let commands = self.patterns.entry(key).or_insert_with(Vec::new);

        if let Some(cmd) = commands.iter_mut().find(|c| c.command == command) {
            if success {
                cmd.record_success();
            } else {
                cmd.record_failure();
            }
        } else {
            let mut cmd = CommandPattern::new(command);
            if success {
                cmd.record_success();
            } else {
                cmd.record_failure();
            }
            commands.push(cmd);
        }

        // Sort by reliability
        commands.sort_by(|a, b| b.reliability.partial_cmp(&a.reliability).unwrap());
    }
}

// ============================================================================
// Part 2: Output Parsers
// ============================================================================

/// A learned output parser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputParser {
    /// Parser ID (e.g., "cpu.core_count")
    pub id: String,
    /// The command this parser works with
    pub command: String,
    /// JSON path to extract value (e.g., "lscpu.CPU(s)")
    pub json_path: Option<String>,
    /// Regex pattern for text extraction
    pub regex_pattern: Option<String>,
    /// Field name in structured output
    pub field_name: String,
    /// Expected value type
    pub value_type: ValueType,
    /// Reliability score
    pub reliability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValueType {
    Integer,
    Float,
    String,
    Boolean,
}

/// Output parser registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputParsers {
    parsers: HashMap<String, OutputParser>,
}

impl OutputParsers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(BRAIN_DIR).join("output_parsers.json");
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::with_builtins()
        }
    }

    /// Create with built-in parsers
    pub fn with_builtins() -> Self {
        let mut parsers = HashMap::new();

        // CPU core count parser
        parsers.insert("cpu.core_count".to_string(), OutputParser {
            id: "cpu.core_count".to_string(),
            command: "lscpu".to_string(),
            json_path: None,
            regex_pattern: Some(r"^CPU\(s\):\s*(\d+)".to_string()),
            field_name: "cpu_count".to_string(),
            value_type: ValueType::Integer,
            reliability: 0.95,
        });

        // CPU model parser
        parsers.insert("cpu.model".to_string(), OutputParser {
            id: "cpu.model".to_string(),
            command: "lscpu".to_string(),
            json_path: None,
            regex_pattern: Some(r"^Model name:\s*(.+)$".to_string()),
            field_name: "cpu_model".to_string(),
            value_type: ValueType::String,
            reliability: 0.95,
        });

        // Memory total parser
        parsers.insert("mem.total".to_string(), OutputParser {
            id: "mem.total".to_string(),
            command: "free -h".to_string(),
            json_path: None,
            regex_pattern: Some(r"^Mem:\s+(\S+)".to_string()),
            field_name: "total_memory".to_string(),
            value_type: ValueType::String,
            reliability: 0.95,
        });

        // Disk usage parser
        parsers.insert("disk.usage".to_string(), OutputParser {
            id: "disk.usage".to_string(),
            command: "df -h /".to_string(),
            json_path: None,
            regex_pattern: Some(r"(\d+%)\s+/$".to_string()),
            field_name: "disk_usage".to_string(),
            value_type: ValueType::String,
            reliability: 0.90,
        });

        Self { parsers }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(BRAIN_DIR).join("output_parsers.json");
        fs::create_dir_all(BRAIN_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    /// Get parser by ID
    pub fn get(&self, id: &str) -> Option<&OutputParser> {
        self.parsers.get(id)
    }

    /// Find parser for a command
    pub fn find_for_command(&self, command: &str) -> Vec<&OutputParser> {
        self.parsers.values()
            .filter(|p| command.starts_with(&p.command))
            .collect()
    }
}

// ============================================================================
// Part 3: Failure Memory
// ============================================================================

/// A recorded failure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureRecord {
    /// The command that failed
    pub command: String,
    /// Type of failure
    pub failure_type: FailureType,
    /// Question context
    pub question: String,
    /// Timestamp
    pub timestamp: u64,
    /// Error message
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailureType {
    BadOutput,       // Command produced unparseable output
    WrongDomain,     // Command doesn't answer the question
    WrongFlags,      // Command flags were incorrect
    Timeout,         // Command timed out
    PermissionDenied,// Not allowed to run
    CommandNotFound, // Command doesn't exist
}

/// Failure memory to avoid repeating mistakes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FailureMemory {
    failures: Vec<FailureRecord>,
}

impl FailureMemory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = PathBuf::from(BRAIN_DIR).join("failure_memory.json");
        if let Ok(data) = fs::read_to_string(&path) {
            let mut mem: Self = serde_json::from_str(&data).unwrap_or_default();
            mem.cleanup_old();
            mem
        } else {
            Self::default()
        }
    }

    /// Save to disk
    pub fn save(&self) -> std::io::Result<()> {
        let path = PathBuf::from(BRAIN_DIR).join("failure_memory.json");
        fs::create_dir_all(BRAIN_DIR)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(path, data)
    }

    /// Remove entries older than TTL
    fn cleanup_old(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.failures.retain(|f| now - f.timestamp < FAILURE_MEMORY_TTL_SECS);
    }

    /// Record a failure
    pub fn record(&mut self, command: &str, failure_type: FailureType, question: &str, error: Option<&str>) {
        let record = FailureRecord {
            command: command.to_string(),
            failure_type,
            question: question.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_message: error.map(|s| s.to_string()),
        };
        self.failures.push(record);
        self.cleanup_old();
    }

    /// Check if a command has recently failed
    pub fn has_recent_failure(&self, command: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.failures.iter().any(|f| {
            f.command == command && (now - f.timestamp) < FAILURE_MEMORY_TTL_SECS
        })
    }

    /// Get failure count for a command
    pub fn failure_count(&self, command: &str) -> usize {
        self.failures.iter().filter(|f| f.command == command).count()
    }
}

// ============================================================================
// Part 4: Brain Core
// ============================================================================

/// Brain decision result
#[derive(Debug, Clone)]
pub enum BrainDecision {
    /// Brain can answer directly without LLM
    SelfSolve {
        answer: String,
        command_used: String,
        confidence: f64,
    },
    /// Brain has partial answer, can help Junior
    PartialSolve {
        partial_answer: String,
        suggested_command: String,
        confidence: f64,
    },
    /// Brain cannot answer, escalate to LLM
    Escalate {
        reason: String,
        suggested_commands: Vec<String>,
    },
}

/// Brain summary for debug output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainSummary {
    pub pattern_match: bool,
    pub matched_key: Option<String>,
    pub stored_command: Option<String>,
    pub previous_success_score: Option<f64>,
    pub decision: String,
    pub time_ms: u64,
}

impl Default for BrainSummary {
    fn default() -> Self {
        Self {
            pattern_match: false,
            matched_key: None,
            stored_command: None,
            previous_success_score: None,
            decision: "escalate".to_string(),
            time_ms: 0,
        }
    }
}

/// The Anna Core Brain
pub struct Brain {
    pub command_library: CommandLibrary,
    pub output_parsers: OutputParsers,
    pub failure_memory: FailureMemory,
}

impl Brain {
    /// Create a new Brain, loading state from disk
    pub fn new() -> Self {
        Self {
            command_library: CommandLibrary::load(),
            output_parsers: OutputParsers::load(),
            failure_memory: FailureMemory::load(),
        }
    }

    /// Save all state to disk
    pub fn save(&self) -> std::io::Result<()> {
        self.command_library.save()?;
        self.output_parsers.save()?;
        self.failure_memory.save()?;
        Ok(())
    }

    /// Try to answer a question without LLM
    pub fn try_answer(&self, question: &str) -> (BrainDecision, BrainSummary) {
        let start = Instant::now();
        let mut summary = BrainSummary::default();

        // Check command library for known pattern
        if let Some(pattern) = self.command_library.get_best_command(question) {
            summary.pattern_match = true;
            summary.matched_key = Some(CommandLibrary::normalize_question(question));
            summary.stored_command = Some(pattern.command.clone());
            summary.previous_success_score = Some(pattern.reliability);

            // Check if command has recent failures
            if self.failure_memory.has_recent_failure(&pattern.command) {
                summary.decision = "escalate_due_to_recent_failure".to_string();
                summary.time_ms = start.elapsed().as_millis() as u64;
                return (BrainDecision::Escalate {
                    reason: format!("Command '{}' has recent failures", pattern.command),
                    suggested_commands: self.get_alternative_commands(question),
                }, summary);
            }

            // High reliability - can self-solve
            if pattern.reliability >= 0.90 {
                summary.decision = "self_solve".to_string();
                summary.time_ms = start.elapsed().as_millis() as u64;
                return (BrainDecision::SelfSolve {
                    answer: String::new(), // Will be filled after command execution
                    command_used: pattern.command.clone(),
                    confidence: pattern.reliability,
                }, summary);
            }

            // Medium reliability - partial solve
            if pattern.reliability >= 0.70 {
                summary.decision = "partial_solve".to_string();
                summary.time_ms = start.elapsed().as_millis() as u64;
                return (BrainDecision::PartialSolve {
                    partial_answer: String::new(),
                    suggested_command: pattern.command.clone(),
                    confidence: pattern.reliability,
                }, summary);
            }
        }

        // No pattern match - escalate
        summary.decision = "escalate_no_pattern".to_string();
        summary.time_ms = start.elapsed().as_millis() as u64;
        (BrainDecision::Escalate {
            reason: "No known pattern matches this question".to_string(),
            suggested_commands: self.get_suggested_commands_for_question(question),
        }, summary)
    }

    /// Get alternative commands when primary fails
    fn get_alternative_commands(&self, question: &str) -> Vec<String> {
        let normalized = CommandLibrary::normalize_question(question);
        if let Some(commands) = self.command_library.patterns.get(&normalized) {
            commands.iter()
                .filter(|c| !self.failure_memory.has_recent_failure(&c.command))
                .map(|c| c.command.clone())
                .take(3)
                .collect()
        } else {
            vec![]
        }
    }

    /// Suggest commands based on question keywords
    fn get_suggested_commands_for_question(&self, question: &str) -> Vec<String> {
        let q = question.to_lowercase();
        let mut suggestions = Vec::new();

        if q.contains("cpu") || q.contains("core") || q.contains("processor") {
            suggestions.push("lscpu".to_string());
        }
        if q.contains("memory") || q.contains("ram") || q.contains("mem") {
            suggestions.push("free -h".to_string());
        }
        if q.contains("disk") || q.contains("storage") || q.contains("space") {
            suggestions.push("df -h".to_string());
            suggestions.push("lsblk".to_string());
        }
        if q.contains("network") || q.contains("ip") || q.contains("interface") {
            suggestions.push("ip addr".to_string());
        }
        if q.contains("kernel") || q.contains("version") || q.contains("uname") {
            suggestions.push("uname -a".to_string());
        }
        if q.contains("uptime") || q.contains("running") {
            suggestions.push("uptime".to_string());
        }
        if q.contains("process") || q.contains("running") {
            suggestions.push("ps aux".to_string());
        }
        if q.contains("service") || q.contains("systemd") {
            suggestions.push("systemctl status".to_string());
        }

        suggestions
    }

    /// Learn from a successful answer
    pub fn learn_success(&mut self, question: &str, command: &str) {
        self.command_library.register_command(question, command, true);
        let _ = self.command_library.save();
    }

    /// Learn from a failed answer
    pub fn learn_failure(&mut self, question: &str, command: &str, failure_type: FailureType, error: Option<&str>) {
        self.command_library.register_command(question, command, false);
        self.failure_memory.record(command, failure_type, question, error);
        let _ = self.command_library.save();
        let _ = self.failure_memory.save();
    }
}

impl Default for Brain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_pattern_reliability() {
        let mut pattern = CommandPattern::new("lscpu");
        pattern.record_success();
        pattern.record_success();
        pattern.record_failure();
        assert!((pattern.reliability - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_normalize_question() {
        assert_eq!(
            CommandLibrary::normalize_question("How many CPU cores?"),
            "how many cpu cores"
        );
        assert_eq!(
            CommandLibrary::normalize_question("What's my   RAM?"),
            "whats my ram"
        );
    }

    #[test]
    fn test_command_library_registration() {
        let mut lib = CommandLibrary::new();
        lib.register_command("how many cpu cores", "lscpu", true);
        lib.register_command("how many cpu cores", "lscpu", true);

        let cmd = lib.get_best_command("how many cpu cores");
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().success_count, 2);
    }

    #[test]
    fn test_failure_memory() {
        let mut mem = FailureMemory::new();
        mem.record("bad_cmd", FailureType::BadOutput, "test", None);
        assert!(mem.has_recent_failure("bad_cmd"));
        assert!(!mem.has_recent_failure("good_cmd"));
    }

    #[test]
    fn test_brain_escalate_no_pattern() {
        let brain = Brain {
            command_library: CommandLibrary::new(),
            output_parsers: OutputParsers::new(),
            failure_memory: FailureMemory::new(),
        };

        let (decision, summary) = brain.try_answer("What is the meaning of life?");
        assert!(!summary.pattern_match);
        matches!(decision, BrainDecision::Escalate { .. });
    }

    #[test]
    fn test_output_parsers_builtins() {
        let parsers = OutputParsers::with_builtins();
        assert!(parsers.get("cpu.core_count").is_some());
        assert!(parsers.get("mem.total").is_some());
    }

    #[test]
    fn test_brain_suggested_commands() {
        let brain = Brain::default();
        let cmds = brain.get_suggested_commands_for_question("How much RAM do I have?");
        assert!(cmds.contains(&"free -h".to_string()));
    }
}
