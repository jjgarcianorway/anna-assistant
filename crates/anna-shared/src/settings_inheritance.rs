// v0.0.595: Settings Inheritance (Phase 171)
// Inherit settings from parent profiles

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::unified_settings::SettingsCategory;

/// Inheritance mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InheritanceMode {
    /// Inherit all from parent
    #[default]
    Full,
    /// Inherit only specified
    Selective,
    /// Override parent values
    Override,
    /// Merge with parent
    Merge,
}

impl std::fmt::Display for InheritanceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Selective => write!(f, "selective"),
            Self::Override => write!(f, "override"),
            Self::Merge => write!(f, "merge"),
        }
    }
}

/// Inheritance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceRule {
    /// Category
    pub category: SettingsCategory,
    /// Mode
    pub mode: InheritanceMode,
    /// Include keys (for selective)
    pub include: Vec<String>,
    /// Exclude keys
    pub exclude: Vec<String>,
}

impl InheritanceRule {
    /// Create new rule
    pub fn new(category: SettingsCategory, mode: InheritanceMode) -> Self {
        Self {
            category,
            mode,
            include: Vec::new(),
            exclude: Vec::new(),
        }
    }

    /// Add include key
    pub fn include(mut self, key: impl Into<String>) -> Self {
        self.include.push(key.into());
        self
    }

    /// Add exclude key
    pub fn exclude(mut self, key: impl Into<String>) -> Self {
        self.exclude.push(key.into());
        self
    }

    /// Check if key should be inherited
    pub fn should_inherit(&self, key: &str) -> bool {
        if self.exclude.contains(&key.to_string()) {
            return false;
        }
        if !self.include.is_empty() {
            return self.include.contains(&key.to_string());
        }
        true
    }
}

/// Inheritance chain entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceEntry {
    /// Entry ID
    pub id: String,
    /// Name
    pub name: String,
    /// Parent ID
    pub parent_id: Option<String>,
    /// Rules
    pub rules: Vec<InheritanceRule>,
    /// Priority (higher = applied later)
    pub priority: u32,
    /// Active
    pub active: bool,
}

impl InheritanceEntry {
    /// Create new entry
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            parent_id: None,
            rules: Vec::new(),
            priority: 0,
            active: true,
        }
    }

    /// Set parent
    pub fn parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Add rule
    pub fn add_rule(&mut self, rule: InheritanceRule) {
        self.rules.push(rule);
    }

    /// Set priority
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Get rule for category
    pub fn rule_for(&self, category: SettingsCategory) -> Option<&InheritanceRule> {
        self.rules.iter().find(|r| r.category == category)
    }

    /// Has parent
    pub fn has_parent(&self) -> bool {
        self.parent_id.is_some()
    }
}

/// Inheritance manager
#[derive(Debug, Clone, Default)]
pub struct InheritanceManager {
    /// Entries
    entries: HashMap<String, InheritanceEntry>,
    /// Root entry ID
    root_id: Option<String>,
}

impl InheritanceManager {
    /// Create new manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entry
    pub fn add(&mut self, entry: InheritanceEntry) -> String {
        let id = entry.id.clone();
        if self.root_id.is_none() && entry.parent_id.is_none() {
            self.root_id = Some(id.clone());
        }
        self.entries.insert(id.clone(), entry);
        id
    }

    /// Get entry
    pub fn get(&self, id: &str) -> Option<&InheritanceEntry> {
        self.entries.get(id)
    }

    /// Get mutable entry
    pub fn get_mut(&mut self, id: &str) -> Option<&mut InheritanceEntry> {
        self.entries.get_mut(id)
    }

    /// Remove entry
    pub fn remove(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Get inheritance chain
    pub fn chain(&self, id: &str) -> Vec<&InheritanceEntry> {
        let mut chain = Vec::new();
        let mut current = self.get(id);

        while let Some(entry) = current {
            chain.push(entry);
            current = entry.parent_id.as_ref().and_then(|pid| self.get(pid));
        }

        chain.reverse();
        chain
    }

    /// Get children of entry
    pub fn children(&self, id: &str) -> Vec<&InheritanceEntry> {
        self.entries.values()
            .filter(|e| e.parent_id.as_deref() == Some(id))
            .collect()
    }

    /// Get root entries
    pub fn roots(&self) -> Vec<&InheritanceEntry> {
        self.entries.values()
            .filter(|e| e.parent_id.is_none())
            .collect()
    }

    /// Entry count
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Depth of entry
    pub fn depth(&self, id: &str) -> usize {
        self.chain(id).len()
    }

    /// Clear all
    pub fn clear(&mut self) {
        self.entries.clear();
        self.root_id = None;
    }
}

/// Format inheritance
pub fn format_inheritance(manager: &InheritanceManager) -> String {
    let mut output = String::new();

    output.push_str("=== Settings Inheritance ===\n\n");
    output.push_str(&format!("Entries: {}\n\n", manager.count()));

    for entry in manager.roots() {
        output.push_str(&format!("{} (root)\n", entry.name));
        for child in manager.children(&entry.id) {
            output.push_str(&format!("  └── {}\n", child.name));
        }
    }

    output
}

/// Check if query is about inheritance
pub fn is_inheritance_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("inherit")
        || lower.contains("parent")
        || lower.contains("extend")
        || lower.contains("derive")
}

/// Fun fact about inheritance
pub fn settings_inheritance_fun_fact() -> &'static str {
    "Anna settings can inherit from parent profiles for easy customization!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(format!("{}", InheritanceMode::Full), "full");
        assert_eq!(format!("{}", InheritanceMode::Merge), "merge");
    }

    #[test]
    fn test_rule_new() {
        let rule = InheritanceRule::new(SettingsCategory::Personality, InheritanceMode::Full);
        assert_eq!(rule.mode, InheritanceMode::Full);
    }

    #[test]
    fn test_rule_should_inherit() {
        let rule = InheritanceRule::new(SettingsCategory::Personality, InheritanceMode::Selective)
            .include("formality")
            .exclude("humor");
        assert!(rule.should_inherit("formality"));
        assert!(!rule.should_inherit("humor"));
    }

    #[test]
    fn test_entry_new() {
        let entry = InheritanceEntry::new("test");
        assert_eq!(entry.name, "test");
        assert!(!entry.has_parent());
    }

    #[test]
    fn test_entry_parent() {
        let entry = InheritanceEntry::new("child").parent("parent-id");
        assert!(entry.has_parent());
    }

    #[test]
    fn test_manager_new() {
        let manager = InheritanceManager::new();
        assert_eq!(manager.count(), 0);
    }

    #[test]
    fn test_manager_add() {
        let mut manager = InheritanceManager::new();
        manager.add(InheritanceEntry::new("root"));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_manager_chain() {
        let mut manager = InheritanceManager::new();
        let root_id = manager.add(InheritanceEntry::new("root"));
        let child_id = manager.add(InheritanceEntry::new("child").parent(&root_id));
        let chain = manager.chain(&child_id);
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_manager_children() {
        let mut manager = InheritanceManager::new();
        let root_id = manager.add(InheritanceEntry::new("root"));
        manager.add(InheritanceEntry::new("child1").parent(&root_id));
        manager.add(InheritanceEntry::new("child2").parent(&root_id));
        let children = manager.children(&root_id);
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_manager_depth() {
        let mut manager = InheritanceManager::new();
        let root_id = manager.add(InheritanceEntry::new("root"));
        let child_id = manager.add(InheritanceEntry::new("child").parent(&root_id));
        assert_eq!(manager.depth(&root_id), 1);
        assert_eq!(manager.depth(&child_id), 2);
    }

    #[test]
    fn test_format_inheritance() {
        let manager = InheritanceManager::new();
        let output = format_inheritance(&manager);
        assert!(output.contains("Inheritance"));
    }

    #[test]
    fn test_is_inheritance_query() {
        assert!(is_inheritance_query("inherit from parent"));
        assert!(is_inheritance_query("extend profile"));
        assert!(!is_inheritance_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = settings_inheritance_fun_fact();
        assert!(fact.contains("inherit"));
    }
}
