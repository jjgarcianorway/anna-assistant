//! Arch Wiki local reader (v0.0.429).
//!
//! Reads Arch Wiki pages from local cache/snapshot.
//! Supports both plain text and HTML formats.

mod file_ops;
mod html;
mod reader;
mod sections;
mod types;

// Re-export public API
pub use file_ops::{find_wiki_page, list_wiki_pages};
pub use reader::read_wiki_page;
pub use types::{get_essential_wiki_pages, WikiReadError};
