//! Shared types for Anna.
//! Contains types for daemon-client communication and shared functionality.
//!
//! ARCHITECTURAL INVARIANT: Anna is system-wide with ZERO state in user home directories.
//! All paths are defined in the `paths` module and use /etc/anna, /var/lib/anna, /run/anna.
//!
//! # Execution Boundary Documentation (Phase 35)
//!
//! This section documents where execution capability exists, does not exist, and what
//! would be required to introduce it. This is a description of absence, not intent.
//!
//! ## Where Execution Could Exist In The Future
//!
//! Execution could be introduced by implementing the `ExecutionAdapter` trait defined
//! in `action_plan.rs`. This trait declares a single method:
//!
//! ```ignore
//! fn execute(&self, plan: &DeterministicActionPlan) -> ExecutionResult;
//! ```
//!
//! A future system could implement this trait on a concrete type, creating an executor
//! that interprets `DeterministicStep` operations and dispatches system commands.
//!
//! ## Where Execution Explicitly Does Not Exist Today
//!
//! 1. **No implementations of `ExecutionAdapter`** - The trait exists but has zero
//!    implementations. No code can call `execute()` because no value has that method.
//!
//! 2. **No functions that interpret plan steps** - `DeterministicStep` contains
//!    `operation` and `target` fields, but no code reads these and performs actions.
//!
//! 3. **No command dispatch** - No code in this crate calls `std::process::Command`,
//!    `tokio::process::Command`, or any system execution API based on plan data.
//!
//! 4. **No RPC handlers that trigger execution** - The `rpc` module defines message
//!    types but no handler executes plans.
//!
//! 5. **No scheduler or daemon loop** - No background task monitors plans and executes
//!    them when ready.
//!
//! ## What Would Be Required To Violate This Boundary
//!
//! To introduce execution capability, ALL of the following would be required:
//!
//! 1. **Add `impl ExecutionAdapter for ConcreteType`** - Create a type that implements
//!    the trait, providing an actual `execute()` method body.
//!
//! 2. **Add step interpretation logic** - Write code that matches on `operation` field
//!    values ("service_restart", "package_install", etc.) and maps them to actions.
//!
//! 3. **Add system command dispatch** - Call `std::process::Command` or equivalent
//!    to actually run commands on the system.
//!
//! 4. **Wire the executor into a code path** - Connect the executor to some trigger
//!    (RPC handler, CLI command, scheduled task) that invokes it.
//!
//! 5. **Explicitly break through `ExecutionGate`** - Decide that `can_execute() == true`
//!    should trigger the executor, making the gate meaningful.
//!
//! None of these exist today. The architecture is sealed at the data layer.
//! Execution is structurally impossible without explicit, reviewable changes.
//!
//! ## Verification Commands
//!
//! ```bash
//! # Verify no ExecutionAdapter implementations exist:
//! grep -rn "impl.*ExecutionAdapter" crates/anna-shared/src/
//! # Expected: zero results
//!
//! # Verify no execute() calls exist:
//! grep -rn "\.execute(" crates/anna-shared/src/
//! # Expected: zero results (only trait definition)
//!
//! # Verify no Command::new based on plan data:
//! grep -rn "Command::new" crates/anna-shared/src/
//! # Expected: zero results
//! ```
//!
//! This phase proves that execution is impossible in the current architecture.
//!
//! # Reserved Execution Interface (Phase 36)
//!
//! This section documents the single sanctioned breach point for future execution.
//!
//! ## Where Execution Would Connect If Ever Allowed
//!
//! The `execution_reservation` module defines `ReservedExecutionInterface`, the ONLY
//! permitted interface for future execution capability. If execution is ever enabled:
//!
//! 1. A concrete type would implement `ReservedExecutionInterface`
//! 2. That type would be instantiated in a controlled context (daemon, CLI, RPC)
//! 3. The instantiation would be wired to:
//!    - `ExecutionGate` check (must pass before invocation)
//!    - Plan retrieval (must be valid and approved)
//!    - `ExecutionAttempt` recording (before and after invocation)
//! 4. The `execute()` method would be called with a `DeterministicActionPlan`
//! 5. The implementation would interpret steps and dispatch system commands
//!
//! ## This Interface Is The Only Sanctioned Breach Point
//!
//! `ReservedExecutionInterface` is the ONLY permitted execution interface.
//! Any execution capability introduced through this interface is:
//!
//! - Explicit and reviewable
//! - Constrained to the defined contract
//! - Auditable at a single location
//!
//! Any execution capability introduced through another path is:
//!
//! - An architectural violation
//! - A security concern
//! - Subject to immediate removal
//!
//! ## No Other Execution Entry Points Are Permitted
//!
//! Do NOT add:
//!
//! - Direct `Command::new` calls based on plan data outside this interface
//! - Alternative execution traits
//! - Executor functions that bypass `ReservedExecutionInterface`
//! - RPC handlers that execute without implementing this interface
//!
//! ## Current State
//!
//! - **Implementations of `ReservedExecutionInterface`**: Zero
//! - **Types that can execute**: Zero
//! - **Code paths that invoke execution**: Zero
//!
//! The interface exists as a reservation. It defines shape without granting power.
//!
//! This interface reserves a future execution boundary without enabling execution.

pub mod action_plan;
pub mod agent;
pub mod capabilities;
pub mod charts;

#[cfg(test)]
mod capabilities_guardrails;
pub mod capability;
pub mod declaration;
pub mod claim_gate;
pub mod command_policy;
#[cfg(test)]
mod adversarial_audit;
#[cfg(test)]
mod command_policy_guardrails;
#[cfg(test)]
mod user_trust_review;
pub mod config;
pub mod deps;
pub mod docs;
pub mod event_bus;
pub mod execution_impossibility;
pub mod execution_request;
pub mod execution_reservation;
pub mod human_execution;
pub mod experiment;
pub mod exposure;
pub mod fingerprint;
pub mod health_report;
pub mod helpers;
pub mod intent_class;
pub mod intention;
pub mod interpretation;
pub mod knowledge;
pub mod live_state;
pub mod memory;
pub mod migration;
pub mod monitor;
pub mod outcome_ledger;
pub mod paths;
pub mod policy;
pub mod prediction;
pub mod proactive;
pub mod probe_ledger;
pub mod probe_stats;
pub mod preferences;
pub mod profile;
pub mod proposal;
pub mod recipe;
pub mod rpc;
pub mod scheduler;
pub mod safe_ops;
pub mod session;
pub mod status;
pub mod stats;
pub mod teaching;
pub mod telemetry_consumer;
pub mod timeline;
pub mod update_ledger;
pub mod user_context;
pub mod version;
pub mod web_search;
pub mod wiki;

// Re-export paths for convenience
pub use paths::{paths, Paths};

// Socket path (uses system paths, can be overridden with ANNA_SOCKET env var)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET")
        .unwrap_or_else(|_| paths().socket_file().to_string_lossy().to_string())
}

// Version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// GitHub repo for updates
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";

// Default update check interval (60 seconds)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;
