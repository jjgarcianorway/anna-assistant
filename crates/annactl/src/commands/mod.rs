//! Command modules for annactl v5.2.4
//!
//! Commands are separated by responsibility:
//! - status: Anna's own health and status
//! - stats: Anna's behavior and history statistics
//! - knowledge: Overview of what Anna knows
//! - knowledge_stats: Detailed knowledge statistics
//! - knowledge_detail: Full profile of a single object

pub mod status;
pub mod stats;
pub mod knowledge;
pub mod knowledge_stats;
pub mod knowledge_detail;
