// v0.0.760: Settings Acre Main
// Main settings acre structure

use super::config::AcreConfig;
use super::measurement::{AcreMeasurement, AcreSurveyor};
use super::stats::AcreStats;

/// Settings acre
#[derive(Debug, Clone, Default)]
pub struct SettingsAcre {
    /// Config
    config: AcreConfig,
    /// Measurements
    measurements: Vec<AcreMeasurement>,
    /// Surveyors
    surveyors: Vec<AcreSurveyor>,
    /// Stats
    stats: AcreStats,
}

impl SettingsAcre {
    /// Create new acre system
    pub fn new(config: AcreConfig) -> Self {
        Self {
            config,
            measurements: Vec::new(),
            surveyors: Vec::new(),
            stats: AcreStats::default(),
        }
    }

    /// Add measurement
    pub fn add_measurement(&mut self, measurement: AcreMeasurement) -> bool {
        if self.measurements.len() >= self.config.max_measurements {
            return false;
        }
        self.measurements.push(measurement);
        self.update_stats();
        true
    }

    /// Get measurement
    pub fn get_measurement(&self, id: &str) -> Option<&AcreMeasurement> {
        self.measurements.iter().find(|m| m.id == id)
    }

    /// Get measurement mut
    pub fn get_measurement_mut(&mut self, id: &str) -> Option<&mut AcreMeasurement> {
        self.measurements.iter_mut().find(|m| m.id == id)
    }

    /// Add surveyor
    pub fn add_surveyor(&mut self, surveyor: AcreSurveyor) {
        self.surveyors.push(surveyor);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.measurements, self.config.acre_type);
    }

    /// Get stats
    pub fn stats(&self) -> &AcreStats {
        &self.stats
    }

    /// Measurement count
    pub fn measurement_count(&self) -> usize {
        self.measurements.len()
    }
}
