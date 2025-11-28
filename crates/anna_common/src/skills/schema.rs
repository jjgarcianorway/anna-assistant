//! Skill Schema v0.42.0
//!
//! Defines the data structures for Anna's learned skills.
//! Skills are parameterized command templates that can be reused.
//!
//! v0.42.0: Added trust_score for pain-driven learning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version for migrations
pub const SKILL_SCHEMA_VERSION: u32 = 1;

/// Parameter type for skill templates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    String,
    Integer,
    Boolean,
    Duration,
    Path,
}

impl ParamType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ParamType::String => "string",
            ParamType::Integer => "integer",
            ParamType::Boolean => "boolean",
            ParamType::Duration => "duration",
            ParamType::Path => "path",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "string" => ParamType::String,
            "integer" => ParamType::Integer,
            "boolean" => ParamType::Boolean,
            "duration" => ParamType::Duration,
            "path" => ParamType::Path,
            _ => ParamType::String,
        }
    }
}

/// Parameter schema for a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamSchema {
    /// Parameter name (e.g., "service_name", "since")
    pub name: String,
    /// Parameter type
    pub param_type: ParamType,
    /// Human-readable description
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<String>,
    /// Example values for the LLM
    pub examples: Vec<String>,
}

/// Statistics for a skill
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillStats {
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Average latency in milliseconds
    pub avg_latency_ms: u64,
    /// Reliability score [0.0, 1.0]
    pub reliability_score: f64,
    /// Trust score [0, 100] - v0.42.0 pain tracking
    /// Starts at 50, increases on success, decreases on failure
    /// Skill is "untrusted" if trust_score < 40
    pub trust_score: u8,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// First learned timestamp
    pub first_learned: DateTime<Utc>,
}

impl SkillStats {
    /// Trust threshold - below this, skill is "untrusted"
    pub const UNTRUSTED_THRESHOLD: u8 = 40;
    /// Default starting trust
    pub const DEFAULT_TRUST: u8 = 50;
    /// Trust gain on success
    pub const TRUST_GAIN: u8 = 5;
    /// Trust loss on failure
    pub const TRUST_LOSS: u8 = 10;

    pub fn new() -> Self {
        Self {
            success_count: 0,
            failure_count: 0,
            avg_latency_ms: 0,
            reliability_score: 0.5, // Start neutral
            trust_score: Self::DEFAULT_TRUST,
            last_used: None,
            first_learned: Utc::now(),
        }
    }

    /// Record a successful execution
    pub fn record_success(&mut self, latency_ms: u64) {
        let total = self.success_count + self.failure_count;
        self.avg_latency_ms = if total == 0 {
            latency_ms
        } else {
            (self.avg_latency_ms * total + latency_ms) / (total + 1)
        };
        self.success_count += 1;
        self.last_used = Some(Utc::now());
        self.update_reliability();
        // Increase trust on success (capped at 100)
        self.trust_score = self.trust_score.saturating_add(Self::TRUST_GAIN).min(100);
    }

    /// Record a failed execution
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_used = Some(Utc::now());
        self.update_reliability();
        // Decrease trust on failure (floored at 0)
        self.trust_score = self.trust_score.saturating_sub(Self::TRUST_LOSS);
    }

    fn update_reliability(&mut self) {
        let total = self.success_count + self.failure_count;
        if total > 0 {
            self.reliability_score = self.success_count as f64 / total as f64;
        }
    }

    /// Check if skill is reliable enough to use
    pub fn is_reliable(&self) -> bool {
        let total = self.success_count + self.failure_count;
        // Need at least 3 uses and 70% success rate
        total >= 3 && self.reliability_score >= 0.7
    }

    /// Check if skill is trusted (trust_score >= 40)
    pub fn is_trusted(&self) -> bool {
        self.trust_score >= Self::UNTRUSTED_THRESHOLD
    }

    /// Check if skill should be retried (not completely broken)
    pub fn should_retry(&self) -> bool {
        // Don't retry if 5+ failures and <30% success
        let total = self.success_count + self.failure_count;
        !(total >= 5 && self.reliability_score < 0.3)
    }

    /// Get trust level as percentage (0-100)
    pub fn trust_percent(&self) -> u8 {
        self.trust_score
    }
}

/// A learned skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill identifier (e.g., "journalctl.service_window")
    pub skill_id: String,
    /// Version number for evolution
    pub version: u32,
    /// Intent category (e.g., "logs_for_service", "disk_usage")
    pub intent: String,
    /// Human-readable description
    pub description: String,
    /// Command template with {{param}} placeholders
    pub command_template: String,
    /// Default parameter values
    pub default_parameters: HashMap<String, String>,
    /// Parameter schema
    pub parameter_schema: Vec<ParamSchema>,
    /// How to parse the output
    pub parser_spec: String,
    /// Example questions this skill can answer
    pub question_examples: Vec<String>,
    /// Usage statistics
    pub stats: SkillStats,
    /// Whether this is a built-in skill (not learned)
    pub builtin: bool,
}

impl Skill {
    /// Create a new skill
    pub fn new(skill_id: &str, intent: &str, description: &str, command_template: &str) -> Self {
        Self {
            skill_id: skill_id.to_string(),
            version: 1,
            intent: intent.to_string(),
            description: description.to_string(),
            command_template: command_template.to_string(),
            default_parameters: HashMap::new(),
            parameter_schema: Vec::new(),
            parser_spec: "text".to_string(),
            question_examples: Vec::new(),
            stats: SkillStats::new(),
            builtin: false,
        }
    }

    /// Add a parameter to the schema
    pub fn with_param(
        mut self,
        name: &str,
        param_type: ParamType,
        description: &str,
        required: bool,
    ) -> Self {
        self.parameter_schema.push(ParamSchema {
            name: name.to_string(),
            param_type,
            description: description.to_string(),
            required,
            default: None,
            examples: Vec::new(),
        });
        self
    }

    /// Add default value for a parameter
    pub fn with_default(mut self, name: &str, value: &str) -> Self {
        self.default_parameters.insert(name.to_string(), value.to_string());
        self
    }

    /// Add example question
    pub fn with_example(mut self, question: &str) -> Self {
        self.question_examples.push(question.to_string());
        self
    }

    /// Mark as builtin
    pub fn builtin(mut self) -> Self {
        self.builtin = true;
        self
    }

    /// Set parser spec
    pub fn with_parser(mut self, parser: &str) -> Self {
        self.parser_spec = parser.to_string();
        self
    }

    /// Build the command with given parameters
    pub fn build_command(&self, params: &HashMap<String, String>) -> Result<Vec<String>, String> {
        let mut cmd = self.command_template.clone();

        // Apply defaults first
        for (key, default) in &self.default_parameters {
            let placeholder = format!("{{{{{}}}}}", key);
            if !params.contains_key(key) {
                cmd = cmd.replace(&placeholder, default);
            }
        }

        // Apply provided parameters
        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            cmd = cmd.replace(&placeholder, value);
        }

        // Check for missing required parameters
        for param in &self.parameter_schema {
            let placeholder = format!("{{{{{}}}}}", param.name);
            if cmd.contains(&placeholder) && param.required {
                return Err(format!("Missing required parameter: {}", param.name));
            }
        }

        // Split into command parts
        // Simple split on spaces (doesn't handle quoted strings yet)
        let parts: Vec<String> = cmd
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        Ok(parts)
    }

    /// Check if this skill matches a question intent
    pub fn matches_intent(&self, question_intent: &str) -> bool {
        self.intent.eq_ignore_ascii_case(question_intent)
    }

    /// Calculate match score against a question
    pub fn match_score(&self, question: &str) -> f64 {
        let q_lower = question.to_lowercase();
        let mut score = 0.0;

        // Check question examples
        for example in &self.question_examples {
            let ex_lower = example.to_lowercase();
            if q_lower.contains(&ex_lower) || ex_lower.contains(&q_lower) {
                score += 0.5;
            }
            // Word overlap
            let ex_words: std::collections::HashSet<_> = ex_lower.split_whitespace().collect();
            let q_words: std::collections::HashSet<_> = q_lower.split_whitespace().collect();
            let overlap = ex_words.intersection(&q_words).count();
            if overlap > 0 {
                score += 0.1 * overlap as f64;
            }
        }

        // Check description
        let desc_lower = self.description.to_lowercase();
        let desc_words: std::collections::HashSet<_> = desc_lower.split_whitespace().collect();
        let q_words: std::collections::HashSet<_> = q_lower.split_whitespace().collect();
        let overlap = desc_words.intersection(&q_words).count();
        score += 0.05 * overlap as f64;

        // Boost by reliability
        score *= 0.5 + 0.5 * self.stats.reliability_score;

        score.min(1.0)
    }
}

/// Query filter for skills
#[derive(Debug, Clone, Default)]
pub struct SkillQuery {
    /// Filter by intent
    pub intent: Option<String>,
    /// Filter by minimum reliability
    pub min_reliability: Option<f64>,
    /// Filter by builtin status
    pub builtin_only: Option<bool>,
    /// Limit results
    pub limit: Option<usize>,
}

impl SkillQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intent(mut self, intent: &str) -> Self {
        self.intent = Some(intent.to_string());
        self
    }

    pub fn min_reliability(mut self, min: f64) -> Self {
        self.min_reliability = Some(min);
        self
    }

    pub fn builtin_only(mut self) -> Self {
        self.builtin_only = Some(true);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
}

/// Anna's progression levels based on skill mastery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnnaLevel {
    Intern,
    Journeyman,
    Master,
    Mythic,
}

impl AnnaLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnnaLevel::Intern => "Intern",
            AnnaLevel::Journeyman => "Journeyman",
            AnnaLevel::Master => "Master",
            AnnaLevel::Mythic => "Mythic",
        }
    }

    /// Determine level based on stats
    pub fn from_stats(skill_count: usize, avg_reliability: f64, total_answers: u64) -> Self {
        if skill_count >= 50 && avg_reliability >= 0.95 && total_answers >= 500 {
            AnnaLevel::Mythic
        } else if skill_count >= 20 && avg_reliability >= 0.85 && total_answers >= 100 {
            AnnaLevel::Master
        } else if skill_count >= 5 && avg_reliability >= 0.70 && total_answers >= 20 {
            AnnaLevel::Journeyman
        } else {
            AnnaLevel::Intern
        }
    }
}

/// System-level statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemStats {
    /// Total questions answered
    pub total_questions: u64,
    /// Questions answered via skills
    pub skill_answers: u64,
    /// Questions requiring re-planning
    pub replan_count: u64,
    /// Questions needing user clarification
    pub clarification_count: u64,
    /// Average response latency
    pub avg_latency_ms: u64,
    /// Current level
    pub level: Option<AnnaLevel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_creation() {
        let skill = Skill::new(
            "journalctl.service_window",
            "logs_for_service",
            "Show logs for a systemd service",
            "journalctl -u {{service_name}} --since \"{{since}}\" --no-pager -n {{max_lines}}",
        )
        .with_param("service_name", ParamType::String, "The systemd service name", true)
        .with_param("since", ParamType::Duration, "Time window start", true)
        .with_param("max_lines", ParamType::Integer, "Maximum lines to show", false)
        .with_default("max_lines", "200")
        .with_example("show the log of annad service for the last 6 hours");

        assert_eq!(skill.skill_id, "journalctl.service_window");
        assert_eq!(skill.parameter_schema.len(), 3);
    }

    #[test]
    fn test_build_command() {
        let skill = Skill::new(
            "test.skill",
            "test",
            "Test skill",
            "echo {{message}} -n {{count}}",
        )
        .with_default("count", "5");

        let mut params = HashMap::new();
        params.insert("message".to_string(), "hello".to_string());

        let cmd = skill.build_command(&params).unwrap();
        assert_eq!(cmd, vec!["echo", "hello", "-n", "5"]);
    }

    #[test]
    fn test_skill_stats() {
        let mut stats = SkillStats::new();
        assert!(!stats.is_reliable());

        stats.record_success(100);
        stats.record_success(150);
        stats.record_success(120);

        assert!(stats.is_reliable());
        assert_eq!(stats.success_count, 3);
        assert!(stats.reliability_score == 1.0);
    }

    #[test]
    fn test_anna_level() {
        assert_eq!(AnnaLevel::from_stats(0, 0.0, 0), AnnaLevel::Intern);
        assert_eq!(AnnaLevel::from_stats(5, 0.75, 25), AnnaLevel::Journeyman);
        assert_eq!(AnnaLevel::from_stats(25, 0.90, 150), AnnaLevel::Master);
        assert_eq!(AnnaLevel::from_stats(60, 0.98, 600), AnnaLevel::Mythic);
    }

    #[test]
    fn test_match_score() {
        let skill = Skill::new(
            "journalctl.service_window",
            "logs_for_service",
            "Show systemd service logs",
            "journalctl -u {{service}}",
        )
        .with_example("show the log of annad service");

        let score = skill.match_score("show the log of annad service for the last 6 hours");
        assert!(score > 0.0);
    }
}
