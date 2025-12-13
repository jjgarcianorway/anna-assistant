//! Context Memory Store - Phase 102
//!
//! Stores and retrieves conversational context.
//! Enables Anna to remember important information across sessions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MemoryType {
    #[default]
    ShortTerm,
    LongTerm,
    Working,
    Episodic,
    Semantic,
}

impl MemoryType {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryType::ShortTerm => "Short-term",
            MemoryType::LongTerm => "Long-term",
            MemoryType::Working => "Working",
            MemoryType::Episodic => "Episodic",
            MemoryType::Semantic => "Semantic",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            MemoryType::ShortTerm => "○",
            MemoryType::LongTerm => "●",
            MemoryType::Working => "◐",
            MemoryType::Episodic => "◑",
            MemoryType::Semantic => "◒",
        }
    }
}

/// Memory importance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub enum MemoryImportance {
    Low,
    #[default]
    Normal,
    High,
    Critical,
}

impl MemoryImportance {
    pub fn name(&self) -> &'static str {
        match self {
            MemoryImportance::Low => "Low",
            MemoryImportance::Normal => "Normal",
            MemoryImportance::High => "High",
            MemoryImportance::Critical => "Critical",
        }
    }

    pub fn score(&self) -> u8 {
        match self {
            MemoryImportance::Low => 1,
            MemoryImportance::Normal => 2,
            MemoryImportance::High => 3,
            MemoryImportance::Critical => 4,
        }
    }
}

/// A memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Memory key
    pub key: String,
    /// Memory content
    pub content: String,
    /// Memory type
    pub memory_type: MemoryType,
    /// Importance level
    pub importance: MemoryImportance,
    /// Access count
    pub access_count: u64,
    /// Created timestamp
    pub created_at: u64,
    /// Last accessed timestamp
    pub last_accessed: u64,
    /// Expires at (optional)
    pub expires_at: Option<u64>,
}

/// Context memory store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextMemoryStore {
    /// All memories
    pub memories: Vec<MemoryEntry>,
    /// Count by type
    pub by_type: HashMap<String, u64>,
    /// Count by importance
    pub by_importance: HashMap<String, u64>,
    /// Total accesses
    pub total_accesses: u64,
    /// Max memories (for pruning)
    pub max_memories: usize,
}

impl ContextMemoryStore {
    pub fn new() -> Self {
        Self {
            max_memories: 1000,
            ..Default::default()
        }
    }

    /// Store a memory
    pub fn store(&mut self, key: String, content: String, memory_type: MemoryType, importance: MemoryImportance, timestamp: u64) {
        // Check if key exists
        let found = self.memories.iter().position(|m| m.key == key);
        if let Some(idx) = found {
            // Update existing
            self.memories[idx].content = content;
            self.memories[idx].last_accessed = timestamp;
        } else {
            // New memory
            let entry = MemoryEntry {
                key,
                content,
                memory_type,
                importance,
                access_count: 0,
                created_at: timestamp,
                last_accessed: timestamp,
                expires_at: None,
            };
            *self.by_type.entry(memory_type.name().to_string()).or_insert(0) += 1;
            *self.by_importance.entry(importance.name().to_string()).or_insert(0) += 1;
            self.memories.push(entry);
        }

        // Prune if over limit
        if self.memories.len() > self.max_memories {
            self.prune();
        }
    }

    /// Retrieve a memory
    pub fn retrieve(&mut self, key: &str, timestamp: u64) -> Option<&str> {
        let found = self.memories.iter().position(|m| m.key == key);
        if let Some(idx) = found {
            self.memories[idx].access_count += 1;
            self.memories[idx].last_accessed = timestamp;
            self.total_accesses += 1;
            Some(&self.memories[idx].content)
        } else {
            None
        }
    }

    /// Get memory without updating access
    pub fn get(&self, key: &str) -> Option<&MemoryEntry> {
        self.memories.iter().find(|m| m.key == key)
    }

    /// Search memories by content
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> {
        let q = query.to_lowercase();
        self.memories
            .iter()
            .filter(|m| m.content.to_lowercase().contains(&q) || m.key.to_lowercase().contains(&q))
            .collect()
    }

    /// Get memories by type
    pub fn by_mem_type(&self, memory_type: MemoryType) -> Vec<&MemoryEntry> {
        self.memories.iter().filter(|m| m.memory_type == memory_type).collect()
    }

    /// Get important memories
    pub fn important(&self) -> Vec<&MemoryEntry> {
        self.memories
            .iter()
            .filter(|m| m.importance >= MemoryImportance::High)
            .collect()
    }

    /// Delete a memory
    pub fn delete(&mut self, key: &str) -> bool {
        let found = self.memories.iter().position(|m| m.key == key);
        if let Some(idx) = found {
            let mem = &self.memories[idx];
            if let Some(count) = self.by_type.get_mut(mem.memory_type.name()) {
                *count = count.saturating_sub(1);
            }
            if let Some(count) = self.by_importance.get_mut(mem.importance.name()) {
                *count = count.saturating_sub(1);
            }
            self.memories.remove(idx);
            true
        } else {
            false
        }
    }

    /// Prune least important/accessed memories
    pub fn prune(&mut self) {
        // Sort by importance (ascending) then by access count (ascending)
        self.memories.sort_by(|a, b| {
            a.importance.score().cmp(&b.importance.score())
                .then(a.access_count.cmp(&b.access_count))
        });

        // Remove first 10%
        let remove_count = self.memories.len() / 10;
        for _ in 0..remove_count {
            if let Some(mem) = self.memories.first() {
                let mem_type = mem.memory_type;
                let mem_importance = mem.importance;
                if let Some(count) = self.by_type.get_mut(mem_type.name()) {
                    *count = count.saturating_sub(1);
                }
                if let Some(count) = self.by_importance.get_mut(mem_importance.name()) {
                    *count = count.saturating_sub(1);
                }
            }
            self.memories.remove(0);
        }
    }

    /// Total memory count
    pub fn total_count(&self) -> usize {
        self.memories.len()
    }

    /// Important memory count
    pub fn important_count(&self) -> usize {
        self.important().len()
    }
}

/// Format memory store for display
pub fn format_memory_store(store: &ContextMemoryStore) -> String {
    let mut lines = vec!["=== Context Memory Store ===".to_string()];
    lines.push(String::new());

    if store.memories.is_empty() {
        lines.push("No memories stored yet.".to_string());
        return lines.join("\n");
    }

    // Summary
    lines.push(format!("Total memories: {}", store.total_count()));
    lines.push(format!("Important: {}", store.important_count()));
    lines.push(format!("Total accesses: {}", store.total_accesses));

    // By type
    if !store.by_type.is_empty() {
        lines.push(String::new());
        lines.push("By type:".to_string());
        for (t, count) in &store.by_type {
            lines.push(format!("  {}: {}", t, count));
        }
    }

    // Important memories
    let important = store.important();
    if !important.is_empty() {
        lines.push(String::new());
        lines.push("Important memories:".to_string());
        for mem in important.iter().take(10) {
            lines.push(format!("  [{}] {} = {}", mem.memory_type.symbol(), mem.key, mem.content));
        }
    }

    lines.join("\n")
}

/// Format memory store compact
pub fn format_memory_store_compact(store: &ContextMemoryStore) -> String {
    format!(
        "Memory: {} stored | {} important | {} accesses",
        store.total_count(),
        store.important_count(),
        store.total_accesses
    )
}

/// Format memory store one-line
pub fn format_memory_store_oneline(store: &ContextMemoryStore) -> String {
    format!("{} memories ({} important)", store.total_count(), store.important_count())
}

/// Check if query is about memory
pub fn is_memory_query(query: &str) -> bool {
    let q = query.to_lowercase();
    let keywords = [
        "memory",
        "remember",
        "what do you remember",
        "stored context",
        "recall",
    ];
    keywords.iter().any(|k| q.contains(k))
}

/// Generate fun fact about memory
pub fn memory_fun_fact(store: &ContextMemoryStore) -> String {
    if store.memories.is_empty() {
        return "No memories stored yet!".to_string();
    }

    let facts = [
        format!("Anna remembers {} things.", store.total_count()),
        format!("{} memories are marked as important.", store.important_count()),
        format!("Memories have been accessed {} times.", store.total_accesses),
    ];

    facts[store.total_count() % facts.len()].clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type() {
        assert_eq!(MemoryType::LongTerm.name(), "Long-term");
        assert_eq!(MemoryType::ShortTerm.symbol(), "○");
    }

    #[test]
    fn test_memory_importance() {
        assert_eq!(MemoryImportance::High.name(), "High");
        assert_eq!(MemoryImportance::Critical.score(), 4);
        assert!(MemoryImportance::Critical > MemoryImportance::Normal);
    }

    #[test]
    fn test_store_memory() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);

        assert_eq!(store.total_count(), 1);
        assert!(store.get("user_name").is_some());
    }

    #[test]
    fn test_retrieve_memory() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);

        let content = store.retrieve("user_name", 2000);
        assert_eq!(content, Some("Alice"));
        assert_eq!(store.total_accesses, 1);
    }

    #[test]
    fn test_update_memory() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);
        store.store("user_name".to_string(), "Bob".to_string(), MemoryType::LongTerm, MemoryImportance::High, 2000);

        assert_eq!(store.total_count(), 1);
        assert_eq!(store.get("user_name").unwrap().content, "Bob");
    }

    #[test]
    fn test_search_memory() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice Smith".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);
        store.store("user_email".to_string(), "alice@example.com".to_string(), MemoryType::LongTerm, MemoryImportance::Normal, 1000);

        let results = store.search("alice");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_by_type() {
        let mut store = ContextMemoryStore::new();
        store.store("m1".to_string(), "v1".to_string(), MemoryType::ShortTerm, MemoryImportance::Normal, 1000);
        store.store("m2".to_string(), "v2".to_string(), MemoryType::LongTerm, MemoryImportance::Normal, 1000);

        assert_eq!(store.by_mem_type(MemoryType::ShortTerm).len(), 1);
        assert_eq!(store.by_mem_type(MemoryType::LongTerm).len(), 1);
    }

    #[test]
    fn test_delete_memory() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);
        store.delete("user_name");

        assert_eq!(store.total_count(), 0);
    }

    #[test]
    fn test_important() {
        let mut store = ContextMemoryStore::new();
        store.store("m1".to_string(), "v1".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);
        store.store("m2".to_string(), "v2".to_string(), MemoryType::LongTerm, MemoryImportance::Low, 1000);

        assert_eq!(store.important_count(), 1);
    }

    #[test]
    fn test_format_store() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);

        let output = format_memory_store(&store);
        assert!(output.contains("Context Memory Store"));
        assert!(output.contains("Total memories: 1"));
    }

    #[test]
    fn test_is_memory_query() {
        assert!(is_memory_query("what do you remember?"));
        assert!(is_memory_query("recall my name"));
        assert!(!is_memory_query("what is the weather?"));
    }

    #[test]
    fn test_fun_fact() {
        let mut store = ContextMemoryStore::new();
        store.store("test".to_string(), "value".to_string(), MemoryType::ShortTerm, MemoryImportance::Normal, 1000);

        let fact = memory_fun_fact(&store);
        assert!(!fact.is_empty());
    }
}
