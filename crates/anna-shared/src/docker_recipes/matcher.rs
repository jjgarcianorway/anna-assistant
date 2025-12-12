//! Docker recipe matcher (v0.0.235).

use super::recipes::builtin_recipes;
use super::types::{DockerFeature, DockerRecipe};

/// Match a query to a Docker recipe
pub fn match_query(query: &str) -> Option<&'static DockerRecipe> {
    let q = query.to_lowercase();

    // Must mention docker or compose to match
    let docker_related = q.contains("docker") || q.contains("compose") || q.contains("container");

    if !docker_related {
        return None;
    }

    // Detect feature from query
    let feature = detect_feature(&q)?;

    // Get recipes (leaked for 'static lifetime - safe for CLI app)
    static RECIPES: std::sync::OnceLock<Vec<DockerRecipe>> = std::sync::OnceLock::new();
    let recipes = RECIPES.get_or_init(builtin_recipes);

    recipes.iter().find(|r| r.feature == feature)
}

/// Detect Docker feature from query
pub fn detect_feature(query: &str) -> Option<DockerFeature> {
    let q = query.to_lowercase();

    // Check for specific features by keywords
    if q.contains("debug")
        || q.contains("troubleshoot")
        || q.contains("failing")
        || q.contains("not working")
    {
        return Some(DockerFeature::Debug);
    }

    if q.contains("cleanup") || q.contains("clean") || q.contains("prune") || q.contains("remove") {
        return Some(DockerFeature::Cleanup);
    }

    if q.contains("exec") || q.contains("shell") || q.contains("bash") || q.contains("attach") {
        return Some(DockerFeature::ExecContainer);
    }

    if q.contains("log") {
        return Some(DockerFeature::ViewLogs);
    }

    if q.contains("pull") || q.contains("download") {
        return Some(DockerFeature::PullImages);
    }

    if q.contains("build") {
        return Some(DockerFeature::BuildImages);
    }

    if q.contains("list") || q.contains(" ps") || q.contains("running") {
        return Some(DockerFeature::ListContainers);
    }

    if q.contains("stop") || q.contains("down") || q.contains("shutdown") {
        return Some(DockerFeature::StopServices);
    }

    if q.contains("start") || q.contains(" up") || q.contains("run") || q.contains("launch") {
        return Some(DockerFeature::StartServices);
    }

    // Default to create compose if talking about docker compose
    if q.contains("create") || q.contains("write") || q.contains("yml") || q.contains("yaml") {
        return Some(DockerFeature::CreateCompose);
    }

    // Generic docker compose questions default to create
    if q.contains("compose") {
        return Some(DockerFeature::CreateCompose);
    }

    None
}
