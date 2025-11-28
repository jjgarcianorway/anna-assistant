//! Skills Module v0.40.0
//!
//! Generic, parameterized skills for Anna.
//! Skills are learned patterns that can be reused across variants.
//!
//! v0.40.0: Initial implementation - generic skills, skill store, built-ins.
//!
//! ## Architecture
//!
//! ```text
//! +----------------+     +---------------+     +----------------+
//! |   Question     | --> | Skill Matcher | --> | Skill Executor |
//! +----------------+     +---------------+     +----------------+
//!                              |                      |
//!                              v                      v
//!                        +------------+        +-------------+
//!                        | Skill Store|        | safe_command|
//!                        +------------+        +-------------+
//! ```
//!
//! ## Key Concepts
//!
//! - **Skill**: A parameterized command template with stats
//! - **Intent**: What the skill does (e.g., "logs_for_service")
//! - **Parameters**: Variables in the template (e.g., service_name, since)
//! - **safe_command**: Execution primitive with whitelist validation
//!
//! ## Usage
//!
//! ```ignore
//! use anna_common::skills::{SkillStore, Skill, execute_skill};
//!
//! // Open skill store
//! let store = SkillStore::open_default()?;
//!
//! // Find matching skill
//! let matches = store.find_by_question("show annad logs", 0.3);
//! if let Some((skill, score)) = matches.first() {
//!     let mut params = HashMap::new();
//!     params.insert("service_name".to_string(), "annad.service".to_string());
//!     params.insert("since".to_string(), "6 hours ago".to_string());
//!
//!     let result = execute_skill(&skill, &params)?;
//!     if result.success {
//!         store.record_success(&skill.skill_id, result.latency_ms)?;
//!     }
//! }
//! ```

pub mod builtins;
pub mod executor;
pub mod schema;
pub mod store;

pub use builtins::{builtin_skills, init_builtin_skills};
pub use executor::{
    execute_safe_command, execute_safe_command_async, execute_skill, validate_params,
    SkillExecutionResult,
};
pub use schema::{
    AnnaLevel, ParamSchema, ParamType, Skill, SkillQuery, SkillStats, SystemStats,
    SKILL_SCHEMA_VERSION,
};
pub use store::{SkillStore, SKILLS_DIR};
