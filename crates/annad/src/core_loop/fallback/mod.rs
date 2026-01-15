//! Fallback command hints for when LLM is unavailable.
//! v0.0.932: Added profile-based command suggestions
//! v0.0.992: Integrated comprehensive monitoring system

mod health;
mod patterns;
mod warmup;

pub use health::{
    get_cached_health, get_health_summary, run_health_checks, HealthCheckResult, HealthStatus,
};
pub use patterns::{get_fallback_commands, get_fallback_commands_with_intent, get_profile_based_commands};
pub use warmup::warm_up_cache;
