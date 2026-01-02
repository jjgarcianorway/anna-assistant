// v0.0.758: Settings Plot (Phase 334)
// Plot registry

use std::collections::HashMap;
use super::plot::SettingsPlot;

/// Plot registry
#[derive(Debug, Clone, Default)]
pub struct PlotRegistry {
    /// Plots by ID
    plots: HashMap<String, SettingsPlot>,
}

impl PlotRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register plot
    pub fn register(&mut self, id: impl Into<String>, plot: SettingsPlot) {
        self.plots.insert(id.into(), plot);
    }

    /// Unregister plot
    pub fn unregister(&mut self, id: &str) -> bool {
        self.plots.remove(id).is_some()
    }

    /// Get plot
    pub fn get(&self, id: &str) -> Option<&SettingsPlot> {
        self.plots.get(id)
    }

    /// Get plot mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsPlot> {
        self.plots.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.plots.len()
    }
}
