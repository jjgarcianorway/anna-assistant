//! Annactl library - exposes modules for testing
//!
//! v6.57.0: Brutal cleanup - single pipeline architecture
//!
//! All queries flow through the unified pipeline:
//! planner_core → executor_core → interpreter_core → trace_renderer

pub mod action_plan_executor;
pub mod cli_output;
pub mod context_engine;
pub mod debug;
pub mod diagnostic_formatter;
pub mod dialogue_v3_json;
pub mod errors;
pub mod hardware_questions;
pub mod health;
pub mod internal_dialogue;
pub mod llm_integration;
pub mod logging;
pub mod net_diagnostics;
pub mod output;
pub mod power_diagnostics;
pub mod startup;
// REMOVED in 6.57.0: query_handler - legacy recipe-based handler
// REMOVED in 6.57.0: recipe_formatter - legacy recipe formatting
pub mod rpc_client;
pub mod status_health;
// REMOVED in 6.57.0: sysadmin_answers - hardcoded answer templates
pub mod system_prompt_v2;
pub mod system_prompt_v3_json;
pub mod system_query;
pub mod system_report;
pub mod systemd;
pub mod telemetry_truth;
pub mod unified_query_handler;
// REMOVED in v7.0.0: planner_query_handler - replaced by query_handler_v7
pub mod query_handler_v7; // v7.0.0: Clean brain architecture
pub mod query_handler_v8; // v8.0.0: Pure LLM-driven architecture
pub mod intent_router; // Intent classification
pub mod approval_ui; // v6.43.0: User approval for plans
