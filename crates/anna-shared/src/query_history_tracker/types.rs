// v0.0.537: Query History Tracker Types (Phase 113)
// Core types for query tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::utils::normalize_query;

/// Query category for classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryCategory {
    System,
    Network,
    Storage,
    Audio,
    Video,
    Desktop,
    Security,
    Package,
    Service,
    Editor,
    Shell,
    Hardware,
    Configuration,
    General,
    Custom(String),
}

impl Default for QueryCategory {
    fn default() -> Self {
        Self::General
    }
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "System"),
            Self::Network => write!(f, "Network"),
            Self::Storage => write!(f, "Storage"),
            Self::Audio => write!(f, "Audio"),
            Self::Video => write!(f, "Video"),
            Self::Desktop => write!(f, "Desktop"),
            Self::Security => write!(f, "Security"),
            Self::Package => write!(f, "Package"),
            Self::Service => write!(f, "Service"),
            Self::Editor => write!(f, "Editor"),
            Self::Shell => write!(f, "Shell"),
            Self::Hardware => write!(f, "Hardware"),
            Self::Configuration => write!(f, "Configuration"),
            Self::General => write!(f, "General"),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Query resolution outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum QueryOutcome {
    #[default]
    Pending,
    Resolved,
    Escalated,
    Failed,
    Deferred,
}

impl std::fmt::Display for QueryOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Failed => write!(f, "Failed"),
            Self::Deferred => write!(f, "Deferred"),
        }
    }
}

/// Single query record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRecord {
    pub id: String,
    pub query_text: String,
    pub normalized_text: String,
    pub category: QueryCategory,
    pub outcome: QueryOutcome,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub ticket_id: Option<String>,
    pub similar_count: u32,
}

impl QueryRecord {
    /// Create new query record
    pub fn new(id: impl Into<String>, query_text: impl Into<String>) -> Self {
        let text = query_text.into();
        Self {
            id: id.into(),
            normalized_text: normalize_query(&text),
            query_text: text,
            category: QueryCategory::default(),
            outcome: QueryOutcome::default(),
            timestamp: Utc::now(),
            response_time_ms: None,
            ticket_id: None,
            similar_count: 0,
        }
    }

    /// Set category
    pub fn with_category(mut self, category: QueryCategory) -> Self {
        self.category = category;
        self
    }

    /// Set ticket ID
    pub fn with_ticket(mut self, ticket_id: impl Into<String>) -> Self {
        self.ticket_id = Some(ticket_id.into());
        self
    }
}
