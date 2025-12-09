//! Systemd recipe matcher (v0.0.233).

use super::recipes::builtin_recipes;
use super::types::{SystemdFeature, SystemdRecipe};

/// Match a query to a systemd recipe
pub fn match_query(query: &str) -> Option<&'static SystemdRecipe> {
    let q = query.to_lowercase();

    // Must mention systemd, service, unit, or timer to match
    let systemd_related = q.contains("systemd")
        || q.contains("unit file")
        || q.contains("service file")
        || (q.contains("service") && (q.contains("create") || q.contains("write")))
        || (q.contains("timer") && !q.contains("time"));

    if !systemd_related {
        return None;
    }

    // Detect feature from query
    let feature = detect_feature(&q)?;

    // Get recipes (leaked for 'static lifetime - safe for CLI app)
    static RECIPES: std::sync::OnceLock<Vec<SystemdRecipe>> = std::sync::OnceLock::new();
    let recipes = RECIPES.get_or_init(builtin_recipes);

    recipes.iter().find(|r| r.feature == feature)
}

/// Detect systemd feature from query
pub fn detect_feature(query: &str) -> Option<SystemdFeature> {
    let q = query.to_lowercase();

    // Check for specific features by keywords
    if q.contains("timer") || q.contains("schedule") || q.contains("periodic") || q.contains("cron")
    {
        return Some(SystemdFeature::CreateTimer);
    }

    if q.contains("user service") || (q.contains("user") && q.contains("systemd")) {
        return Some(SystemdFeature::CreateUserService);
    }

    if q.contains("socket") && q.contains("activ") {
        return Some(SystemdFeature::SocketActivation);
    }

    if q.contains("harden") || q.contains("secur") || q.contains("sandbox") {
        return Some(SystemdFeature::HardenService);
    }

    if q.contains("log") || q.contains("journalctl") || q.contains("journal") {
        return Some(SystemdFeature::ViewLogs);
    }

    if q.contains("debug") || q.contains("failing") || q.contains("failed") || q.contains("fix") {
        return Some(SystemdFeature::DebugService);
    }

    if q.contains("enable") || q.contains("start") {
        return Some(SystemdFeature::EnableService);
    }

    // Default to create service if talking about systemd/unit files
    if q.contains("create")
        || q.contains("new")
        || q.contains("write")
        || q.contains("make")
        || q.contains("unit")
    {
        return Some(SystemdFeature::CreateService);
    }

    None
}
