//! Configuration-related module declarations
//! All configuration modules

// Core config modules
#[path = "config_intent/mod.rs"]
pub mod config_intent;
#[path = "config_parser/mod.rs"]
pub mod config_parser; // v0.0.236
#[path = "config_types.rs"]
pub mod config_types; // v0.0.264: Config types (ConfigTarget, ConfigIntent)

// Natural language configuration modules
#[path = "debug_config.rs"]
pub mod debug_config; // v0.0.473: Debug configuration via natural language
#[path = "notification_config/mod.rs"]
pub mod notification_config; // v0.0.470: Notification config via natural language
#[path = "preference_config.rs"]
pub mod preference_config; // v0.0.467: Natural language preference configuration
#[path = "risk_config.rs"]
pub mod risk_config; // v0.0.474: Risk level configuration via natural language

// Specific configuration modules
#[path = "personality_config.rs"]
pub mod personality_config; // v0.0.542: Personality config
#[path = "risk_level_config.rs"]
pub mod risk_level_config; // v0.0.543: Risk level config
#[path = "learning_mode_config.rs"]
pub mod learning_mode_config; // v0.0.544: Learning mode config
#[path = "escalation_policy_config/mod.rs"]
pub mod escalation_policy_config; // v0.0.545: Escalation policy config
#[path = "verbosity_config.rs"]
pub mod verbosity_config; // v0.0.546: Verbosity config
#[path = "confirmation_behavior_config.rs"]
pub mod confirmation_behavior_config; // v0.0.547: Confirmation behavior config
#[path = "timeout_config.rs"]
pub mod timeout_config; // v0.0.548: Timeout config
#[path = "output_style_config.rs"]
pub mod output_style_config; // v0.0.549: Output style config
#[path = "privacy_config.rs"]
pub mod privacy_config; // v0.0.550: Privacy config
#[path = "backup_config.rs"]
pub mod backup_config; // v0.0.551: Backup config
#[path = "update_config.rs"]
pub mod update_config; // v0.0.552: Update config
#[path = "model_config.rs"]
pub mod model_config; // v0.0.553: Model config

// UI configuration
#[path = "ui_config.rs"]
pub mod ui_config; // v0.0.413: UI configuration (mode, spinner, etc.)
