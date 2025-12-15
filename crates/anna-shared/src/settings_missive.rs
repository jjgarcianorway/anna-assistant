// v0.0.716: Settings Missive (Phase 292)
// Formal letters about settings changes

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Missive type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MissiveType {
    /// Formal missive
    #[default]
    Formal,
    /// Informal missive
    Informal,
    /// Personal missive
    Personal,
    /// Business missive
    Business,
}

impl std::fmt::Display for MissiveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Formal => write!(f, "formal"),
            Self::Informal => write!(f, "informal"),
            Self::Personal => write!(f, "personal"),
            Self::Business => write!(f, "business"),
        }
    }
}

/// Missive delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MissiveDelivery {
    /// Standard delivery
    #[default]
    Standard,
    /// Express delivery
    Express,
    /// Priority delivery
    Priority,
    /// Certified delivery
    Certified,
}

impl std::fmt::Display for MissiveDelivery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Express => write!(f, "express"),
            Self::Priority => write!(f, "priority"),
            Self::Certified => write!(f, "certified"),
        }
    }
}

/// Missive config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveConfig {
    /// Name
    pub name: String,
    /// Missive type
    pub missive_type: MissiveType,
    /// Delivery method
    pub delivery: MissiveDelivery,
    /// Max missives
    pub max_missives: usize,
}

impl MissiveConfig {
    /// Create new config
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            missive_type: MissiveType::Formal,
            delivery: MissiveDelivery::Standard,
            max_missives: 250,
        }
    }

    /// Set type
    pub fn missive_type(mut self, mt: MissiveType) -> Self {
        self.missive_type = mt;
        self
    }

    /// Set delivery
    pub fn delivery(mut self, d: MissiveDelivery) -> Self {
        self.delivery = d;
        self
    }

    /// Set max missives
    pub fn max_missives(mut self, max: usize) -> Self {
        self.max_missives = max;
        self
    }
}

impl Default for MissiveConfig {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Missive letter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveLetter {
    /// Letter ID
    pub id: String,
    /// Subject
    pub subject: String,
    /// Content
    pub content: String,
    /// From
    pub from: String,
    /// To
    pub to: String,
    /// Delivered
    pub delivered: bool,
}

impl MissiveLetter {
    /// Create new letter
    pub fn new(id: impl Into<String>, subject: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            content: content.into(),
            from: String::new(),
            to: String::new(),
            delivered: false,
        }
    }

    /// Set from
    pub fn from(mut self, f: impl Into<String>) -> Self {
        self.from = f.into();
        self
    }

    /// Set to
    pub fn to(mut self, t: impl Into<String>) -> Self {
        self.to = t.into();
        self
    }

    /// Mark delivered
    pub fn deliver(&mut self) {
        self.delivered = true;
    }
}

/// Missive enclosure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissiveEnclosure {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Letter ID
    pub letter_id: String,
}

impl MissiveEnclosure {
    /// Create new enclosure
    pub fn new(key: impl Into<String>, value: impl Into<String>, letter_id: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            letter_id: letter_id.into(),
        }
    }
}

/// Missive stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MissiveStats {
    /// Total missives
    pub total_missives: usize,
    /// Delivered missives
    pub delivered: usize,
    /// Priority missives
    pub priority_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl MissiveStats {
    /// Update from letters
    pub fn update(&mut self, letters: &[MissiveLetter], missive_type: MissiveType) {
        self.total_missives = letters.len();
        self.delivered = letters.iter().filter(|l| l.delivered).count();
        *self.by_type.entry(missive_type.to_string()).or_insert(0) += 1;
    }

    /// Delivery rate
    pub fn delivery_rate(&self) -> f64 {
        if self.total_missives == 0 { 0.0 } else { self.delivered as f64 / self.total_missives as f64 * 100.0 }
    }
}

/// Settings missive
#[derive(Debug, Clone, Default)]
pub struct SettingsMissive {
    /// Config
    config: MissiveConfig,
    /// Letters
    letters: Vec<MissiveLetter>,
    /// Enclosures
    enclosures: Vec<MissiveEnclosure>,
    /// Stats
    stats: MissiveStats,
}

impl SettingsMissive {
    /// Create new missive system
    pub fn new(config: MissiveConfig) -> Self {
        Self {
            config,
            letters: Vec::new(),
            enclosures: Vec::new(),
            stats: MissiveStats::default(),
        }
    }

    /// Add letter
    pub fn add_letter(&mut self, letter: MissiveLetter) -> bool {
        if self.letters.len() >= self.config.max_missives {
            return false;
        }
        self.letters.push(letter);
        self.update_stats();
        true
    }

    /// Get letter
    pub fn get_letter(&self, id: &str) -> Option<&MissiveLetter> {
        self.letters.iter().find(|l| l.id == id)
    }

    /// Get letter mut
    pub fn get_letter_mut(&mut self, id: &str) -> Option<&mut MissiveLetter> {
        self.letters.iter_mut().find(|l| l.id == id)
    }

    /// Add enclosure
    pub fn add_enclosure(&mut self, enclosure: MissiveEnclosure) {
        self.enclosures.push(enclosure);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.letters, self.config.missive_type);
    }

    /// Get stats
    pub fn stats(&self) -> &MissiveStats {
        &self.stats
    }

    /// Letter count
    pub fn letter_count(&self) -> usize {
        self.letters.len()
    }
}

/// Missive registry
#[derive(Debug, Clone, Default)]
pub struct MissiveRegistry {
    /// Missives by ID
    missives: HashMap<String, SettingsMissive>,
}

impl MissiveRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register missive
    pub fn register(&mut self, id: impl Into<String>, missive: SettingsMissive) {
        self.missives.insert(id.into(), missive);
    }

    /// Unregister missive
    pub fn unregister(&mut self, id: &str) -> bool {
        self.missives.remove(id).is_some()
    }

    /// Get missive
    pub fn get(&self, id: &str) -> Option<&SettingsMissive> {
        self.missives.get(id)
    }

    /// Get missive mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMissive> {
        self.missives.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.missives.len()
    }
}

/// Format missive registry
pub fn format_missive_registry(registry: &MissiveRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Missive Registry:\n");
    output.push_str(&format!("  Missives: {}\n", registry.count()));
    output
}

/// Check if query is about missive
pub fn is_missive_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings missive") || lower.contains("missive settings") || lower.contains("formal letter")
}

/// Fun fact about missive
pub fn missive_fun_fact() -> &'static str {
    "Anna's settings missive delivers formal letters about configuration changes!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missive_type_display() {
        assert_eq!(format!("{}", MissiveType::Formal), "formal");
        assert_eq!(format!("{}", MissiveType::Business), "business");
    }

    #[test]
    fn test_delivery_display() {
        assert_eq!(format!("{}", MissiveDelivery::Standard), "standard");
        assert_eq!(format!("{}", MissiveDelivery::Certified), "certified");
    }

    #[test]
    fn test_config_new() {
        let c = MissiveConfig::new("test");
        assert_eq!(c.name, "test");
    }

    #[test]
    fn test_config_builder() {
        let c = MissiveConfig::new("test")
            .missive_type(MissiveType::Personal)
            .delivery(MissiveDelivery::Express);
        assert_eq!(c.missive_type, MissiveType::Personal);
        assert_eq!(c.delivery, MissiveDelivery::Express);
    }

    #[test]
    fn test_letter_new() {
        let l = MissiveLetter::new("l1", "Subject", "Content");
        assert_eq!(l.id, "l1");
    }

    #[test]
    fn test_letter_builder() {
        let l = MissiveLetter::new("l1", "Subject", "Content")
            .from("sender")
            .to("recipient");
        assert_eq!(l.from, "sender");
        assert_eq!(l.to, "recipient");
    }

    #[test]
    fn test_letter_deliver() {
        let mut l = MissiveLetter::new("l1", "Subject", "Content");
        l.deliver();
        assert!(l.delivered);
    }

    #[test]
    fn test_enclosure_new() {
        let e = MissiveEnclosure::new("key", "value", "l1");
        assert_eq!(e.letter_id, "l1");
    }

    #[test]
    fn test_stats_update() {
        let mut s = MissiveStats::default();
        let mut letter = MissiveLetter::new("l1", "Subject", "Content");
        letter.deliver();
        s.update(&[letter], MissiveType::Formal);
        assert_eq!(s.total_missives, 1);
        assert_eq!(s.delivered, 1);
    }

    #[test]
    fn test_missive_new() {
        let m = SettingsMissive::new(MissiveConfig::default());
        assert_eq!(m.letter_count(), 0);
    }

    #[test]
    fn test_missive_add_letter() {
        let mut m = SettingsMissive::new(MissiveConfig::default());
        m.add_letter(MissiveLetter::new("l1", "Subject", "Content"));
        assert_eq!(m.letter_count(), 1);
    }

    #[test]
    fn test_registry_new() {
        let r = MissiveRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MissiveRegistry::new();
        r.register("m1", SettingsMissive::new(MissiveConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_missive_query() {
        assert!(is_missive_query("settings missive"));
        assert!(!is_missive_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = missive_fun_fact();
        assert!(fact.contains("missive"));
    }
}
