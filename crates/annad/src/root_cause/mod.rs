//! Root Cause Analysis - Find WHY problems happen, not just WHAT happened.

mod types;
mod analyzer;

pub use types::{
    SystemEvent, EventType, CausalLink, RootCauseAnalysis, RootCause,
    DependencyGraph, ComponentNode, ComponentType, DependencyEdge, DependencyType,
};
pub use analyzer::{analyze_symptom, collect_recent_events};
