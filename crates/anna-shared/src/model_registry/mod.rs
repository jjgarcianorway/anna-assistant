//! Model registry for role-model bindings and hardware-aware selection (v0.0.201).
//!
//! Tracks which models are assigned to which roles (team + specialist role).
//! Provides hardware-aware model selection based on system capabilities.
//!
//! v0.0.29: Initial implementation.
//! v0.0.201: Modularized into domain-focused submodules.

mod persistence;
mod registry;
mod tests;
mod types;

// Re-export all types and functions
pub use persistence::{
    load_model_registry, model_registry_path, parse_ollama_list, save_model_registry,
};
pub use registry::ModelRegistry;
pub use types::{recommended_model_for_tier, HardwareTier, ModelSpec, ModelState, RoleBinding};
