// v0.0.751: Settings Borough Types
// Core types and enums for borough system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Borough type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BoroughType {
    /// Urban borough
    #[default]
    Urban,
    /// Metropolitan borough
    Metropolitan,
    /// London borough
    London,
    /// Municipal borough
    Municipal,
}

impl std::fmt::Display for BoroughType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Urban => write!(f, "urban"),
            Self::Metropolitan => write!(f, "metropolitan"),
            Self::London => write!(f, "london"),
            Self::Municipal => write!(f, "municipal"),
        }
    }
}

/// Borough status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BoroughStatus {
    /// Established status
    #[default]
    Established,
    /// Active status
    Active,
    /// Reformed status
    Reformed,
    /// Abolished status
    Abolished,
}

impl std::fmt::Display for BoroughStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Established => write!(f, "established"),
            Self::Active => write!(f, "active"),
            Self::Reformed => write!(f, "reformed"),
            Self::Abolished => write!(f, "abolished"),
        }
    }
}

/// Borough config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughConfig {
    /// Name
    pub name: String,
    /// Borough type
    pub borough_type: BoroughType,
    /// Status
    pub status: BoroughStatus,
    /// Max resolutions
    pub max_resolutions: usize,
}

impl BoroughConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            borough_type: BoroughType::Urban,
            status: BoroughStatus::Established,
            max_resolutions: 100,
        }
    }

    /// Set type
    pub fn borough_type(mut self, bt: BoroughType) -> Self {
        self.borough_type = bt;
        self
    }

    /// Set status
    pub fn status(mut self, s: BoroughStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max resolutions
    pub fn max_resolutions(mut self, max: usize) -> Self {
        self.max_resolutions = max;
        self
    }
}

impl Default for BoroughConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Borough resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughResolution {
    /// Resolution ID
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

impl BoroughResolution {
    /// Create new resolution
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            section: 0,
            adopted: true,
        }
    }

    /// Set section
    pub fn section(mut self, s: u32) -> Self {
        self.section = s;
        self
    }

    /// Make adopted
    pub fn make_adopted(&mut self) {
        self.adopted = true;
    }

    /// Make rescinded
    pub fn make_rescinded(&mut self) {
        self.adopted = false;
    }
}

/// Borough representative
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoroughRepresentative {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Resolution ID
    pub resolution_id: String,
}

impl BoroughRepresentative {
    /// Create new representative
    pub fn new(key: impl Into<String>, name: impl Into<String>, resolution_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            resolution_id: resolution_id.into(),
        }
    }
}

/// Borough stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BoroughStats {
    /// Total resolutions
    pub total_resolutions: usize,
    /// Adopted resolutions
    pub adopted: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl BoroughStats {
    /// Update from resolutions
    pub fn update(&mut self, resolutions: &[BoroughResolution], borough_type: BoroughType) {
        self.total_resolutions = resolutions.len();
        self.adopted = resolutions.iter().filter(|r| r.adopted).count();
        *self.by_type.entry(borough_type.to_string()).or_insert(0) += 1;
    }

    /// Adopted rate
    pub fn adopted_rate(&self) -> f64 {
        if self.total_resolutions == 0 { 0.0 } else { self.adopted as f64 / self.total_resolutions as f64 * 100.0 }
    }
}
