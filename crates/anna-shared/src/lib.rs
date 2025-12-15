//! Shared types and utilities for Anna components.
//! v0.0.73: Single source of truth for version via version module.
//! v0.0.74: Model selector with Qwen3-VL preference.
//! v0.0.75: UX realism, stats/RPG backend, recipes learned, citations.
//! v0.0.408: Research-first knowledge system with evidence-based answers.
//! v0.0.410: Evidence pipeline - real knowledge engine (probes, docs, wiki).
//! v0.0.414: Doc-first reasoning with citations and honesty tracking.
//! v0.0.415: Strict specialist contract - fast, honest, grounded answers.
//! v0.0.416: Knowledge engine and self-learning recipes.
//! v0.0.417: Strict reliability - direct answers only, no tutorials.
//! v0.0.418: Full recipe learning system - learn from tickets, execute without LLM.
//! v0.0.420: Recipe V2 - Clean learning engine with global/user recipes.
//! v0.0.421: Specialist V2 - Stable, schema-driven responses, no parse errors.
//! v0.0.422: Knowledge V2 - Research-first layer (Arch Wiki, man pages, help).
//! v0.0.423: Recipe V3 - Safe learning/execution engine with preconditions and risk levels.
//! v0.0.424: Knowledge V4 - Complete local knowledge engine with citations.
//! v0.0.425: Specialist V3 - Strict JSON contract, robust parser, no parse errors.
//! v0.0.426: Strict ticket lifecycle, honest metrics, reality-based stats.
//! v0.0.427: Self-learning recipe engine with evidence-based matching.
//! v0.0.428: Strict specialist protocol, no-bullshit policy, honest stats.
//! v0.0.429: Documentation brain - Arch Wiki, man pages, help as local knowledge graph.
//! v0.0.430: Background workers, idle-time learning, alerts, and long-running tickets.
//! v0.0.431: Hollywood UX - Unified transcript and terminal renderer.
//! v0.0.432: Knowledge pipeline - Priority-ordered knowledge fetching and learning.
//! v0.0.433: Robustness layer - Timeouts, failure handling, and truthful stats.
//! v0.0.434: Hardware-aware model selection, local model management, and helper installation.
//! v0.0.435: Evidence-first knowledge engine - citations, probe primitives, recipes with promotion.
//! v0.0.436: Anna Protocol v1 - Unbreakable typed JSON communication, no more parse errors.
//! v0.0.437: Question Contract - Fix understanding and answer minimality.
//! v0.0.438: Fast Pipeline - Hard budgets, no streaming for parsed calls, reliability stats.
//! v0.0.439: Deterministic Routing - Fix "Sofia handles everything" with intent-to-department map.
//! v0.0.440: Specialist Contract v1 - Strict JSON schema, retries, fallback summarizer.
//! v0.0.441: ERA Pipeline - Universal Evidence → Reasoning → Answer architecture.
//! v0.0.442: Ticket Integrity - Honest stats, clarification-first, package/system separation.
//! v0.0.443: Source Layer - Citations, trace observability, clean inventories, honest stats/UI.
//! v0.0.444: Reliability Metrics - Canonical outcomes, honest stats, accurate inventories.
//! v0.0.444: Debug Mode - 3-level debug output, sanitization, reason codes, routing transparency.
//! v0.0.445: Hard Reliability Gate - No answer without evidence, no fake success.
//! v0.0.448: Deterministic Probes - Intent-specific probe mapping, no LLM guessing.

pub mod advice;
pub mod anna_proto; // v0.0.436: Unbreakable typed model communication protocol
pub mod answer_contract;
pub mod answer_shaper; // v0.0.415: Shape answers for users
pub mod background_worker; // v0.0.430: Background job system
pub mod brief;
pub mod budget;
pub mod canonical_intents; // v0.0.416: Canonical intents and topics
pub mod change;
pub mod change_history;
pub mod change_transaction;
pub mod claims;
pub mod clarify;
pub mod clarify_v2;
pub mod comms_render; // v0.0.407: Internal comms rendering from ticket state
pub mod config_intent;
pub mod config_parser; // v0.0.236
pub mod config_seed_recipes; // v0.0.264: Seed recipes for editor configs
pub mod config_types; // v0.0.264: Config types (ConfigTarget, ConfigIntent)
pub mod context_memory; // v0.0.246
pub mod cron_recipes; // v0.0.234
pub mod database_recipes; // v0.0.461: Database backup/restore recipes
pub mod debug_config; // v0.0.473: Debug configuration via natural language
pub mod debug_mode; // v0.0.444: Debug levels, sanitization, reason codes
pub mod kubernetes_recipes; // v0.0.459: Kubernetes pod/deployment recipes
pub mod deterministic_probes; // v0.0.448: Intent → probes deterministic mapping
pub mod deterministic_routing; // v0.0.439: Deterministic routing with intent-to-department map
pub mod distro_utils; // v0.0.383: Distro-aware package recommendations
pub mod doc_brain; // v0.0.406: Unified doc search (man pages, wiki, help)
pub mod doc_engine; // v0.0.429: Documentation brain - local knowledge graph
pub mod doc_fetcher; // v0.0.410: Enhanced doc fetchers
pub mod doc_first_workflow; // v0.0.414: Doc-first specialist reasoning
pub mod doc_search; // v0.0.408: Local documentation search
pub mod doc_snippet; // v0.0.412: Documentation source integration
pub mod docker_recipes; // v0.0.235
pub mod editor_recipe_data;
pub mod editor_recipes;
pub mod email; // v0.0.113
pub mod era_pipeline; // v0.0.441: ERA Pipeline - Evidence → Reasoning → Answer
pub mod error;
pub mod error_output; // v0.0.407: User-friendly error messages
pub mod evidence_engine; // v0.0.410: Evidence engine core types
pub mod evidence_first; // v0.0.435: Evidence-first knowledge engine
pub mod evidence_gatherer; // v0.0.410: Evidence orchestration
pub mod evidence_pipeline; // v0.0.410: Full evidence integration
pub mod facts;
pub mod facts_types;
pub mod facts_maintenance; // v0.0.471: Facts lifecycle maintenance
pub mod fast_pipeline; // v0.0.438: Fast Pipeline - hard budgets, no streaming, reliability stats
pub mod fastpath;
pub mod followup_hints; // v0.0.384: Context-aware follow-up suggestions
pub mod git_recipes;
pub mod greeting_insights; // v0.0.245
pub mod grounding;
pub mod guard;
pub mod hardware_aware; // v0.0.434: Hardware-aware model selection and helper management
pub mod health_brief;
pub mod health_delta;
pub mod health_tips; // v0.0.244
pub mod health_view;
pub mod helpers;
pub mod hollywood_ux; // v0.0.431: Unified transcript and Hollywood terminal renderer
pub mod honest_metrics; // v0.0.426: Reality-based stats (no fake 100%)
pub mod honest_stats; // v0.0.415: Honest stats tracking
pub mod idle_tips; // v0.0.240
pub mod intake;
pub mod intent_handlers; // v0.0.417: Deterministic intent handlers
pub mod intent_policy; // v0.0.414: Intent-based routing (no hardcoded NL)
pub mod inventory;
pub mod knowledge;
pub mod knowledge_config; // v0.0.414: Knowledge source configuration
pub mod knowledge_engine; // v0.0.416: Knowledge engine (man, help, wiki)
pub mod knowledge_executor; // v0.0.414: Knowledge query executor
pub mod knowledge_index; // v0.0.410: Compiled knowledge store
pub mod knowledge_item; // v0.0.408: Knowledge item abstraction
pub mod knowledge_learning; // v0.0.414: Self-learning from docs and tickets
pub mod knowledge_pipeline; // v0.0.432: Priority-ordered knowledge fetching and learning
pub mod knowledge_query; // v0.0.414: Doc-first knowledge query interface
pub mod knowledge_v2; // v0.0.422: Research-first knowledge layer
pub mod knowledge_v4; // v0.0.424: Complete local knowledge engine with citations
pub mod learned_recipes; // v0.0.416: Self-learning recipe schema
pub mod learning_engine; // v0.0.427: Self-learning recipe engine with evidence-based matching
pub mod learning_explanations; // v0.0.457: Learning mode command explanations
pub mod ledger;
pub mod llm_parse; // v0.0.407: Strict LLM JSON parsing with error handling
pub mod long_task; // v0.0.455: Long-running task detection and handling
pub mod model_registry;
pub mod model_selector;
pub mod narrator;
pub mod network_recipes; // v0.0.462: Network troubleshooting recipes
pub mod notification_config; // v0.0.470: Notification config via natural language
pub mod package_recipes;
pub mod parsers;
pub mod pending;
pub mod person_stats;
pub mod probe_registry; // v0.0.410: Composable probe definitions
pub mod probe_spine;
pub mod progress;
pub mod question_contract; // v0.0.437: Question Contract - typed intent and answer shape enforcement
pub mod recipe;
pub mod recipe_candidate; // v0.0.408: Recipe candidate storage for learning
pub mod recipe_converter; // v0.0.412: Ticket-to-recipe conversion
pub mod recipe_eligibility; // v0.0.418: Recipe learning eligibility checker
pub mod recipe_engine; // v0.0.412: Self-learning recipe system
pub mod recipe_exec_helpers; // v0.0.412: Execution helper functions
pub mod recipe_executor; // v0.0.412: Recipe execution engine
pub mod recipe_extractor; // v0.0.418: Extract recipes from tickets
pub mod recipe_fast_path; // v0.0.416: Recipe execution before specialists
pub mod recipe_feedback;
pub mod recipe_file; // v0.0.406: TOML-based authored recipes
pub mod recipe_index;
pub mod recipe_learner; // v0.0.416: Recipe learning engine
pub mod recipe_learning;
pub mod recipe_matcher;
pub mod recipe_matcher_v2; // v0.0.418: Runtime recipe matching
pub mod recipe_runtime; // v0.0.418: Recipe execution engine
pub mod recipe_schema; // v0.0.418: Recipe data model
pub mod recipe_stats; // v0.0.416: Recipe usage stats
pub mod recipe_storage; // v0.0.418: Recipe file storage and indexing
pub mod recipe_store_v2; // v0.0.412: Persistent recipe storage
pub mod recipe_telemetry; // v0.0.418: Recipe usage telemetry
pub mod recipe_templates; // v0.0.412: Generic parameterized recipes
pub mod recipe_v2; // v0.0.420: Clean learning engine with global/user recipes
pub mod recipe_v3; // v0.0.423: Safe learning/execution engine with risk levels
pub mod regression_tests; // v0.0.415: Shape validation tests
pub mod reliability;
pub mod reliability_gate; // v0.0.445: Hard reliability gate, claim/evidence model, deterministic-first
pub mod reliability_metrics; // v0.0.444: Canonical outcomes, real reliability stats
pub mod repl_greeting; // v0.0.413: Stats-based REPL greeting
pub mod report;
pub mod resource_limits;
pub mod risk_config; // v0.0.474: Risk level configuration via natural language
pub mod review;
pub mod review_gate;
pub mod review_prompts;
pub mod revision;
pub mod robustness; // v0.0.433: Timeouts, failure handling, and truthful stats
pub mod roster;
pub mod rpc;
pub mod seed_recipes; // v0.0.418: Initial seed recipes
pub mod senior_strategic; // v0.0.458: Senior idle-time strategic thinking
pub mod service_recipes;
pub mod session_display; // v0.0.475: Session summary display
pub mod settings_display; // v0.0.476: Unified settings display
pub mod shell_recipes;
pub mod snapshot;
pub mod solver_prompts; // v0.0.408: Evidence-focused solver prompts
pub mod source_layer; // v0.0.443: Source providers, citations, trace, inventories
pub mod specialist_contract_v1; // v0.0.440: Specialist Contract v1 - strict JSON, retries, fallback
pub mod specialist_protocol; // v0.0.428: Strict specialist protocol, no-bullshit policy, honest stats
pub mod specialist_response; // v0.0.409: Unified specialist response schema
pub mod specialist_v2; // v0.0.421: Stable, schema-driven specialist responses
pub mod specialist_v3; // v0.0.425: Strict JSON contract, robust parser, no parse errors
pub mod specialists;
pub mod ssh_recipes;
pub mod stats;
pub mod status;
pub mod status_snapshot;
pub mod strict_contract; // v0.0.415: Strict specialist JSON contract
pub mod strict_prompts; // v0.0.415: Strict specialist prompts
pub mod systemd_recipes; // v0.0.233
pub mod teams;
pub mod telemetry;
pub mod ticket;
pub mod ticket_integrity; // v0.0.442: Honest stats, clarification-first, package/system separation
pub mod ticket_lifecycle; // v0.0.426: Strict ticket lifecycle state machine
pub mod ticket_log; // v0.0.406: Structured ticket logs for learning
pub mod ticket_packet;
pub mod ticket_state; // v0.0.407: Explicit ticket lifecycle and states
pub mod ticket_stats; // v0.0.407: Truthful ticket statistics
pub mod trace;
pub mod transcript;
pub mod transcript_ext;
pub mod transcript_render; // v0.0.413: Cinematic/debug transcript renderer
pub mod transcript_segment; // v0.0.413: Transcript segment data model
pub mod translator_contract; // v0.0.415: Strict translator schema
pub mod truth_ledger;
pub mod ui;
pub mod ui_config; // v0.0.413: UI configuration (mode, spinner, etc.)
pub mod update_ledger;
pub mod user_alarms; // v0.0.456: Natural language alarms and reminders
pub mod preference_config; // v0.0.467: Natural language preference configuration
pub mod verify;
pub mod webserver_recipes; // v0.0.460: Nginx/Apache configuration recipes
pub mod wiki_cache; // v0.0.472: Arch Wiki local caching
pub mod xp_display; // v0.0.478: XP/Level RPG-style progression display
pub mod fun_stats_display; // v0.0.479: Fun statistics display
pub mod capabilities_display; // v0.0.480: Capabilities display
pub mod query_type_router; // v0.0.481: Query type router
pub mod contextual_tips; // v0.0.482: Contextual tips system
pub mod command_shortcuts; // v0.0.483: Command shortcuts
pub mod quick_status; // v0.0.484: Quick status summary
pub mod repeated_questions; // v0.0.485: Repeated questions detection
pub mod response_length; // v0.0.486: Response length tracking
pub mod resolution_time; // v0.0.487: Resolution time tracking
pub mod interaction_counter; // v0.0.488: Interaction counter
pub mod expert_stats; // v0.0.489: Expert ticket statistics
pub mod recipe_stats_display; // v0.0.490: Recipe statistics display
pub mod stats_dashboard; // v0.0.491: Aggregated stats dashboard
pub mod uptime_tracker; // v0.0.492: Uptime tracking
pub mod ticket_history_display; // v0.0.493: Ticket history display
pub mod error_summary_display; // v0.0.494: Error summary display
pub mod team_performance_display; // v0.0.495: Team performance display
pub mod anna_progress_report; // v0.0.496: Anna progress report
pub mod user_activity_summary; // v0.0.497: User activity summary
pub mod system_health_score; // v0.0.498: System health score
pub mod knowledge_base_stats; // v0.0.499: Knowledge base stats
pub mod boot_time_tracking; // v0.0.500: Boot time tracking
pub mod command_execution_log; // v0.0.501: Command execution logging
pub mod specialist_conversation; // v0.0.502: Specialist conversation display
pub mod backup_history; // v0.0.503: Backup history tracking
pub mod package_install_tracker; // v0.0.504: Package installation tracker
pub mod service_management_tracker; // v0.0.505: Service management tracker
pub mod config_change_tracker; // v0.0.506: Config change tracker
pub mod helper_tracker; // v0.0.507: Helper tool tracker
pub mod email_notification; // v0.0.508: Email notification system
pub mod idle_time_detector; // v0.0.509: Idle time detector
pub mod ticket_resolution_stats; // v0.0.510: Ticket resolution stats
pub mod specialist_roster; // v0.0.511: Specialist roster
pub mod llm_assignment; // v0.0.512: LLM assignment tracker
pub mod dialogue_renderer; // v0.0.513: Dialogue renderer
pub mod alarm_scheduler; // v0.0.514: Alarm scheduler
pub mod strategic_thinking; // v0.0.515: Strategic thinking tracker
pub mod hardware_capability; // v0.0.516: Hardware capability detector
pub mod dependency_tracker; // v0.0.517: Dependency tracker
pub mod session_history; // v0.0.518: Session history tracker
pub mod query_pattern_analyzer; // v0.0.519: Query pattern analyzer
pub mod resource_usage_tracker; // v0.0.520: Resource usage tracker
pub mod error_recovery_tracker; // v0.0.521: Error recovery tracker
pub mod user_preference_learner; // v0.0.522: User preference learner
pub mod task_priority_manager; // v0.0.523: Task priority manager
pub mod anna_metrics_dashboard; // v0.0.524: Anna metrics dashboard (Phase 100!)
pub mod workflow_automation; // v0.0.525: Workflow automation tracker
pub mod context_memory_store; // v0.0.526: Context memory store
pub mod skill_proficiency; // v0.0.527: Skill proficiency tracker
pub mod team_specialist_roster; // v0.0.528: Team specialist roster
pub mod escalation_tracker; // v0.0.529: Escalation tracker
pub mod knowledge_citation; // v0.0.530: Knowledge citation tracker
pub mod llm_model_registry; // v0.0.531: LLM model registry
pub mod helper_install_tracker; // v0.0.532: Helper install tracker
pub mod notification_tracker; // v0.0.533: Notification tracker
pub mod long_task_manager; // v0.0.534: Long task manager
pub mod greeting_generator; // v0.0.535: Greeting generator
pub mod display_mode_manager; // v0.0.536: Display mode manager
pub mod query_history_tracker; // v0.0.537: Query history tracker
pub mod response_time_tracker; // v0.0.538: Response time tracker
pub mod team_consultation_tracker; // v0.0.539: Team consultation tracker
pub mod installation_tracker; // v0.0.540: Installation date tracker
pub mod tips_system; // v0.0.541: Tips system
pub mod personality_config; // v0.0.542: Personality config
pub mod risk_level_config; // v0.0.543: Risk level config
pub mod learning_mode_config; // v0.0.544: Learning mode config
pub mod escalation_policy_config; // v0.0.545: Escalation policy config
pub mod verbosity_config; // v0.0.546: Verbosity config
pub mod confirmation_behavior_config; // v0.0.547: Confirmation behavior config
pub mod timeout_config; // v0.0.548: Timeout config
pub mod output_style_config; // v0.0.549: Output style config
pub mod privacy_config; // v0.0.550: Privacy config
pub mod backup_config; // v0.0.551: Backup config
pub mod update_config; // v0.0.552: Update config
pub mod model_config; // v0.0.553: Model config
pub mod unified_settings; // v0.0.554: Unified settings manager
pub mod settings_persistence; // v0.0.555: Settings persistence
pub mod settings_migration; // v0.0.556: Settings migration
pub mod settings_validation; // v0.0.557: Settings validation
pub mod settings_export; // v0.0.558: Settings export/import
pub mod settings_cli; // v0.0.559: Settings CLI interface
pub mod settings_watcher; // v0.0.560: Settings watcher
pub mod settings_diff; // v0.0.561: Settings diff
pub mod settings_presets; // v0.0.562: Settings presets
pub mod settings_history; // v0.0.563: Settings history
pub mod settings_sync; // v0.0.564: Settings sync
pub mod settings_profiles; // v0.0.565: Settings profiles
pub mod settings_search; // v0.0.566: Settings search
pub mod settings_notifications; // v0.0.567: Settings notifications
pub mod settings_scheduler; // v0.0.568: Settings scheduler
pub mod settings_templates; // v0.0.569: Settings templates
pub mod settings_constraints; // v0.0.570: Settings constraints
pub mod settings_hooks; // v0.0.571: Settings hooks
pub mod settings_wizard; // v0.0.572: Settings wizard
pub mod settings_audit; // v0.0.573: Settings audit
pub mod settings_orchestrator; // v0.0.574: Settings orchestrator
pub mod settings_backup; // v0.0.575: Settings backup manager
pub mod settings_restore; // v0.0.576: Settings restore
pub mod settings_analytics; // v0.0.577: Settings analytics
pub mod settings_recommendations; // v0.0.578: Settings recommendations
pub mod settings_dashboard; // v0.0.579: Settings dashboard
pub mod settings_api; // v0.0.580: Settings API
pub mod settings_events; // v0.0.581: Settings events
pub mod settings_permissions; // v0.0.582: Settings permissions
pub mod settings_diagnostics; // v0.0.583: Settings diagnostics
pub mod settings_metrics; // v0.0.584: Settings metrics
pub mod settings_logging; // v0.0.585: Settings logging
pub mod settings_cache; // v0.0.586: Settings cache
pub mod settings_transactions; // v0.0.587: Settings transactions
pub mod settings_versioning; // v0.0.588: Settings versioning
pub mod settings_throttling; // v0.0.589: Settings throttling
pub mod settings_middleware; // v0.0.590: Settings middleware
pub mod settings_observer; // v0.0.591: Settings observer
pub mod settings_snapshot; // v0.0.592: Settings snapshot
pub mod settings_lock; // v0.0.593: Settings lock
pub mod settings_encryption; // v0.0.594: Settings encryption
pub mod settings_inheritance; // v0.0.595: Settings inheritance
pub mod settings_query; // v0.0.596: Settings query
pub mod settings_validator_chain; // v0.0.597: Settings validator chain
pub mod settings_transformer; // v0.0.598: Settings transformer
pub mod settings_resolver; // v0.0.599: Settings resolver
pub mod settings_aggregator; // v0.0.600: Settings aggregator
pub mod settings_comparator; // v0.0.601: Settings comparator
pub mod settings_serializer; // v0.0.602: Settings serializer
pub mod settings_router; // v0.0.603: Settings router
pub mod settings_compiler; // v0.0.604: Settings compiler
pub mod settings_linker; // v0.0.605: Settings linker
pub mod settings_bundler; // v0.0.606: Settings bundler
pub mod settings_deployer; // v0.0.607: Settings deployer
pub mod settings_monitor; // v0.0.608: Settings monitor
pub mod settings_reporter; // v0.0.609: Settings reporter
pub mod settings_task_scheduler; // v0.0.610: Settings task scheduler
pub mod settings_queue; // v0.0.611: Settings queue
pub mod settings_worker; // v0.0.612: Settings worker
pub mod settings_executor; // v0.0.613: Settings executor
pub mod settings_pipeline; // v0.0.614: Settings pipeline
pub mod settings_processor; // v0.0.615: Settings processor
pub mod settings_handler; // v0.0.616: Settings handler
pub mod settings_dispatcher; // v0.0.617: Settings dispatcher
pub mod settings_coordinator; // v0.0.618: Settings coordinator
pub mod settings_controller; // v0.0.619: Settings controller
pub mod settings_service; // v0.0.620: Settings service
pub mod settings_manager; // v0.0.621: Settings manager
pub mod settings_registry; // v0.0.622: Settings registry
pub mod settings_index; // v0.0.623: Settings index
pub mod settings_catalog; // v0.0.624: Settings catalog
pub mod settings_gateway; // v0.0.625: Settings gateway
pub mod settings_proxy; // v0.0.626: Settings proxy
pub mod settings_facade; // v0.0.627: Settings facade
pub mod settings_adapter; // v0.0.628: Settings adapter
pub mod settings_bridge; // v0.0.629: Settings bridge
pub mod settings_connector; // v0.0.630: Settings connector
pub mod settings_provider; // v0.0.631: Settings provider
pub mod settings_consumer; // v0.0.632: Settings consumer
pub mod settings_subscriber; // v0.0.633: Settings subscriber
pub mod settings_publisher; // v0.0.634: Settings publisher
pub mod settings_broadcaster; // v0.0.635: Settings broadcaster
pub mod settings_listener; // v0.0.636: Settings listener
pub mod settings_poller; // v0.0.637: Settings poller
pub mod settings_tracker; // v0.0.638: Settings tracker
pub mod settings_notifier; // v0.0.639: Settings notifier
pub mod settings_report_generator; // v0.0.640: Settings report generator
pub mod settings_inspector; // v0.0.641: Settings inspector
pub mod settings_analyzer; // v0.0.642: Settings analyzer
pub mod settings_sanitizer; // v0.0.643: Settings sanitizer
pub mod settings_formatter; // v0.0.644: Settings formatter
pub mod settings_normalizer; // v0.0.645: Settings normalizer
pub mod settings_parser; // v0.0.646: Settings parser
pub mod settings_renderer; // v0.0.647: Settings renderer
pub mod settings_encoder; // v0.0.648: Settings encoder
pub mod settings_decoder; // v0.0.649: Settings decoder
pub mod settings_converter; // v0.0.650: Settings converter
pub mod settings_mapper; // v0.0.651: Settings mapper
pub mod settings_binder; // v0.0.652: Settings binder
pub mod settings_extractor; // v0.0.653: Settings extractor
pub mod settings_injector; // v0.0.654: Settings injector
pub mod settings_merger; // v0.0.655: Settings merger
pub mod settings_splitter; // v0.0.656: Settings splitter
pub mod settings_cloner; // v0.0.657: Settings cloner
pub mod settings_archiver; // v0.0.658: Settings archiver
pub mod settings_restorer; // v0.0.659: Settings restorer
pub mod settings_versioner; // v0.0.660: Settings versioner
pub mod settings_differ; // v0.0.661: Settings differ
pub mod settings_patcher; // v0.0.662: Settings patcher
pub mod settings_graph; // v0.0.663: Settings graph
pub mod settings_resolution; // v0.0.664: Settings resolution
pub mod settings_validator_hub; // v0.0.665: Settings validator hub
pub mod settings_transform; // v0.0.666: Settings transform
pub mod settings_normalization; // v0.0.667: Settings normalization
pub mod settings_denormalization; // v0.0.668: Settings denormalization
pub mod settings_indexer; // v0.0.669: Settings indexer
pub mod settings_query_engine; // v0.0.670: Settings query engine
pub mod settings_aggregation; // v0.0.671: Settings aggregation
pub mod settings_projector; // v0.0.672: Settings projector
pub mod settings_selector; // v0.0.673: Settings selector
pub mod settings_filter; // v0.0.674: Settings filter
pub mod settings_sorter; // v0.0.675: Settings sorter
pub mod settings_grouper; // v0.0.676: Settings grouper
pub mod settings_reducer; // v0.0.677: Settings reducer
pub mod settings_partitioner; // v0.0.678: Settings partitioner
pub mod settings_flattener; // v0.0.679: Settings flattener
pub mod settings_expander; // v0.0.680: Settings expander
pub mod settings_iterator; // v0.0.681: Settings iterator
pub mod settings_collector; // v0.0.682: Settings collector
pub mod settings_zipper; // v0.0.683: Settings zipper
pub mod settings_scanner; // v0.0.684: Settings scanner
pub mod settings_finder; // v0.0.685: Settings finder
pub mod settings_counter; // v0.0.686: Settings counter
pub mod settings_matcher; // v0.0.687: Settings matcher
pub mod settings_validator; // v0.0.688: Settings validator
pub mod settings_comparer; // v0.0.689: Settings comparer
pub mod settings_combiner; // v0.0.690: Settings combiner
pub mod settings_auditor; // v0.0.691: Settings auditor
pub mod settings_chronicle; // v0.0.692: Settings chronicle
pub mod settings_ledger; // v0.0.693: Settings ledger
pub mod settings_diary; // v0.0.694: Settings diary
pub mod settings_folio; // v0.0.695: Settings folio
pub mod settings_album; // v0.0.696: Settings album
pub mod settings_dossier; // v0.0.697: Settings dossier
pub mod settings_portfolio; // v0.0.698: Settings portfolio
pub mod settings_catalog_v2; // v0.0.699: Settings catalog v2
pub mod settings_compendium; // v0.0.700: Settings compendium (Milestone!)
pub mod settings_anthology; // v0.0.701: Settings anthology
pub mod settings_archive_v2; // v0.0.702: Settings archive v2
pub mod settings_repertoire; // v0.0.703: Settings repertoire
pub mod settings_gazette; // v0.0.704: Settings gazette
pub mod settings_almanac; // v0.0.705: Settings almanac
pub mod settings_bulletin; // v0.0.706: Settings bulletin
pub mod version;

// v0.0.67: Service desk narrative modules
pub mod citations;
pub mod render;
pub mod stats_store;

// v0.0.75: UX realism + stats/RPG + recipes + citations
pub mod citation;
pub mod event_log;
pub mod presentation;
pub mod recipe_store;
pub mod result_signals;

// v0.0.81: Service Desk Theatre - cinematic narrative
pub mod theatre;

// v0.0.86: Streak calculations for stats/RPG
pub mod streaks;

// v0.0.454: Dynamic team availability based on hardware
pub mod team_availability;

// v0.0.87: Dialogue variety for theatre
pub mod dialogue;

// v0.0.89: Personalized greetings and context-aware dialogue
pub mod greetings;

// v0.0.90: Achievement badges for stats/RPG
pub mod achievements;

// v0.0.105: Service Desk Foundation - tickets and user profiles
pub mod ticket_tracker;
pub mod user_profile;

// v0.0.107: Staff performance tracking
pub mod staff_stats;

// v0.0.256: Synonym expansion for recipe matching
pub mod synonyms;

// v0.0.257: Desktop configuration recipes (wallpaper, themes)
pub mod desktop_recipes;

// v0.0.258: Pending ticket retry queue
pub mod pending_queue;

// v0.0.268: Query scenario test corpus (100+ queries)
pub mod query_scenarios;

// v0.0.275: LLM-generated greeting context
pub mod greeting_context;
pub mod greeting_tips; // v0.0.468: Configuration tips in greetings

// v0.0.280: System telemetry tracking
pub mod system_telemetry;
pub mod system_monitors; // v0.0.469: Proactive system monitoring

// v0.0.281: Proactive health alerts
pub mod health_alerts;

// v0.0.282: LLM-based recipe similarity scoring
pub mod recipe_similarity;

// v0.0.282: Idle-time learning suggestions
pub mod learning_suggestions;

// v0.0.286: Proactive maintenance actions
pub mod maintenance_actions;

// v0.0.288: Learning progress tracking
pub mod learning_progress;

// v0.0.289: Interesting facts for greetings
pub mod interesting_facts;

// v0.0.322: Probe effectiveness learning
pub mod probe_learning;

// v0.0.401: Specialist knowledge capture
pub mod clarification_learning;
pub mod learning_stats; // v0.0.401: Learning progress statistics
pub mod specialist_learning;
pub mod specialist_patterns; // v0.0.401: Generic pattern matching
pub mod specialist_recipes; // v0.0.401: Recipes from specialist lessons // v0.0.401: Learning from user clarifications

// v0.0.404: JSON-only specialist contract
pub mod specialist_contract;

pub use error::AnnaError;
pub use ledger::{Ledger, LedgerEntry, LedgerEntryKind};
pub use rpc::{
    Capabilities, DaemonInfo, HardwareSummary, ProbeParams, ProbeType, RpcMethod, RpcRequest,
    RpcResponse, RuntimeContext,
};
pub use status::{
    BenchmarkResult, DaemonState, DaemonStatus, HardwareInfo, LlmState, LlmStatus, ModelInfo,
    OllamaStatus, ProgressInfo, UpdateStatus,
};
// v0.0.73: Re-export version constants for backward compatibility
pub use version::{VersionInfo, BUILD_DATE, GIT_SHA, PROTOCOL_VERSION, VERSION};

/// Socket path for annad (use socket_path() for env override support)
pub const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// State directory for Anna (use state_dir() for env override support)
pub const STATE_DIR: &str = "/var/lib/anna";

/// Get socket path with env override support (ANNA_SOCKET)
pub fn socket_path() -> String {
    std::env::var("ANNA_SOCKET").unwrap_or_else(|_| SOCKET_PATH.to_string())
}

/// Get state directory with env override support (ANNA_STATE_DIR)
pub fn state_dir() -> String {
    std::env::var("ANNA_STATE_DIR").unwrap_or_else(|_| STATE_DIR.to_string())
}

/// Ledger file path
pub const LEDGER_PATH: &str = "/var/lib/anna/ledger.json";

/// Config file path
pub const CONFIG_PATH: &str = "/var/lib/anna/config.json";

/// Update check interval in seconds (default, can be overridden by config)
pub const DEFAULT_UPDATE_CHECK_INTERVAL: u64 = 60;

/// GitHub repository for version checks
pub const GITHUB_REPO: &str = "jjgarcianorway/anna-assistant";
