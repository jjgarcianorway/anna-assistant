//! Knowledge Store Schema v0.11.0
//!
//! Defines the data structures for Anna's persistent knowledge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Schema version for migrations
pub const SCHEMA_VERSION: u32 = 1;

/// Fact status in the knowledge store
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FactStatus {
    /// Fact is current and reliable
    Active,
    /// Fact has not been validated recently
    Stale,
    /// Fact has been superseded by newer information
    Deprecated,
    /// Fact conflicts with other evidence
    Conflicted,
}

impl FactStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FactStatus::Active => "active",
            FactStatus::Stale => "stale",
            FactStatus::Deprecated => "deprecated",
            FactStatus::Conflicted => "conflicted",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "active" => FactStatus::Active,
            "stale" => FactStatus::Stale,
            "deprecated" => FactStatus::Deprecated,
            "conflicted" => FactStatus::Conflicted,
            _ => FactStatus::Active,
        }
    }

    pub fn is_usable(&self) -> bool {
        matches!(self, FactStatus::Active | FactStatus::Stale)
    }
}

/// A fact in the knowledge store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    /// Internal identifier (UUID)
    pub id: String,
    /// Entity identifier (e.g., "cpu:0", "pkg:vim", "location:editor.vim.config")
    pub entity: String,
    /// Attribute name (e.g., "cores", "version", "path")
    pub attribute: String,
    /// Value (string or JSON)
    pub value: String,
    /// Source identifier (e.g., "probe:cpu.info:2025-11-27T12:00:00Z")
    pub source: String,
    /// First observed timestamp
    pub first_seen: DateTime<Utc>,
    /// Last validated timestamp
    pub last_seen: DateTime<Utc>,
    /// Confidence score [0.0, 1.0]
    pub confidence: f64,
    /// Current status
    pub status: FactStatus,
    /// Optional notes for conflicts or revisions
    pub notes: Option<String>,
}

impl Fact {
    /// Create a new fact from probe evidence
    pub fn from_probe(
        entity: String,
        attribute: String,
        value: String,
        probe_id: &str,
        confidence: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity,
            attribute,
            value,
            source: format!("probe:{}:{}", probe_id, now.to_rfc3339()),
            first_seen: now,
            last_seen: now,
            confidence,
            status: FactStatus::Active,
            notes: None,
        }
    }

    /// Create a new fact from LLM inference
    pub fn from_llm(
        entity: String,
        attribute: String,
        value: String,
        reasoning: &str,
        confidence: f64,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity,
            attribute,
            value,
            source: format!("llm:{}:{}", reasoning, now.to_rfc3339()),
            first_seen: now,
            last_seen: now,
            confidence,
            status: FactStatus::Active,
            notes: None,
        }
    }

    /// Mark fact as validated with fresh evidence
    pub fn refresh(&mut self, new_source: Option<&str>) {
        self.last_seen = Utc::now();
        if let Some(src) = new_source {
            self.source = src.to_string();
        }
        self.status = FactStatus::Active;
    }

    /// Mark fact as stale
    pub fn mark_stale(&mut self, reason: Option<&str>) {
        self.status = FactStatus::Stale;
        if let Some(r) = reason {
            self.notes = Some(r.to_string());
        }
    }

    /// Mark fact as deprecated
    pub fn deprecate(&mut self, reason: &str) {
        self.status = FactStatus::Deprecated;
        self.notes = Some(reason.to_string());
    }

    /// Check if fact is stale based on age threshold
    pub fn is_stale_by_age(&self, max_age_hours: i64) -> bool {
        let age = Utc::now().signed_duration_since(self.last_seen);
        age.num_hours() > max_age_hours
    }
}

/// History record for fact changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactHistory {
    /// Fact ID this history belongs to
    pub fact_id: String,
    /// Previous value
    pub old_value: String,
    /// New value
    pub new_value: String,
    /// Previous status
    pub old_status: FactStatus,
    /// New status
    pub new_status: FactStatus,
    /// Reason for change
    pub reason: String,
    /// Timestamp of change
    pub changed_at: DateTime<Utc>,
}

/// Query filter for facts
#[derive(Debug, Clone, Default)]
pub struct FactQuery {
    /// Filter by entity (exact match or prefix with *)
    pub entity: Option<String>,
    /// Filter by attribute
    pub attribute: Option<String>,
    /// Filter by minimum confidence
    pub min_confidence: Option<f64>,
    /// Filter by status
    pub status: Option<Vec<FactStatus>>,
    /// Filter by last_seen after this time
    pub seen_after: Option<DateTime<Utc>>,
    /// Limit results
    pub limit: Option<usize>,
}

impl FactQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entity(mut self, entity: &str) -> Self {
        self.entity = Some(entity.to_string());
        self
    }

    pub fn entity_prefix(mut self, prefix: &str) -> Self {
        self.entity = Some(format!("{}*", prefix));
        self
    }

    pub fn attribute(mut self, attr: &str) -> Self {
        self.attribute = Some(attr.to_string());
        self
    }

    pub fn min_confidence(mut self, conf: f64) -> Self {
        self.min_confidence = Some(conf);
        self
    }

    pub fn active_only(mut self) -> Self {
        self.status = Some(vec![FactStatus::Active]);
        self
    }

    pub fn usable_only(mut self) -> Self {
        self.status = Some(vec![FactStatus::Active, FactStatus::Stale]);
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Filter by specific statuses
    pub fn status(mut self, statuses: Vec<FactStatus>) -> Self {
        self.status = Some(statuses);
        self
    }

    /// Filter by last seen after timestamp
    pub fn seen_after(mut self, after: Option<DateTime<Utc>>) -> Self {
        self.seen_after = after;
        self
    }
}

/// Entity types for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    Cpu,
    Gpu,
    Disk,
    Filesystem,
    Package,
    Service,
    App,
    Config,
    User,
    Location,
    Network,
    System,
    Unknown,
}

impl EntityType {
    pub fn from_entity(entity: &str) -> Self {
        let prefix = entity.split(':').next().unwrap_or("");
        match prefix {
            "cpu" => EntityType::Cpu,
            "gpu" => EntityType::Gpu,
            "disk" => EntityType::Disk,
            "fs" => EntityType::Filesystem,
            "pkg" => EntityType::Package,
            "svc" => EntityType::Service,
            "app" => EntityType::App,
            "cfg" => EntityType::Config,
            "user" => EntityType::User,
            "location" => EntityType::Location,
            "net" => EntityType::Network,
            "system" => EntityType::System,
            _ => EntityType::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_from_probe() {
        let fact = Fact::from_probe(
            "cpu:0".to_string(),
            "cores".to_string(),
            "8".to_string(),
            "cpu.info",
            0.95,
        );
        assert_eq!(fact.entity, "cpu:0");
        assert_eq!(fact.attribute, "cores");
        assert!(fact.source.starts_with("probe:cpu.info:"));
        assert_eq!(fact.status, FactStatus::Active);
    }

    #[test]
    fn test_fact_status() {
        assert!(FactStatus::Active.is_usable());
        assert!(FactStatus::Stale.is_usable());
        assert!(!FactStatus::Deprecated.is_usable());
        assert!(!FactStatus::Conflicted.is_usable());
    }

    #[test]
    fn test_entity_type() {
        assert_eq!(EntityType::from_entity("cpu:0"), EntityType::Cpu);
        assert_eq!(EntityType::from_entity("pkg:vim"), EntityType::Package);
        assert_eq!(
            EntityType::from_entity("location:editor.vim.config"),
            EntityType::Location
        );
    }

    #[test]
    fn test_fact_query() {
        let query = FactQuery::new()
            .entity_prefix("pkg:")
            .min_confidence(0.8)
            .active_only();
        assert!(query.entity.unwrap().ends_with('*'));
        assert_eq!(query.min_confidence, Some(0.8));
    }
}
