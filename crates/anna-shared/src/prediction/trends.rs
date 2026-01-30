//! Time-series trend analysis.
//!
//! Implements simple linear regression and trend detection.

use serde::{Deserialize, Serialize};

/// Trend direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Values increasing over time
    Increasing,
    /// Values decreasing over time
    Decreasing,
    /// No significant trend (stable)
    Stable,
}

/// Result of trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Direction of the trend
    pub direction: TrendDirection,
    /// Slope of the linear regression (units per day)
    pub slope: f64,
    /// R-squared (goodness of fit, 0-1)
    pub r_squared: f64,
    /// Current value
    pub current: f64,
    /// Predicted value after N days
    pub predicted: f64,
    /// Number of samples used
    pub sample_count: usize,
}

impl TrendAnalysis {
    /// Check if trend is significant (high R-squared).
    pub fn is_significant(&self) -> bool {
        self.r_squared > 0.5 && self.sample_count >= 5
    }

    /// Get days until a threshold is reached (if increasing).
    pub fn days_until(&self, threshold: f64) -> Option<f64> {
        if self.direction != TrendDirection::Increasing || self.slope <= 0.0 {
            return None;
        }
        if self.current >= threshold {
            return Some(0.0);
        }
        Some((threshold - self.current) / self.slope)
    }
}

/// Analyze trend from a series of values.
/// Values are assumed to be evenly spaced (e.g., daily).
pub fn analyze_trend(values: &[f64], forecast_days: usize) -> Option<TrendAnalysis> {
    if values.len() < 3 {
        return None;
    }

    let n = values.len() as f64;

    // Calculate means
    let x_mean = (n - 1.0) / 2.0;
    let y_mean: f64 = values.iter().sum::<f64>() / n;

    // Calculate slope and intercept using least squares
    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean) * (x - x_mean);
    }

    if denominator == 0.0 {
        return None;
    }

    let slope = numerator / denominator;
    let intercept = y_mean - slope * x_mean;

    // Calculate R-squared
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        let y_pred = slope * x + intercept;
        ss_res += (y - y_pred).powi(2);
        ss_tot += (y - y_mean).powi(2);
    }

    let r_squared = if ss_tot > 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    // Determine direction (threshold relative to data range)
    let data_range = values.iter().cloned().fold(f64::NAN, f64::max)
        - values.iter().cloned().fold(f64::NAN, f64::min);
    let stability_threshold = (data_range * 0.1).max(0.1);

    let direction = if slope.abs() < stability_threshold {
        TrendDirection::Stable
    } else if slope > 0.0 {
        TrendDirection::Increasing
    } else {
        TrendDirection::Decreasing
    };

    // Current and predicted values
    let current = *values.last().unwrap();
    let predicted = slope * (values.len() as f64 + forecast_days as f64 - 1.0) + intercept;

    Some(TrendAnalysis {
        direction,
        slope,
        r_squared,
        current,
        predicted,
        sample_count: values.len(),
    })
}

/// Detect if there's a memory leak (consistently increasing memory usage).
pub fn detect_memory_leak(memory_samples: &[f64]) -> Option<TrendAnalysis> {
    let analysis = analyze_trend(memory_samples, 7)?;

    // Memory leak criteria:
    // - Increasing trend
    // - Significant slope (> 0.5% per sample)
    // - Good fit (R² > 0.6)
    if analysis.direction == TrendDirection::Increasing
        && analysis.slope > 0.5
        && analysis.r_squared > 0.6
    {
        Some(analysis)
    } else {
        None
    }
}

/// Detect boot time degradation.
pub fn detect_boot_degradation(boot_times: &[f64]) -> Option<TrendAnalysis> {
    let analysis = analyze_trend(boot_times, 5)?;

    // Degradation criteria:
    // - Increasing trend
    // - Significant slope (> 0.5s per boot)
    // - Some fit (R² > 0.4)
    if analysis.direction == TrendDirection::Increasing
        && analysis.slope > 0.5
        && analysis.r_squared > 0.4
    {
        Some(analysis)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increasing_trend() {
        let values = vec![10.0, 12.0, 14.0, 16.0, 18.0, 20.0];
        let analysis = analyze_trend(&values, 7).unwrap();

        assert_eq!(analysis.direction, TrendDirection::Increasing);
        assert!(analysis.slope > 1.5);
        assert!(analysis.r_squared > 0.99);
    }

    #[test]
    fn test_decreasing_trend() {
        let values = vec![20.0, 18.0, 16.0, 14.0, 12.0, 10.0];
        let analysis = analyze_trend(&values, 7).unwrap();

        assert_eq!(analysis.direction, TrendDirection::Decreasing);
        assert!(analysis.slope < -1.5);
    }

    #[test]
    fn test_stable_trend() {
        let values = vec![50.0, 50.1, 49.9, 50.0, 50.1, 49.9];
        let analysis = analyze_trend(&values, 7).unwrap();

        assert_eq!(analysis.direction, TrendDirection::Stable);
    }

    #[test]
    fn test_days_until_threshold() {
        let values = vec![70.0, 72.0, 74.0, 76.0, 78.0, 80.0];
        let analysis = analyze_trend(&values, 7).unwrap();

        // Should reach 90% in about 5 days
        let days = analysis.days_until(90.0);
        assert!(days.is_some());
        assert!(days.unwrap() > 0.0 && days.unwrap() < 10.0);
    }
}
