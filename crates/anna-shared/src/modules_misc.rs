//! Miscellaneous module declarations
//! Tracking, monitoring, and utility modules

// Background and task management
#[path = "background_worker/mod.rs"]
pub mod background_worker; // v0.0.430: Background job system
#[path = "long_task.rs"]
pub mod long_task; // v0.0.455: Long-running task detection and handling
#[path = "long_task_manager/mod.rs"]
pub mod long_task_manager; // v0.0.534: Long task manager (modularized)
#[path = "task_priority_manager/mod.rs"]
pub mod task_priority_manager; // v0.0.523: Task priority manager

// Model management
#[path = "model_registry/mod.rs"]
pub mod model_registry;
#[path = "model_selector/mod.rs"]
pub mod model_selector;
#[path = "llm_model_registry/mod.rs"]
pub mod llm_model_registry; // v0.0.531: LLM model registry
#[path = "llm_assignment/mod.rs"]
pub mod llm_assignment; // v0.0.512: LLM assignment tracker

// Hardware and system
#[path = "hardware_aware/mod.rs"]
pub mod hardware_aware; // v0.0.434: Hardware-aware model selection and helper management
#[path = "hardware_capability/mod.rs"]
pub mod hardware_capability; // v0.0.516: Hardware capability detector
#[path = "distro_utils.rs"]
pub mod distro_utils; // v0.0.383: Distro-aware package recommendations
#[path = "system_monitors/mod.rs"]
pub mod system_monitors; // v0.0.469: Proactive system monitoring

// Tracking modules
#[path = "boot_time_tracking/mod.rs"]
pub mod boot_time_tracking; // v0.0.500: Boot time tracking
#[path = "command_execution_log/mod.rs"]
pub mod command_execution_log; // v0.0.501: Command execution logging
#[path = "backup_history/mod.rs"]
pub mod backup_history; // v0.0.503: Backup history tracking
#[path = "package_install_tracker/mod.rs"]
pub mod package_install_tracker; // v0.0.504: Package installation tracker (modularized)
#[path = "service_management_tracker/mod.rs"]
pub mod service_management_tracker; // v0.0.505: Service management tracker
#[path = "config_change_tracker/mod.rs"]
pub mod config_change_tracker; // v0.0.506: Config change tracker
#[path = "helper_tracker/mod.rs"]
pub mod helper_tracker; // v0.0.507: Helper tool tracker (modularized)
#[path = "helper_install_tracker/mod.rs"]
pub mod helper_install_tracker; // v0.0.532: Helper install tracker (modularized)
#[path = "dependency_tracker/mod.rs"]
pub mod dependency_tracker; // v0.0.517: Dependency tracker
#[path = "resource_usage_tracker.rs"]
pub mod resource_usage_tracker; // v0.0.520: Resource usage tracker
#[path = "error_recovery_tracker/mod.rs"]
pub mod error_recovery_tracker; // v0.0.521: Error recovery tracker (modularized)
#[path = "escalation_tracker/mod.rs"]
pub mod escalation_tracker; // v0.0.529: Escalation tracker (modularized)
#[path = "notification_tracker/mod.rs"]
pub mod notification_tracker; // v0.0.533: Notification tracker
#[path = "query_history_tracker/mod.rs"]
pub mod query_history_tracker; // v0.0.537: Query history tracker
#[path = "response_time_tracker/mod.rs"]
pub mod response_time_tracker; // v0.0.538: Response time tracker
#[path = "team_consultation_tracker/mod.rs"]
pub mod team_consultation_tracker; // v0.0.539: Team consultation tracker
#[path = "installation_tracker.rs"]
pub mod installation_tracker; // v0.0.540: Installation date tracker
#[path = "workflow_automation/mod.rs"]
pub mod workflow_automation; // v0.0.525: Workflow automation tracker

// Detection and analysis
#[path = "idle_time_detector/mod.rs"]
pub mod idle_time_detector; // v0.0.509: Idle time detector
#[path = "repeated_questions/mod.rs"]
pub mod repeated_questions; // v0.0.485: Repeated questions detection
#[path = "query_pattern_analyzer/mod.rs"]
pub mod query_pattern_analyzer; // v0.0.519: Query pattern analyzer

// Counters and metrics
#[path = "interaction_counter/mod.rs"]
pub mod interaction_counter; // v0.0.488: Interaction counter
#[path = "response_length/mod.rs"]
pub mod response_length; // v0.0.486: Response length tracking
#[path = "resolution_time/mod.rs"]
pub mod resolution_time; // v0.0.487: Resolution time tracking

// Notifications and alerts
#[path = "email_notification/mod.rs"]
pub mod email_notification; // v0.0.508: Email notification system
#[path = "user_alarms/mod.rs"]
pub mod user_alarms; // v0.0.456: Natural language alarms and reminders
#[path = "alarm_scheduler/mod.rs"]
pub mod alarm_scheduler; // v0.0.514: Alarm scheduler

// Context and memory
#[path = "context_memory.rs"]
pub mod context_memory; // v0.0.246
#[path = "context_memory_store/mod.rs"]
pub mod context_memory_store; // v0.0.526: Context memory store (modularized v0.0.555)
#[path = "session_history/mod.rs"]
pub mod session_history; // v0.0.518: Session history tracker (modularized)

// Maintenance and strategic
#[path = "maintenance_actions.rs"]
pub mod maintenance_actions; // v0.0.286: Proactive maintenance actions
#[path = "senior_strategic/mod.rs"]
pub mod senior_strategic; // v0.0.458: Senior idle-time strategic thinking
#[path = "strategic_thinking/mod.rs"]
pub mod strategic_thinking; // v0.0.515: Strategic thinking tracker

// Synonyms and matching
#[path = "synonyms.rs"]
pub mod synonyms; // v0.0.256: Synonym expansion for recipe matching

// Query scenarios
#[path = "query_scenarios/mod.rs"]
pub mod query_scenarios; // v0.0.268: Query scenario test corpus (100+ queries)

// Shortcuts and commands
#[path = "command_shortcuts/mod.rs"]
pub mod command_shortcuts; // v0.0.483: Command shortcuts

// Skill proficiency
#[path = "skill_proficiency/mod.rs"]
pub mod skill_proficiency; // v0.0.527: Skill proficiency tracker

// Debug mode
#[path = "debug_mode/mod.rs"]
pub mod debug_mode; // v0.0.444: Debug levels, sanitization, reason codes
