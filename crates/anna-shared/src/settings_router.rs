// v0.0.603: Settings Router (Phase 179)
// Routing logic for settings operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Route type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteType {
    /// Direct access
    Direct,
    /// Via middleware
    Middleware,
    /// Via proxy
    Proxy,
    /// Cached
    Cached,
    /// Redirect
    Redirect,
}

impl std::fmt::Display for RouteType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Direct => write!(f, "direct"),
            Self::Middleware => write!(f, "middleware"),
            Self::Proxy => write!(f, "proxy"),
            Self::Cached => write!(f, "cached"),
            Self::Redirect => write!(f, "redirect"),
        }
    }
}

/// Route action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteAction {
    /// Get value
    Get,
    /// Set value
    Set,
    /// Delete value
    Delete,
    /// List values
    List,
    /// Query values
    Query,
}

impl std::fmt::Display for RouteAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "get"),
            Self::Set => write!(f, "set"),
            Self::Delete => write!(f, "delete"),
            Self::List => write!(f, "list"),
            Self::Query => write!(f, "query"),
        }
    }
}

/// Route definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteDef {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Route type
    pub route_type: RouteType,
    /// Supported actions
    pub actions: Vec<RouteAction>,
    /// Target categories
    pub categories: Vec<SettingsCategory>,
    /// Priority
    pub priority: i32,
    /// Enabled
    pub enabled: bool,
}

impl RouteDef {
    /// Create new route
    pub fn new(id: impl Into<String>, route_type: RouteType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            route_type,
            actions: Vec::new(),
            categories: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Set name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add action
    pub fn action(mut self, action: RouteAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Enable/disable
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Supports action
    pub fn supports(&self, action: RouteAction) -> bool {
        self.actions.is_empty() || self.actions.contains(&action)
    }

    /// Applies to category
    pub fn applies_to(&self, category: SettingsCategory) -> bool {
        self.categories.is_empty() || self.categories.contains(&category)
    }
}

/// Route match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteMatch {
    /// Route ID
    pub route_id: String,
    /// Route type
    pub route_type: RouteType,
    /// Score
    pub score: i32,
}

impl RouteMatch {
    /// Create new match
    pub fn new(route_id: impl Into<String>, route_type: RouteType, score: i32) -> Self {
        Self {
            route_id: route_id.into(),
            route_type,
            score,
        }
    }
}

/// Route table
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteTable {
    /// Routes
    routes: Vec<RouteDef>,
}

impl RouteTable {
    /// Create new table
    pub fn new() -> Self {
        Self::default()
    }

    /// Add route
    pub fn add(&mut self, route: RouteDef) {
        self.routes.push(route);
        self.routes.sort_by_key(|r| r.priority);
    }

    /// Remove route
    pub fn remove(&mut self, id: &str) -> Option<RouteDef> {
        if let Some(pos) = self.routes.iter().position(|r| r.id == id) {
            Some(self.routes.remove(pos))
        } else {
            None
        }
    }

    /// Get route
    pub fn get(&self, id: &str) -> Option<&RouteDef> {
        self.routes.iter().find(|r| r.id == id)
    }

    /// Find matching routes
    pub fn find(&self, action: RouteAction, category: SettingsCategory) -> Vec<RouteMatch> {
        self.routes
            .iter()
            .filter(|r| r.enabled && r.supports(action) && r.applies_to(category))
            .map(|r| RouteMatch::new(&r.id, r.route_type, r.priority))
            .collect()
    }

    /// Best match
    pub fn best_match(&self, action: RouteAction, category: SettingsCategory) -> Option<RouteMatch> {
        self.find(action, category).into_iter().next()
    }

    /// Route count
    pub fn count(&self) -> usize {
        self.routes.len()
    }

    /// Enabled count
    pub fn enabled_count(&self) -> usize {
        self.routes.iter().filter(|r| r.enabled).count()
    }
}

/// Settings router
#[derive(Debug, Clone, Default)]
pub struct SettingsRouter {
    /// Route tables by name
    tables: HashMap<String, RouteTable>,
    /// Default table
    default_table: RouteTable,
    /// Route stats
    stats: RouteStats,
}

/// Route stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteStats {
    /// Total requests
    pub total: usize,
    /// By action
    pub by_action: HashMap<String, usize>,
    /// Cache hits
    pub cache_hits: usize,
    /// Redirects
    pub redirects: usize,
}

impl RouteStats {
    /// Record request
    pub fn record(&mut self, action: RouteAction, route_type: RouteType) {
        self.total += 1;
        *self.by_action.entry(action.to_string()).or_insert(0) += 1;
        match route_type {
            RouteType::Cached => self.cache_hits += 1,
            RouteType::Redirect => self.redirects += 1,
            _ => {}
        }
    }
}

impl SettingsRouter {
    /// Create new router
    pub fn new() -> Self {
        Self::default()
    }

    /// Add table
    pub fn add_table(&mut self, name: impl Into<String>, table: RouteTable) {
        self.tables.insert(name.into(), table);
    }

    /// Get table
    pub fn get_table(&self, name: &str) -> Option<&RouteTable> {
        self.tables.get(name)
    }

    /// Set default table
    pub fn set_default(&mut self, table: RouteTable) {
        self.default_table = table;
    }

    /// Route request
    pub fn route(&mut self, action: RouteAction, category: SettingsCategory) -> Option<RouteMatch> {
        let m = self.default_table.best_match(action, category);
        if let Some(ref match_result) = m {
            self.stats.record(action, match_result.route_type);
        }
        m
    }

    /// Get stats
    pub fn stats(&self) -> &RouteStats {
        &self.stats
    }

    /// Table count
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }
}

/// Format router
pub fn format_router(router: &SettingsRouter) -> String {
    let mut output = String::new();
    output.push_str("Settings Router:\n");
    output.push_str(&format!("  Tables: {}\n", router.table_count()));
    output.push_str(&format!("  Default routes: {}\n", router.default_table.count()));
    output.push_str(&format!("  Total requests: {}\n", router.stats.total));
    output
}

/// Check if query is about router
pub fn is_router_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("router")
        || lower.contains("route settings")
        || lower.contains("settings routing")
}

/// Fun fact about router
pub fn router_fun_fact() -> &'static str {
    "Anna uses smart routing to efficiently handle settings operations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_type_display() {
        assert_eq!(format!("{}", RouteType::Direct), "direct");
        assert_eq!(format!("{}", RouteType::Cached), "cached");
    }

    #[test]
    fn test_route_action_display() {
        assert_eq!(format!("{}", RouteAction::Get), "get");
        assert_eq!(format!("{}", RouteAction::Set), "set");
    }

    #[test]
    fn test_route_def_new() {
        let r = RouteDef::new("r1", RouteType::Direct);
        assert_eq!(r.id, "r1");
        assert!(r.enabled);
    }

    #[test]
    fn test_route_def_builder() {
        let r = RouteDef::new("r1", RouteType::Middleware)
            .name("Main")
            .action(RouteAction::Get)
            .category(SettingsCategory::Personality)
            .priority(50);
        assert!(r.supports(RouteAction::Get));
    }

    #[test]
    fn test_route_match_new() {
        let m = RouteMatch::new("r1", RouteType::Direct, 100);
        assert_eq!(m.score, 100);
    }

    #[test]
    fn test_table_new() {
        let t = RouteTable::new();
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn test_table_add_remove() {
        let mut t = RouteTable::new();
        t.add(RouteDef::new("r1", RouteType::Direct));
        assert_eq!(t.count(), 1);
        t.remove("r1");
        assert_eq!(t.count(), 0);
    }

    #[test]
    fn test_table_find() {
        let mut t = RouteTable::new();
        t.add(RouteDef::new("r1", RouteType::Direct).action(RouteAction::Get));
        let matches = t.find(RouteAction::Get, SettingsCategory::Personality);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_router_new() {
        let r = SettingsRouter::new();
        assert_eq!(r.table_count(), 0);
    }

    #[test]
    fn test_router_add_table() {
        let mut r = SettingsRouter::new();
        r.add_table("test", RouteTable::new());
        assert_eq!(r.table_count(), 1);
    }

    #[test]
    fn test_is_router_query() {
        assert!(is_router_query("settings routing"));
        assert!(!is_router_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = router_fun_fact();
        assert!(fact.contains("routing"));
    }
}
