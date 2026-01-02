// v0.0.599: Settings Resolver Core (Phase 175)
// Core resolver implementation for settings conflicts

use std::collections::HashSet;

use super::config::ResolverConfig;
use super::types::{Conflict, Dependency, Resolution};

/// Settings resolver
#[derive(Debug, Clone, Default)]
pub struct SettingsResolver {
    /// Configuration
    config: ResolverConfig,
    /// Dependencies
    dependencies: Vec<Dependency>,
    /// Conflict history
    conflicts: Vec<Conflict>,
    /// Resolution history
    resolutions: Vec<Resolution>,
}

impl SettingsResolver {
    /// Create new resolver
    pub fn new() -> Self {
        Self::default()
    }

    /// With config
    pub fn with_config(config: ResolverConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Add dependency
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    /// Remove dependency
    pub fn remove_dependency(&mut self, source: &str, depends_on: &str) -> bool {
        if let Some(pos) = self
            .dependencies
            .iter()
            .position(|d| d.source == source && d.depends_on == depends_on)
        {
            self.dependencies.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get dependencies for key
    pub fn dependencies_for(&self, source: &str) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.source == source).collect()
    }

    /// Get dependents of key
    pub fn dependents_of(&self, key: &str) -> Vec<&Dependency> {
        self.dependencies.iter().filter(|d| d.depends_on == key).collect()
    }

    /// Check for circular dependencies
    pub fn has_circular(&self, start: &str) -> bool {
        let mut visited = HashSet::new();
        self.check_circular_internal(start, &mut visited)
    }

    fn check_circular_internal(&self, current: &str, visited: &mut HashSet<String>) -> bool {
        if visited.contains(current) {
            return true;
        }
        visited.insert(current.to_string());

        for dep in self.dependencies_for(current) {
            if self.check_circular_internal(&dep.depends_on, visited) {
                return true;
            }
        }
        visited.remove(current);
        false
    }

    /// Record conflict
    pub fn record_conflict(&mut self, conflict: Conflict) {
        self.conflicts.push(conflict);
    }

    /// Record resolution
    pub fn record_resolution(&mut self, resolution: Resolution) {
        self.resolutions.push(resolution);
    }

    /// Get conflicts
    pub fn conflicts(&self) -> &[Conflict] {
        &self.conflicts
    }

    /// Get dependency count
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Get resolutions
    pub fn resolutions(&self) -> &[Resolution] {
        &self.resolutions
    }

    /// Conflict count
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Resolution count
    pub fn resolution_count(&self) -> usize {
        self.resolutions.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.conflicts.clear();
        self.resolutions.clear();
    }
}
