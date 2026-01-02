// v0.0.578: Settings Recommendations - Types (Phase 154)
// Type definitions for recommendations

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

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
}
