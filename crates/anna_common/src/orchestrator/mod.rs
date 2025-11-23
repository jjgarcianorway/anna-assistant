//! Orchestrator - Adaptive planner with Arch Wiki consultation
//!
//! 6.2.0: Minimal vertical slice for DNS scenario

pub mod knowledge;
pub mod planner;
pub mod telemetry;

pub use knowledge::{get_arch_help_dns, KnowledgeSourceKind, KnowledgeSourceRef, WikiSummary};
pub use planner::{plan_dns_fix, Plan, PlanStep, PlanStepKind};
pub use telemetry::TelemetrySummary;
