// v0.0.646: Settings Parser Result (Phase 222)
// Parse results and statistics

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{ParseSource, ParseError};

/// Parse result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    /// Was successful
    pub success: bool,
    /// Parsed values
    pub values: HashMap<String, String>,
    /// Errors
    pub errors: Vec<ParseError>,
    /// Source type
    pub source: ParseSource,
}

impl ParseResult {
    /// Create success result
    pub fn success(values: HashMap<String, String>, source: ParseSource) -> Self {
        Self {
            success: true,
            values,
            errors: Vec::new(),
            source,
        }
    }

    /// Create failure result
    pub fn failure(errors: Vec<ParseError>, source: ParseSource) -> Self {
        Self {
            success: false,
            values: HashMap::new(),
            errors,
            source,
        }
    }

    /// Value count
    pub fn value_count(&self) -> usize {
        self.values.len()
    }

    /// Error count
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

/// Parser stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParserStats {
    /// Total parses
    pub total_parses: usize,
    /// Successful parses
    pub successful: usize,
    /// Failed parses
    pub failed: usize,
    /// By source
    pub by_source: HashMap<String, usize>,
    /// Total values parsed
    pub total_values: usize,
}

impl ParserStats {
    /// Record parse
    pub fn record(&mut self, source: ParseSource, success: bool, value_count: usize) {
        self.total_parses += 1;
        if success {
            self.successful += 1;
            self.total_values += value_count;
        } else {
            self.failed += 1;
        }
        *self.by_source.entry(source.to_string()).or_insert(0) += 1;
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_parses == 0 {
            0.0
        } else {
            self.successful as f64 / self.total_parses as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_success() {
        let mut values = HashMap::new();
        values.insert("key".to_string(), "value".to_string());
        let r = ParseResult::success(values, ParseSource::Json);
        assert!(r.success);
        assert_eq!(r.value_count(), 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = ParserStats::default();
        s.record(ParseSource::Json, true, 5);
        s.record(ParseSource::Json, false, 0);
        assert_eq!(s.total_parses, 2);
        assert_eq!(s.successful, 1);
    }
}
