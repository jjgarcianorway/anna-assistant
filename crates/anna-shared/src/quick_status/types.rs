//! Core types for quick status functionality.

use serde::{Deserialize, Serialize};

/// Quick status health level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthLevel {
    /// Everything is fine
    Good,
    /// Minor issues, not critical
    Warning,
    /// Critical issues need attention
    Critical,
    /// Status unknown
    Unknown,
}

impl Default for HealthLevel {
    fn default() -> Self {
        Self::Unknown
    }
}

impl HealthLevel {
    /// Get display symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Good => "[OK]",
            Self::Warning => "[!]",
            Self::Critical => "[X]",
            Self::Unknown => "[?]",
        }
    }

    /// Get color hint for display
    pub fn color_hint(&self) -> &'static str {
        match self {
            Self::Good => "green",
            Self::Warning => "yellow",
            Self::Critical => "red",
            Self::Unknown => "gray",
        }
    }
}

/// Individual status item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusItem {
    /// Name of the item
    pub name: String,
    /// Health level
    pub health: HealthLevel,
    /// Brief status message
    pub message: String,
    /// Optional value (e.g., "85%")
    pub value: Option<String>,
}

impl StatusItem {
    /// Create a new status item
    pub fn new(name: &str, health: HealthLevel, message: &str) -> Self {
        Self {
            name: name.to_string(),
            health,
            message: message.to_string(),
            value: None,
        }
    }

    /// Add a value
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }

    /// Format for display
    pub fn format(&self) -> String {
        let value_str = self
            .value
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();

        format!(
            "{} {}: {}{}",
            self.health.symbol(),
            self.name,
            self.message,
            value_str
        )
    }
}

/// Quick status summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickStatus {
    /// Individual status items
    pub items: Vec<StatusItem>,
    /// Overall health (worst of all items)
    pub overall: HealthLevel,
    /// Summary message
    pub summary: String,
}

impl QuickStatus {
    /// Create empty status
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a status item
    pub fn add(&mut self, item: StatusItem) {
        // Update overall health to worst level
        self.overall = match (&self.overall, &item.health) {
            (_, HealthLevel::Critical) => HealthLevel::Critical,
            (HealthLevel::Critical, _) => HealthLevel::Critical,
            (_, HealthLevel::Warning) => HealthLevel::Warning,
            (HealthLevel::Warning, _) => HealthLevel::Warning,
            (HealthLevel::Unknown, HealthLevel::Good) => HealthLevel::Good,
            (HealthLevel::Good, _) => HealthLevel::Good,
            _ => self.overall,
        };

        self.items.push(item);
    }

    /// Add multiple items
    pub fn add_all(&mut self, items: Vec<StatusItem>) {
        for item in items {
            self.add(item);
        }
    }

    /// Set summary message
    pub fn set_summary(&mut self, summary: &str) {
        self.summary = summary.to_string();
    }

    /// Generate summary from items
    pub fn generate_summary(&mut self) {
        let critical = self
            .items
            .iter()
            .filter(|i| i.health == HealthLevel::Critical)
            .count();
        let warning = self
            .items
            .iter()
            .filter(|i| i.health == HealthLevel::Warning)
            .count();

        self.summary = if critical > 0 {
            format!(
                "{} critical issue{}",
                critical,
                if critical > 1 { "s" } else { "" }
            )
        } else if warning > 0 {
            format!("{} warning{}", warning, if warning > 1 { "s" } else { "" })
        } else {
            "All systems operational".to_string()
        };
    }

    /// Get count of issues by health level
    pub fn count_by_health(&self, health: HealthLevel) -> usize {
        self.items.iter().filter(|i| i.health == health).count()
    }

    /// Check if there are any critical issues
    pub fn has_critical(&self) -> bool {
        self.items.iter().any(|i| i.health == HealthLevel::Critical)
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.items.iter().any(|i| i.health == HealthLevel::Warning)
    }

    /// Check if everything is good
    pub fn all_good(&self) -> bool {
        self.items.iter().all(|i| i.health == HealthLevel::Good)
    }
}
