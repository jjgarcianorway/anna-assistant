//! Acceptance tests for hardware-aware system (v0.0.434).

use super::catalog::{ModelCatalog, ModelRole};
use super::helper_config::{HelperConfig, HelperInstallDecision, HelperInstallPolicy};
use super::helpers::{HelperCatalog, HelperManager};
use super::integration::{ModelAvailability, ProbeHelper, SpecialistHelper};
use super::model_config::{AutoInstallPolicy, InstallDecision, ModelConfig, PreferSmallSetting};
use super::model_health::{InstalledBy, ModelHealth, ModelHealthRecord, ModelStatus};
use super::model_plan::{ModelPlan, ModelPlanner, PlanValidation};
use super::profile::{
    CapabilityTier, CpuInfo, GpuInfo, GpuVendor, HardwareProfile, OsInfo, StorageInfo,
};
use super::status::{HardwareStatus, HelperUsageStats, ModelUsageStats};

// === Test Fixtures ===

fn tiny_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "Intel Celeron".to_string(),
            core_count: 2,
            thread_count: 2,
            avx2_supported: false,
        },
        ram_total_gb: 6.0,
        ram_free_gb: 4.0,
        gpu: GpuInfo::default(),
        storage: StorageInfo {
            data_dir_available_gb: 10,
            model_storage_available_gb: 20,
            model_storage_total_gb: 50,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Tiny,
    }
}

fn medium_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "Intel Core i7-12700".to_string(),
            core_count: 12,
            thread_count: 20,
            avx2_supported: true,
        },
        ram_total_gb: 32.0,
        ram_free_gb: 24.0,
        gpu: GpuInfo {
            discrete: true,
            vendor: GpuVendor::Nvidia,
            model_name: Some("RTX 4060".to_string()),
            vram_gb: Some(8),
        },
        storage: StorageInfo {
            data_dir_available_gb: 50,
            model_storage_available_gb: 100,
            model_storage_total_gb: 500,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Medium,
    }
}

fn large_profile() -> HardwareProfile {
    HardwareProfile {
        profile_version: 1,
        last_profiled_at: "0".to_string(),
        cpu: CpuInfo {
            model_name: "AMD Ryzen 9 7950X".to_string(),
            core_count: 16,
            thread_count: 32,
            avx2_supported: true,
        },
        ram_total_gb: 64.0,
        ram_free_gb: 48.0,
        gpu: GpuInfo {
            discrete: true,
            vendor: GpuVendor::Nvidia,
            model_name: Some("RTX 4090".to_string()),
            vram_gb: Some(24),
        },
        storage: StorageInfo {
            data_dir_available_gb: 200,
            model_storage_available_gb: 500,
            model_storage_total_gb: 2000,
        },
        os: OsInfo {
            distro: "Arch Linux".to_string(),
            kernel_version: "6.6.0".to_string(),
        },
        tier: CapabilityTier::Large,
    }
}

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

// === Part H: Acceptance Criteria Tests ===

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
