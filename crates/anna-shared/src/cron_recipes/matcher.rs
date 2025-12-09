//! Cron recipe matcher (v0.0.234).

use super::recipes::builtin_recipes;
use super::types::{CronFeature, CronRecipe};

/// Match a query to a cron recipe
pub fn match_query(query: &str) -> Option<&'static CronRecipe> {
    let q = query.to_lowercase();

    // Must mention cron, crontab, or schedule to match
    let cron_related =
        q.contains("cron") || q.contains("crontab") || q.contains("scheduled task");

    if !cron_related {
        return None;
    }

    // Detect feature from query
    let feature = detect_feature(&q)?;

    // Get recipes (leaked for 'static lifetime - safe for CLI app)
    static RECIPES: std::sync::OnceLock<Vec<CronRecipe>> = std::sync::OnceLock::new();
    let recipes = RECIPES.get_or_init(builtin_recipes);

    recipes.iter().find(|r| r.feature == feature)
}

/// Detect cron feature from query
pub fn detect_feature(query: &str) -> Option<CronFeature> {
    let q = query.to_lowercase();

    // Check for specific features by keywords
    if q.contains("syntax") || q.contains("format") || q.contains("expression") {
        return Some(CronFeature::SyntaxHelp);
    }

    if q.contains("environment") || q.contains("env") || q.contains("path") {
        return Some(CronFeature::Environment);
    }

    if q.contains("debug")
        || q.contains("not running")
        || q.contains("failing")
        || q.contains("troubleshoot")
    {
        return Some(CronFeature::DebugJob);
    }

    if q.contains("log") || q.contains("output") {
        return Some(CronFeature::ViewLogs);
    }

    if q.contains("remove") || q.contains("delete") {
        return Some(CronFeature::RemoveJob);
    }

    if q.contains("list") || q.contains("show") || q.contains("view") {
        return Some(CronFeature::ListJobs);
    }

    if q.contains("edit") || q.contains("modify") {
        return Some(CronFeature::EditCrontab);
    }

    // Default to add job if talking about cron
    if q.contains("add") || q.contains("create") || q.contains("new") || q.contains("schedule") {
        return Some(CronFeature::AddJob);
    }

    // Generic cron questions default to syntax help
    Some(CronFeature::SyntaxHelp)
}
