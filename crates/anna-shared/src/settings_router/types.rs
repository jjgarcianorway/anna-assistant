// v0.0.603: Settings Router Types (Phase 179)
// Type definitions for routing logic

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
