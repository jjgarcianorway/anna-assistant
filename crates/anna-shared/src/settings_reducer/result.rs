// v0.0.677: Settings Reducer Result (Phase 253)
// Result types for reduction operations

use serde::{Deserialize, Serialize};
use super::types::ReduceOp;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
