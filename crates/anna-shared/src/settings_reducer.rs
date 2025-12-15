// v0.0.677: Settings Reducer (Phase 253)
// Reduce settings to aggregated values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reduce operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ReduceOp {
    /// Count entries
    #[default]
    Count,
    /// Sum numeric values
    Sum,
    /// Average numeric values
    Average,
    /// Find minimum
    Min,
    /// Find maximum
    Max,
    /// Concatenate values
    Concat,
}

impl std::fmt::Display for ReduceOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Count => write!(f, "count"),
            Self::Sum => write!(f, "sum"),
            Self::Average => write!(f, "average"),
            Self::Min => write!(f, "min"),
            Self::Max => write!(f, "max"),
            Self::Concat => write!(f, "concat"),
        }
    }
}

/// Reduce target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ReduceTarget {
    /// Reduce values
    #[default]
    Values,
    /// Reduce keys
    Keys,
    /// Reduce both
    Both,
}

impl std::fmt::Display for ReduceTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Values => write!(f, "values"),
            Self::Keys => write!(f, "keys"),
            Self::Both => write!(f, "both"),
        }
    }
}

/// Reducer config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReducerConfig {
    /// Default operation
    pub default_op: ReduceOp,
    /// Default target
    pub default_target: ReduceTarget,
    /// Concat separator
    pub concat_separator: String,
    /// Skip non-numeric
    pub skip_non_numeric: bool,
}

impl ReducerConfig {
    /// Create new config
    pub fn new(op: ReduceOp) -> Self {
        Self {
            default_op: op,
            default_target: ReduceTarget::Values,
            concat_separator: ", ".to_string(),
            skip_non_numeric: true,
        }
    }

    /// Set target
    pub fn target(mut self, target: ReduceTarget) -> Self {
        self.default_target = target;
        self
    }

    /// Set concat separator
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.concat_separator = sep.into();
        self
    }

    /// Set skip non-numeric
    pub fn skip_non_numeric(mut self, skip: bool) -> Self {
        self.skip_non_numeric = skip;
        self
    }
}

impl Default for ReducerConfig {
    fn default() -> Self {
        Self::new(ReduceOp::Count)
    }
}

/// Reduced value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReducedValue {
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// None
    None,
}

impl ReducedValue {
    /// As float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Integer(i) => Some(*i as f64),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// As string
    pub fn as_string(&self) -> String {
        match self {
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::String(s) => s.clone(),
            Self::None => "none".to_string(),
        }
    }
}

impl Default for ReducedValue {
    fn default() -> Self {
        Self::None
    }
}

/// Reduce result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReduceResult {
    /// Reduced value
    pub value: ReducedValue,
    /// Operation used
    pub operation: ReduceOp,
    /// Entries processed
    pub entries_processed: usize,
    /// Entries skipped
    pub entries_skipped: usize,
}

impl ReduceResult {
    /// Create new result
    pub fn new(value: ReducedValue, op: ReduceOp, processed: usize, skipped: usize) -> Self {
        Self {
            value,
            operation: op,
            entries_processed: processed,
            entries_skipped: skipped,
        }
    }

    /// Total entries
    pub fn total_entries(&self) -> usize {
        self.entries_processed + self.entries_skipped
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.total_entries();
        if total == 0 {
            0.0
        } else {
            self.entries_processed as f64 / total as f64
        }
    }
}

impl Default for ReduceResult {
    fn default() -> Self {
        Self::new(ReducedValue::None, ReduceOp::Count, 0, 0)
    }
}

/// Reducer stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReducerStats {
    /// Total reductions
    pub total_reductions: usize,
    /// Total processed
    pub total_processed: usize,
    /// Total skipped
    pub total_skipped: usize,
    /// By operation
    pub by_op: HashMap<String, usize>,
}

impl ReducerStats {
    /// Record reduction
    pub fn record(&mut self, result: &ReduceResult) {
        self.total_reductions += 1;
        self.total_processed += result.entries_processed;
        self.total_skipped += result.entries_skipped;
        *self.by_op.entry(result.operation.to_string()).or_insert(0) += 1;
    }

    /// Average processed
    pub fn average_processed(&self) -> f64 {
        if self.total_reductions == 0 {
            0.0
        } else {
            self.total_processed as f64 / self.total_reductions as f64
        }
    }
}

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

/// Reducer registry
#[derive(Debug, Clone, Default)]
pub struct ReducerRegistry {
    /// Reducers by ID
    reducers: HashMap<String, SettingsReducer>,
}

impl ReducerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register reducer
    pub fn register(&mut self, id: impl Into<String>, reducer: SettingsReducer) {
        self.reducers.insert(id.into(), reducer);
    }

    /// Unregister reducer
    pub fn unregister(&mut self, id: &str) -> bool {
        self.reducers.remove(id).is_some()
    }

    /// Get reducer
    pub fn get(&self, id: &str) -> Option<&SettingsReducer> {
        self.reducers.get(id)
    }

    /// Get reducer mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsReducer> {
        self.reducers.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.reducers.len()
    }
}

/// Format reducer registry
pub fn format_reducer_registry(registry: &ReducerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Reducer Registry:\n");
    output.push_str(&format!("  Reducers: {}\n", registry.count()));
    output
}

/// Check if query is about reducer
pub fn is_reducer_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("reduce settings") || lower.contains("settings reducer") || lower.contains("aggregate settings")
}

/// Fun fact about reducer
pub fn reducer_fun_fact() -> &'static str {
    "Anna's settings reducer aggregates your settings into meaningful summaries!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_op_display() {
        assert_eq!(format!("{}", ReduceOp::Count), "count");
        assert_eq!(format!("{}", ReduceOp::Sum), "sum");
    }

    #[test]
    fn test_reduce_target_display() {
        assert_eq!(format!("{}", ReduceTarget::Values), "values");
        assert_eq!(format!("{}", ReduceTarget::Keys), "keys");
    }

    #[test]
    fn test_config_new() {
        let c = ReducerConfig::new(ReduceOp::Sum);
        assert_eq!(c.default_op, ReduceOp::Sum);
    }

    #[test]
    fn test_config_builder() {
        let c = ReducerConfig::new(ReduceOp::Concat)
            .separator("; ")
            .target(ReduceTarget::Keys);
        assert_eq!(c.concat_separator, "; ");
        assert_eq!(c.default_target, ReduceTarget::Keys);
    }

    #[test]
    fn test_reduced_value_as_float() {
        let v = ReducedValue::Float(3.14);
        assert!((v.as_float().unwrap() - 3.14).abs() < 0.001);

        let i = ReducedValue::Integer(42);
        assert!((i.as_float().unwrap() - 42.0).abs() < 0.001);
    }

    #[test]
    fn test_reduced_value_as_string() {
        let v = ReducedValue::Float(3.14);
        assert!(v.as_string().contains("3.14"));

        let s = ReducedValue::String("hello".to_string());
        assert_eq!(s.as_string(), "hello");
    }

    #[test]
    fn test_result_new() {
        let r = ReduceResult::new(ReducedValue::Integer(5), ReduceOp::Count, 5, 0);
        assert_eq!(r.entries_processed, 5);
        assert_eq!(r.total_entries(), 5);
    }

    #[test]
    fn test_result_success_rate() {
        let r = ReduceResult::new(ReducedValue::Float(10.0), ReduceOp::Sum, 8, 2);
        assert!((r.success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ReducerStats::default();
        let r = ReduceResult::new(ReducedValue::Integer(3), ReduceOp::Count, 3, 0);
        s.record(&r);
        assert_eq!(s.total_reductions, 1);
        assert_eq!(s.total_processed, 3);
    }

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

    #[test]
    fn test_registry_new() {
        let r = ReducerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = ReducerRegistry::new();
        r.register("r1", SettingsReducer::new(ReducerConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_reducer_query() {
        assert!(is_reducer_query("reduce settings"));
        assert!(!is_reducer_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = reducer_fun_fact();
        assert!(fact.contains("reducer"));
    }
}
