// v0.0.718: Settings Directive (Phase 294)
// Authoritative directives for settings management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Directive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DirectiveType {
    /// Mandatory directive
    #[default]
    Mandatory,
    /// Recommended directive
    Recommended,
    /// Advisory directive
    Advisory,
    /// Optional directive
    Optional,
}

impl std::fmt::Display for DirectiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mandatory => write!(f, "mandatory"),
            Self::Recommended => write!(f, "recommended"),
            Self::Advisory => write!(f, "advisory"),
            Self::Optional => write!(f, "optional"),
        }
    }
}

/// Directive authority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DirectiveAuthority {
    /// System authority
    #[default]
    System,
    /// Admin authority
    Admin,
    /// Policy authority
    Policy,
    /// Executive authority
    Executive,
}

impl std::fmt::Display for DirectiveAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::System => write!(f, "system"),
            Self::Admin => write!(f, "admin"),
            Self::Policy => write!(f, "policy"),
            Self::Executive => write!(f, "executive"),
        }
    }
}

/// Directive config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveConfig {
    /// Name
    pub name: String,
    /// Directive type
    pub directive_type: DirectiveType,
    /// Authority
    pub authority: DirectiveAuthority,
    /// Max directives
    pub max_directives: usize,
}

impl DirectiveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            directive_type: DirectiveType::Mandatory,
            authority: DirectiveAuthority::System,
            max_directives: 150,
        }
    }

    /// Set type
    pub fn directive_type(mut self, dt: DirectiveType) -> Self {
        self.directive_type = dt;
        self
    }

    /// Set authority
    pub fn authority(mut self, a: DirectiveAuthority) -> Self {
        self.authority = a;
        self
    }

    /// Set max directives
    pub fn max_directives(mut self, max: usize) -> Self {
        self.max_directives = max;
        self
    }
}

impl Default for DirectiveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Directive order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveOrder {
    /// Order ID
    pub id: String,
    /// Title
    pub title: String,
    /// Instructions
    pub instructions: String,
    /// Authority
    pub authority: DirectiveAuthority,
    /// Enforced
    pub enforced: bool,
}

impl DirectiveOrder {
    /// Create new order
    pub fn new(id: impl Into<String>, title: impl Into<String>, instructions: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            instructions: instructions.into(),
            authority: DirectiveAuthority::System,
            enforced: false,
        }
    }

    /// Set authority
    pub fn authority(mut self, a: DirectiveAuthority) -> Self {
        self.authority = a;
        self
    }

    /// Enforce directive
    pub fn enforce(&mut self) {
        self.enforced = true;
    }
}

/// Directive supplement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveSupplement {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Order ID
    pub order_id: String,
}

impl DirectiveSupplement {
    /// Create new supplement
    pub fn new(key: impl Into<String>, value: impl Into<String>, order_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            order_id: order_id.into(),
        }
    }
}

/// Directive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DirectiveStats {
    /// Total directives
    pub total_directives: usize,
    /// Enforced directives
    pub enforced: usize,
    /// Mandatory count
    pub mandatory_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl DirectiveStats {
    /// Update from orders
    pub fn update(&mut self, orders: &[DirectiveOrder], directive_type: DirectiveType) {
        self.total_directives = orders.len();
        self.enforced = orders.iter().filter(|o| o.enforced).count();
        if directive_type == DirectiveType::Mandatory {
            self.mandatory_count = orders.len();
        }
        *self.by_type.entry(directive_type.to_string()).or_insert(0) += 1;
    }

    /// Enforcement rate
    pub fn enforcement_rate(&self) -> f64 {
        if self.total_directives == 0 { 0.0 } else { self.enforced as f64 / self.total_directives as f64 * 100.0 }
    }
}

/// Settings directive
#[derive(Debug, Clone, Default)]
pub struct SettingsDirective {
    /// Config
    config: DirectiveConfig,
    /// Orders
    orders: Vec<DirectiveOrder>,
    /// Supplements
    supplements: Vec<DirectiveSupplement>,
    /// Stats
    stats: DirectiveStats,
}

impl SettingsDirective {
    /// Create new directive system
    pub fn new(config: DirectiveConfig) -> Self {
        Self {
            config,
            orders: Vec::new(),
            supplements: Vec::new(),
            stats: DirectiveStats::default(),
        }
    }

    /// Add order
    pub fn add_order(&mut self, order: DirectiveOrder) -> bool {
        if self.orders.len() >= self.config.max_directives {
            return false;
        }
        self.orders.push(order);
        self.update_stats();
        true
    }

    /// Get order
    pub fn get_order(&self, id: &str) -> Option<&DirectiveOrder> {
        self.orders.iter().find(|o| o.id == id)
    }

    /// Get order mut
    pub fn get_order_mut(&mut self, id: &str) -> Option<&mut DirectiveOrder> {
        self.orders.iter_mut().find(|o| o.id == id)
    }

    /// Add supplement
    pub fn add_supplement(&mut self, supplement: DirectiveSupplement) {
        self.supplements.push(supplement);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.orders, self.config.directive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &DirectiveStats {
        &self.stats
    }

    /// Order count
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }
}

/// Directive registry
#[derive(Debug, Clone, Default)]
pub struct DirectiveRegistry {
    /// Directives by ID
    directives: HashMap<String, SettingsDirective>,
}

impl DirectiveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register directive
    pub fn register(&mut self, id: impl Into<String>, directive: SettingsDirective) {
        self.directives.insert(id.into(), directive);
    }

    /// Unregister directive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.directives.remove(id).is_some()
    }

    /// Get directive
    pub fn get(&self, id: &str) -> Option<&SettingsDirective> {
        self.directives.get(id)
    }

    /// Get directive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsDirective> {
        self.directives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.directives.len()
    }
}

/// Format directive registry
pub fn format_directive_registry(registry: &DirectiveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Directive Registry:\n");
    output.push_str(&format!("  Directives: {}\n", registry.count()));
    output
}

/// Check if query is about directive
pub fn is_directive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings directive") || lower.contains("directive settings") || lower.contains("mandatory directive")
}

/// Fun fact about directive
pub fn directive_fun_fact() -> &'static str {
    "Anna's settings directive issues authoritative orders for configuration management!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directive_type_display() {
        assert_eq!(format!("{}", DirectiveType::Mandatory), "mandatory");
        assert_eq!(format!("{}", DirectiveType::Optional), "optional");
    }

    #[test]
    fn test_authority_display() {
        assert_eq!(format!("{}", DirectiveAuthority::System), "system");
        assert_eq!(format!("{}", DirectiveAuthority::Executive), "executive");
    }

    #[test]
    fn test_config_new() {
        let c = DirectiveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = DirectiveConfig::new("test")
            .directive_type(DirectiveType::Recommended)
            .authority(DirectiveAuthority::Admin);
        assert_eq!(c.directive_type, DirectiveType::Recommended);
        assert_eq!(c.authority, DirectiveAuthority::Admin);
    }

    #[test]
    fn test_order_new() {
        let o = DirectiveOrder::new("o1", "Title", "Instructions");
        assert_eq!(o.id, "o1");
    }

    #[test]
    fn test_order_builder() {
        let o = DirectiveOrder::new("o1", "Title", "Instructions")
            .authority(DirectiveAuthority::Policy);
        assert_eq!(o.authority, DirectiveAuthority::Policy);
    }

    #[test]
    fn test_order_enforce() {
        let mut o = DirectiveOrder::new("o1", "Title", "Instructions");
        o.enforce();
        assert!(o.enforced);
    }

    #[test]
    fn test_supplement_new() {
        let s = DirectiveSupplement::new("key", "value", "o1");
        assert_eq!(s.order_id, "o1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = DirectiveStats::default();
        let mut order = DirectiveOrder::new("o1", "Title", "Instructions");
        order.enforce();
        s.update(&[order], DirectiveType::Mandatory);
        assert_eq!(s.total_directives, 1);
        assert_eq!(s.enforced, 1);
        assert_eq!(s.mandatory_count, 1);
    }

    #[test]
    fn test_directive_new() {
        let d = SettingsDirective::new(DirectiveConfig::default());
        assert_eq!(d.order_count(), 0);
    }

    #[test]
    fn test_directive_add_order() {
        let mut d = SettingsDirective::new(DirectiveConfig::default());
        d.add_order(DirectiveOrder::new("o1", "Title", "Instructions"));
        assert_eq!(d.order_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = DirectiveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = DirectiveRegistry::new();
        r.register("d1", SettingsDirective::new(DirectiveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_directive_query() {
        assert!(is_directive_query("settings directive"));
        assert!(!is_directive_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = directive_fun_fact();
        assert!(fact.contains("directive"));
    }
}
