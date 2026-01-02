// v0.0.662: Settings Patcher Core (Phase 238)
// Main patcher implementation

use std::collections::HashMap;

use super::{
    config::PatcherConfig,
    entry::PatchEntry,
    result::{PatchResult, PatcherStats},
    types::{PatchMode, PatchOperation},
};

/// Settings patcher
#[derive(Debug, Clone, Default)]
pub struct SettingsPatcher {
    /// Config
    config: PatcherConfig,
    /// Results
    results: Vec<PatchResult>,
    /// Stats
    stats: PatcherStats,
}

impl SettingsPatcher {
    /// Create new patcher
    pub fn new(config: PatcherConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: PatcherStats::default(),
        }
    }

    /// Apply patches to settings
    pub fn apply(
        &mut self,
        target: &mut HashMap<String, String>,
        patches: &[PatchEntry],
    ) -> PatchResult {
        let mut result = PatchResult::new(self.config.mode);
        let backup = target.clone();

        for patch in patches {
            let success = match patch.operation {
                PatchOperation::Add => {
                    if !target.contains_key(&patch.key) {
                        if let Some(value) = &patch.value {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value.clone());
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Remove => {
                    if target.contains_key(&patch.key) {
                        if self.config.mode != PatchMode::DryRun {
                            target.remove(&patch.key);
                        }
                        true
                    } else {
                        false
                    }
                }
                PatchOperation::Replace => {
                    if target.contains_key(&patch.key) {
                        if let Some(value) = &patch.value {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value.clone());
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Copy => {
                    if let Some(source_key) = &patch.source_key {
                        if let Some(value) = target.get(source_key).cloned() {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                PatchOperation::Move => {
                    if let Some(source_key) = &patch.source_key {
                        if let Some(value) = target.remove(source_key) {
                            if self.config.mode != PatchMode::DryRun {
                                target.insert(patch.key.clone(), value);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            };

            if success {
                result.add_applied(patch.key.clone());
                self.stats.record(patch.operation, 1);
            } else {
                match self.config.mode {
                    PatchMode::Strict | PatchMode::Atomic => {
                        result.add_failed(patch.key.clone());
                    }
                    PatchMode::Lenient | PatchMode::DryRun => {
                        result.add_skipped(patch.key.clone());
                    }
                }
            }
        }

        // Rollback for atomic mode on failure
        if self.config.mode == PatchMode::Atomic && result.has_failures() {
            *target = backup;
        }

        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[PatchResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &PatcherStats {
        &self.stats
    }

    /// Result count
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results.clear();
    }
}
