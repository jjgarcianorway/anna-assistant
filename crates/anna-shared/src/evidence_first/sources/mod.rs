//! Knowledge sources (v0.0.435).
//!
//! Abstractions for retrieving evidence from local sources.

mod error;
mod help_text;
mod local_docs;
mod man_page;
mod types;

// Re-export all public types
pub use error::SourceError;
pub use help_text::{HelpTextSource, HelpVariant};
pub use local_docs::LocalDocsSource;
pub use man_page::{ManPageSource, ManSection};
pub use types::KnowledgeSource;
