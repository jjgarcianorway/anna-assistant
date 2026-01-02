//! Core component tests for hardware-aware system (v0.0.434).
//!
//! Tests Parts A-D:
//! - Hardware profiling
//! - Model catalog and selection
//! - Model planning
//! - Model health tracking
//! - Model configuration

use super::catalog::{ModelCatalog, ModelRole};
use super::model_config::{AutoInstallPolicy, InstallDecision, ModelConfig, PreferSmallSetting};
use super::model_health::{ModelHealth, ModelHealthRecord, ModelStatus};
use super::model_plan::{ModelPlanner, PlanValidation};
use super::profile::CapabilityTier;
use super::test_fixtures::{large_profile, medium_profile, tiny_profile};

// === Part A: Hardware Profiling Tests ===

/// Test 1: Capability tier calculation.
#[test]
fn test_capability_tier_calculation() {
    // Tiny: low RAM, no GPU
    let profile = tiny_profile();
    assert_eq!(profile.tier, CapabilityTier::Tiny);

    // Medium: good RAM + discrete GPU
    let profile = medium_profile();
    assert_eq!(profile.tier, CapabilityTier::Medium);

    // Large: high RAM + strong GPU
    let profile = large_profile();
    assert_eq!(profile.tier, CapabilityTier::Large);
}

/// Test 2: Profile version tracking.
#[test]
fn test_profile_version_tracking() {
    let profile = medium_profile();
    assert_eq!(profile.profile_version, 1);

    // Outdated profile triggers reprofile
    let mut old_profile = profile.clone();
    old_profile.profile_version = 0;
    assert!(old_profile.needs_reprofile());
}

// === Part B: Model Catalog Tests ===

/// Test 3: Model selection respects tier constraints.
#[test]
fn test_model_selection_tier_constraints() {
    let catalog = ModelCatalog::default_catalog();

    // Tiny tier should only get tiny-compatible models
    let translator = catalog.select_model(ModelRole::Translator, CapabilityTier::Tiny, 4.0, false);
    assert!(translator.is_some());
    assert_eq!(translator.unwrap().min_tier, CapabilityTier::Tiny);

    // Medium tier can access more models
    let senior = catalog.select_model(ModelRole::Senior, CapabilityTier::Medium, 20.0, false);
    assert!(senior.is_some());
}

/// Test 4: Prefer small models option works.
#[test]
fn test_prefer_small_models() {
    let catalog = ModelCatalog::default_catalog();

    // Without prefer_small, get higher priority
    let big = catalog.select_model(ModelRole::Senior, CapabilityTier::Large, 64.0, false);
    // With prefer_small, get lower priority
    let small = catalog.select_model(ModelRole::Senior, CapabilityTier::Large, 64.0, true);

    assert!(big.is_some());
    assert!(small.is_some());
    assert!(small.unwrap().priority <= big.unwrap().priority);
}

// === Part B/C: Model Plan Tests ===

/// Test 5: Fresh install generates coherent plan.
#[test]
fn test_fresh_install_plan() {
    let planner = ModelPlanner::new();
    let profile = medium_profile();

    let plan = planner.generate_plan(&profile).unwrap();

    assert_eq!(plan.tier, CapabilityTier::Medium);
    assert!(!plan.translator_model.is_empty());
    assert!(!plan.junior_model.is_empty());
    assert!(!plan.senior_model.is_empty());
    assert!(plan.estimated_disk_gb > 0);
}

/// Test 6: Plan validation detects mismatches.
#[test]
fn test_plan_validation() {
    let planner = ModelPlanner::new();
    let medium = medium_profile();
    let tiny = tiny_profile();

    let plan = planner.generate_plan(&medium).unwrap();

    // Valid for same profile
    let validation = planner.validate_plan(&plan, &medium);
    assert!(validation.is_valid());

    // Invalid for different tier
    let validation = planner.validate_plan(&plan, &tiny);
    assert!(!validation.is_valid());
}

// === Part C: Model Health Tests ===

/// Test 7: Missing model detection.
#[test]
fn test_missing_model_detection() {
    let mut health = ModelHealth::new();
    let planner = ModelPlanner::new();
    let profile = medium_profile();
    let plan = planner.generate_plan(&profile).unwrap();

    // Initially all missing
    let missing = health.missing_models(&plan);
    assert_eq!(missing.len(), plan.model_names().len());

    // After adding one
    health.models.insert(
        plan.translator_model.clone(),
        ModelHealthRecord::installed_by_anna(&plan.translator_model),
    );

    let missing = health.missing_models(&plan);
    assert!(missing.len() < plan.model_names().len());
}

/// Test 8: Model removal detection.
#[test]
fn test_model_removal_detection() {
    let mut health = ModelHealth::new();

    // Model was installed
    health.models.insert(
        "qwen3:4b".to_string(),
        ModelHealthRecord::installed_by_anna("qwen3:4b"),
    );
    assert_eq!(health.status("qwen3:4b"), ModelStatus::Unverified);

    // Model is now missing (simulating ollama rm)
    health.models.get_mut("qwen3:4b").unwrap().status = ModelStatus::Missing;
    assert_eq!(health.status("qwen3:4b"), ModelStatus::Missing);
}

// === Part D: Model Config Tests ===

/// Test 9: Auto-install policy "always".
#[test]
fn test_auto_install_always() {
    let mut config = ModelConfig::new();
    config.auto_install = AutoInstallPolicy::Always;

    assert_eq!(config.can_install("any_model"), InstallDecision::Allowed);
}

/// Test 10: Auto-install policy "never".
#[test]
fn test_auto_install_never() {
    let mut config = ModelConfig::new();
    config.auto_install = AutoInstallPolicy::Never;

    assert!(!config.can_install("any_model").can_proceed());
}

/// Test 11: Auto-install policy "ask-per-model".
#[test]
fn test_auto_install_ask() {
    let mut config = ModelConfig::new();
    config.auto_install = AutoInstallPolicy::AskPerModel;

    // First time: needs approval
    assert_eq!(
        config.can_install("new_model"),
        InstallDecision::NeedsApproval
    );

    // After approval: allowed
    config.record_decision("new_model", true, None);
    assert_eq!(config.can_install("new_model"), InstallDecision::Allowed);

    // After denial: denied
    config.record_decision("denied_model", false, Some("Too large"));
    assert!(!config.can_install("denied_model").can_proceed());
}

/// Test 12: Disk limit enforcement.
#[test]
fn test_disk_limit_enforcement() {
    let config = ModelConfig::new(); // Default 25GB limit

    assert!(!config.would_exceed_disk_limit(10, 10)); // 20GB OK
    assert!(config.would_exceed_disk_limit(20, 10)); // 30GB exceeds
}

/// Test 25: Prefer small setting resolves correctly.
#[test]
fn test_prefer_small_setting() {
    // Auto resolves based on RAM
    assert!(PreferSmallSetting::Auto.resolve(8.0)); // < 16GB
    assert!(!PreferSmallSetting::Auto.resolve(32.0)); // >= 16GB

    // Explicit settings
    assert!(PreferSmallSetting::Yes.resolve(64.0));
    assert!(!PreferSmallSetting::No.resolve(4.0));
}

// === Acceptance Criteria: Fresh Install ===

/// Acceptance 1: Fresh install on medium-tier shows coherent status.
#[test]
fn acceptance_fresh_install_medium_tier() {
    let profile = medium_profile();
    let planner = ModelPlanner::new();
    let plan = planner.generate_plan(&profile).unwrap();

    // Verify plan is coherent
    assert_eq!(plan.tier, CapabilityTier::Medium);
    assert!(!plan.translator_model.is_empty());
    assert!(!plan.junior_model.is_empty());
    assert!(!plan.senior_model.is_empty());

    // Verify no model duplication in unique names
    let names = plan.model_names();
    let mut sorted = names.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(names.len(), sorted.len());
}
