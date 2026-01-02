//! Unified documentation query API (v0.0.429).
//!
//! Main interface for specialists and recipe engine to query docs.

mod query_engine;
mod query_types;

pub use query_engine::DocEngine;
pub use query_types::{score_snippet, DocEngineStats, IndexStats, RefreshStats};
