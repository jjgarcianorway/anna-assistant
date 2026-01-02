// v0.0.670: Settings Query Engine Types (Phase 246)
// Query types and data structures

use serde::{Deserialize, Serialize};

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
}
