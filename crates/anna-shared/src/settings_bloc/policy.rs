// v0.0.740: Settings Bloc Policy (Phase 316)
// Bloc policy management

use serde::{Deserialize, Serialize};

/// Bloc policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlocPolicy {
    /// Policy ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Priority level
    pub priority: u32,
    /// Coordinated
    pub coordinated: bool,
}

impl BlocPolicy {
    /// Create new policy
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            priority: 0,
            coordinated: true,
        }
    }

    /// Set priority
    pub fn priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    /// Make coordinated
    pub fn make_coordinated(&mut self) {
        self.coordinated = true;
    }

    /// Make independent
    pub fn make_independent(&mut self) {
        self.coordinated = false;
    }
}
