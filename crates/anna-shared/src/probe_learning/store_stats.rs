//! Statistics, health, and quality tracking.
//! Provides analytics and confidence metrics for the learning system.

use super::store::ProbeLearningStore;
use super::store_feedback::now_secs;
use super::types::{LearningHealth, LearningStats, QualityTrend, TrendDirection};

impl ProbeLearningStore {
    /// Get learning stats for display
    pub fn learning_stats(&self) -> LearningStats {
        LearningStats {
            total_queries: self.successful_patterns.len() + self.negative_patterns.len(),
            successful_patterns: self.successful_patterns.len(),
            negative_patterns: self.negative_patterns.len(),
            keywords_learned: self.keyword_probes.len(),
            categories_with_data: self.effectiveness.len(),
            avg_quality: self
                .successful_patterns
                .iter()
                .map(|p| p.quality as f32)
                .sum::<f32>()
                / self.successful_patterns.len().max(1) as f32,
        }
    }

    /// v0.0.331: Get quality trend (comparing recent vs previous period)
    pub fn quality_trend(&self) -> Option<QualityTrend> {
        let now = now_secs();
        let week = 7 * 24 * 60 * 60;
        let recent: Vec<_> = self
            .successful_patterns
            .iter()
            .filter(|p| now - p.timestamp < week)
            .collect();
        let previous: Vec<_> = self
            .successful_patterns
            .iter()
            .filter(|p| {
                let age = now - p.timestamp;
                age >= week && age < 2 * week
            })
            .collect();

        if recent.is_empty() && previous.is_empty() {
            return None;
        }

        let current_avg = if recent.is_empty() {
            0.0
        } else {
            recent.iter().map(|p| p.quality as f32).sum::<f32>() / recent.len() as f32
        };
        let previous_avg = if previous.is_empty() {
            current_avg
        } else {
            previous.iter().map(|p| p.quality as f32).sum::<f32>() / previous.len() as f32
        };

        let change = current_avg - previous_avg;
        let trend = if change > 0.3 {
            TrendDirection::Improving
        } else if change < -0.3 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        Some(QualityTrend {
            current_avg,
            previous_avg,
            trend,
            change,
        })
    }

    /// v0.0.332: Get confidence factor (0.0-1.0) based on learning health
    pub fn confidence_factor(&self) -> f32 {
        let stats = self.learning_stats();
        let volume = (stats.total_queries as f32 / 50.0).min(1.0);
        let quality = stats.avg_quality / 5.0;
        let diversity = (stats.keywords_learned as f32 / 30.0).min(1.0);
        let trend = match self.quality_trend() {
            Some(t) => match t.trend {
                TrendDirection::Improving => 1.1,
                TrendDirection::Stable => 1.0,
                TrendDirection::Declining => 0.8,
            },
            None => 0.9,
        };
        ((volume * 0.4 + quality * 0.3 + diversity * 0.3) * trend).clamp(0.0, 1.0)
    }

    /// v0.0.332: Should we trust learned recommendations?
    /// Returns true if confidence is above threshold
    pub fn should_use_learning(&self) -> bool {
        self.confidence_factor() >= 0.3
    }

    /// v0.0.332: Get learning health status
    pub fn health_status(&self) -> LearningHealth {
        let confidence = self.confidence_factor();
        let trend = self.quality_trend();
        if confidence >= 0.7 {
            LearningHealth::Excellent
        } else if confidence >= 0.5 {
            if let Some(t) = &trend {
                if t.trend == TrendDirection::Declining {
                    return LearningHealth::NeedsAttention;
                }
            }
            LearningHealth::Good
        } else if confidence >= 0.3 {
            LearningHealth::Developing
        } else {
            LearningHealth::Insufficient
        }
    }
}
