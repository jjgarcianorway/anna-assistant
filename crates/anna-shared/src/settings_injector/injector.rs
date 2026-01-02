// v0.0.654: Settings Injector Implementation
// Core injector logic for settings injection

use std::collections::HashMap;

use super::types::{InjectionResult, InjectionStrategy, InjectionType, InjectorConfig, InjectorStats};

/// Settings injector
#[derive(Debug, Clone, Default)]
pub struct SettingsInjector {
    /// Config
    config: InjectorConfig,
    /// Results
    results: Vec<InjectionResult>,
    /// Stats
    stats: InjectorStats,
}

impl SettingsInjector {
    /// Create new injector
    pub fn new(config: InjectorConfig) -> Self {
        Self {
            config,
            results: Vec::new(),
            stats: InjectorStats::default(),
        }
    }

    /// Inject settings
    pub fn inject(&mut self, target: &mut HashMap<String, String>, source: &HashMap<String, String>) -> InjectionResult {
        let mut result = InjectionResult::new(self.config.injection_type);

        for (key, value) in source {
            let exists = target.contains_key(key);

            match self.config.injection_type {
                InjectionType::Insert => {
                    if exists {
                        match self.config.strategy {
                            InjectionStrategy::FailOnConflict => result.add_failed(key.clone()),
                            InjectionStrategy::SkipOnConflict => result.add_skipped(key.clone()),
                            InjectionStrategy::OverwriteOnConflict => {
                                if !self.config.dry_run {
                                    target.insert(key.clone(), value.clone());
                                }
                                result.add_updated(key.clone());
                            }
                            InjectionStrategy::MergeOnConflict => {
                                result.add_skipped(key.clone());
                            }
                        }
                    } else if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                        result.add_inserted(key.clone());
                    } else {
                        result.add_inserted(key.clone());
                    }
                }
                InjectionType::Update => {
                    if exists {
                        if !self.config.dry_run {
                            target.insert(key.clone(), value.clone());
                        }
                        result.add_updated(key.clone());
                    } else {
                        result.add_skipped(key.clone());
                    }
                }
                InjectionType::Upsert => {
                    if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                    }
                    if exists {
                        result.add_updated(key.clone());
                    } else {
                        result.add_inserted(key.clone());
                    }
                }
                InjectionType::Replace => {
                    if !self.config.dry_run {
                        target.insert(key.clone(), value.clone());
                    }
                    result.add_updated(key.clone());
                }
                InjectionType::Append => {
                    if exists {
                        if !self.config.dry_run {
                            let existing = target.get(key).cloned().unwrap_or_default();
                            target.insert(key.clone(), format!("{}{}", existing, value));
                        }
                        result.add_updated(key.clone());
                    } else {
                        if !self.config.dry_run {
                            target.insert(key.clone(), value.clone());
                        }
                        result.add_inserted(key.clone());
                    }
                }
            }
        }

        self.stats.record(
            self.config.injection_type,
            result.inserted.len(),
            result.updated.len(),
            result.skipped.len(),
        );
        self.results.push(result.clone());
        result
    }

    /// Get results
    pub fn results(&self) -> &[InjectionResult] {
        &self.results
    }

    /// Get stats
    pub fn stats(&self) -> &InjectorStats {
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
