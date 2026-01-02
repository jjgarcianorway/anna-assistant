//! Memory querying and retrieval operations

use super::store::ContextMemoryStore;
use super::types::{MemoryEntry, MemoryType};

impl ContextMemoryStore {
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
    use crate::context_memory_store::types::{MemoryImportance, MemoryType};

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
