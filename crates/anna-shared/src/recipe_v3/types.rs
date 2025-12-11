//! Core recipe types (v0.0.423).
//!
//! The Recipe struct and all supporting types for the learning and reuse engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Origin of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeOrigin {
    /// Built-in seed recipe
    #[default]
    BuiltIn,
    /// Learned from a successful ticket
    LearnedFromTicket,
    /// Manually authored by user
    UserAuthored,
}

/// Author of a recipe
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeAuthor {
    /// System-generated recipe
    #[default]
    System,
    /// Specialist-generated recipe
    Specialist(String),
    /// User-authored recipe
    User(String),
}

/// Domain/category of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeDomain {
    #[default]
    General,
    Service,
    Package,
    Network,
    Disk,
    Memory,
    Process,
    User,
    Config,
    Git,
    Docker,
    Editor,
    Shell,
    Systemd,
    Cron,
}

impl RecipeDomain {
    /// Parse domain from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "service" | "systemd" => Self::Systemd,
            "package" | "pacman" | "yay" | "paru" => Self::Package,
            "network" | "net" | "wifi" | "ethernet" => Self::Network,
            "disk" | "storage" | "filesystem" | "mount" => Self::Disk,
            "memory" | "ram" | "swap" => Self::Memory,
            "process" | "proc" | "ps" | "kill" => Self::Process,
            "user" | "account" | "group" => Self::User,
            "config" | "conf" | "settings" => Self::Config,
            "git" | "github" | "repo" => Self::Git,
            "docker" | "container" | "podman" => Self::Docker,
            "editor" | "vim" | "nvim" | "nano" | "emacs" => Self::Editor,
            "shell" | "bash" | "zsh" | "fish" => Self::Shell,
            "cron" | "timer" | "schedule" => Self::Cron,
            _ => Self::General,
        }
    }
}

/// Risk level of a recipe
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipeRiskLevel {
    /// Safe, read-only operations (probes, queries)
    #[default]
    None,
    /// Low risk, easily reversible (config changes with backup)
    Low,
    /// Medium risk, may require manual intervention to reverse
    Medium,
    /// High risk, potentially destructive (rm, format, etc.)
    High,
}

impl RecipeRiskLevel {
    /// Whether this risk level requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, Self::Medium | Self::High)
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "none" | "safe" | "readonly" => Self::None,
            "low" => Self::Low,
            "medium" | "med" => Self::Medium,
            "high" | "dangerous" => Self::High,
            _ => Self::Medium,
        }
    }
}

/// Confirmation policy for recipe execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationPolicy {
    /// Never ask for confirmation (trusted/safe recipes)
    Never,
    /// Ask once before starting execution
    #[default]
    Once,
    /// Ask before each risky step
    PerStep,
    /// Always ask (high-risk recipes)
    Always,
}

/// Recipe matching criteria
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeMatcher {
    /// Target domain
    pub domain: RecipeDomain,
    /// Matching intents (e.g., "restart", "enable", "check")
    pub intents: Vec<String>,
    /// Required keywords for matching
    pub keywords: Vec<String>,
    /// Optional entity patterns (e.g., service names)
    pub entity_patterns: Vec<String>,
    /// Similarity key for fuzzy matching
    pub similarity_key: String,
}

impl RecipeMatcher {
    /// Create a new matcher
    pub fn new(domain: RecipeDomain) -> Self {
        Self {
            domain,
            ..Default::default()
        }
    }

    /// Add intents
    pub fn with_intents(mut self, intents: &[&str]) -> Self {
        self.intents = intents.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add keywords
    pub fn with_keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add entity patterns
    pub fn with_entities(mut self, patterns: &[&str]) -> Self {
        self.entity_patterns = patterns.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set similarity key
    pub fn with_similarity_key(mut self, key: &str) -> Self {
        self.similarity_key = key.to_string();
        self
    }
}

/// Recipe usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecipeStats {
    /// Total times matched
    pub times_matched: u32,
    /// Times executed
    pub times_executed: u32,
    /// Successful executions
    pub times_succeeded: u32,
    /// Failed executions
    pub times_failed: u32,
    /// Times user skipped/rejected
    pub times_skipped: u32,
    /// Last used timestamp (Unix epoch seconds)
    pub last_used: Option<u64>,
    /// Average execution time in ms
    pub avg_execution_ms: u64,
}

impl RecipeStats {
    /// Success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.times_executed == 0 {
            0.0
        } else {
            self.times_succeeded as f32 / self.times_executed as f32
        }
    }

    /// Whether recipe is mature (enough usage data)
    pub fn is_mature(&self) -> bool {
        self.times_executed >= super::MIN_MATURE_USES
    }

    /// Record a match
    pub fn record_match(&mut self) {
        self.times_matched += 1;
        self.last_used = Some(now_epoch());
    }

    /// Record successful execution
    pub fn record_success(&mut self, execution_ms: u64) {
        self.times_executed += 1;
        self.times_succeeded += 1;
        self.update_avg_time(execution_ms);
    }

    /// Record failed execution
    pub fn record_failure(&mut self) {
        self.times_executed += 1;
        self.times_failed += 1;
    }

    /// Record user skip
    pub fn record_skip(&mut self) {
        self.times_skipped += 1;
    }

    fn update_avg_time(&mut self, new_time: u64) {
        if self.avg_execution_ms == 0 {
            self.avg_execution_ms = new_time;
        } else {
            // Rolling average
            self.avg_execution_ms = (self.avg_execution_ms * 3 + new_time) / 4;
        }
    }
}

/// A complete recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeV3 {
    /// Unique identifier
    pub id: String,
    /// Version number (increments on updates)
    pub version: u32,
    /// Human-readable title
    pub title: String,
    /// Description of what this recipe does
    pub description: String,
    /// Origin (built-in, learned, user-authored)
    pub origin: RecipeOrigin,
    /// Author information
    pub author: RecipeAuthor,
    /// Matching criteria
    pub matcher: RecipeMatcher,
    /// Preconditions that must be true
    pub preconditions: Vec<super::RecipeCondition>,
    /// Steps to execute
    pub steps: Vec<super::RecipeStep>,
    /// Expected outcomes/assertions
    pub postconditions: Vec<super::RecipeCondition>,
    /// Risk level
    pub risk_level: RecipeRiskLevel,
    /// Confirmation policy
    pub confirmation: ConfirmationPolicy,
    /// Source citations
    pub citations: Vec<String>,
    /// Usage statistics
    pub stats: RecipeStats,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Whether recipe is enabled
    pub enabled: bool,
    /// Source ticket ID (if learned)
    pub source_ticket_id: Option<String>,
    /// Variables/parameters this recipe accepts
    pub parameters: HashMap<String, String>,
}

impl Default for RecipeV3 {
    fn default() -> Self {
        Self {
            id: String::new(),
            version: 1,
            title: String::new(),
            description: String::new(),
            origin: RecipeOrigin::default(),
            author: RecipeAuthor::default(),
            matcher: RecipeMatcher::default(),
            preconditions: vec![],
            steps: vec![],
            postconditions: vec![],
            risk_level: RecipeRiskLevel::default(),
            confirmation: ConfirmationPolicy::default(),
            citations: vec![],
            stats: RecipeStats::default(),
            tags: vec![],
            enabled: true,
            source_ticket_id: None,
            parameters: HashMap::new(),
        }
    }
}

impl RecipeV3 {
    /// Create a new recipe with ID
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            ..Default::default()
        }
    }

    /// Builder: set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Builder: set origin
    pub fn with_origin(mut self, origin: RecipeOrigin) -> Self {
        self.origin = origin;
        self
    }

    /// Builder: set matcher
    pub fn with_matcher(mut self, matcher: RecipeMatcher) -> Self {
        self.matcher = matcher;
        self
    }

    /// Builder: add precondition
    pub fn with_precondition(mut self, cond: super::RecipeCondition) -> Self {
        self.preconditions.push(cond);
        self
    }

    /// Builder: add step
    pub fn with_step(mut self, step: super::RecipeStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Builder: set risk level
    pub fn with_risk(mut self, risk: RecipeRiskLevel) -> Self {
        self.risk_level = risk;
        self
    }

    /// Builder: add citation
    pub fn with_citation(mut self, citation: &str) -> Self {
        self.citations.push(citation.to_string());
        self
    }

    /// Builder: add tag
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.tags.push(tag.to_string());
        self
    }

    /// Check if recipe is healthy (good success rate)
    pub fn is_healthy(&self) -> bool {
        !self.stats.is_mature() || self.stats.success_rate() >= super::MIN_SUCCESS_RATE
    }

    /// Get primary domain
    pub fn domain(&self) -> RecipeDomain {
        self.matcher.domain
    }
}

/// Get current Unix epoch seconds
fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_domain_parse() {
        assert_eq!(RecipeDomain::from_str("systemd"), RecipeDomain::Systemd);
        assert_eq!(RecipeDomain::from_str("pacman"), RecipeDomain::Package);
        assert_eq!(RecipeDomain::from_str("unknown"), RecipeDomain::General);
    }

    #[test]
    fn test_risk_level_confirmation() {
        assert!(!RecipeRiskLevel::None.requires_confirmation());
        assert!(!RecipeRiskLevel::Low.requires_confirmation());
        assert!(RecipeRiskLevel::Medium.requires_confirmation());
        assert!(RecipeRiskLevel::High.requires_confirmation());
    }

    #[test]
    fn test_recipe_stats() {
        let mut stats = RecipeStats::default();
        stats.record_success(100);
        stats.record_success(200);
        assert_eq!(stats.times_executed, 2);
        assert_eq!(stats.success_rate(), 1.0);

        stats.record_failure();
        assert!(stats.success_rate() < 1.0);
    }

    #[test]
    fn test_recipe_builder() {
        let recipe = RecipeV3::new("test-1", "Test Recipe")
            .with_description("A test")
            .with_risk(RecipeRiskLevel::Low)
            .with_tag("test");

        assert_eq!(recipe.id, "test-1");
        assert_eq!(recipe.risk_level, RecipeRiskLevel::Low);
        assert!(recipe.tags.contains(&"test".to_string()));
    }
}
