//! Predictive Forecasting Engine for Anna v0.14.0 "Orion III"
//!
//! Lightweight statistical models for trend prediction

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::history::{HistoryEntry, HistoryManager};
use crate::learning::LearningEngine;

/// Forecast window
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForecastWindow {
    SevenDay,
    ThirtyDay,
}

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub window_days: u64,              // 7 or 30
    pub current_score: f64,            // Current overall score
    pub predicted_score: f64,          // Forecasted score
    pub confidence_lower: f64,         // Lower confidence bound
    pub confidence_upper: f64,         // Upper confidence bound
    pub trajectory: String,            // "improving", "stable", "degrading"
    pub deviation: f64,                // Current vs predicted delta
    pub moving_average: f64,           // MA over window
    pub exponential_weighted: f64,     // EWM forecast (α=0.5)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavioral_trend: Option<BehavioralTrendScore>,  // Phase 2.2: Learning integration
}

/// Behavioral trend score (from learning engine)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralTrendScore {
    pub overall_trust: f32,           // 0.0 to 1.0
    pub acceptance_rate: f32,         // 0.0 to 1.0
    pub automation_readiness: f32,    // 0.0 to 1.0
    pub trend_direction: String,      // "improving", "stable", "declining"
}

/// Forecasting engine
pub struct ForecastEngine {
    history_mgr: HistoryManager,
    learning: Option<LearningEngine>,
}

impl ForecastEngine {
    /// Create new forecasting engine
    pub fn new() -> Result<Self> {
        let history_mgr = HistoryManager::new()?;
        let learning = LearningEngine::new().ok();
        Ok(Self { history_mgr, learning })
    }

    /// Generate 7-day forecast
    pub fn forecast_7d(&self) -> Result<Option<Forecast>> {
        self.forecast(ForecastWindow::SevenDay)
    }

    /// Generate 30-day forecast
    pub fn forecast_30d(&self) -> Result<Option<Forecast>> {
        self.forecast(ForecastWindow::ThirtyDay)
    }

    /// Generate forecast for given window
    fn forecast(&self, window: ForecastWindow) -> Result<Option<Forecast>> {
        let entries = self.history_mgr.load_all()?;

        if entries.len() < 3 {
            return Ok(None); // Need at least 3 data points
        }

        let window_days = match window {
            ForecastWindow::SevenDay => 7,
            ForecastWindow::ThirtyDay => 30,
        };

        let window_seconds = window_days * 86400;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Filter entries within window
        let window_entries: Vec<&HistoryEntry> = entries
            .iter()
            .filter(|e| e.timestamp >= now.saturating_sub(window_seconds))
            .collect();

        if window_entries.len() < 2 {
            return Ok(None);
        }

        // Current score
        let current_score = entries.last().unwrap().overall_score as f64;

        // Moving average
        let ma = self.moving_average(&window_entries);

        // Exponential weighted average (α=0.5)
        let ewm = self.exponential_weighted(&window_entries, 0.5);

        // Linear regression for trend
        let (slope, _intercept) = self.linear_regression(&window_entries);

        // Predicted score (blend MA and EWM)
        let predicted_score = (ma * 0.4 + ewm * 0.6).clamp(0.0, 10.0);

        // Confidence bounds (±1.5 based on recent variance)
        let variance = self.variance(&window_entries);
        let std_dev = variance.sqrt();
        let confidence_interval = std_dev * 1.5;

        let confidence_lower = (predicted_score - confidence_interval).max(0.0);
        let confidence_upper = (predicted_score + confidence_interval).min(10.0);

        // Trajectory classification
        let trajectory = if slope >= 0.15 {
            "improving"
        } else if slope <= -0.15 {
            "degrading"
        } else {
            "stable"
        };

        // Deviation
        let deviation = current_score - predicted_score;

        // Behavioral trend (Phase 2.2 integration)
        let behavioral_trend = self.get_behavioral_trend();

        Ok(Some(Forecast {
            window_days,
            current_score,
            predicted_score,
            confidence_lower,
            confidence_upper,
            trajectory: trajectory.to_string(),
            deviation,
            moving_average: ma,
            exponential_weighted: ewm,
            behavioral_trend,
        }))
    }

    /// Compute moving average
    fn moving_average(&self, entries: &[&HistoryEntry]) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }

        let sum: f64 = entries.iter().map(|e| e.overall_score as f64).sum();
        sum / entries.len() as f64
    }

    /// Compute exponential weighted average
    fn exponential_weighted(&self, entries: &[&HistoryEntry], alpha: f64) -> f64 {
        if entries.is_empty() {
            return 0.0;
        }

        let mut ewm = entries[0].overall_score as f64;

        for entry in entries.iter().skip(1) {
            let value = entry.overall_score as f64;
            ewm = alpha * value + (1.0 - alpha) * ewm;
        }

        ewm
    }

    /// Compute variance
    fn variance(&self, entries: &[&HistoryEntry]) -> f64 {
        if entries.len() < 2 {
            return 0.0;
        }

        let mean = self.moving_average(entries);
        let sum_squared_diff: f64 = entries
            .iter()
            .map(|e| {
                let diff = e.overall_score as f64 - mean;
                diff * diff
            })
            .sum();

        sum_squared_diff / entries.len() as f64
    }

    /// Linear regression (returns slope, intercept)
    fn linear_regression(&self, entries: &[&HistoryEntry]) -> (f64, f64) {
        if entries.len() < 2 {
            return (0.0, 0.0);
        }

        // Use indices as X (time progression)
        let n = entries.len() as f64;
        let x_values: Vec<f64> = (0..entries.len()).map(|i| i as f64).collect();
        let y_values: Vec<f64> = entries.iter().map(|e| e.overall_score as f64).collect();

        let x_mean: f64 = x_values.iter().sum::<f64>() / n;
        let y_mean: f64 = y_values.iter().sum::<f64>() / n;

        let numerator: f64 = x_values
            .iter()
            .zip(y_values.iter())
            .map(|(x, y)| (x - x_mean) * (y - y_mean))
            .sum();

        let denominator: f64 = x_values.iter().map(|x| (x - x_mean).powi(2)).sum();

        if denominator == 0.0 {
            return (0.0, y_mean);
        }

        let slope = numerator / denominator;
        let intercept = y_mean - slope * x_mean;

        (slope, intercept)
    }

    /// Get trajectory emoji
    pub fn trajectory_emoji(trajectory: &str) -> &'static str {
        match trajectory {
            "improving" => "↗",
            "degrading" => "↘",
            _ => "→",
        }
    }

    /// Get trajectory color
    pub fn trajectory_color(trajectory: &str) -> &'static str {
        match trajectory {
            "improving" => "\x1b[32m",  // Green
            "degrading" => "\x1b[31m",  // Red
            _ => "\x1b[36m",             // Cyan
        }
    }

    /// Get behavioral trend from learning engine
    fn get_behavioral_trend(&self) -> Option<BehavioralTrendScore> {
        let learning = self.learning.as_ref()?;
        let trend = learning.get_trend();

        let trend_direction = if trend.overall_trust > 0.65 {
            "improving"
        } else if trend.overall_trust < 0.35 {
            "declining"
        } else {
            "stable"
        };

        Some(BehavioralTrendScore {
            overall_trust: trend.overall_trust,
            acceptance_rate: trend.acceptance_trend,
            automation_readiness: trend.automation_readiness,
            trend_direction: trend_direction.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_entries() -> Vec<HistoryEntry> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        vec![
            HistoryEntry {
                timestamp: now - 14 * 86400,
                hardware_score: 7,
                software_score: 6,
                user_score: 7,
                overall_score: 7,
                top_recommendations: vec![],
            },
            HistoryEntry {
                timestamp: now - 10 * 86400,
                hardware_score: 7,
                software_score: 7,
                user_score: 7,
                overall_score: 7,
                top_recommendations: vec![],
            },
            HistoryEntry {
                timestamp: now - 7 * 86400,
                hardware_score: 8,
                software_score: 7,
                user_score: 8,
                overall_score: 8,
                top_recommendations: vec![],
            },
            HistoryEntry {
                timestamp: now - 3 * 86400,
                hardware_score: 8,
                software_score: 8,
                user_score: 8,
                overall_score: 8,
                top_recommendations: vec![],
            },
            HistoryEntry {
                timestamp: now,
                hardware_score: 9,
                software_score: 8,
                user_score: 8,
                overall_score: 8,
                top_recommendations: vec![],
            },
        ]
    }

    #[test]
    fn test_moving_average() {
        let engine = ForecastEngine::new().unwrap();
        let entries = mock_entries();
        let entry_refs: Vec<&HistoryEntry> = entries.iter().collect();

        let ma = engine.moving_average(&entry_refs);

        // Average of [7, 7, 8, 8, 8] = 7.6
        assert!((ma - 7.6).abs() < 0.01);
    }

    #[test]
    fn test_exponential_weighted() {
        let engine = ForecastEngine::new().unwrap();
        let entries = mock_entries();
        let entry_refs: Vec<&HistoryEntry> = entries.iter().collect();

        let ewm = engine.exponential_weighted(&entry_refs, 0.5);

        // EWM should be between MA and latest value
        assert!(ewm >= 7.0 && ewm <= 9.0);
    }

    #[test]
    fn test_variance() {
        let engine = ForecastEngine::new().unwrap();
        let entries = mock_entries();
        let entry_refs: Vec<&HistoryEntry> = entries.iter().collect();

        let variance = engine.variance(&entry_refs);

        // Should be non-zero (data has variation)
        assert!(variance > 0.0);
    }

    #[test]
    fn test_linear_regression() {
        let engine = ForecastEngine::new().unwrap();
        let entries = mock_entries();
        let entry_refs: Vec<&HistoryEntry> = entries.iter().collect();

        let (slope, intercept) = engine.linear_regression(&entry_refs);

        // Slope should be positive (improving trend: 7 → 8)
        assert!(slope > 0.0);
        assert!(intercept > 0.0);
    }

    #[test]
    fn test_trajectory_classification() {
        let engine = ForecastEngine::new().unwrap();
        let entries = mock_entries();
        let entry_refs: Vec<&HistoryEntry> = entries.iter().collect();

        let (slope, _) = engine.linear_regression(&entry_refs);

        let trajectory = if slope >= 0.15 {
            "improving"
        } else if slope <= -0.15 {
            "degrading"
        } else {
            "stable"
        };

        // With improving data, should be "improving"
        assert_eq!(trajectory, "improving");
    }

    #[test]
    fn test_trajectory_emoji() {
        assert_eq!(ForecastEngine::trajectory_emoji("improving"), "↗");
        assert_eq!(ForecastEngine::trajectory_emoji("degrading"), "↘");
        assert_eq!(ForecastEngine::trajectory_emoji("stable"), "→");
    }

    #[test]
    fn test_forecast_with_insufficient_data() {
        let engine = ForecastEngine::new().unwrap();

        // Engine won't have history in test env
        // This test verifies graceful handling
        let result = engine.forecast_7d();
        assert!(result.is_ok());
    }

    #[test]
    fn test_forecast_serialization() {
        let forecast = Forecast {
            window_days: 7,
            current_score: 8.0,
            predicted_score: 8.2,
            confidence_lower: 7.0,
            confidence_upper: 9.0,
            trajectory: "improving".to_string(),
            deviation: -0.2,
            moving_average: 7.8,
            exponential_weighted: 8.1,
        };

        let json = serde_json::to_string(&forecast).unwrap();
        let parsed: Forecast = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.window_days, 7);
        assert_eq!(parsed.trajectory, "improving");
    }
}
