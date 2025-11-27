//! Knowledge Store v0.16.1
//!
//! Persistent knowledge store for Anna's understanding of the system.
//! Facts are stored as structured records with evidence, confidence, and timestamps.
//!
//! v0.16.1: Added fact validation to double-check user/LLM facts before storing.
//!
//! Schema version: 1

pub mod learning;
pub mod schema;
pub mod store;
pub mod telemetry;
pub mod validation;

pub use learning::*;
pub use schema::*;
pub use store::*;
pub use telemetry::*;
pub use validation::*;
