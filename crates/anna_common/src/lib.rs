//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).
//!
//! ## v6.59.0 Architecture - Unified Tool Catalog & Typed Actions
//!
//! All queries flow through a single unified pipeline:
//! `tooling::actions` → `tooling::executor` → LLM summarization
//!
//! v6.59.0 fixes the NL executor with:
//! - Single `tooling::catalog` with `ToolId` enum (NO arbitrary shell)
//! - Typed `Action` vocabulary that LLM selects from
//! - All execution goes through `ToolExecutor`
//! - Honest error messages (no "not in catalog" spam)
//! - Self-test feature validates all tools
//!
//! The LLM can ONLY select from predefined Actions. It cannot generate shell.
//! There are NO legacy handlers, NO hardcoded recipes, NO shortcut paths.

pub mod action_plan; // Beta.66: Secure ACTION_PLAN execution with validation
pub mod action_plan_v3; // Beta.143: JSON runtime contract for LLM
pub mod advice_cache;
pub mod anna_config; // 6.18.0: User configuration system (emoji, color preferences)
pub mod anna_hardware_profile; // 6.11.0: Hardware profile tracking and LLM recommendations
pub mod anna_self_health; // 6.11.0: Anna self-health checks (deps, permissions, LLM)
pub mod answer_validator; // Beta.87: Multi-pass answer validation - zero hallucination guarantee
pub mod arch_wiki_corpus; // 6.13.0: Local embedded Arch Wiki snippets for concrete fixes
pub mod audio; // Audio system detection (PipeWire, Pulse, ALSA)
pub mod backup_detection; // Backup tool detection (timeshift, snapper, borg, restic)
pub mod beautiful;
pub mod boot; // Boot system detection (UEFI/BIOS, Secure Boot, bootloader)
// REMOVED in 6.57.0: caretaker_brain - legacy analysis engine with hardcoded rules
pub mod command_intelligence; // 6.15.0: Dynamic command derivation (CIL)
// REMOVED in 6.57.0: wiki_answer_engine - pattern-based shortcut answers
pub mod output_style; // 6.17.0: Professional output styling with capability detection
pub mod output_engine; // 6.31.0: Professional Output Engine v1 - Unified formatting
pub mod categories;
pub mod change_log; // Phase 5.1: Change logging and rollback
pub mod change_log_db; // Phase 5.1: SQLite persistence for change logs
pub mod change_recipe; // Phase 7: Safe change recipes with strict guardrails
pub mod change_recipe_display; // Phase 7: UI display for change recipes
pub mod command_meta;
// REMOVED in 6.57.0: command_recipe - legacy recipe system
pub mod config;
pub mod config_file; // Desktop config file parsing (Hyprland, i3, Sway)
pub mod config_parser;
pub mod container_virt_perf; // Container and virtualization performance
pub mod context;
pub mod cpu_performance; // CPU performance detection (governors, microcode, flags)
pub mod cpu_throttling; // CPU throttling and power state detection
pub mod desktop; // Desktop environment detection (Hyprland, i3, KDE, etc.)
pub mod desktop_automation; // Desktop automation helpers
pub mod disk_analysis;
pub mod display;
pub mod display_issues; // Display driver and multi-monitor issue detection
pub mod file_backup; // File backup system with SHA256 verification
pub mod file_index; // Beta.84: File-level indexing
pub mod filesystem; // Filesystem features detection (TRIM, LUKS, Btrfs)
pub mod filesystem_health; // Filesystem health detection
pub mod github_releases;
pub mod gpu_compute; // GPU compute capabilities (CUDA, OpenCL, ROCm, oneAPI)
pub mod gpu_throttling; // GPU throttling detection
pub mod graphics; // Graphics and display detection (Vulkan, OpenGL, session type)
pub mod hardware_capability; // Hardware capability detection for local LLM
pub mod historian; // Historian - Long-term memory and trend analysis system
pub mod insights_engine; // v6.24.0: Insights Engine - Historical Metrics & Trend Analysis
pub mod session_context; // v6.26.0: Session Context - Deep Context Memory
pub mod proactive_commentary; // v6.27.0: Proactive Commentary Engine
pub mod predictive_diagnostics; // v6.28.0: Predictive Diagnostics Engine
pub mod insight_summaries; // v6.29.0: Insight Summaries Engine
pub mod meta_telemetry; // v6.30.0: Meta Insight Telemetry
pub mod optimization_engine; // v6.30.0: Optimization Engine - Self-tuning profiles
pub mod reflection_engine; // v6.35.0: Reflection Engine - Anna's self-aware status block
pub mod progress_indicator; // v6.36.0: Progress Indicator
pub mod de_wm_detector; // v6.40.0: DE/WM Detector
pub mod system_info; // v6.41.0: Deterministic system information
pub mod system_report; // v6.41.0: System Report v2 - Fully deterministic

// === UNIFIED PIPELINE (v6.57.0) ===
pub mod llm_client; // v6.42.0: LLM Client abstraction
pub mod tool_inventory; // v6.42.0: Tool Inventory - available system tools
pub mod planner_core; // v6.41.0: Planner Core - LLM-driven command planning
pub mod executor_core; // v6.41.0: Executor Core - Safe command execution
pub mod interpreter_core; // v6.41.0: Interpreter Core - LLM-driven output interpretation
pub mod trace_renderer; // v6.41.0: Trace Renderer - Visible thinking trace
pub mod command_validator; // v6.44.0: Command Validation - safety rails
pub mod validation_loop; // v6.45.0: Multi-round LLM validation

// === TOOLCHAIN REALITY LOCK (v6.58.0) ===
pub mod strict_tool_catalog; // v6.58.0: Strict catalog of allowed commands
pub mod command_exec; // v6.58.0: Real command execution with honest output

// === UNIFIED TOOL CATALOG (v6.59.0) ===
pub mod tooling; // v6.59.0: Single source of truth for all tools Anna can execute

pub mod interactive_mode; // v6.46.0: Interactive Mode
pub mod greeting_engine; // v6.47.0: Greeting Engine
pub mod learning_engine; // v6.47.0: Learning Engine
pub mod telemetry_diff; // v6.47.0: Telemetry Diff
pub mod reality_check; // v6.48.0: Reality Check Engine
pub mod action_episodes; // v6.49.0: Episodic Action Log
pub mod rollback_engine; // v6.49.0: Rollback Engine
pub mod episode_storage; // v6.49.0: Episode Storage
pub mod execution_safety; // v6.50.0: Execution Safety
pub mod confirmation_ui; // v6.50.0: Confirmation UI
pub mod episode_recorder; // v6.50.0: Episode Recorder
pub mod post_validation; // v6.50.0: Post-Validation
pub mod change_journal; // v6.51.0: Change Journal
pub mod journal_query_intent; // v6.51.0: Journal Query Intent
pub mod journal_renderer; // v6.51.0: Journal Renderer
pub mod config_diff; // v6.51.0: Config Diff
pub mod policy_engine; // v6.52.0: Policy Engine
pub mod machine_identity; // v6.54.0: Machine identity
pub mod user_identity; // v6.54.0: User identity
pub mod knowledge_scope; // v6.54.0: System vs User scoped data
pub mod knowledge_domain; // v6.55.1: Knowledge domain types
pub mod knowledge_introspection; // v6.55.1: Knowledge introspection
pub mod knowledge_export; // v6.55.1: Knowledge export
pub mod knowledge_import; // v6.55.1: Knowledge import
pub mod knowledge_pruning; // v6.55.1: Knowledge pruning
pub mod self_stats; // v6.56.0: Self statistics

pub mod ignore_filters;
pub mod initramfs; // Initramfs configuration detection
pub mod insights; // Phase 5.2: Behavioral insights engine
pub mod installation_source;
pub mod ipc;
pub mod kernel_modules; // Kernel and boot detection
pub mod language; // Language system
pub mod wiki_adapter; // 6.9.0: Arch Wiki adaptation pipeline
// REMOVED in 6.57.0: answer_formatter - static answer templates
// REMOVED in 6.57.0: executor - legacy executor (superseded by executor_core)
// REMOVED in 6.57.0: orchestrator - hardcoded planners
// REMOVED in 6.57.0: selftest - legacy selftest depending on orchestrator
pub mod learning;
pub mod llm; // Task 12: LLM abstraction layer
pub mod log_noise_filter; // v6.39.0: Intelligent hardware error filtering
pub mod llm_benchmark; // Beta.68: LLM benchmarking harness
pub mod llm_context; // LLM Contextualization
pub mod llm_upgrade; // Step 3: Hardware upgrade detection
pub mod memory_usage; // Memory usage detection
pub mod model_profiles; // Data-driven model selection
pub mod network_config; // Network configuration detection
pub mod network_monitoring; // Network monitoring
pub mod ollama_installer; // Automatic local LLM bootstrap
pub mod orphaned_packages; // Orphaned package detection
pub mod package_health; // Package health detection
pub mod package_mgmt; // Package management configuration
pub mod paths;
pub mod personality; // Phase 5.1: Conversational personality controls
pub mod power; // Power and battery detection
pub mod prediction;
pub mod prediction_actions;
pub mod profile;
pub mod prompt_builder; // Phase 9: LLM prompt construction
pub mod qa_scenarios; // Beta.67: Real-world QA scenarios
// REMOVED in 6.57.0: recipe_executor - legacy recipe execution
// REMOVED in 6.57.0: recipe_planner - legacy recipe planning
// REMOVED in 6.57.0: recipe_validator - legacy recipe validation
pub mod reddit_qa_validator; // Beta.78: Reddit-based validation
pub mod rollback;
pub mod security; // Security configuration
pub mod security_features; // Security features
pub mod self_healing;
pub mod sensors; // Hardware sensors detection
pub mod storage; // Storage detection
pub mod suggestion_engine; // Task 8: Deep Caretaker v0.1 - Rule-based suggestions
pub mod suggestions; // Phase 5.1: Suggestion engine
pub mod system_health; // System health detection
pub mod system_knowledge; // 6.12.0: System Knowledge Base
pub mod systemd_health; // Systemd health detection
pub mod telemetry; // Telemetry structures from annad
// REMOVED in 6.57.0: template_library - hardcoded templates
pub mod terminal_format; // Phase 8: Beautiful terminal formatting
pub mod tlp_planner; // 6.13.0: TLP fix planner
pub mod trend_detectors; // Milestone 1.3: Trend-based detection
pub mod types;
pub mod updater;
pub mod user_behavior; // User behavior pattern detection
pub mod virtualization;
pub mod voltage_monitoring; // Voltage monitoring
pub mod wiki_llm; // 6.23.0: LLM integration for wiki reasoning
pub mod wiki_reasoner; // 6.23.0: Wiki Reasoning Engine
pub mod wiki_topics; // 6.23.0: Wiki topic classification

// v7.0.0: Clean brain architecture rewrite
pub mod brain_v7;

pub use advice_cache::*;
pub use beautiful::*;
pub use categories::*;
pub use config::*;
pub use config_parser::*;
pub use file_index::*;
pub use ignore_filters::*;
pub use ipc::*;
pub use paths::*;
pub use profile::*;
pub use rollback::*;
pub use types::*;
pub use updater::*;
