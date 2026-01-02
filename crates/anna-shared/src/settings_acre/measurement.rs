// v0.0.760: Settings Acre Measurement
// Measurement and surveyor structures

use serde::{Deserialize, Serialize};

/// Acre measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreMeasurement {
    /// Measurement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Chain number
    pub chain: u32,
    /// Certified
    pub certified: bool,
}

impl AcreMeasurement {
    /// Create new measurement
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            chain: 0,
            certified: true,
        }
    }

    /// Set chain
    pub fn chain(mut self, c: u32) -> Self {
        self.chain = c;
        self
    }

    /// Make certified
    pub fn make_certified(&mut self) {
        self.certified = true;
    }

    /// Make uncertified
    pub fn make_uncertified(&mut self) {
        self.certified = false;
    }
}

/// Acre surveyor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcreSurveyor {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Measurement ID
    pub measurement_id: String,
}

impl AcreSurveyor {
    /// Create new surveyor
    pub fn new(key: impl Into<String>, name: impl Into<String>, measurement_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            measurement_id: measurement_id.into(),
        }
    }
}
