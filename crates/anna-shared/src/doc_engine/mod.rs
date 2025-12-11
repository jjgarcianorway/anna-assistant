//! Documentation Engine (v0.0.429)
//!
//! Anna's documentation brain - local knowledge graph from:
//! 1. Arch Wiki (local copy/snapshot)
//! 2. Man pages
//! 3. Tool help output (--help)
//! 4. Local config/doc files
//!
//! Priority: Probes first, docs second. Docs interpret, not invent.

pub mod types;
pub mod index;
pub mod man_parser;
pub mod help_extractor;
pub mod wiki_reader;
pub mod query;
pub mod recipe_integration;
pub mod translator_policy;

pub use types::*;
pub use index::*;
pub use query::*;

/// Maximum snippet size for storage (bytes)
pub const MAX_SNIPPET_SIZE: usize = 2000;

/// Maximum snippets per query result
pub const MAX_QUERY_RESULTS: usize = 5;

/// Cache expiry for help output (7 days)
pub const HELP_CACHE_DAYS: u64 = 7;

/// Cache expiry for man pages (30 days)
pub const MAN_CACHE_DAYS: u64 = 30;

/// Default doc storage path
pub const DOC_STORAGE_PATH: &str = "/var/lib/anna/docs";

/// Alternative doc storage (user home)
pub const DOC_STORAGE_ALT: &str = "~/.anna/docs";

/// Arch Wiki cache path
pub const WIKI_CACHE_PATH: &str = "/var/lib/anna/wiki-cache";

/// Alternative wiki cache (user home)
pub const WIKI_CACHE_ALT: &str = "~/.anna/wiki-cache";

#[cfg(test)]
mod tests;
