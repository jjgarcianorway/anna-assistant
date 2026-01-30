//! Agent implementations.
//!
//! This module contains concrete agent implementations:
//! - SpecialistAgent: Wraps existing department specialists as agents

mod specialist_agent;

pub use specialist_agent::SpecialistAgent;

use crate::agent_registry::AgentRegistry;
use crate::department::specialists::{get_department, Specialist};
use anna_shared::agent::AgentMemoryStore;
use std::sync::Arc;
use tracing::info;

/// Build the default agent registry with all specialists.
pub fn build_default_registry() -> AgentRegistry {
    let mut registry = AgentRegistry::new();
    let department = get_department();
    let memory_store = AgentMemoryStore::load();

    for specialist in &department.specialists {
        let agent = SpecialistAgent::from_specialist(specialist, &memory_store);
        registry.register(Arc::new(agent));
    }

    info!(
        "Built agent registry with {} specialists",
        registry.count()
    );

    registry
}
