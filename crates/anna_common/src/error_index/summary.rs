//! Error Summary Types - v5.2.3

use std::collections::HashMap;
use super::category::LogCategory;

/// Grouped error summary for display
#[derive(Debug, Clone)]
pub struct GroupedErrorSummary {
    pub service_name: String,
    pub error_count: u64,
    pub cause_summary: String,
    pub example_message: Option<String>,
}

/// v5.2.3: Universal error summary by object type
#[derive(Debug, Clone, Default)]
pub struct UniversalErrorSummary {
    pub services: Vec<ObjectErrorEntry>,
    pub packages: Vec<ObjectErrorEntry>,
    pub executables: Vec<ObjectErrorEntry>,
    pub filesystem: Vec<ObjectErrorEntry>,
    pub kernel: Vec<ObjectErrorEntry>,
}

impl UniversalErrorSummary {
    /// Total error count across all categories
    pub fn total_errors(&self) -> usize {
        self.services.iter().map(|e| e.error_count).sum::<usize>()
            + self.packages.iter().map(|e| e.error_count).sum::<usize>()
            + self.executables.iter().map(|e| e.error_count).sum::<usize>()
            + self.filesystem.iter().map(|e| e.error_count).sum::<usize>()
            + self.kernel.iter().map(|e| e.error_count).sum::<usize>()
    }

    /// Check if any errors exist
    pub fn has_errors(&self) -> bool {
        !self.services.is_empty()
            || !self.packages.is_empty()
            || !self.executables.is_empty()
            || !self.filesystem.is_empty()
            || !self.kernel.is_empty()
    }
}

/// v5.2.3: Object error entry for universal summary
#[derive(Debug, Clone)]
pub struct ObjectErrorEntry {
    pub name: String,
    pub error_count: usize,
    pub cause: String,
    pub example: Option<String>,
    pub categories: HashMap<LogCategory, usize>,
}
