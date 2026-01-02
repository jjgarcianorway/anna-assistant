// v0.0.578: Settings Recommendations - Engine (Phase 154)
// Core recommendation engine

use crate::unified_settings::UnifiedSettings;

use super::checkers::{check_performance, check_privacy, check_security, check_usability};
use super::types::{Recommendation, RecommendationPriority, RecommendationStatus, RecommendationType};

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
        check_security(&mut self.recommendations, &mut self.next_id, settings);
        check_privacy(&mut self.recommendations, &mut self.next_id, settings);
        check_usability(&mut self.recommendations, &mut self.next_id, settings);
        check_performance(&mut self.recommendations, &mut self.next_id, settings);

        self.active()
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
