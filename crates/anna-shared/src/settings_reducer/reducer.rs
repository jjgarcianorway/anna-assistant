// v0.0.677: Settings Reducer Core (Phase 253)
// Core reducer implementation

use std::collections::HashMap;
use super::config::ReducerConfig;
use super::result::{ReducedValue, ReduceResult};
use super::stats::ReducerStats;
use super::types::ReduceOp;

/// Settings reducer
#[derive(Debug, Clone, Default)]
pub struct SettingsReducer {
    /// Config
    config: ReducerConfig,
    /// Stats
    stats: ReducerStats,
}

impl SettingsReducer {
    /// Create new reducer
    pub fn new(config: ReducerConfig) -> Self {
        Self {
            config,
            stats: ReducerStats::default(),
        }
    }

    /// Count entries
    pub fn count(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let result = ReduceResult::new(
            ReducedValue::Integer(settings.len() as i64),
            ReduceOp::Count,
            settings.len(),
            0,
        );
        self.stats.record(&result);
        result
    }

    /// Sum numeric values
    pub fn sum(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let mut sum = 0.0;
        let mut processed = 0;
        let mut skipped = 0;

        for value in settings.values() {
            if let Ok(num) = value.parse::<f64>() {
                sum += num;
                processed += 1;
            } else {
                skipped += 1;
            }
        }

        let result = ReduceResult::new(ReducedValue::Float(sum), ReduceOp::Sum, processed, skipped);
        self.stats.record(&result);
        result
    }

    /// Average numeric values
    pub fn average(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let mut sum = 0.0;
        let mut count = 0;
        let mut skipped = 0;

        for value in settings.values() {
            if let Ok(num) = value.parse::<f64>() {
                sum += num;
                count += 1;
            } else {
                skipped += 1;
            }
        }

        let avg = if count > 0 { sum / count as f64 } else { 0.0 };
        let result = ReduceResult::new(ReducedValue::Float(avg), ReduceOp::Average, count, skipped);
        self.stats.record(&result);
        result
    }

    /// Find minimum value
    pub fn min(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let mut min_val: Option<f64> = None;
        let mut processed = 0;
        let mut skipped = 0;

        for value in settings.values() {
            if let Ok(num) = value.parse::<f64>() {
                min_val = Some(min_val.map_or(num, |m| m.min(num)));
                processed += 1;
            } else {
                skipped += 1;
            }
        }

        let value = min_val.map_or(ReducedValue::None, ReducedValue::Float);
        let result = ReduceResult::new(value, ReduceOp::Min, processed, skipped);
        self.stats.record(&result);
        result
    }

    /// Find maximum value
    pub fn max(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let mut max_val: Option<f64> = None;
        let mut processed = 0;
        let mut skipped = 0;

        for value in settings.values() {
            if let Ok(num) = value.parse::<f64>() {
                max_val = Some(max_val.map_or(num, |m| m.max(num)));
                processed += 1;
            } else {
                skipped += 1;
            }
        }

        let value = max_val.map_or(ReducedValue::None, ReducedValue::Float);
        let result = ReduceResult::new(value, ReduceOp::Max, processed, skipped);
        self.stats.record(&result);
        result
    }

    /// Concatenate values
    pub fn concat(&mut self, settings: &HashMap<String, String>) -> ReduceResult {
        let values: Vec<&str> = settings.values().map(|s| s.as_str()).collect();
        let concatenated = values.join(&self.config.concat_separator);
        let processed = settings.len();

        let result = ReduceResult::new(
            ReducedValue::String(concatenated),
            ReduceOp::Concat,
            processed,
            0,
        );
        self.stats.record(&result);
        result
    }

    /// Get stats
    pub fn stats(&self) -> &ReducerStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reducer_count() {
        let mut r = SettingsReducer::new(ReducerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "1".to_string());
        settings.insert("b".to_string(), "2".to_string());

        let result = r.count(&settings);
        assert_eq!(result.value.as_float(), Some(2.0));
    }

    #[test]
    fn test_reducer_sum() {
        let mut r = SettingsReducer::new(ReducerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "10".to_string());
        settings.insert("b".to_string(), "20".to_string());

        let result = r.sum(&settings);
        assert!((result.value.as_float().unwrap() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_reducer_average() {
        let mut r = SettingsReducer::new(ReducerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "10".to_string());
        settings.insert("b".to_string(), "20".to_string());

        let result = r.average(&settings);
        assert!((result.value.as_float().unwrap() - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_reducer_min_max() {
        let mut r = SettingsReducer::new(ReducerConfig::default());
        let mut settings = HashMap::new();
        settings.insert("a".to_string(), "5".to_string());
        settings.insert("b".to_string(), "15".to_string());
        settings.insert("c".to_string(), "10".to_string());

        let min_result = r.min(&settings);
        assert!((min_result.value.as_float().unwrap() - 5.0).abs() < 0.001);

        let max_result = r.max(&settings);
        assert!((max_result.value.as_float().unwrap() - 15.0).abs() < 0.001);
    }
}
