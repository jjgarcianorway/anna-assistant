// v0.0.719: Settings Edict - Main Edict
// Core settings edict implementation

use super::config::EdictConfig;
use super::proclamation::{EdictProclamation, EdictAnnotation};
use super::stats::EdictStats;

/// Settings edict
#[derive(Debug, Clone, Default)]
pub struct SettingsEdict {
    /// Config
    config: EdictConfig,
    /// Proclamations
    proclamations: Vec<EdictProclamation>,
    /// Annotations
    annotations: Vec<EdictAnnotation>,
    /// Stats
    stats: EdictStats,
}

impl SettingsEdict {
    /// Create new edict system
    pub fn new(config: EdictConfig) -> Self {
        Self {
            config,
            proclamations: Vec::new(),
            annotations: Vec::new(),
            stats: EdictStats::default(),
        }
    }

    /// Add proclamation
    pub fn add_proclamation(&mut self, proclamation: EdictProclamation) -> bool {
        if self.proclamations.len() >= self.config.max_edicts {
            return false;
        }
        self.proclamations.push(proclamation);
        self.update_stats();
        true
    }

    /// Get proclamation
    pub fn get_proclamation(&self, id: &str) -> Option<&EdictProclamation> {
        self.proclamations.iter().find(|p| p.id == id)
    }

    /// Get proclamation mut
    pub fn get_proclamation_mut(&mut self, id: &str) -> Option<&mut EdictProclamation> {
        self.proclamations.iter_mut().find(|p| p.id == id)
    }

    /// Add annotation
    pub fn add_annotation(&mut self, annotation: EdictAnnotation) {
        self.annotations.push(annotation);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.proclamations, self.config.edict_type);
    }

    /// Get stats
    pub fn stats(&self) -> &EdictStats {
        &self.stats
    }

    /// Proclamation count
    pub fn proclamation_count(&self) -> usize {
        self.proclamations.len()
    }
}
