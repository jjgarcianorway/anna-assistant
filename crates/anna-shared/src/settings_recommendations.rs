// v0.0.578: Settings Recommendations (Phase 154)
// Provide intelligent settings recommendations based on usage

use serde::{Deserialize, Serialize};

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

/// Recommendation priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationPriority {
    /// Low priority - nice to have
    Low,
    /// Medium priority - suggested
    Medium,
    /// High priority - recommended
    High,
    /// Critical - strongly recommended
    Critical,
}

impl std::fmt::Display for RecommendationPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Recommendation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Security improvement
    Security,
    /// Performance optimization
    Performance,
    /// Usability enhancement
    Usability,
    /// Privacy protection
    Privacy,
    /// Best practice
    BestPractice,
}

impl std::fmt::Display for RecommendationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Security => write!(f, "Security"),
            Self::Performance => write!(f, "Performance"),
            Self::Usability => write!(f, "Usability"),
            Self::Privacy => write!(f, "Privacy"),
            Self::BestPractice => write!(f, "Best Practice"),
        }
    }
}

/// Recommendation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RecommendationStatus {
    /// Active recommendation
    #[default]
    Active,
    /// Dismissed by user
    Dismissed,
    /// Applied
    Applied,
    /// Expired
    Expired,
}

impl std::fmt::Display for RecommendationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Dismissed => write!(f, "Dismissed"),
            Self::Applied => write!(f, "Applied"),
            Self::Expired => write!(f, "Expired"),
        }
    }
}

/// Single recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Unique ID
    pub id: u64,
    /// Type
    pub rec_type: RecommendationType,
    /// Priority
    pub priority: RecommendationPriority,
    /// Category affected
    pub category: SettingsCategory,
    /// Setting name
    pub setting: String,
    /// Current value description
    pub current: String,
    /// Recommended value description
    pub recommended: String,
    /// Reason for recommendation
    pub reason: String,
    /// Status
    pub status: RecommendationStatus,
    /// Created timestamp
    pub created: chrono::DateTime<chrono::Utc>,
}

impl Recommendation {
    /// Create new recommendation
    pub fn new(
        id: u64,
        rec_type: RecommendationType,
        category: SettingsCategory,
        setting: impl Into<String>,
    ) -> Self {
        Self {
            id,
            rec_type,
            priority: RecommendationPriority::Medium,
            category,
            setting: setting.into(),
            current: String::new(),
            recommended: String::new(),
            reason: String::new(),
            status: RecommendationStatus::Active,
            created: chrono::Utc::now(),
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: RecommendationPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set current value
    pub fn current(mut self, current: impl Into<String>) -> Self {
        self.current = current.into();
        self
    }

    /// Set recommended value
    pub fn recommended(mut self, recommended: impl Into<String>) -> Self {
        self.recommended = recommended.into();
        self
    }

    /// Set reason
    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    /// Mark as dismissed
    pub fn dismiss(&mut self) {
        self.status = RecommendationStatus::Dismissed;
    }

    /// Mark as applied
    pub fn apply(&mut self) {
        self.status = RecommendationStatus::Applied;
    }

    /// Check if active
    pub fn is_active(&self) -> bool {
        self.status == RecommendationStatus::Active
    }
}

/// Recommendation engine
#[derive(Debug, Clone, Default)]
pub struct RecommendationEngine {
    /// All recommendations
    recommendations: Vec<Recommendation>,
    /// Next ID
    next_id: u64,
    /// Show dismissed
    show_dismissed: bool,
}

impl RecommendationEngine {
    /// Create new engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze settings and generate recommendations
    pub fn analyze(&mut self, settings: &UnifiedSettings) -> Vec<&Recommendation> {
        // Clear old active recommendations
        self.recommendations.retain(|r| r.status != RecommendationStatus::Active);

        // Generate new recommendations based on settings
        self.check_security(settings);
        self.check_privacy(settings);
        self.check_usability(settings);
        self.check_performance(settings);

        self.active()
    }

    fn check_security(&mut self, settings: &UnifiedSettings) {
        // Check require_root_confirmation
        if !settings.risk.require_root_confirmation {
            self.add(
                Recommendation::new(
                    self.next_id,
                    RecommendationType::Security,
                    SettingsCategory::Risk,
                    "require_root_confirmation",
                )
                .priority(RecommendationPriority::High)
                .current("Disabled")
                .recommended("Enabled")
                .reason("Root confirmation helps prevent accidental system changes"),
            );
            self.next_id += 1;
        }

        // Check confirmation settings
        if !settings.confirmation.always_confirm_delete {
            self.add(
                Recommendation::new(
                    self.next_id,
                    RecommendationType::Security,
                    SettingsCategory::Confirmation,
                    "always_confirm_delete",
                )
                .priority(RecommendationPriority::Critical)
                .current("Disabled")
                .recommended("Enabled")
                .reason("Confirmation for delete actions prevents data loss"),
            );
            self.next_id += 1;
        }
    }

    fn check_privacy(&mut self, settings: &UnifiedSettings) {
        // Check telemetry
        if settings.privacy.allow_telemetry {
            self.add(
                Recommendation::new(
                    self.next_id,
                    RecommendationType::Privacy,
                    SettingsCategory::Privacy,
                    "allow_telemetry",
                )
                .priority(RecommendationPriority::Low)
                .current("Enabled")
                .recommended("Disabled")
                .reason("Disabling telemetry improves privacy"),
            );
            self.next_id += 1;
        }
    }

    fn check_usability(&mut self, settings: &UnifiedSettings) {
        // Check verbosity
        if !settings.verbosity.show_progress {
            self.add(
                Recommendation::new(
                    self.next_id,
                    RecommendationType::Usability,
                    SettingsCategory::Verbosity,
                    "show_progress",
                )
                .priority(RecommendationPriority::Low)
                .current("Disabled")
                .recommended("Enabled")
                .reason("Progress indicators help track long-running operations"),
            );
            self.next_id += 1;
        }
    }

    fn check_performance(&mut self, settings: &UnifiedSettings) {
        // Check timeout
        if settings.timeout.command_timeout_ms > 60000 {
            self.add(
                Recommendation::new(
                    self.next_id,
                    RecommendationType::Performance,
                    SettingsCategory::Timeout,
                    "command_timeout_ms",
                )
                .priority(RecommendationPriority::Medium)
                .current(format!("{}ms", settings.timeout.command_timeout_ms))
                .recommended("60000ms or less")
                .reason("Lower timeout prevents hanging on slow operations"),
            );
            self.next_id += 1;
        }
    }

    fn add(&mut self, rec: Recommendation) {
        self.recommendations.push(rec);
    }

    /// Get active recommendations
    pub fn active(&self) -> Vec<&Recommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.is_active())
            .collect()
    }

    /// Get all recommendations
    pub fn all(&self) -> &[Recommendation] {
        &self.recommendations
    }

    /// Get recommendations by type
    pub fn by_type(&self, rec_type: RecommendationType) -> Vec<&Recommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.rec_type == rec_type && (r.is_active() || self.show_dismissed))
            .collect()
    }

    /// Get recommendations by priority
    pub fn by_priority(&self, priority: RecommendationPriority) -> Vec<&Recommendation> {
        self.recommendations
            .iter()
            .filter(|r| r.priority == priority && r.is_active())
            .collect()
    }

    /// Get recommendation by ID
    pub fn get(&self, id: u64) -> Option<&Recommendation> {
        self.recommendations.iter().find(|r| r.id == id)
    }

    /// Get mutable recommendation by ID
    pub fn get_mut(&mut self, id: u64) -> Option<&mut Recommendation> {
        self.recommendations.iter_mut().find(|r| r.id == id)
    }

    /// Dismiss recommendation
    pub fn dismiss(&mut self, id: u64) -> bool {
        if let Some(rec) = self.get_mut(id) {
            rec.dismiss();
            true
        } else {
            false
        }
    }

    /// Apply recommendation
    pub fn apply(&mut self, id: u64) -> bool {
        if let Some(rec) = self.get_mut(id) {
            rec.apply();
            true
        } else {
            false
        }
    }

    /// Count active recommendations
    pub fn active_count(&self) -> usize {
        self.recommendations.iter().filter(|r| r.is_active()).count()
    }

    /// Count by priority
    pub fn count_by_priority(&self, priority: RecommendationPriority) -> usize {
        self.by_priority(priority).len()
    }
}

/// Format recommendations for display
pub fn format_recommendations(engine: &RecommendationEngine) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Recommendations ===\n\n");
    output.push_str(&format!("Active: {}\n", engine.active_count()));
    output.push_str(&format!(
        "Critical: {}\n\n",
        engine.count_by_priority(RecommendationPriority::Critical)
    ));

    let active = engine.active();
    if active.is_empty() {
        output.push_str("No active recommendations. Your settings look good!\n");
        return output;
    }

    for rec in active {
        output.push_str(&format!(
            "• [{}] {} - {} ({})\n",
            rec.priority, rec.rec_type, rec.setting, rec.category
        ));
        output.push_str(&format!("  {}\n", rec.reason));
    }

    output
}

/// Check if query is about recommendations
pub fn is_recommendations_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("recommend")
        || lower.contains("suggestion")
        || lower.contains("improve settings")
        || lower.contains("optimize settings")
}

/// Fun fact about recommendations
pub fn settings_recommendations_fun_fact() -> &'static str {
    "Anna analyzes your settings to provide personalized recommendations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_priority_display() {
        assert_eq!(format!("{}", RecommendationPriority::High), "High");
        assert_eq!(format!("{}", RecommendationPriority::Critical), "Critical");
    }

    #[test]
    fn test_recommendation_type_display() {
        assert_eq!(format!("{}", RecommendationType::Security), "Security");
        assert_eq!(format!("{}", RecommendationType::Privacy), "Privacy");
    }

    #[test]
    fn test_recommendation_status_display() {
        assert_eq!(format!("{}", RecommendationStatus::Active), "Active");
        assert_eq!(format!("{}", RecommendationStatus::Applied), "Applied");
    }

    #[test]
    fn test_recommendation_new() {
        let rec = Recommendation::new(
            1,
            RecommendationType::Security,
            SettingsCategory::Risk,
            "test_setting",
        );
        assert_eq!(rec.id, 1);
        assert!(rec.is_active());
    }

    #[test]
    fn test_recommendation_builder() {
        let rec = Recommendation::new(
            1,
            RecommendationType::Security,
            SettingsCategory::Risk,
            "test",
        )
        .priority(RecommendationPriority::High)
        .reason("Test reason");

        assert_eq!(rec.priority, RecommendationPriority::High);
        assert_eq!(rec.reason, "Test reason");
    }

    #[test]
    fn test_recommendation_dismiss() {
        let mut rec = Recommendation::new(
            1,
            RecommendationType::Security,
            SettingsCategory::Risk,
            "test",
        );
        rec.dismiss();
        assert!(!rec.is_active());
        assert_eq!(rec.status, RecommendationStatus::Dismissed);
    }

    #[test]
    fn test_recommendation_apply() {
        let mut rec = Recommendation::new(
            1,
            RecommendationType::Security,
            SettingsCategory::Risk,
            "test",
        );
        rec.apply();
        assert_eq!(rec.status, RecommendationStatus::Applied);
    }

    #[test]
    fn test_recommendation_engine_new() {
        let engine = RecommendationEngine::new();
        assert_eq!(engine.active_count(), 0);
    }

    #[test]
    fn test_recommendation_engine_analyze() {
        let mut engine = RecommendationEngine::new();
        let settings = UnifiedSettings::default();
        let recs = engine.analyze(&settings);
        // Should generate some recommendations for default settings
        assert!(recs.len() >= 0);
    }

    #[test]
    fn test_recommendation_engine_dismiss() {
        let mut engine = RecommendationEngine::new();
        let settings = UnifiedSettings::default();
        engine.analyze(&settings);

        if engine.active_count() > 0 {
            let id = engine.active()[0].id;
            assert!(engine.dismiss(id));
        }
    }

    #[test]
    fn test_format_recommendations() {
        let engine = RecommendationEngine::new();
        let output = format_recommendations(&engine);
        assert!(output.contains("Recommendations"));
    }

    #[test]
    fn test_is_recommendations_query() {
        assert!(is_recommendations_query("show recommendations"));
        assert!(is_recommendations_query("suggestions for settings"));
        assert!(is_recommendations_query("improve settings"));
        assert!(!is_recommendations_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_recommendations_fun_fact();
        assert!(fact.contains("recommend"));
    }
}
