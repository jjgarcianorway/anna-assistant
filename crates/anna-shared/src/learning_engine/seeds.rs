//! Seed recipes for learning engine (v0.0.427).
//!
//! Minimal hardcoded recipes for common tasks:
//! - Check free RAM
//! - Check disk space
//! - Check service status
//!
//! These provide a foundation until the system learns more.

use super::{
    AnswerKind, AnswerTemplate, LearnedRecipe, LogicType, RecipeInputs, RecipeLogic, RecipeOrigin,
    RecipePattern, RecipeProbe, RecipeSafety, RiskLevel,
};

/// Create all seed recipes
pub fn create_seeds() -> Vec<LearnedRecipe> {
    vec![
        seed_check_free_ram(),
        seed_check_disk_space(),
        seed_check_service_status(),
        seed_check_uptime(),
        seed_check_memory_usage(),
    ]
}

/// Seed recipe: Check free RAM
fn seed_check_free_ram() -> LearnedRecipe {
    LearnedRecipe {
        id: "seed-check-free-ram".to_string(),
        domain: "performance.memory".to_string(),
        pattern: RecipePattern {
            intent: "check_free_ram".to_string(),
            keywords: vec![
                "ram".to_string(),
                "memory".to_string(),
                "free".to_string(),
                "available".to_string(),
            ],
            required_signals: vec!["probe:free".to_string()],
            optional_signals: vec![],
        },
        inputs: RecipeInputs::default(),
        probes: vec![RecipeProbe {
            id: "probe:free".to_string(),
            tool: "probe.free".to_string(),
            params: vec![],
            optional: false,
            timeout_ms: 5000,
        }],
        logic: RecipeLogic {
            logic_type: LogicType::Template,
            answer_kind: AnswerKind::Diagnostic,
            steps: vec!["Run free -h to check memory".to_string()],
            conditionals: Default::default(),
        },
        answer_template: AnswerTemplate {
            short: "Available RAM: {{available_mem}}".to_string(),
            detailed: "Memory Status:\n  Total: {{total_mem}}\n  Used: {{used_mem}}\n  Free: {{free_mem}}\n  Available: {{available_mem}}".to_string(),
            variables: vec![
                "total_mem".to_string(),
                "used_mem".to_string(),
                "free_mem".to_string(),
                "available_mem".to_string(),
            ],
        },
        safety: RecipeSafety {
            risk: RiskLevel::Low,
            needs_backup: false,
            requires_sudo: false,
            warning: None,
        },
        origin: seed_origin(),
        stats: Default::default(),
        version: 1,
        enabled: true,
    }
}

/// Seed recipe: Check disk space
fn seed_check_disk_space() -> LearnedRecipe {
    LearnedRecipe {
        id: "seed-check-disk-space".to_string(),
        domain: "storage.disk".to_string(),
        pattern: RecipePattern {
            intent: "check_disk_space".to_string(),
            keywords: vec![
                "disk".to_string(),
                "space".to_string(),
                "storage".to_string(),
                "full".to_string(),
            ],
            required_signals: vec!["probe:df".to_string()],
            optional_signals: vec![],
        },
        inputs: RecipeInputs::default(),
        probes: vec![RecipeProbe {
            id: "probe:df".to_string(),
            tool: "probe.df".to_string(),
            params: vec![],
            optional: false,
            timeout_ms: 5000,
        }],
        logic: RecipeLogic {
            logic_type: LogicType::Template,
            answer_kind: AnswerKind::Diagnostic,
            steps: vec!["Run df -h to check disk usage".to_string()],
            conditionals: Default::default(),
        },
        answer_template: AnswerTemplate {
            short: "Disk usage: {{disk_percent}} ({{disk_used}} used, {{disk_available}} available)".to_string(),
            detailed: "Disk Space Status:\n  Used: {{disk_used}}\n  Available: {{disk_available}}\n  Usage: {{disk_percent}}".to_string(),
            variables: vec![
                "disk_used".to_string(),
                "disk_available".to_string(),
                "disk_percent".to_string(),
            ],
        },
        safety: RecipeSafety {
            risk: RiskLevel::Low,
            needs_backup: false,
            requires_sudo: false,
            warning: None,
        },
        origin: seed_origin(),
        stats: Default::default(),
        version: 1,
        enabled: true,
    }
}

/// Seed recipe: Check service status
fn seed_check_service_status() -> LearnedRecipe {
    let mut inputs = RecipeInputs::default();
    inputs.params.insert(
        "service_name".to_string(),
        "Name of the systemd service".to_string(),
    );

    LearnedRecipe {
        id: "seed-check-service-status".to_string(),
        domain: "services.systemd".to_string(),
        pattern: RecipePattern {
            intent: "check_service_status".to_string(),
            keywords: vec![
                "service".to_string(),
                "systemctl".to_string(),
                "status".to_string(),
                "running".to_string(),
            ],
            required_signals: vec!["probe:systemctl".to_string()],
            optional_signals: vec![],
        },
        inputs,
        probes: vec![RecipeProbe {
            id: "probe:systemctl".to_string(),
            tool: "probe.systemctl_status".to_string(),
            params: vec!["{{service_name}}".to_string()],
            optional: false,
            timeout_ms: 5000,
        }],
        logic: RecipeLogic {
            logic_type: LogicType::Template,
            answer_kind: AnswerKind::Diagnostic,
            steps: vec!["Run systemctl status to check service".to_string()],
            conditionals: Default::default(),
        },
        answer_template: AnswerTemplate {
            short: "Service {{service_name}}: {{probe:systemctl_output}}".to_string(),
            detailed: "Service Status for {{service_name}}:\n{{probe:systemctl_output}}"
                .to_string(),
            variables: vec!["service_name".to_string()],
        },
        safety: RecipeSafety {
            risk: RiskLevel::Low,
            needs_backup: false,
            requires_sudo: false,
            warning: None,
        },
        origin: seed_origin(),
        stats: Default::default(),
        version: 1,
        enabled: true,
    }
}

/// Seed recipe: Check system uptime
fn seed_check_uptime() -> LearnedRecipe {
    LearnedRecipe {
        id: "seed-check-uptime".to_string(),
        domain: "system".to_string(),
        pattern: RecipePattern {
            intent: "check_uptime".to_string(),
            keywords: vec![
                "uptime".to_string(),
                "running".to_string(),
                "boot".to_string(),
                "reboot".to_string(),
            ],
            required_signals: vec!["probe:uptime".to_string()],
            optional_signals: vec![],
        },
        inputs: RecipeInputs::default(),
        probes: vec![RecipeProbe {
            id: "probe:uptime".to_string(),
            tool: "probe.uptime".to_string(),
            params: vec![],
            optional: false,
            timeout_ms: 5000,
        }],
        logic: RecipeLogic {
            logic_type: LogicType::Template,
            answer_kind: AnswerKind::Diagnostic,
            steps: vec!["Run uptime to check system uptime".to_string()],
            conditionals: Default::default(),
        },
        answer_template: AnswerTemplate {
            short: "System uptime: {{uptime}}".to_string(),
            detailed: "System has been running for: {{uptime}}".to_string(),
            variables: vec!["uptime".to_string()],
        },
        safety: RecipeSafety {
            risk: RiskLevel::Low,
            needs_backup: false,
            requires_sudo: false,
            warning: None,
        },
        origin: seed_origin(),
        stats: Default::default(),
        version: 1,
        enabled: true,
    }
}

/// Seed recipe: Check memory usage (detailed)
fn seed_check_memory_usage() -> LearnedRecipe {
    LearnedRecipe {
        id: "seed-check-memory-usage".to_string(),
        domain: "performance.memory".to_string(),
        pattern: RecipePattern {
            intent: "check_memory_usage".to_string(),
            keywords: vec![
                "memory".to_string(),
                "usage".to_string(),
                "using".to_string(),
                "consuming".to_string(),
            ],
            required_signals: vec!["probe:free".to_string()],
            optional_signals: vec!["probe:vmstat".to_string()],
        },
        inputs: RecipeInputs::default(),
        probes: vec![
            RecipeProbe {
                id: "probe:free".to_string(),
                tool: "probe.free".to_string(),
                params: vec![],
                optional: false,
                timeout_ms: 5000,
            },
            RecipeProbe {
                id: "probe:vmstat".to_string(),
                tool: "probe.vmstat".to_string(),
                params: vec![],
                optional: true,
                timeout_ms: 5000,
            },
        ],
        logic: RecipeLogic {
            logic_type: LogicType::Template,
            answer_kind: AnswerKind::Diagnostic,
            steps: vec![
                "Run free -h to check memory".to_string(),
                "Run vmstat for additional stats".to_string(),
            ],
            conditionals: Default::default(),
        },
        answer_template: AnswerTemplate {
            short: "Memory: {{used_mem}} of {{total_mem}} used ({{available_mem}} available)".to_string(),
            detailed: "Memory Usage:\n  Total: {{total_mem}}\n  Used: {{used_mem}}\n  Free: {{free_mem}}\n  Available: {{available_mem}}\n\nNote: 'Available' includes reclaimable cache.".to_string(),
            variables: vec![
                "total_mem".to_string(),
                "used_mem".to_string(),
                "free_mem".to_string(),
                "available_mem".to_string(),
            ],
        },
        safety: RecipeSafety {
            risk: RiskLevel::Low,
            needs_backup: false,
            requires_sudo: false,
            warning: None,
        },
        origin: seed_origin(),
        stats: Default::default(),
        version: 1,
        enabled: true,
    }
}

/// Common origin for seed recipes
fn seed_origin() -> RecipeOrigin {
    RecipeOrigin {
        created_from_ticket: None,
        created_by: "system".to_string(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        sources: vec!["seed".to_string()],
        is_seed: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeds_created() {
        let seeds = create_seeds();
        assert_eq!(seeds.len(), 5);

        // All seeds should be marked as seeds
        for seed in &seeds {
            assert!(seed.origin.is_seed);
            assert!(seed.enabled);
        }
    }

    #[test]
    fn test_seed_ids_unique() {
        let seeds = create_seeds();
        let mut ids: Vec<&str> = seeds.iter().map(|s| s.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), seeds.len());
    }

    #[test]
    fn test_ram_seed_pattern() {
        let seed = seed_check_free_ram();
        assert_eq!(seed.pattern.intent, "check_free_ram");
        assert!(seed.pattern.keywords.contains(&"ram".to_string()));
        assert!(!seed.probes.is_empty());
    }

    #[test]
    fn test_disk_seed_pattern() {
        let seed = seed_check_disk_space();
        assert_eq!(seed.pattern.intent, "check_disk_space");
        assert!(seed.pattern.keywords.contains(&"disk".to_string()));
    }

    #[test]
    fn test_service_seed_has_params() {
        let seed = seed_check_service_status();
        assert!(seed.inputs.params.contains_key("service_name"));
    }
}
