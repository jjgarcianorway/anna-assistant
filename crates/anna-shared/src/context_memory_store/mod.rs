//! Context Memory Store - Phase 102
//!
//! Stores and retrieves conversational context.
//! Enables Anna to remember important information across sessions.

mod types;
mod store;
mod query;
mod format;

// Re-export types
pub use types::{MemoryEntry, MemoryImportance, MemoryType};

// Re-export store
pub use store::ContextMemoryStore;

// Re-export query functions
pub use query::{is_memory_query, memory_fun_fact};

// Re-export format functions
pub use format::{format_memory_store, format_memory_store_compact, format_memory_store_oneline};
