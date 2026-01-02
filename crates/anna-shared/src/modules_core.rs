//! Core module re-exports
//! Essential modules for Anna's operation

// Version and protocol
#[path = "version.rs"]
pub mod version;
#[path = "anna_proto/mod.rs"]
pub mod anna_proto;

// Core types and errors
#[path = "error.rs"]
pub mod error;
#[path = "rpc/mod.rs"]
pub mod rpc;
#[path = "ledger.rs"]
pub mod ledger;
#[path = "truth_ledger.rs"]
pub mod truth_ledger;
#[path = "update_ledger.rs"]
pub mod update_ledger;

// Ticket system
#[path = "ticket/mod.rs"]
pub mod ticket;
#[path = "ticket_tracker/mod.rs"]
pub mod ticket_tracker;
#[path = "ticket_state/mod.rs"]
pub mod ticket_state;
#[path = "ticket_log/mod.rs"]
pub mod ticket_log;
#[path = "ticket_lifecycle/mod.rs"]
pub mod ticket_lifecycle;
#[path = "ticket_integrity/mod.rs"]
pub mod ticket_integrity;
#[path = "ticket_packet/mod.rs"]
pub mod ticket_packet;
#[path = "pending/mod.rs"]
pub mod pending;
#[path = "pending_queue.rs"]
pub mod pending_queue;

// User and team management
#[path = "user_profile/mod.rs"]
pub mod user_profile;
#[path = "roster/mod.rs"]
pub mod roster;
#[path = "teams.rs"]
pub mod teams;
#[path = "specialist_roster/mod.rs"]
pub mod specialist_roster;
#[path = "team_specialist_roster/mod.rs"]
pub mod team_specialist_roster;
#[path = "team_availability.rs"]
pub mod team_availability;

// Specialists
#[path = "specialists.rs"]
pub mod specialists;
#[path = "specialist_response/mod.rs"]
pub mod specialist_response;
#[path = "specialist_contract/mod.rs"]
pub mod specialist_contract;
#[path = "specialist_contract_v1/mod.rs"]
pub mod specialist_contract_v1;
#[path = "specialist_protocol/mod.rs"]
pub mod specialist_protocol;
#[path = "specialist_v2/mod.rs"]
pub mod specialist_v2;
#[path = "specialist_v3/mod.rs"]
pub mod specialist_v3;
#[path = "specialist_conversation/mod.rs"]
pub mod specialist_conversation;

// Contracts and protocols
#[path = "question_contract/mod.rs"]
pub mod question_contract;
#[path = "answer_contract/mod.rs"]
pub mod answer_contract;
#[path = "answer_shaper.rs"]
pub mod answer_shaper;
#[path = "strict_contract/mod.rs"]
pub mod strict_contract;
#[path = "strict_prompts.rs"]
pub mod strict_prompts;
#[path = "translator_contract.rs"]
pub mod translator_contract;

// Intent and routing
#[path = "intake/mod.rs"]
pub mod intake;
#[path = "canonical_intents/mod.rs"]
pub mod canonical_intents;
#[path = "intent_handlers/mod.rs"]
pub mod intent_handlers;
#[path = "intent_policy.rs"]
pub mod intent_policy;
#[path = "deterministic_routing/mod.rs"]
pub mod deterministic_routing;
#[path = "query_type_router.rs"]
pub mod query_type_router;

// Pipelines
#[path = "era_pipeline/mod.rs"]
pub mod era_pipeline;
#[path = "fast_pipeline/mod.rs"]
pub mod fast_pipeline;
#[path = "fastpath/mod.rs"]
pub mod fastpath;

// Review and verification
#[path = "review/mod.rs"]
pub mod review;
#[path = "review_gate/mod.rs"]
pub mod review_gate;
#[path = "review_prompts/mod.rs"]
pub mod review_prompts;
#[path = "verify/mod.rs"]
pub mod verify;

// Claims, facts, and grounding
#[path = "claims.rs"]
pub mod claims;
#[path = "facts/mod.rs"]
pub mod facts;
#[path = "facts_types.rs"]
pub mod facts_types;
#[path = "facts_maintenance.rs"]
pub mod facts_maintenance;
#[path = "grounding/mod.rs"]
pub mod grounding;

// Clarification
#[path = "clarify/mod.rs"]
pub mod clarify;
#[path = "clarify_v2/mod.rs"]
pub mod clarify_v2;

// Guard and reliability
#[path = "guard/mod.rs"]
pub mod guard;
#[path = "reliability/mod.rs"]
pub mod reliability;
#[path = "reliability_gate/mod.rs"]
pub mod reliability_gate;
#[path = "reliability_metrics/mod.rs"]
pub mod reliability_metrics;

// Metrics and telemetry
#[path = "honest_metrics.rs"]
pub mod honest_metrics;
#[path = "telemetry/mod.rs"]
pub mod telemetry;
#[path = "system_telemetry/mod.rs"]
pub mod system_telemetry;

// Parsers and prompts
#[path = "parsers/mod.rs"]
pub mod parsers;
#[path = "llm_parse.rs"]
pub mod llm_parse;
#[path = "solver_prompts.rs"]
pub mod solver_prompts;

// Transcript and logging
#[path = "transcript/mod.rs"]
pub mod transcript;
#[path = "transcript_ext.rs"]
pub mod transcript_ext;
#[path = "transcript_segment.rs"]
pub mod transcript_segment;
#[path = "trace/mod.rs"]
pub mod trace;
#[path = "event_log/mod.rs"]
pub mod event_log;

// Citations and sources
#[path = "citation.rs"]
pub mod citation;
#[path = "citations.rs"]
pub mod citations;
#[path = "source_layer/mod.rs"]
pub mod source_layer;

// Regression tests
#[path = "regression_tests.rs"]
pub mod regression_tests;

// Result signals
#[path = "result_signals.rs"]
pub mod result_signals;

// Advice and briefs
#[path = "advice.rs"]
pub mod advice;
#[path = "brief/mod.rs"]
pub mod brief;

// Helpers
#[path = "helpers/mod.rs"]
pub mod helpers;

// Budget and resources
#[path = "budget/mod.rs"]
pub mod budget;
#[path = "resource_limits.rs"]
pub mod resource_limits;
#[path = "robustness/mod.rs"]
pub mod robustness;

// Revision and change management
#[path = "revision/mod.rs"]
pub mod revision;
#[path = "change.rs"]
pub mod change;
#[path = "change_history.rs"]
pub mod change_history;
#[path = "change_transaction.rs"]
pub mod change_transaction;

// Snapshot
#[path = "snapshot/mod.rs"]
pub mod snapshot;

// Report
#[path = "report/mod.rs"]
pub mod report;

// Inventory
#[path = "inventory/mod.rs"]
pub mod inventory;

// Narrator
#[path = "narrator/mod.rs"]
pub mod narrator;

// Email
#[path = "email/mod.rs"]
pub mod email;
