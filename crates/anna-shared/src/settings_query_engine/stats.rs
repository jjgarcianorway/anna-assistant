// v0.0.670: Settings Query Engine Stats (Phase 246)
// Statistics tracking for query engine

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::QueryResult;

/// Query engine stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryEngineStats {
    /// Total queries
    pub total_queries: usize,
    /// Total matches
    pub total_matches: usize,
    /// Cache hits
    pub cache_hits: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl QueryEngineStats {
    /// Record query
    pub fn record(&mut self, result: &QueryResult) {
        self.total_queries += 1;
        self.total_matches += result.total_matches;
        *self.by_type.entry(result.query_type.to_string()).or_insert(0) += 1;
    }

    /// Record cache hit
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    /// Matches per query
    pub fn matches_per_query(&self) -> f64 {
        if self.total_queries == 0 {
            0.0
        } else {
            self.total_matches as f64 / self.total_queries as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_query_engine::types::{QueryResult, QueryType};

    #[test]
    fn test_stats_record() {
        let mut s = QueryEngineStats::default();
        let r = QueryResult::success(vec![("k".to_string(), "v".to_string())], QueryType::Select);
        s.record(&r);
        assert_eq!(s.total_queries, 1);
        assert_eq!(s.total_matches, 1);
    }
}
