//! Built-in recipe matchers for shell, git, SSH, systemd, cron, and Docker configurations.
//!
//! Extracted from recipe_fast_path.rs (v0.0.163) for modularization.
//! v0.0.233: Added systemd unit file recipes.
//! v0.0.234: Added cron job recipes.
//! v0.0.235: Added Docker Compose recipes.

use anna_shared::cron_recipes;
use anna_shared::docker_recipes;
use anna_shared::git_recipes;
use anna_shared::recipe::{Recipe, RecipeAction, RecipeKind};
use anna_shared::shell_recipes;
use anna_shared::ssh_recipes;
use anna_shared::systemd_recipes;
use tracing::info;

use crate::recipe_fast_path::{ticket_from_recipe, RecipeFastPathResult};

/// Check query against built-in shell recipes
pub fn check_shell_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Detect shell from query or environment
    let shell = if q.contains("bash") || q.contains("bashrc") {
        Some(shell_recipes::Shell::Bash)
    } else if q.contains("zsh") || q.contains("zshrc") {
        Some(shell_recipes::Shell::Zsh)
    } else if q.contains("fish") {
        Some(shell_recipes::Shell::Fish)
    } else {
        shell_recipes::Shell::detect()
    };

    // Detect feature from query
    let feature = shell_recipes::detect_feature(&q)?;
    let shell = shell?;

    // Find matching recipe
    let recipe = shell_recipes::find_recipe(shell, feature)?;

    // Build a synthetic Recipe for the result
    let synthetic_recipe = Recipe {
        id: format!("shell-{}-{:?}", shell, feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "desktop",
            "request",
            "shell_config",
            query,
        ),
        team: anna_shared::teams::Team::Desktop,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: format!(
            "To {} in {}:\n\nAdd to ~/{}\n```\n{}\n```\n\n{}",
            feature.display_name(),
            shell.display_name(),
            shell.config_path().display(),
            recipe.lines.join("\n"),
            recipe
                .rollback_hint
                .as_deref()
                .unwrap_or("To undo: remove the added lines")
        ),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::ShellConfig,
        target: None,
        action: RecipeAction::EnsureLine {
            line: recipe.lines.join("\n"),
        },
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec![shell.display_name().to_lowercase()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!(
        "Shell recipe match: {} in {}",
        feature.display_name(),
        shell.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            shell.display_name().to_lowercase(),
            feature.display_name().to_string(),
        ],
        skip_llm: true,
    })
}

/// Check query against built-in git recipes
pub fn check_git_recipes(query: &str) -> Option<RecipeFastPathResult> {
    let q = query.to_lowercase();

    // Must mention "git" to match git recipes
    if !q.contains("git") {
        return None;
    }

    // Detect feature from query
    let feature = git_recipes::detect_feature(&q)?;

    // Find matching recipes
    let recipes = git_recipes::find_recipe(feature);
    if recipes.is_empty() {
        return None;
    }

    let recipe = &recipes[0];

    // Build answer from recipe
    let answer = if recipe.needs_parameters() {
        format!(
            "To configure {}:\n\nCommands:\n{}\n\nNote: Replace {{name}} and {{email}} with your values.\n\n{}",
            feature.display_name(),
            recipe.commands.iter().map(|c| format!("  {}", c)).collect::<Vec<_>>().join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    } else {
        format!(
            "To configure {}:\n\nRun:\n{}\n\n{}",
            feature.display_name(),
            recipe
                .commands
                .iter()
                .map(|c| format!("  {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            recipe.rollback_hint.as_deref().unwrap_or("")
        )
    };

    // Build a synthetic Recipe
    let synthetic_recipe = Recipe {
        id: format!("git-{:?}", feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "git_config",
            query,
        ),
        team: anna_shared::teams::Team::General,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: answer,
        created_at: 0,
        success_count: 100,
        reliability_score: 95,
        kind: RecipeKind::GitConfig,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: feature.keywords().iter().map(|s| s.to_string()).collect(),
        targets: vec!["git".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!("Git recipe match: {}", feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec!["git".to_string(), feature.display_name().to_string()],
        skip_llm: true,
    })
}

/// Check query against built-in SSH recipes (v0.0.104)
pub fn check_ssh_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the SSH recipe matcher
    let ssh_recipe = ssh_recipes::match_query(query)?;

    // Build a synthetic Recipe from the SSH recipe
    let synthetic_recipe = Recipe {
        id: format!("ssh-{:?}", ssh_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "ssh_config",
            query,
        ),
        team: anna_shared::teams::Team::Security,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: ssh_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::SshConfig,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: ssh_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["ssh".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!("SSH recipe match: {}", ssh_recipe.feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "ssh".to_string(),
            ssh_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
    })
}

/// Check query against built-in systemd recipes (v0.0.233)
pub fn check_systemd_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the systemd recipe matcher
    let systemd_recipe = systemd_recipes::match_query(query)?;

    // Build a synthetic Recipe from the systemd recipe
    let synthetic_recipe = Recipe {
        id: format!("systemd-{:?}", systemd_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "systemd_unit",
            query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: systemd_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::SystemdUnit,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: systemd_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["systemd".to_string(), "service".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!(
        "Systemd recipe match: {}",
        systemd_recipe.feature.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "systemd".to_string(),
            systemd_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
    })
}

/// Check query against built-in cron recipes (v0.0.234)
pub fn check_cron_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the cron recipe matcher
    let cron_recipe = cron_recipes::match_query(query)?;

    // Build a synthetic Recipe from the cron recipe
    let synthetic_recipe = Recipe {
        id: format!("cron-{:?}", cron_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "cron_job",
            query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: cron_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::CronJob,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: cron_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["cron".to_string(), "crontab".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!("Cron recipe match: {}", cron_recipe.feature.display_name());

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "cron".to_string(),
            cron_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
    })
}

/// Check query against built-in Docker recipes (v0.0.235)
pub fn check_docker_recipes(query: &str) -> Option<RecipeFastPathResult> {
    // Use the Docker recipe matcher
    let docker_recipe = docker_recipes::match_query(query)?;

    // Build a synthetic Recipe from the Docker recipe
    let synthetic_recipe = Recipe {
        id: format!("docker-{:?}", docker_recipe.feature),
        signature: anna_shared::recipe::RecipeSignature::new(
            "system",
            "request",
            "docker_compose",
            query,
        ),
        team: anna_shared::teams::Team::Services,
        risk_level: anna_shared::ticket::RiskLevel::LowRiskChange,
        required_evidence_kinds: vec![],
        probe_sequence: vec![],
        answer_template: docker_recipe.answer_template.clone(),
        created_at: 0,
        success_count: 100, // Built-in = mature
        reliability_score: 95,
        kind: RecipeKind::DockerCompose,
        target: None,
        action: RecipeAction::None,
        rollback: None,
        clarification_slots: vec![],
        default_question_id: None,
        populates_facts: vec![],
        intent_tags: docker_recipe
            .feature
            .keywords()
            .iter()
            .map(|s| s.to_string())
            .collect(),
        targets: vec!["docker".to_string(), "compose".to_string()],
        preconditions: vec![],
        clarify_prereqs: vec![],
    };

    info!(
        "Docker recipe match: {}",
        docker_recipe.feature.display_name()
    );

    Some(RecipeFastPathResult {
        matched: true,
        ticket: Some(ticket_from_recipe(&synthetic_recipe)),
        recipe: Some(synthetic_recipe),
        score: 90,
        matched_tokens: vec![
            "docker".to_string(),
            docker_recipe.feature.display_name().to_string(),
        ],
        skip_llm: true,
    })
}
