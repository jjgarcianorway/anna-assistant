// v0.0.616: Settings Handler (Phase 192)
// Handle settings operations with routing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Handler type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HandlerType {
    /// Read handler
    Read,
    /// Write handler
    Write,
    /// Delete handler
    Delete,
    /// Validate handler
    Validate,
    /// Transform handler
    Transform,
    /// Notify handler
    Notify,
}

impl std::fmt::Display for HandlerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read => write!(f, "read"),
            Self::Write => write!(f, "write"),
            Self::Delete => write!(f, "delete"),
            Self::Validate => write!(f, "validate"),
            Self::Transform => write!(f, "transform"),
            Self::Notify => write!(f, "notify"),
        }
    }
}

/// Handler status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HandlerStatus {
    /// Ready
    #[default]
    Ready,
    /// Busy
    Busy,
    /// Disabled
    Disabled,
    /// Error
    Error,
}

impl std::fmt::Display for HandlerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ready => write!(f, "ready"),
            Self::Busy => write!(f, "busy"),
            Self::Disabled => write!(f, "disabled"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// Handler definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerDef {
    /// Unique ID
    pub id: String,
    /// Name
    pub name: String,
    /// Type
    pub handler_type: HandlerType,
    /// Categories
    pub categories: Vec<SettingsCategory>,
    /// Priority
    pub priority: u32,
    /// Enabled
    pub enabled: bool,
}

impl HandlerDef {
    /// Create new handler def
    pub fn new(id: impl Into<String>, handler_type: HandlerType) -> Self {
        Self {
            id: id.into(),
            name: String::new(),
            handler_type,
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

    /// Add category
    pub fn category(mut self, category: SettingsCategory) -> Self {
        self.categories.push(category);
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Handler instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerInstance {
    /// Definition ID
    pub def_id: String,
    /// Status
    pub status: HandlerStatus,
    /// Invocations
    pub invocations: usize,
    /// Errors
    pub errors: usize,
    /// Last invoked
    pub last_invoked: Option<u64>,
}

impl HandlerInstance {
    /// Create new instance
    pub fn new(def_id: impl Into<String>) -> Self {
        Self {
            def_id: def_id.into(),
            status: HandlerStatus::Ready,
            invocations: 0,
            errors: 0,
            last_invoked: None,
        }
    }

    /// Invoke
    pub fn invoke(&mut self, timestamp: u64) {
        self.status = HandlerStatus::Busy;
        self.invocations += 1;
        self.last_invoked = Some(timestamp);
    }

    /// Complete
    pub fn complete(&mut self) {
        self.status = HandlerStatus::Ready;
    }

    /// Error
    pub fn error(&mut self) {
        self.status = HandlerStatus::Error;
        self.errors += 1;
    }

    /// Disable
    pub fn disable(&mut self) {
        self.status = HandlerStatus::Disabled;
    }

    /// Enable
    pub fn enable(&mut self) {
        if self.status == HandlerStatus::Disabled {
            self.status = HandlerStatus::Ready;
        }
    }

    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.invocations == 0 {
            1.0
        } else {
            (self.invocations - self.errors) as f64 / self.invocations as f64
        }
    }
}

/// Settings handler registry
#[derive(Debug, Clone, Default)]
pub struct SettingsHandlerRegistry {
    /// Definitions
    definitions: HashMap<String, HandlerDef>,
    /// Instances
    instances: HashMap<String, HandlerInstance>,
}

impl SettingsHandlerRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register handler
    pub fn register(&mut self, def: HandlerDef) {
        let id = def.id.clone();
        self.definitions.insert(id.clone(), def);
        self.instances.insert(id.clone(), HandlerInstance::new(id));
    }

    /// Unregister handler
    pub fn unregister(&mut self, id: &str) {
        self.definitions.remove(id);
        self.instances.remove(id);
    }

    /// Get definition
    pub fn get_def(&self, id: &str) -> Option<&HandlerDef> {
        self.definitions.get(id)
    }

    /// Get instance
    pub fn get_instance(&self, id: &str) -> Option<&HandlerInstance> {
        self.instances.get(id)
    }

    /// Get instance mut
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut HandlerInstance> {
        self.instances.get_mut(id)
    }

    /// Find by type
    pub fn find_by_type(&self, handler_type: HandlerType) -> Vec<&HandlerDef> {
        self.definitions.values()
            .filter(|d| d.enabled && d.handler_type == handler_type)
            .collect()
    }

    /// Handler count
    pub fn count(&self) -> usize {
        self.definitions.len()
    }

    /// Ready count
    pub fn ready_count(&self) -> usize {
        self.instances.values()
            .filter(|i| i.status == HandlerStatus::Ready)
            .count()
    }
}

/// Format registry
pub fn format_handler_registry(registry: &SettingsHandlerRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Handler Registry:\n");
    output.push_str(&format!("  Handlers: {}\n", registry.count()));
    output.push_str(&format!("  Ready: {}\n", registry.ready_count()));
    output
}

/// Check if query is about handler
pub fn is_handler_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("handler")
        || lower.contains("handle settings")
        || lower.contains("settings handler")
}

/// Fun fact about handler
pub fn handler_fun_fact() -> &'static str {
    "Anna uses specialized handlers for different types of settings operations!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(format!("{}", HandlerType::Read), "read");
        assert_eq!(format!("{}", HandlerType::Write), "write");
    }

    #[test]
    fn test_status_display() {
        assert_eq!(format!("{}", HandlerStatus::Ready), "ready");
        assert_eq!(format!("{}", HandlerStatus::Busy), "busy");
    }

    #[test]
    fn test_def_new() {
        let d = HandlerDef::new("h1", HandlerType::Read);
        assert!(d.enabled);
    }

    #[test]
    fn test_def_builder() {
        let d = HandlerDef::new("h1", HandlerType::Write)
            .name("Writer")
            .priority(10);
        assert_eq!(d.priority, 10);
    }

    #[test]
    fn test_instance_new() {
        let i = HandlerInstance::new("h1");
        assert_eq!(i.status, HandlerStatus::Ready);
    }

    #[test]
    fn test_instance_lifecycle() {
        let mut i = HandlerInstance::new("h1");
        i.invoke(100);
        assert_eq!(i.status, HandlerStatus::Busy);
        i.complete();
        assert_eq!(i.status, HandlerStatus::Ready);
    }

    #[test]
    fn test_instance_success_rate() {
        let mut i = HandlerInstance::new("h1");
        i.invocations = 10;
        i.errors = 2;
        assert!((i.success_rate() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_registry_new() {
        let r = SettingsHandlerRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = SettingsHandlerRegistry::new();
        r.register(HandlerDef::new("h1", HandlerType::Read));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_registry_find_by_type() {
        let mut r = SettingsHandlerRegistry::new();
        r.register(HandlerDef::new("h1", HandlerType::Read));
        r.register(HandlerDef::new("h2", HandlerType::Write));
        assert_eq!(r.find_by_type(HandlerType::Read).len(), 1);
    }

    #[test]
    fn test_is_handler_query() {
        assert!(is_handler_query("settings handler"));
        assert!(!is_handler_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = handler_fun_fact();
        assert!(fact.contains("handler"));
    }
}
