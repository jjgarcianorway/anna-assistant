//! Context memory store - storage and management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{MemoryEntry, MemoryImportance, MemoryType};

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

    /// Get important memories
    pub fn important(&self) -> Vec<&MemoryEntry> {
        self.memories
            .iter()
            .filter(|m| m.importance >= MemoryImportance::High)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
