// v0.0.670: Settings Query Engine Implementation (Phase 246)
// Core query engine implementation

use std::collections::HashMap;

use super::config::QueryEngineConfig;
use super::stats::QueryEngineStats;
use super::types::{Query, QueryCondition, QueryOperator, QueryResult, QueryType};

/// Settings query engine
#[derive(Debug, Clone, Default)]
pub struct SettingsQueryEngine {
    /// Config
    config: QueryEngineConfig,
    /// Stats
    stats: QueryEngineStats,
}

impl SettingsQueryEngine {
    /// Create new engine
    pub fn new(config: QueryEngineConfig) -> Self {
        Self {
            config,
            stats: QueryEngineStats::default(),
        }
    }

    /// Execute query
    pub fn execute(&mut self, query: &Query, settings: &HashMap<String, String>) -> QueryResult {
        let mut matches: Vec<(String, String)> = settings.iter()
            .filter(|(key, value)| {
                query.conditions.iter().all(|c| c.evaluate(key, value, self.config.case_insensitive))
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Apply offset
        if query.offset > 0 {
            matches = matches.into_iter().skip(query.offset).collect();
        }

        // Apply limit
        if let Some(limit) = query.limit {
            matches.truncate(limit.min(self.config.max_results));
        } else {
            matches.truncate(self.config.max_results);
        }

        let result = QueryResult::success(matches, query.query_type);
        self.stats.record(&result);
        result
    }

    /// Select all
    pub fn select_all(&mut self, settings: &HashMap<String, String>) -> QueryResult {
        self.execute(&Query::default(), settings)
    }

    /// Select where key starts with
    pub fn select_where_key_starts(&mut self, prefix: &str, settings: &HashMap<String, String>) -> QueryResult {
        let query = Query::new(QueryType::Select)
            .condition(QueryCondition::new("key", QueryOperator::StartsWith, prefix));
        self.execute(&query, settings)
    }

    /// Get stats
    pub fn stats(&self) -> &QueryEngineStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_new() {
        let e = SettingsQueryEngine::new(QueryEngineConfig::default());
        assert_eq!(e.stats().total_queries, 0);
    }

    #[test]
    fn test_engine_select_all() {
        let mut e = SettingsQueryEngine::new(QueryEngineConfig::default());
        let mut settings = HashMap::new();
        settings.insert("k1".to_string(), "v1".to_string());
        settings.insert("k2".to_string(), "v2".to_string());

        let result = e.select_all(&settings);
        assert_eq!(result.total_matches, 2);
    }

    #[test]
    fn test_engine_select_where() {
        let mut e = SettingsQueryEngine::new(QueryEngineConfig::default());
        let mut settings = HashMap::new();
        settings.insert("app.name".to_string(), "test".to_string());
        settings.insert("app.version".to_string(), "1.0".to_string());
        settings.insert("db.host".to_string(), "localhost".to_string());

        let result = e.select_where_key_starts("app.", &settings);
        assert_eq!(result.total_matches, 2);
    }
}
