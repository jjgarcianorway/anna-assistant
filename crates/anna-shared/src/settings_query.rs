// v0.0.596: Settings Query (Phase 172)
// Query DSL for settings

use serde::{Deserialize, Serialize};

use crate::unified_settings::SettingsCategory;

/// Query operator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryOperator {
    /// Equal
    Eq,
    /// Not equal
    Ne,
    /// Greater than
    Gt,
    /// Greater or equal
    Gte,
    /// Less than
    Lt,
    /// Less or equal
    Lte,
    /// Contains
    Contains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Matches regex
    Matches,
}

impl std::fmt::Display for QueryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eq => write!(f, "=="),
            Self::Ne => write!(f, "!="),
            Self::Gt => write!(f, ">"),
            Self::Gte => write!(f, ">="),
            Self::Lt => write!(f, "<"),
            Self::Lte => write!(f, "<="),
            Self::Contains => write!(f, "contains"),
            Self::StartsWith => write!(f, "starts_with"),
            Self::EndsWith => write!(f, "ends_with"),
            Self::Matches => write!(f, "matches"),
        }
    }
}

/// Query condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryCondition {
    /// Field path
    pub field: String,
    /// Operator
    pub operator: QueryOperator,
    /// Value to compare
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

    /// Check if matches string value
    pub fn matches(&self, actual: &str) -> bool {
        match self.operator {
            QueryOperator::Eq => actual == self.value,
            QueryOperator::Ne => actual != self.value,
            QueryOperator::Gt => actual > self.value.as_str(),
            QueryOperator::Gte => actual >= self.value.as_str(),
            QueryOperator::Lt => actual < self.value.as_str(),
            QueryOperator::Lte => actual <= self.value.as_str(),
            QueryOperator::Contains => actual.contains(&self.value),
            QueryOperator::StartsWith => actual.starts_with(&self.value),
            QueryOperator::EndsWith => actual.ends_with(&self.value),
            QueryOperator::Matches => {
                regex::Regex::new(&self.value)
                    .map(|re| re.is_match(actual))
                    .unwrap_or(false)
            }
        }
    }
}

/// Query builder
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsQuery {
    /// Target categories (empty = all)
    pub categories: Vec<SettingsCategory>,
    /// Conditions (AND)
    pub conditions: Vec<QueryCondition>,
    /// Field selection (empty = all)
    pub select: Vec<String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset
    pub offset: usize,
    /// Order by field
    pub order_by: Option<String>,
    /// Descending order
    pub descending: bool,
}

impl SettingsQuery {
    /// Create new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Add condition
    pub fn where_field(mut self, field: impl Into<String>, op: QueryOperator, value: impl Into<String>) -> Self {
        self.conditions.push(QueryCondition::new(field, op, value));
        self
    }

    /// Select fields
    pub fn select_field(mut self, field: impl Into<String>) -> Self {
        self.select.push(field.into());
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

    /// Order by field
    pub fn order_by(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(field.into());
        self
    }

    /// Descending order
    pub fn desc(mut self) -> Self {
        self.descending = true;
        self
    }

    /// Check if matches category
    pub fn matches_category(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }

    /// Condition count
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }

    /// Has conditions
    pub fn has_conditions(&self) -> bool {
        !self.conditions.is_empty()
    }
}

/// Query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Category
    pub category: SettingsCategory,
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Matched conditions
    pub matched: usize,
}

impl QueryResult {
    /// Create new result
    pub fn new(category: SettingsCategory, key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            category,
            key: key.into(),
            value: value.into(),
            matched: 0,
        }
    }

    /// Set matched count
    pub fn matched(mut self, count: usize) -> Self {
        self.matched = count;
        self
    }
}

/// Query executor
#[derive(Debug, Clone, Default)]
pub struct QueryExecutor {
    /// Query history
    history: Vec<SettingsQuery>,
    /// Max history
    max_history: usize,
}

impl QueryExecutor {
    /// Create new executor
    pub fn new() -> Self {
        Self {
            max_history: 100,
            ..Default::default()
        }
    }

    /// Record query
    pub fn record(&mut self, query: SettingsQuery) {
        self.history.push(query);
        while self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Get history
    pub fn history(&self) -> &[SettingsQuery] {
        &self.history
    }

    /// Recent queries
    pub fn recent(&self, count: usize) -> Vec<&SettingsQuery> {
        self.history.iter().rev().take(count).collect()
    }

    /// History count
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Format query
pub fn format_query(query: &SettingsQuery) -> String {
    let mut output = String::new();

    output.push_str("SELECT ");
    if query.select.is_empty() {
        output.push('*');
    } else {
        output.push_str(&query.select.join(", "));
    }

    output.push_str(" FROM settings");

    if !query.categories.is_empty() {
        let cats: Vec<_> = query.categories.iter().map(|c| format!("{}", c)).collect();
        output.push_str(&format!(" IN [{}]", cats.join(", ")));
    }

    if !query.conditions.is_empty() {
        output.push_str(" WHERE ");
        let conds: Vec<_> = query.conditions.iter()
            .map(|c| format!("{} {} '{}'", c.field, c.operator, c.value))
            .collect();
        output.push_str(&conds.join(" AND "));
    }

    if let Some(ref order) = query.order_by {
        output.push_str(&format!(" ORDER BY {}", order));
        if query.descending {
            output.push_str(" DESC");
        }
    }

    if let Some(limit) = query.limit {
        output.push_str(&format!(" LIMIT {}", limit));
    }

    output
}

/// Check if query is about settings query
pub fn is_settings_query_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("query")
        || lower.contains("select")
        || lower.contains("find")
        || lower.contains("search settings")
}

/// Fun fact about queries
pub fn settings_query_fun_fact() -> &'static str {
    "Anna supports SQL-like query syntax to search and filter your settings!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_display() {
        assert_eq!(format!("{}", QueryOperator::Eq), "==");
        assert_eq!(format!("{}", QueryOperator::Contains), "contains");
    }

    #[test]
    fn test_condition_new() {
        let cond = QueryCondition::new("field", QueryOperator::Eq, "value");
        assert_eq!(cond.field, "field");
    }

    #[test]
    fn test_condition_matches() {
        let cond = QueryCondition::new("f", QueryOperator::Contains, "test");
        assert!(cond.matches("this is a test"));
        assert!(!cond.matches("no match"));
    }

    #[test]
    fn test_query_new() {
        let query = SettingsQuery::new();
        assert!(query.categories.is_empty());
        assert!(!query.has_conditions());
    }

    #[test]
    fn test_query_builder() {
        let query = SettingsQuery::new()
            .category(SettingsCategory::Personality)
            .where_field("key", QueryOperator::Eq, "value")
            .limit(10);
        assert_eq!(query.categories.len(), 1);
        assert_eq!(query.condition_count(), 1);
    }

    #[test]
    fn test_query_matches_category() {
        let query = SettingsQuery::new().category(SettingsCategory::Personality);
        assert!(query.matches_category(SettingsCategory::Personality));
        assert!(!query.matches_category(SettingsCategory::Risk));
    }

    #[test]
    fn test_result_new() {
        let result = QueryResult::new(SettingsCategory::Privacy, "key", "value");
        assert_eq!(result.key, "key");
    }

    #[test]
    fn test_executor_new() {
        let executor = QueryExecutor::new();
        assert_eq!(executor.history_count(), 0);
    }

    #[test]
    fn test_executor_record() {
        let mut executor = QueryExecutor::new();
        executor.record(SettingsQuery::new());
        assert_eq!(executor.history_count(), 1);
    }

    #[test]
    fn test_format_query() {
        let query = SettingsQuery::new()
            .category(SettingsCategory::Personality)
            .where_field("formality", QueryOperator::Eq, "formal");
        let output = format_query(&query);
        assert!(output.contains("SELECT"));
        assert!(output.contains("WHERE"));
    }

    #[test]
    fn test_is_settings_query_query() {
        assert!(is_settings_query_query("query settings"));
        assert!(is_settings_query_query("select from"));
        assert!(!is_settings_query_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_query_fun_fact();
        assert!(fact.contains("query"));
    }
}
