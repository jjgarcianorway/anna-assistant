//! Knowledge fetcher pipeline (v0.0.422).
//!
//! Orchestrates knowledge fetching from multiple sources:
//! 1. Man pages
//! 2. Help output
//! 3. Arch Wiki (cache)
//! 4. Local docs
//!
//! Produces normalized KnowledgeSnippet objects.

mod core;
mod helpers;
mod tests;
mod types;

// Re-export public types
pub use core::KnowledgeFetcher;
pub use types::FetchResult;
