//! Knowledge V2 - Research-First Knowledge Layer (v0.0.422).
//!
//! Anna's brain is wired to Arch Wiki, man pages, and help output.
//! This module provides a clean, prioritized knowledge pipeline:
//!
//! Priority order:
//! 1. Local system docs (man, help, pacman metadata)
//! 2. Arch Wiki (online or cached)
//! 3. Official docs (/usr/share/doc, /usr/share/help)
//!
//! Key principles:
//! - Research-first, not opinion-first
//! - Every claim backed by citations
//! - No arbitrary web search
//! - Clean snippets, not text blobs

pub mod cache;
pub mod fetcher;
pub mod policy;
pub mod snippet;
pub mod sources;

// Re-export main types
pub use cache::{WikiCache, WikiCacheEntry};
pub use fetcher::{FetchResult, KnowledgeFetcher};
pub use policy::{get_knowledge_topics, needs_knowledge, ResearchPolicy};
pub use snippet::{KnowledgeSnippet, KnowledgeSource};
pub use sources::{fetch_arch_wiki, fetch_help_output, fetch_local_doc, fetch_man_page};

/// Maximum snippets per ticket
pub const MAX_SNIPPETS_PER_TICKET: usize = 5;

/// Maximum raw excerpt length (chars)
pub const MAX_EXCERPT_LENGTH: usize = 2000;

/// Maximum summary length (chars)
pub const MAX_SUMMARY_LENGTH: usize = 400;

/// Maximum key points per snippet
pub const MAX_KEY_POINTS: usize = 7;

/// Cache TTL in seconds (7 days)
pub const CACHE_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// Fetch timeout in milliseconds
pub const FETCH_TIMEOUT_MS: u64 = 3000;
