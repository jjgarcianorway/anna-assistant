// v0.0.609: Settings Reporter
// Reporter for settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Report kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReportKind {
    #[default]
    Status,
    Health,
    Usage,
}

impl std::fmt::Display for ReportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status => write!(f, "status"),
            Self::Health => write!(f, "health"),
            Self::Usage => write!(f, "usage"),
        }
    }
}

/// Reporter config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
    pub kind: ReportKind,
    pub category: Option<SettingsCategory>,
    pub enabled: bool,
}

impl ReporterConfig {
    pub fn new(kind: ReportKind) -> Self {
        Self { kind, category: None, enabled: true }
    }
}

/// Settings reporter
#[derive(Debug, Clone, Default)]
pub struct SettingsReporter {
    configs: HashMap<String, ReporterConfig>,
}

impl SettingsReporter {
    pub fn new() -> Self { Self::default() }
    pub fn register(&mut self, id: String, config: ReporterConfig) {
        self.configs.insert(id, config);
    }
    pub fn count(&self) -> usize { self.configs.len() }
}

pub fn is_reporter_query(query: &str) -> bool {
    query.to_lowercase().contains("reporter")
}

pub fn reporter_fun_fact() -> &'static str {
    "Anna's settings reporters generate status reports!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_display() {
        assert_eq!(format!("{}", ReportKind::Status), "status");
    }

    #[test]
    fn test_reporter_new() {
        let r = SettingsReporter::new();
        assert_eq!(r.count(), 0);
    }
}
