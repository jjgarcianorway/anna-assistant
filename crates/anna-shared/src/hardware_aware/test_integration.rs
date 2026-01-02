//! Integration and acceptance tests for hardware-aware system (v0.0.434).
//!
//! Tests Parts E-H:
//! - Helper catalog and configuration
//! - Integration with probing and specialists
//! - Status display and formatting
//! - Usage stats tracking
//! - Acceptance criteria

use super::catalog::ModelRole;
use super::helper_config::{HelperConfig, HelperInstallDecision, HelperInstallPolicy};
use super::helper_entry::HelperCatalog;
use super::helper_manager::HelperManager;
use super::integration::{ModelAvailability, ProbeHelper, SpecialistHelper};
use super::model_config::ModelConfig;
use super::model_health::{ModelHealth, ModelHealthRecord, ModelStatus};
use super::model_plan::ModelPlanner;
use super::profile::CapabilityTier;
use super::status::{HardwareStatus, HelperUsageStats, ModelUsageStats};
use super::test_fixtures::medium_profile;

// === Part E: Helper Tests ===

/// Test 13: Helper catalog has expected entries.
#[test]
fn test_helper_catalog() {
    let catalog = HelperCatalog::default_catalog();

    assert!(catalog.get("lm_sensors").is_some());
    assert!(catalog.get("smartmontools").is_some());
    assert!(catalog.get("nvme_cli").is_some());
}

/// Test 14: Helper install policy "ask-per-helper".
#[test]
fn test_helper_install_ask() {
    let mut config = HelperConfig::new();
    config.auto_install = HelperInstallPolicy::AskPerHelper;

    // First time: needs approval
    assert_eq!(
        config.can_install("lm_sensors"),
        HelperInstallDecision::NeedsApproval
    );

    // After approval: allowed
    config.record_decision("lm_sensors", true, None);
    assert_eq!(
        config.can_install("lm_sensors"),
        HelperInstallDecision::Allowed
    );
}

/// Test 15: Helper install policy "never".
#[test]
fn test_helper_install_never() {
    let mut config = HelperConfig::new();
    config.auto_install = HelperInstallPolicy::Never;

    assert!(!config.can_install("lm_sensors").can_proceed());
}

/// Test 16: Anna-installed vs user-installed tracking.
#[test]
fn test_helper_install_tracking() {
    let mut manager = HelperManager::new();

    manager.record_anna_install("lm_sensors");
    manager.record_detected("smartmontools");

    assert!(manager.anna_installed().contains(&"lm_sensors"));
    assert!(manager.user_installed().contains(&"smartmontools"));
}

// === Part F: Integration Tests ===

/// Test 17: Probe helper fallback behavior.
#[test]
fn test_probe_helper_fallback() {
    let helper = ProbeHelper::new();
    let manager = HelperManager::new();

    // Without lm_sensors, temperature probe uses fallback
    let cmd = helper.best_command("temperature", &manager);
    assert!(cmd.available);
    assert!(cmd.is_fallback);
    assert!(cmd.note.is_some());
}

/// Test 18: Model availability with fallback.
#[test]
fn test_model_availability_fallback() {
    let helper = SpecialistHelper::new();
    let planner = ModelPlanner::new();
    let profile = medium_profile();
    let plan = planner.generate_plan(&profile).unwrap();

    let mut health = ModelHealth::new();

    // Junior is available
    health.models.insert(
        plan.junior_model.clone(),
        ModelHealthRecord::installed_by_anna(&plan.junior_model),
    );
    health.models.get_mut(&plan.junior_model).unwrap().status = ModelStatus::Ok;

    // Senior is missing
    let avail = helper.check_model_availability(ModelRole::Senior, &plan, &health);

    // Should have fallback to junior
    match avail {
        ModelAvailability::Missing { fallback, .. } => {
            assert!(fallback.is_some());
        }
        _ => panic!("Expected Missing status"),
    }
}

/// Test 19: Helper suggestion for specialists.
#[test]
fn test_helper_suggestion() {
    let helper = SpecialistHelper::new();

    let suggestion =
        helper.suggest_helper_step("lm_sensors", "To diagnose overheating", "Arch Linux");

    assert!(suggestion.is_some());
    let s = suggestion.unwrap();
    assert!(s.install_command.contains("pacman"));
}

// === Part G: Status Display Tests ===

/// Test 20: Status shows coherent system profile.
#[test]
fn test_status_system_profile() {
    let profile = medium_profile();
    let planner = ModelPlanner::new();
    let plan = planner.generate_plan(&profile).unwrap();
    let health = ModelHealth::new();
    let model_config = ModelConfig::new();
    let helper_manager = HelperManager::new();
    let helper_catalog = HelperCatalog::default_catalog();
    let helper_config = HelperConfig::new();

    let status = HardwareStatus::build(
        &profile,
        &plan,
        &health,
        &model_config,
        &helper_manager,
        &helper_catalog,
        &helper_config,
    );

    assert_eq!(status.profile.tier, "Medium");
    assert_eq!(status.profile.ram_total_gb, 32.0);
}

/// Test 21: Status shows model plan without duplicates.
#[test]
fn test_status_no_duplicate_models() {
    let profile = medium_profile();
    let planner = ModelPlanner::new();
    let plan = planner.generate_plan(&profile).unwrap();
    let health = ModelHealth::new();
    let model_config = ModelConfig::new();
    let helper_manager = HelperManager::new();
    let helper_catalog = HelperCatalog::default_catalog();
    let helper_config = HelperConfig::new();

    let status = HardwareStatus::build(
        &profile,
        &plan,
        &health,
        &model_config,
        &helper_manager,
        &helper_catalog,
        &helper_config,
    );

    // Should have 3 roles
    assert_eq!(status.llm.models.len(), 3);
}

/// Test 22: Status reflects missing models.
#[test]
fn test_status_missing_models() {
    let profile = medium_profile();
    let planner = ModelPlanner::new();
    let plan = planner.generate_plan(&profile).unwrap();
    let health = ModelHealth::new(); // All missing
    let model_config = ModelConfig::new();
    let helper_manager = HelperManager::new();
    let helper_catalog = HelperCatalog::default_catalog();
    let helper_config = HelperConfig::new();

    let status = HardwareStatus::build(
        &profile,
        &plan,
        &health,
        &model_config,
        &helper_manager,
        &helper_catalog,
        &helper_config,
    );

    assert_eq!(status.llm.state, "DEGRADED");
    assert!(status.llm.models.iter().all(|m| m.status == "MISSING"));
}

/// Test 23: Model usage stats tracking.
#[test]
fn test_model_usage_stats() {
    let mut stats = ModelUsageStats::default();

    stats.record_call("qwen3:4b", 500, true);
    stats.record_call("qwen3:4b", 700, true);
    stats.record_call("qwen3:4b", 600, false);
    stats.record_error("qwen3:4b", "ParseError");

    let usage = stats.models.get("qwen3:4b").unwrap();
    assert_eq!(usage.call_count, 3);
    assert_eq!(usage.error_count, 2);

    assert!(stats.last_error.is_some());
    assert_eq!(stats.last_error.as_ref().unwrap().error_type, "ParseError");
}

/// Test 24: Helper usage stats tracking.
#[test]
fn test_helper_usage_stats() {
    let mut stats = HelperUsageStats::default();

    stats.record_use("lm_sensors");
    stats.record_use("lm_sensors");
    stats.record_use("smartmontools");

    assert_eq!(stats.helpers.get("lm_sensors").unwrap().use_count, 2);
    assert_eq!(stats.helpers.get("smartmontools").unwrap().use_count, 1);
}

/// Test 26: Full status format output.
#[test]
fn test_full_status_format() {
    let profile = medium_profile();
    let planner = ModelPlanner::new();
    let plan = planner.generate_plan(&profile).unwrap();

    let mut health = ModelHealth::new();
    health.models.insert(
        plan.translator_model.clone(),
        ModelHealthRecord::installed_by_anna(&plan.translator_model),
    );
    health
        .models
        .get_mut(&plan.translator_model)
        .unwrap()
        .status = ModelStatus::Ok;

    let model_config = ModelConfig::new();

    let mut helper_manager = HelperManager::new();
    helper_manager.record_anna_install("lm_sensors");

    let helper_catalog = HelperCatalog::default_catalog();
    let helper_config = HelperConfig::new();

    let status = HardwareStatus::build(
        &profile,
        &plan,
        &health,
        &model_config,
        &helper_manager,
        &helper_catalog,
        &helper_config,
    );

    let formatted = status.format();

    // Verify all sections present
    assert!(formatted.contains("[system_profile]"));
    assert!(formatted.contains("[llm]"));
    assert!(formatted.contains("[helpers]"));

    // Verify key data
    assert!(formatted.contains("32.0 GB"));
    assert!(formatted.contains("Medium"));
    assert!(formatted.contains("ollama"));
    assert!(formatted.contains("lm_sensors"));
}

// === Part H: Acceptance Criteria Tests ===

/// Acceptance 2: Manual model removal detected and routing degrades.
#[test]
fn acceptance_manual_model_removal() {
    let helper = SpecialistHelper::new();
    let planner = ModelPlanner::new();
    let profile = medium_profile();
    let plan = planner.generate_plan(&profile).unwrap();

    let mut health = ModelHealth::new();

    // Junior is installed and OK
    health.models.insert(
        plan.junior_model.clone(),
        ModelHealthRecord::installed_by_anna(&plan.junior_model),
    );
    health.models.get_mut(&plan.junior_model).unwrap().status = ModelStatus::Ok;

    // Senior was installed but now missing (user ran ollama rm)
    health.models.insert(
        plan.senior_model.clone(),
        ModelHealthRecord::installed_by_anna(&plan.senior_model),
    );
    health.models.get_mut(&plan.senior_model).unwrap().status = ModelStatus::Missing;

    // Check availability - should detect missing and offer fallback
    let avail = helper.check_model_availability(ModelRole::Senior, &plan, &health);

    assert!(avail.can_proceed()); // Can proceed with fallback
    assert!(avail.is_fallback());
    assert_eq!(avail.usable_model(), Some(plan.junior_model.as_str()));
}

/// Acceptance 3: Helper request follows policy.
#[test]
fn acceptance_helper_request_policy() {
    let mut config = HelperConfig::new();

    // Policy: ask-per-helper
    config.auto_install = HelperInstallPolicy::AskPerHelper;
    assert_eq!(
        config.can_install("lm_sensors"),
        HelperInstallDecision::NeedsApproval
    );

    // After confirmation
    config.record_decision("lm_sensors", true, None);
    assert_eq!(
        config.can_install("lm_sensors"),
        HelperInstallDecision::Allowed
    );

    // Policy: never
    config.auto_install = HelperInstallPolicy::Never;
    assert!(!config.can_install("smartmontools").can_proceed());
}

/// Acceptance 4: Anna-installed helpers tracked separately.
#[test]
fn acceptance_helper_tracking_separate() {
    let mut manager = HelperManager::new();

    // Anna installs lm_sensors
    manager.record_anna_install("lm_sensors");

    // User had smartmontools already
    manager.record_detected("smartmontools");

    // Verify separation
    let anna = manager.anna_installed();
    let user = manager.user_installed();

    assert!(anna.contains(&"lm_sensors"));
    assert!(!anna.contains(&"smartmontools"));
    assert!(user.contains(&"smartmontools"));
    assert!(!user.contains(&"lm_sensors"));
}
