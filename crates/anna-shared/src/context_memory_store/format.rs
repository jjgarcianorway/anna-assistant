//! Memory store formatting utilities

use super::store::ContextMemoryStore;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_memory_store::types::{MemoryImportance, MemoryType};

    #[test]
    fn test_format_store() {
        let mut store = ContextMemoryStore::new();
        store.store("user_name".to_string(), "Alice".to_string(), MemoryType::LongTerm, MemoryImportance::High, 1000);

        let output = format_memory_store(&store);
        assert!(output.contains("Context Memory Store"));
        assert!(output.contains("Total memories: 1"));
    }
}
