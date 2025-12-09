//! Type definitions for health view module (v0.0.210).

use serde::{Deserialize, Serialize};

/// Severity level for health items
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthSeverity {
    /// Critical issues requiring immediate attention
    Critical,
    /// Warnings that should be addressed soon
    Warning,
    /// Informational notes (rarely shown)
    Note,
}

impl std::fmt::Display for HealthSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::Warning => write!(f, "warning"),
            Self::Note => write!(f, "note"),
        }
    }
}

/// Health item category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthCategory {
    /// Disk/storage issues
    Disk,
    /// Memory issues
    Memory,
    /// Service failures
    Services,
    /// System changes
    Changes,
}

/// A single health item (issue or warning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    /// Severity level
    pub severity: HealthSeverity,
    /// Short description (one line)
    pub message: String,
    /// Category for grouping
    pub category: HealthCategory,
    /// Raw value (e.g., percentage) for deterministic sorting
    pub sort_key: u32,
}

impl HealthItem {
    /// Create a critical item
    pub fn critical(category: HealthCategory, message: impl Into<String>, sort_key: u32) -> Self {
        Self {
            severity: HealthSeverity::Critical,
            message: message.into(),
            category,
            sort_key,
        }
    }

    /// Create a warning item
    pub fn warning(category: HealthCategory, message: impl Into<String>, sort_key: u32) -> Self {
        Self {
            severity: HealthSeverity::Warning,
            message: message.into(),
            category,
            sort_key,
        }
    }

    /// Create a note item
    pub fn note(category: HealthCategory, message: impl Into<String>) -> Self {
        Self {
            severity: HealthSeverity::Note,
            message: message.into(),
            category,
            sort_key: 0,
        }
    }

    /// Format for display (v0.0.265: ASCII icons)
    pub fn format(&self) -> String {
        let icon = match self.severity {
            HealthSeverity::Critical => "[!!]",
            HealthSeverity::Warning => "[!]",
            HealthSeverity::Note => "[i]",
        };
        format!("{} {}", icon, self.message)
    }
}

/// A change since last snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthChange {
    /// What changed
    pub description: String,
    /// Whether this is a positive change (e.g., service recovered)
    pub positive: bool,
}
