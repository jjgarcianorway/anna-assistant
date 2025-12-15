// v0.0.670: Settings Query Engine (Phase 246)
// Query engine for complex settings queries

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum QueryType {
    /// Select query
    #[default]
    Select,
    /// Filter query
    Filter,
    /// Aggregate query
    Aggregate,
    /// Join query
    Join,
    /// Union query
    Union,
}

impl std::fmt::Display for QueryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Select => write!(f, "select"),
            Self::Filter => write!(f, "filter"),
            Self::Aggregate => write!(f, "aggregate"),
            Self::Join => write!(f, "join"),
            Self::Union => write!(f, "union"),
        }
    }
}

/// Query operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum QueryOperator {
    /// Equals
    #[default]
    Eq,
    /// Not equals
    Ne,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Contains
    Contains,
    /// StartsWith
    StartsWith,
}

impl std::fmt::Display for QueryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "eq"),
            Self::Ne => write!(f, "ne"),
            Self::Gt => write!(f, "gt"),
            Self::Lt => write!(f, "lt"),
            Self::Contains => write!(f, "contains"),
            Self::StartsWith => write!(f, "starts_with"),
        }
    }
}

/// Query engine config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEngineConfig {
    /// Max results
    pub max_results: usize,
    /// Enable caching
    pub enable_cache: bool,
    /// Timeout (ms)
    pub timeout_ms: u64,
    /// Case insensitive
    pub case_insensitive: bool,
}

impl QueryEngineConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            max_results: 1000,
            enable_cache: true,
            timeout_ms: 5000,
            case_insensitive: true,
        }
    }

    /// Set max results
    pub fn max_results(mut self, max: usize) -> Self {
        self.max_results = max;
        self
    }

    /// Set enable cache
    pub fn enable_cache(mut self, enable: bool) -> Self {
        self.enable_cache = enable;
        self
    }

    /// Set case insensitive
    pub fn case_insensitive(mut self, ci: bool) -> Self {
        self.case_insensitive = ci;
        self
    }
}

impl Default for QueryEngineConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Query condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCondition {
    /// Field to match
    pub field: String,
    /// Operator
    pub operator: QueryOperator,
    /// Value to match
    pub value: String,
}

impl QueryCondition {
    /// Create new condition
    pub fn new(field: impl Into<String>, operator: QueryOperator, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            operator,
            value: value.into(),
        }
    }

    /// Evaluate condition
    pub fn evaluate(&self, key: &str, value: &str, case_insensitive: bool) -> bool {
        let target = if self.field == "key" { key } else { value };
        let (target, check) = if case_insensitive {
            (target.to_lowercase(), self.value.to_lowercase())
        } else {
            (target.to_string(), self.value.clone())
        };

        match self.operator {
            QueryOperator::Eq => target == check,
            QueryOperator::Ne => target != check,
            QueryOperator::Gt => target > check,
            QueryOperator::Lt => target < check,
            QueryOperator::Contains => target.contains(&check),
            QueryOperator::StartsWith => target.starts_with(&check),
        }
    }
}

/// Query definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    /// Query type
    pub query_type: QueryType,
    /// Conditions
    pub conditions: Vec<QueryCondition>,
    /// Limit
    pub limit: Option<usize>,
    /// Offset
    pub offset: usize,
}

impl Query {
    /// Create new query
    pub fn new(query_type: QueryType) -> Self {
        Self {
            query_type,
            conditions: Vec::new(),
            limit: None,
            offset: 0,
        }
    }

    /// Add condition
    pub fn condition(mut self, condition: QueryCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new(QueryType::Select)
    }
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Matching entries
    pub entries: Vec<(String, String)>,
    /// Total matches
    pub total_matches: usize,
    /// Query time (ms)
    pub query_time_ms: u64,
    /// Query type used
    pub query_type: QueryType,
    /// Success
    pub success: bool,
}

impl QueryResult {
    /// Create success result
    pub fn success(entries: Vec<(String, String)>, query_type: QueryType) -> Self {
        let total_matches = entries.len();
        Self {
            entries,
            total_matches,
            query_time_ms: 0,
            query_type,
            success: true,
        }
    }

    /// Create empty result
    pub fn empty(query_type: QueryType) -> Self {
        Self::success(Vec::new(), query_type)
    }

    /// With time
    pub fn with_time(mut self, time_ms: u64) -> Self {
        self.query_time_ms = time_ms;
        self
    }

    /// Has results
    pub fn has_results(&self) -> bool {
        !self.entries.is_empty()
    }
}

impl Default for QueryResult {
    fn default() -> Self {
        Self::empty(QueryType::Select)
    }
}

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

/// Query engine registry
#[derive(Debug, Clone, Default)]
pub struct QueryEngineRegistry {
    /// Engines by ID
    engines: HashMap<String, SettingsQueryEngine>,
}

impl QueryEngineRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register engine
    pub fn register(&mut self, id: impl Into<String>, engine: SettingsQueryEngine) {
        self.engines.insert(id.into(), engine);
    }

    /// Unregister engine
    pub fn unregister(&mut self, id: &str) -> bool {
        self.engines.remove(id).is_some()
    }

    /// Get engine
    pub fn get(&self, id: &str) -> Option<&SettingsQueryEngine> {
        self.engines.get(id)
    }

    /// Get engine mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsQueryEngine> {
        self.engines.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.engines.len()
    }
}

/// Format query engine registry
pub fn format_query_engine_registry(registry: &QueryEngineRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Query Engine Registry:\n");
    output.push_str(&format!("  Engines: {}\n", registry.count()));
    output
}

/// Check if query is about query engine
pub fn is_query_engine_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("query engine") || lower.contains("query settings") || lower.contains("search engine")
}

/// Fun fact about query engine
pub fn query_engine_fun_fact() -> &'static str {
    "Anna's query engine supports complex queries with multiple conditions!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_type_display() {
        assert_eq!(format!("{}", QueryType::Select), "select");
        assert_eq!(format!("{}", QueryType::Filter), "filter");
    }

    #[test]
    fn test_operator_display() {
        assert_eq!(format!("{}", QueryOperator::Eq), "eq");
        assert_eq!(format!("{}", QueryOperator::Contains), "contains");
    }

    #[test]
    fn test_config_new() {
        let c = QueryEngineConfig::new();
        assert!(c.enable_cache);
    }

    #[test]
    fn test_config_builder() {
        let c = QueryEngineConfig::new()
            .max_results(100)
            .case_insensitive(false);
        assert_eq!(c.max_results, 100);
        assert!(!c.case_insensitive);
    }

    #[test]
    fn test_condition_evaluate() {
        let c = QueryCondition::new("key", QueryOperator::StartsWith, "app.");
        assert!(c.evaluate("app.name", "value", true));
        assert!(!c.evaluate("db.host", "value", true));
    }

    #[test]
    fn test_query_new() {
        let q = Query::new(QueryType::Select);
        assert!(q.conditions.is_empty());
    }

    #[test]
    fn test_query_builder() {
        let q = Query::new(QueryType::Filter)
            .condition(QueryCondition::new("key", QueryOperator::Eq, "test"))
            .limit(10);
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.limit, Some(10));
    }

    #[test]
    fn test_result_success() {
        let r = QueryResult::success(vec![("k".to_string(), "v".to_string())], QueryType::Select);
        assert!(r.has_results());
        assert_eq!(r.total_matches, 1);
    }

    #[test]
    fn test_stats_record() {
        let mut s = QueryEngineStats::default();
        let r = QueryResult::success(vec![("k".to_string(), "v".to_string())], QueryType::Select);
        s.record(&r);
        assert_eq!(s.total_queries, 1);
        assert_eq!(s.total_matches, 1);
    }

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

    #[test]
    fn test_registry_new() {
        let r = QueryEngineRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = QueryEngineRegistry::new();
        r.register("e1", SettingsQueryEngine::new(QueryEngineConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_query_engine_query() {
        assert!(is_query_engine_query("query settings"));
        assert!(!is_query_engine_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = query_engine_fun_fact();
        assert!(fact.contains("query"));
    }
}
