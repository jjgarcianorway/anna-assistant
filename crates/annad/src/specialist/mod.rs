//! Specialist System - Deterministic dispatch and execution.
//!
//! v0.3.37: Phase 10 - Specialist System Enablement
//!
//! Specialists are routing + execution units (not LLM wrappers).
//! This module provides:
//! - Domain classification for routing
//! - Static registry of 16 specialists (8 domains x 2 levels)
//! - Dispatch engine with escalation rules
//! - Execution contract (structured output only)
//! - Ticket lifecycle events
//! - Per-specialist statistics

pub mod dispatch;
pub mod domain;
pub mod events;
pub mod executor;
pub mod output;
pub mod registry;
pub mod stats;

// Re-export key types
pub use dispatch::{DispatchDecision, DispatchEngine, EscalationReason, CONFIDENCE_HIGH};
pub use domain::Domain;
pub use events::TicketEvent;
pub use executor::{ExecutionContext, SpecialistExecutor};
pub use output::SpecialistOutput;
pub use registry::{
    get_by_id, get_junior, get_senior, get_specialists_for_domain, SpecialistDefinition,
    SpecialistLevel, SPECIALIST_REGISTRY,
};
pub use stats::{SpecialistMetrics, SpecialistStatsStore};
