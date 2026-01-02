// v0.0.603: Settings Router Routing Logic (Phase 179)
// Route table and router implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;
use super::types::{RouteAction, RouteDef, RouteMatch, RouteStats, RouteType};

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
    pub(crate) default_table: RouteTable,
    /// Route stats
    stats: RouteStats,
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
