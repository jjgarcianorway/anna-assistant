// v0.0.728: Settings Protocol - Data Structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ProtocolType;

/// Protocol clause
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolClause {
    /// Clause ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Section number
    pub section: u32,
    /// Adopted
    pub adopted: bool,
}

impl ProtocolClause {
    /// Create new clause
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            adopted: false,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Adopt clause
    pub fn adopt(&mut self) {
        self.adopted = true;
    }

    /// Reject clause
    pub fn reject(&mut self) {
        self.adopted = false;
    }
}

/// Protocol party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParty {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Clause ID
    pub clause_id: String,
}

impl ProtocolParty {
    /// Create new party
    pub fn new(key: impl Into<String>, name: impl Into<String>, clause_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            clause_id: clause_id.into(),
        }
    }
}

/// Protocol stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Total clauses
    pub total_clauses: usize,
    /// Adopted clauses
    pub adopted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ProtocolStats {
    /// Update from clauses
    pub fn update(&mut self, clauses: &[ProtocolClause], protocol_type: ProtocolType) {
        self.total_clauses = clauses.len();
        self.adopted = clauses.iter().filter(|c| c.adopted).count();
        *self.by_type.entry(protocol_type.to_string()).or_insert(0) += 1;
    }

    /// Adoption rate
    pub fn adoption_rate(&self) -> f64 {
        if self.total_clauses == 0 { 0.0 } else { self.adopted as f64 / self.total_clauses as f64 * 100.0 }
    }
}
