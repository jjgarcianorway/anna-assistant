//! Capabilities Display (v0.0.480).
//!
//! Displays Anna's capabilities and what she can help with.
//! Responds to queries like "what can you do?" or "help me".

mod category;
mod formatters;
mod parsers;
mod facts;

#[cfg(test)]
mod tests;

// Re-export all public items to preserve the original API
pub use category::CapabilityCategory;
pub use formatters::{
    format_capabilities,
    format_capability_category,
    format_capabilities_compact,
    format_capabilities_with_teams,
};
pub use parsers::{
    is_capabilities_query,
    parse_capability_category,
};
pub use facts::{
    capability_facts,
    random_capability_fact,
};
