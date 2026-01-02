//! Core types for the documentation engine (v0.0.429).

pub mod query;
pub mod reference;
pub mod snippet;
pub mod source;

// Re-export all public types
pub use query::{DocQuery, DocResult};
pub use reference::DocReference;
pub use snippet::DocSnippet;
pub use source::DocSourceKind;
