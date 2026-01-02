// v0.0.708: Settings Memo (Phase 284)
// Memo registry and utilities

use std::collections::HashMap;
use super::memo::SettingsMemo;

/// Memo registry
#[derive(Debug, Clone, Default)]
pub struct MemoRegistry {
    /// Memos by ID
    memos: HashMap<String, SettingsMemo>,
}

impl MemoRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register memo
    pub fn register(&mut self, id: impl Into<String>, memo: SettingsMemo) {
        self.memos.insert(id.into(), memo);
    }

    /// Unregister memo
    pub fn unregister(&mut self, id: &str) -> bool {
        self.memos.remove(id).is_some()
    }

    /// Get memo
    pub fn get(&self, id: &str) -> Option<&SettingsMemo> {
        self.memos.get(id)
    }

    /// Get memo mut
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SettingsMemo> {
        self.memos.get_mut(id)
    }

    /// Count
    pub fn count(&self) -> usize {
        self.memos.len()
    }
}

/// Format memo registry
pub fn format_memo_registry(registry: &MemoRegistry) -> String {
    let mut output = String::new();
    output.push_str("Settings Memo Registry:\n");
    output.push_str(&format!("  Memos: {}\n", registry.count()));
    output
}

/// Check if query is about memo
pub fn is_memo_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("settings memo") || lower.contains("memo settings") || lower.contains("internal memo")
}

/// Fun fact about memo
pub fn memo_fun_fact() -> &'static str {
    "Anna's settings memo system facilitates internal configuration communication!"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings_memo::config::MemoConfig;

    #[test]
    fn test_registry_new() {
        let r = MemoRegistry::new();
        assert_eq!(r.count(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut r = MemoRegistry::new();
        r.register("m1", SettingsMemo::new(MemoConfig::default()));
        assert_eq!(r.count(), 1);
    }

    #[test]
    fn test_is_memo_query() {
        assert!(is_memo_query("settings memo"));
        assert!(!is_memo_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = memo_fun_fact();
        assert!(fact.contains("memo"));
    }
}
