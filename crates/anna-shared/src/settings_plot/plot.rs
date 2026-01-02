// v0.0.758: Settings Plot (Phase 334)
// Settings plot main structure

use super::config::PlotConfig;
use super::survey::PlotSurvey;
use super::steward::PlotSteward;
use super::stats::PlotStats;

/// Settings plot
#[derive(Debug, Clone, Default)]
pub struct SettingsPlot {
    /// Config
    config: PlotConfig,
    /// Surveys
    surveys: Vec<PlotSurvey>,
    /// Stewards
    stewards: Vec<PlotSteward>,
    /// Stats
    stats: PlotStats,
}

impl SettingsPlot {
    /// Create new plot system
    pub fn new(config: PlotConfig) -> Self {
        Self {
            config,
            surveys: Vec::new(),
            stewards: Vec::new(),
            stats: PlotStats::default(),
        }
    }

    /// Add survey
    pub fn add_survey(&mut self, survey: PlotSurvey) -> bool {
        if self.surveys.len() >= self.config.max_surveys {
            return false;
        }
        self.surveys.push(survey);
        self.update_stats();
        true
    }

    /// Get survey
    pub fn get_survey(&self, id: &str) -> Option<&PlotSurvey> {
        self.surveys.iter().find(|s| s.id == id)
    }

    /// Get survey mut
    pub fn get_survey_mut(&mut self, id: &str) -> Option<&mut PlotSurvey> {
        self.surveys.iter_mut().find(|s| s.id == id)
    }

    /// Add steward
    pub fn add_steward(&mut self, steward: PlotSteward) {
        self.stewards.push(steward);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.surveys, self.config.plot_type);
    }

    /// Get stats
    pub fn stats(&self) -> &PlotStats {
        &self.stats
    }

    /// Survey count
    pub fn survey_count(&self) -> usize {
        self.surveys.len()
    }
}
