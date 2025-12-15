// v0.0.717: Settings Circular (Phase 293)
// Circular notices distributed to all

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Circular type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CircularType {
    /// Policy circular
    #[default]
    Policy,
    /// Information circular
    Information,
    /// Directive circular
    Directive,
    /// Advisory circular
    Advisory,
}

impl std::fmt::Display for CircularType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Policy => write!(f, "policy"),
            Self::Information => write!(f, "information"),
            Self::Directive => write!(f, "directive"),
            Self::Advisory => write!(f, "advisory"),
        }
    }
}

/// Circular scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CircularScope {
    /// All scope
    #[default]
    All,
    /// Department scope
    Department,
    /// Team scope
    Team,
    /// Individual scope
    Individual,
}

impl std::fmt::Display for CircularScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Department => write!(f, "department"),
            Self::Team => write!(f, "team"),
            Self::Individual => write!(f, "individual"),
        }
    }
}

/// Circular config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularConfig {
    /// Name
    pub name: String,
    /// Circular type
    pub circular_type: CircularType,
    /// Scope
    pub scope: CircularScope,
    /// Max circulars
    pub max_circulars: usize,
}

impl CircularConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            circular_type: CircularType::Policy,
            scope: CircularScope::All,
            max_circulars: 200,
        }
    }

    /// Set type
    pub fn circular_type(mut self, ct: CircularType) -> Self {
        self.circular_type = ct;
        self
    }

    /// Set scope
    pub fn scope(mut self, s: CircularScope) -> Self {
        self.scope = s;
        self
    }

    /// Set max circulars
    pub fn max_circulars(mut self, max: usize) -> Self {
        self.max_circulars = max;
        self
    }
}

impl Default for CircularConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Circular notice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularNotice {
    /// Notice ID
    pub id: String,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Reference number
    pub reference: String,
    /// Effective date
    pub effective_date: String,
    /// Active
    pub active: bool,
}

impl CircularNotice {
    /// Create new notice
    pub fn new(id: impl Into<String>, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            content: content.into(),
            reference: String::new(),
            effective_date: String::new(),
            active: true,
        }
    }

    /// Set reference
    pub fn reference(mut self, r: impl Into<String>) -> Self {
        self.reference = r.into();
        self
    }

    /// Set effective date
    pub fn effective_date(mut self, d: impl Into<String>) -> Self {
        self.effective_date = d.into();
        self
    }

    /// Deactivate circular
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Circular attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularAttachment {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Notice ID
    pub notice_id: String,
}

impl CircularAttachment {
    /// Create new attachment
    pub fn new(key: impl Into<String>, value: impl Into<String>, notice_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            notice_id: notice_id.into(),
        }
    }
}

/// Circular stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CircularStats {
    /// Total circulars
    pub total_circulars: usize,
    /// Active circulars
    pub active: usize,
    /// Policy circulars
    pub policy_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl CircularStats {
    /// Update from notices
    pub fn update(&mut self, notices: &[CircularNotice], circular_type: CircularType) {
        self.total_circulars = notices.len();
        self.active = notices.iter().filter(|n| n.active).count();
        if circular_type == CircularType::Policy {
            self.policy_count = notices.len();
        }
        *self.by_type.entry(circular_type.to_string()).or_insert(0) += 1;
    }

    /// Active rate
    pub fn active_rate(&self) -> f64 {
        if self.total_circulars == 0 { 0.0 } else { self.active as f64 / self.total_circulars as f64 * 100.0 }
    }
}

/// Settings circular
#[derive(Debug, Clone, Default)]
pub struct SettingsCircular {
    /// Config
    config: CircularConfig,
    /// Notices
    notices: Vec<CircularNotice>,
    /// Attachments
    attachments: Vec<CircularAttachment>,
    /// Stats
    stats: CircularStats,
}

impl SettingsCircular {
    /// Create new circular system
    pub fn new(config: CircularConfig) -> Self {
        Self {
            config,
            notices: Vec::new(),
            attachments: Vec::new(),
            stats: CircularStats::default(),
        }
    }

    /// Add notice
    pub fn add_notice(&mut self, notice: CircularNotice) -> bool {
        if self.notices.len() >= self.config.max_circulars {
            return false;
        }
        self.notices.push(notice);
        self.update_stats();
        true
    }

    /// Get notice
    pub fn get_notice(&self, id: &str) -> Option<&CircularNotice> {
        self.notices.iter().find(|n| n.id == id)
    }

    /// Get notice mut
    pub fn get_notice_mut(&mut self, id: &str) -> Option<&mut CircularNotice> {
        self.notices.iter_mut().find(|n| n.id == id)
    }

    /// Add attachment
    pub fn add_attachment(&mut self, attachment: CircularAttachment) {
        self.attachments.push(attachment);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.notices, self.config.circular_type);
    }

    /// Get stats
    pub fn stats(&self) -> &CircularStats {
        &self.stats
    }

    /// Notice count
    pub fn notice_count(&self) -> usize {
        self.notices.len()
    }
}

/// Circular registry
#[derive(Debug, Clone, Default)]
pub struct CircularRegistry {
    /// Circulars by ID
    circulars: HashMap<String, SettingsCircular>,
}

impl CircularRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register circular
    pub fn register(&mut self, id: impl Into<String>, circular: SettingsCircular) {
        self.circulars.insert(id.into(), circular);
    }

    /// Unregister circular
    pub fn unregister(&mut self, id: &str) -> bool {
        self.circulars.remove(id).is_some()
    }

    /// Get circular
    pub fn get(&self, id: &str) -> Option<&SettingsCircular> {
        self.circulars.get(id)
    }

    /// Get circular mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsCircular> {
        self.circulars.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.circulars.len()
    }
}

/// Format circular registry
pub fn format_circular_registry(registry: &CircularRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Circular Registry:\n");
    output.push_str(&format!("  Circulars: {}\n", registry.count()));
    output
}

/// Check if query is about circular
pub fn is_circular_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings circular") || lower.contains("circular settings") || lower.contains("policy circular")
}

/// Fun fact about circular
pub fn circular_fun_fact() -> &'static str {
    "Anna's settings circular distributes policy notices to all configuration stakeholders!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circular_type_display() {
        assert_eq!(format!("{}", CircularType::Policy), "policy");
        assert_eq!(format!("{}", CircularType::Advisory), "advisory");
    }

    #[test]
    fn test_scope_display() {
        assert_eq!(format!("{}", CircularScope::All), "all");
        assert_eq!(format!("{}", CircularScope::Team), "team");
    }

    #[test]
    fn test_config_new() {
        let c = CircularConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = CircularConfig::new("test")
            .circular_type(CircularType::Directive)
            .scope(CircularScope::Department);
        assert_eq!(c.circular_type, CircularType::Directive);
        assert_eq!(c.scope, CircularScope::Department);
    }

    #[test]
    fn test_notice_new() {
        let n = CircularNotice::new("n1", "Title", "Content");
        assert_eq!(n.id, "n1");
    }

    #[test]
    fn test_notice_builder() {
        let n = CircularNotice::new("n1", "Title", "Content")
            .reference("REF-001")
            .effective_date("2025-01-01");
        assert_eq!(n.reference, "REF-001");
        assert_eq!(n.effective_date, "2025-01-01");
    }

    #[test]
    fn test_notice_deactivate() {
        let mut n = CircularNotice::new("n1", "Title", "Content");
        n.deactivate();
        assert!(!n.active);
    }

    #[test]
    fn test_attachment_new() {
        let a = CircularAttachment::new("key", "value", "n1");
        assert_eq!(a.notice_id, "n1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = CircularStats::default();
        let notice = CircularNotice::new("n1", "Title", "Content");
        s.update(&[notice], CircularType::Policy);
        assert_eq!(s.total_circulars, 1);
        assert_eq!(s.active, 1);
        assert_eq!(s.policy_count, 1);
    }

    #[test]
    fn test_circular_new() {
        let c = SettingsCircular::new(CircularConfig::default());
        assert_eq!(c.notice_count(), 0);
    }

    #[test]
    fn test_circular_add_notice() {
        let mut c = SettingsCircular::new(CircularConfig::default());
        c.add_notice(CircularNotice::new("n1", "Title", "Content"));
        assert_eq!(c.notice_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = CircularRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = CircularRegistry::new();
        r.register("c1", SettingsCircular::new(CircularConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_circular_query() {
        assert!(is_circular_query("settings circular"));
        assert!(!is_circular_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = circular_fun_fact();
        assert!(fact.contains("circular"));
    }
}
