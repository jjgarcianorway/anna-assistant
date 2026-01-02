//! Ticket Intent Schema (Part A) - v0.0.439.
//!
//! Canonical schema for translator output. Translator must output exactly this
//! JSON structure with temperature=0, max tokens 200, no prose.

pub mod parser;
pub mod schema;
#[cfg(test)]
mod tests;
pub mod types;

// Re-export public types for convenience
pub use parser::{IntentSchemaParser, ParseError};
pub use schema::TicketIntentSchema;
pub use types::{CanonicalIntent, Department, RiskLevel};
