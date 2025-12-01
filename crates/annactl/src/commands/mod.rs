//! Command modules for annactl v5.5.0
//!
//! Commands:
//! - status: Daemon health and status
//! - stats: Daemon activity statistics
//! - knowledge: Knowledge overview by category
//! - knowledge_stats: Coverage and quality statistics
//! - knowledge_category: List objects in a category
//! - knowledge_detail: Full profile of a single object
//! - version: Installation and version info

pub mod status;
pub mod stats;
pub mod knowledge;
pub mod knowledge_stats;
pub mod knowledge_category;
pub mod knowledge_detail;
pub mod version;
