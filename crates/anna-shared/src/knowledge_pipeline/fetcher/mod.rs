//! Knowledge fetcher module (v0.0.432).
//!
//! Modularized knowledge fetching with priority ordering.

mod core;
mod handlers;
mod types;
mod utils;

pub use core::KnowledgeFetcher;
pub use types::{FetchConfig, FetchResult};
