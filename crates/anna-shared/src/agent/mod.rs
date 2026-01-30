//! Multi-agent system for Anna.
//!
//! This module provides the agent abstraction layer for Anna's
//! multi-agent orchestration system.
//!
//! # Components
//!
//! - `traits`: Core Agent trait and domain/model tier enums
//! - `types`: AgentTask, AgentResult, AgentContext structures
//! - `memory`: Per-agent learning and memory persistence

mod memory;
mod traits;
mod types;

pub use memory::{
    AgentMemory, AgentMemoryStore, DomainFact, FailedAttempt, LearnedPattern,
};
pub use traits::{Agent, AgentDomain, ModelTier};
pub use types::{
    detect_domains, AgentCapability, AgentContext, AgentResult, AgentTask,
    Evidence, EvidenceSource, ExecutionBudget, Learning, SystemProfile, TaskContext,
};
