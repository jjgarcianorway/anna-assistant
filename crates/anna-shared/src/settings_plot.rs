// v0.0.758: Settings Plot (Phase 334)
// Land plot for settings allocation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Plot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotType {
    /// Garden plot
    #[default]
    Garden,
    /// Building plot
    Building,
    /// Cemetery plot
    Cemetery,
    /// Allotment plot
    Allotment,
}

impl std::fmt::Display for PlotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Garden => write!(f, "garden"),
            Self::Building => write!(f, "building"),
            Self::Cemetery => write!(f, "cemetery"),
            Self::Allotment => write!(f, "allotment"),
        }
    }
}

/// Plot status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotStatus {
    /// Allocated status
    #[default]
    Allocated,
    /// Cultivated status
    Cultivated,
    /// Fallow status
    Fallow,
    /// Reserved status
    Reserved,
}

impl std::fmt::Display for PlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Allocated => write!(f, "allocated"),
            Self::Cultivated => write!(f, "cultivated"),
            Self::Fallow => write!(f, "fallow"),
            Self::Reserved => write!(f, "reserved"),
        }
    }
}

/// Plot config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotConfig {
    /// Name
    pub name: String,
    /// Plot type
    pub plot_type: PlotType,
    /// Status
    pub status: PlotStatus,
    /// Max surveys
    pub max_surveys: usize,
}

impl PlotConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            plot_type: PlotType::Garden,
            status: PlotStatus::Allocated,
            max_surveys: 100,
        }
    }

    /// Set type
    pub fn plot_type(mut self, pt: PlotType) -> Self {
        self.plot_type = pt;
        self
    }

    /// Set status
    pub fn status(mut self, s: PlotStatus) -> Self {
        self.status = s;
        self
    }

    /// Set max surveys
    pub fn max_surveys(mut self, max: usize) -> Self {
        self.max_surveys = max;
        self
    }
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Plot survey
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSurvey {
    /// Survey ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Row number
    pub row: u32,
    /// Verified
    pub verified: bool,
}

impl PlotSurvey {
    /// Create new survey
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            row: 0,
            verified: true,
        }
    }

    /// Set row
    pub fn row(mut self, r: u32) -> Self {
        self.row = r;
        self
    }

    /// Make verified
    pub fn make_verified(&mut self) {
        self.verified = true;
    }

    /// Make unverified
    pub fn make_unverified(&mut self) {
        self.verified = false;
    }
}

/// Plot steward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotSteward {
    /// Key
    pub key: String,
    /// Name
    pub name: String,
    /// Survey ID
    pub survey_id: String,
}

impl PlotSteward {
    /// Create new steward
    pub fn new(key: impl Into<String>, name: impl Into<String>, survey_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            name: name.into(),
            survey_id: survey_id.into(),
        }
    }
}

/// Plot stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlotStats {
    /// Total surveys
    pub total_surveys: usize,
    /// Verified surveys
    pub verified: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl PlotStats {
    /// Update from surveys
    pub fn update(&mut self, surveys: &[PlotSurvey], plot_type: PlotType) {
        self.total_surveys = surveys.len();
        self.verified = surveys.iter().filter(|s| s.verified).count();
        *self.by_type.entry(plot_type.to_string()).or_insert(0) += 1;
    }

    /// Verified rate
    pub fn verified_rate(&self) -> f64 {
        if self.total_surveys == 0 { 0.0 } else { self.verified as f64 / self.total_surveys as f64 * 100.0 }
    }
}

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

/// Format plot registry
pub fn format_plot_registry(registry: &PlotRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Plot Registry:\n");
    output.push_str(&format!("  Plots: {}\n", registry.count()));
    output
}

/// Check if query is about plot
pub fn is_plot_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings plot") || lower.contains("plot settings") || lower.contains("land plot")
}

/// Fun fact about plot
pub fn plot_fun_fact() -> &'static str {
    "Anna's settings plot establishes allocation boundaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plot_type_display() {
        assert_eq!(format!("{}", PlotType::Garden), "garden");
        assert_eq!(format!("{}", PlotType::Building), "building");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", PlotStatus::Allocated), "allocated");
        assert_eq!(format!("{}", PlotStatus::Cultivated), "cultivated");
    }

    #[test]
    fn test_config_new() {
        let c = PlotConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = PlotConfig::new("test")
            .plot_type(PlotType::Cemetery)
            .status(PlotStatus::Fallow);
        assert_eq!(c.plot_type, PlotType::Cemetery);
        assert_eq!(c.status, PlotStatus::Fallow);
    }

    #[test]
    fn test_survey_new() {
        let s = PlotSurvey::new("s1", "Title", "Content");
        assert_eq!(s.id, "s1");
    }

    #[test]
    fn test_survey_builder() {
        let s = PlotSurvey::new("s1", "Title", "Content")
            .row(1);
        assert_eq!(s.row, 1);
    }

    #[test]
    fn test_survey_verified() {
        let mut s = PlotSurvey::new("s1", "Title", "Content");
        s.make_unverified();
        assert!(!s.verified);
        s.make_verified();
        assert!(s.verified);
    }

    #[test]
    fn test_steward_new() {
        let s = PlotSteward::new("key", "name", "s1");
        assert_eq!(s.survey_id, "s1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = PlotStats::default();
        let survey = PlotSurvey::new("s1", "Title", "Content");
        s.update(&[survey], PlotType::Garden);
        assert_eq!(s.total_surveys, 1);
        assert_eq!(s.verified, 1);
    }

    #[test]
    fn test_plot_new() {
        let p = SettingsPlot::new(PlotConfig::default());
        assert_eq!(p.survey_count(), 0);
    }

    #[test]
    fn test_plot_add_survey() {
        let mut p = SettingsPlot::new(PlotConfig::default());
        p.add_survey(PlotSurvey::new("s1", "Title", "Content"));
        assert_eq!(p.survey_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = PlotRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = PlotRegistry::new();
        r.register("p1", SettingsPlot::new(PlotConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_plot_query() {
        assert!(is_plot_query("settings plot"));
        assert!(!is_plot_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = plot_fun_fact();
        assert!(fact.contains("plot"));
    }
}
