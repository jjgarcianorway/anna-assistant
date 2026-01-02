// v0.0.719: Settings Edict - Proclamation
// Edict proclamations and annotations

use serde::{Deserialize, Serialize};
use super::types::EdictStatus;

/// Edict proclamation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictProclamation {
    /// Proclamation ID
    pub id: String,
    /// Title
    pub title: String,
    /// Decree
    pub decree: String,
    /// Status
    pub status: EdictStatus,
    /// Seal
    pub seal: String,
}

impl EdictProclamation {
    /// Create new proclamation
    pub fn new(id: impl Into<String>, title: impl Into<String>, decree: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            decree: decree.into(),
            status: EdictStatus::Draft,
            seal: String::new(),
        }
    }

    /// Set seal
    pub fn seal(mut self, s: impl Into<String>) -> Self {
        self.seal = s.into();
        self
    }

    /// Proclaim edict
    pub fn proclaim(&mut self) {
        self.status = EdictStatus::Proclaimed;
    }

    /// Activate edict
    pub fn activate(&mut self) {
        self.status = EdictStatus::Active;
    }

    /// Revoke edict
    pub fn revoke(&mut self) {
        self.status = EdictStatus::Revoked;
    }
}

/// Edict annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdictAnnotation {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Proclamation ID
    pub proclamation_id: String,
}

impl EdictAnnotation {
    /// Create new annotation
    pub fn new(key: impl Into<String>, value: impl Into<String>, proclamation_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            proclamation_id: proclamation_id.into(),
        }
    }
}
