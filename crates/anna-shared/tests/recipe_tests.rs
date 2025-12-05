//! Tests for recipe module including v0.0.27 config edit recipes.

use anna_shared::recipe::{
    compute_recipe_id, recipe_filename, should_persist_recipe, Recipe, RecipeAction,
    RecipeKind, RecipeSignature, RecipeTarget, RollbackInfo,
};
use anna_shared::teams::Team;
use anna_shared::ticket::RiskLevel;
use anna_shared::trace::EvidenceKind;
use std::path::PathBuf;

#[test]
fn test_recipe_signature_creation() {
    let sig = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");

    assert_eq!(sig.domain, "system");
    assert_eq!(sig.query_pattern, "how much ram?");
}

#[test]
fn test_recipe_signature_hash_deterministic() {
    let sig1 = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");
    let sig2 = RecipeSignature::new("system", "question", "MemoryUsage", "How much RAM?");

    assert_eq!(sig1.hash_id(), sig2.hash_id());
}

#[test]
fn test_recipe_id_deterministic() {
    let sig = RecipeSignature::new("storage", "investigate", "DiskUsage", "check disk");

    let id1 = compute_recipe_id(&sig, Team::Storage);
    let id2 = compute_recipe_id(&sig, Team::Storage);

    assert_eq!(id1, id2);
    assert_eq!(id1.len(), 16); // 16 hex chars
}

#[test]
fn test_recipe_id_differs_by_team() {
    let sig = RecipeSignature::new("storage", "investigate", "DiskUsage", "check disk");

    let id_storage = compute_recipe_id(&sig, Team::Storage);
    let id_general = compute_recipe_id(&sig, Team::General);

    assert_ne!(id_storage, id_general);
}

#[test]
fn test_recipe_creation() {
    let sig = RecipeSignature::new("network", "question", "NetworkInfo", "show ip");
    let recipe = Recipe::new(
        sig,
        Team::Network,
        RiskLevel::ReadOnly,
        vec![],
        vec!["ip addr show".to_string()],
        "Your IP is {ip}".to_string(),
        85,
    );

    assert_eq!(recipe.team, Team::Network);
    assert_eq!(recipe.success_count, 1);
    assert_eq!(recipe.reliability_score, 85);
    assert!(!recipe.is_mature());
    assert_eq!(recipe.kind, RecipeKind::Query);
    assert!(!recipe.is_config_edit());
}

#[test]
fn test_recipe_maturity() {
    let sig = RecipeSignature::new("system", "question", "CpuInfo", "cpu info");
    let mut recipe = Recipe::new(
        sig,
        Team::Hardware,
        RiskLevel::ReadOnly,
        vec![EvidenceKind::Cpu],
        vec!["lscpu".to_string()],
        String::new(),
        90,
    );

    assert!(!recipe.is_mature());

    recipe.record_success();
    assert!(!recipe.is_mature());

    recipe.record_success();
    assert!(recipe.is_mature()); // 3 successes
}

#[test]
fn test_should_persist_recipe() {
    assert!(should_persist_recipe(true, 80));
    assert!(should_persist_recipe(true, 100));
    assert!(!should_persist_recipe(true, 79));
    assert!(!should_persist_recipe(false, 100));
    assert!(!should_persist_recipe(false, 50));
}

#[test]
fn test_recipe_serialization() {
    let sig = RecipeSignature::new("storage", "question", "DiskUsage", "disk space");
    let recipe = Recipe::new(
        sig,
        Team::Storage,
        RiskLevel::ReadOnly,
        vec![EvidenceKind::Disk, EvidenceKind::BlockDevices],
        vec!["df -h".to_string(), "lsblk".to_string()],
        "Disk usage: {usage}".to_string(),
        88,
    );

    let json = serde_json::to_string(&recipe).unwrap();
    let parsed: Recipe = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, recipe.id);
    assert_eq!(parsed.team, Team::Storage);
    assert_eq!(parsed.probe_sequence.len(), 2);
}

#[test]
fn test_recipe_filename() {
    let path = recipe_filename("abc123def456");
    assert!(path.to_string_lossy().contains("abc123def456.json"));
    assert!(path.to_string_lossy().contains("recipes"));
}

// v0.0.27 tests

#[test]
fn test_recipe_kind_default() {
    assert_eq!(RecipeKind::default(), RecipeKind::Query);
}

#[test]
fn test_recipe_action_default() {
    assert_eq!(RecipeAction::default(), RecipeAction::None);
}

#[test]
fn test_recipe_target_expand_path() {
    let target = RecipeTarget::new("vim", "$HOME/.vimrc");
    let expanded = target.expand_path();

    // Should not contain $HOME after expansion
    assert!(!expanded.to_string_lossy().contains("$HOME"));
    assert!(expanded.to_string_lossy().contains(".vimrc"));
}

#[test]
fn test_config_edit_recipe_creation() {
    let sig = RecipeSignature::new("desktop", "request", "VimConfig", "enable syntax highlighting");
    let target = RecipeTarget::new("vim", "$HOME/.vimrc");
    let action = RecipeAction::EnsureLine {
        line: "syntax on".to_string(),
    };

    let recipe = Recipe::config_edit(sig, Team::Desktop, target, action, 90);

    assert_eq!(recipe.kind, RecipeKind::ConfigEnsureLine);
    assert!(recipe.is_config_edit());
    assert_eq!(recipe.risk_level, RiskLevel::LowRiskChange);
    assert!(recipe.target.is_some());
    assert!(matches!(recipe.action, RecipeAction::EnsureLine { .. }));
}

#[test]
fn test_config_edit_recipe_with_rollback() {
    let sig = RecipeSignature::new("desktop", "request", "VimConfig", "enable syntax");
    let target = RecipeTarget::new("vim", "$HOME/.vimrc");
    let action = RecipeAction::EnsureLine {
        line: "syntax on".to_string(),
    };
    let rollback = RollbackInfo::new(
        PathBuf::from("/tmp/backup/.vimrc.bak"),
        "Restore original .vimrc from backup",
    );

    let recipe = Recipe::config_edit(sig, Team::Desktop, target, action, 90)
        .with_rollback(rollback);

    assert!(recipe.rollback.is_some());
    let rb = recipe.rollback.unwrap();
    assert!(rb.backup_path.to_string_lossy().contains("backup"));
    assert!(!rb.tested);
}

#[test]
fn test_config_edit_recipe_serialization() {
    let sig = RecipeSignature::new("desktop", "request", "VimConfig", "enable syntax");
    let target = RecipeTarget::new("vim", "$HOME/.vimrc");
    let action = RecipeAction::EnsureLine {
        line: "syntax on".to_string(),
    };

    let recipe = Recipe::config_edit(sig, Team::Desktop, target, action, 90);

    let json = serde_json::to_string_pretty(&recipe).unwrap();
    let parsed: Recipe = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.id, recipe.id);
    assert_eq!(parsed.kind, RecipeKind::ConfigEnsureLine);
    assert!(parsed.target.is_some());

    let target = parsed.target.unwrap();
    assert_eq!(target.app_id, "vim");
}

#[test]
fn test_recipe_kind_serialization() {
    // Test Query
    let json = serde_json::to_string(&RecipeKind::Query).unwrap();
    assert!(json.contains("query"));

    // Test ConfigEnsureLine
    let json = serde_json::to_string(&RecipeKind::ConfigEnsureLine).unwrap();
    assert!(json.contains("config_ensure_line"));

    // Roundtrip
    let parsed: RecipeKind = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed, RecipeKind::ConfigEnsureLine);
}

#[test]
fn test_recipe_action_serialization() {
    let action = RecipeAction::EnsureLine {
        line: "syntax on".to_string(),
    };
    let json = serde_json::to_string(&action).unwrap();

    assert!(json.contains("ensure_line"));
    assert!(json.contains("syntax on"));

    let parsed: RecipeAction = serde_json::from_str(&json).unwrap();
    match parsed {
        RecipeAction::EnsureLine { line } => assert_eq!(line, "syntax on"),
        _ => panic!("Expected EnsureLine"),
    }
}

#[test]
fn test_rollback_info_creation() {
    let rollback = RollbackInfo::new(
        PathBuf::from("/backup/file.bak"),
        "Restore from backup",
    );

    assert_eq!(rollback.backup_path, PathBuf::from("/backup/file.bak"));
    assert_eq!(rollback.description, "Restore from backup");
    assert!(!rollback.tested);
}
