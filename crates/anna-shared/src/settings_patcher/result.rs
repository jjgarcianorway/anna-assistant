// v0.0.662: Settings Patcher Result (Phase 238)
// Result tracking and statistics for patch operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{PatchMode, PatchOperation};

/// Patch result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResult {
    /// Applied patches
    pub applied: Vec<String>,
    /// Skipped patches
    pub skipped: Vec<String>,
    /// Failed patches
    pub failed: Vec<String>,
    /// Patch mode used
    pub mode: PatchMode,
}

impl PatchResult {
    /// Create new result
    pub fn new(mode: PatchMode) -> Self {
        Self {
            applied: Vec::new(),
            skipped: Vec::new(),
            failed: Vec::new(),
            mode,
        }
    }

    /// Add applied
    pub fn add_applied(&mut self, key: String) {
        self.applied.push(key);
    }

    /// Add skipped
    pub fn add_skipped(&mut self, key: String) {
        self.skipped.push(key);
    }

    /// Add failed
    pub fn add_failed(&mut self, key: String) {
        self.failed.push(key);
    }

    /// Total applied
    pub fn total_applied(&self) -> usize {
        self.applied.len()
    }

    /// Has failures
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// Success
    pub fn success(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Patcher stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatcherStats {
    /// Total patches applied
    pub total_patches: usize,
    /// Total operations
    pub total_operations: usize,
    /// By operation
    pub by_operation: HashMap<String, usize>,
}

impl PatcherStats {
    /// Record patch
    pub fn record(&mut self, operation: PatchOperation, count: usize) {
        self.total_patches += 1;
        self.total_operations += count;
        *self.by_operation.entry(operation.to_string()).or_insert(0) += count;
    }

    /// Average operations per patch
    pub fn average_operations(&self) -> f64 {
        if self.total_patches == 0 {
            0.0
        } else {
            self.total_operations as f64 / self.total_patches as f64
        }
    }
}
