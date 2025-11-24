//! Annad library
//!
//! Exposes public modules for testing

// Beta.279: Expose historian module for testing
#[path = "historian/mod.rs"]
pub mod historian;

// Expose intel module for proactive engine access
pub mod intel;

// Phase 0.9: System steward
#[path = "steward/mod.rs"]
pub mod steward;

// 6.7.0: Reflection - Anna's self-awareness and concrete error reporting
pub mod reflection;
pub mod reflection_builder;
