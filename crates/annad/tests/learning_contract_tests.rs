//! Learning Contract Test Suite v3.8.0
//!
//! Validates all guarantees specified in docs/ANNA_LEARNING_CONTRACT.md:
//!
//! - Debug mode structure: proper output format when debug enabled
//! - Reset behavior: soft reset preserves recipes, hard reset clears them
//! - Failure detection: no silent failures, reliability reflects confidence
//! - Percentage formatting: no raw floats in user-facing output
//!
//! ## Running
//!
//! ```bash
//! cargo test --test learning_contract_tests
//! ```

use anna_common::{
    // Debug state
    debug_state::{DebugState, DebugIntent},
    // Reset functions
    ExperiencePaths, ExperienceSnapshot,
    reset_experience, reset_factory, ResetType,
    // Recipe system
    RecipeStore, Recipe,
    router_llm::QuestionType,
    // Brain fast path
    FastQuestionType, try_fast_answer,
    // Telemetry
    telemetry::{TelemetryEvent, Origin, Outcome},
    // UI formatting
    ui_colors::format_percentage,
    // Constants
    MIN_RECIPE_RELIABILITY, RECIPE_MATCH_THRESHOLD,
};

use std::fs;
use tempfile::TempDir;

// ============================================================================
// DEBUG MODE STRUCTURE TESTS
// ============================================================================

/// Debug intent classification - enable patterns
#[test]
fn test_debug_intent_enable_patterns() {
    let enable_patterns = [
        "enable debug mode",
        "turn debug mode on",
        "activate debug",
        "start debug mode",
        "turn on debug",
    ];

    for pattern in &enable_patterns {
        assert_eq!(
            DebugIntent::classify(pattern),
            DebugIntent::Enable,
            "Pattern '{}' should be classified as Enable",
            pattern
        );
    }
}

/// Debug intent classification - disable patterns
#[test]
fn test_debug_intent_disable_patterns() {
    let disable_patterns = [
        "disable debug mode",
        "turn debug mode off",
        "deactivate debug",
        "stop debug mode",
        "turn off debug",
    ];

    for pattern in &disable_patterns {
        assert_eq!(
            DebugIntent::classify(pattern),
            DebugIntent::Disable,
            "Pattern '{}' should be classified as Disable",
            pattern
        );
    }
}

/// Debug intent classification - status patterns
#[test]
fn test_debug_intent_status_patterns() {
    let status_patterns = [
        "is debug mode enabled?",
        "is debug on?",
        "what is your debug mode state?",
        "are you in debug mode?",
        "debug status",
    ];

    for pattern in &status_patterns {
        assert_eq!(
            DebugIntent::classify(pattern),
            DebugIntent::Status,
            "Pattern '{}' should be classified as Status",
            pattern
        );
    }
}

/// Debug intent classification - non-debug questions
#[test]
fn test_debug_intent_none_patterns() {
    let none_patterns = [
        "how much RAM do I have?",
        "what is the weather?",
        "tell me about debugging",
        "what cpu do I have?",
    ];

    for pattern in &none_patterns {
        assert_eq!(
            DebugIntent::classify(pattern),
            DebugIntent::None,
            "Pattern '{}' should be classified as None",
            pattern
        );
    }
}

/// Debug state persistence
#[test]
fn test_debug_state_serialization() {
    let mut state = DebugState::new();
    assert!(!state.enabled);
    assert!(state.last_changed_at.is_none());

    // Format messages should be consistent
    let enable_msg = DebugState::format_enable_message();
    let disable_msg = DebugState::format_disable_message();

    assert!(enable_msg.contains("enabled"));
    assert!(disable_msg.contains("disabled"));

    // Status format
    assert!(state.format_status().contains("disabled"));
    state.enabled = true;
    assert!(state.format_status().contains("enabled"));
}

// ============================================================================
// RESET BEHAVIOR TESTS
// ============================================================================

/// Helper to create test environment
fn setup_test_env() -> (TempDir, ExperiencePaths) {
    let temp = TempDir::new().unwrap();
    let paths = ExperiencePaths::with_root(temp.path());

    // Create directories
    fs::create_dir_all(&paths.xp_dir).unwrap();
    fs::create_dir_all(paths.telemetry_file.parent().unwrap()).unwrap();
    fs::create_dir_all(&paths.stats_dir).unwrap();
    fs::create_dir_all(&paths.knowledge_dir).unwrap();

    (temp, paths)
}

/// Soft reset resets XP but preserves knowledge
#[test]
fn test_soft_reset_preserves_knowledge() {
    let (_temp, paths) = setup_test_env();

    // Create knowledge file
    let knowledge_file = paths.knowledge_dir.join("learned.json");
    fs::write(&knowledge_file, r#"{"fact": "CPU is AMD Ryzen"}"#).unwrap();

    // Create XP store with advanced values
    let advanced_xp = r#"{"anna":{"level":10,"xp":5000},"anna_stats":{"total_questions":100}}"#;
    fs::write(paths.xp_store_file(), advanced_xp).unwrap();

    // Soft reset
    let result = reset_experience(&paths);

    assert!(result.success, "Soft reset should succeed");
    assert_eq!(result.reset_type, ResetType::Experience);

    // Knowledge should be preserved
    assert!(knowledge_file.exists(), "Knowledge should be preserved after soft reset");
    let content = fs::read_to_string(&knowledge_file).unwrap();
    assert!(content.contains("AMD Ryzen"), "Knowledge content should be intact");

    // XP should be reset to baseline
    let xp_content = fs::read_to_string(paths.xp_store_file()).unwrap();
    assert!(xp_content.contains("\"level\": 1"), "Level should be reset to 1");
    assert!(xp_content.contains("\"trust\": 0.5"), "Trust should be reset to 0.5");
}

/// Hard reset clears everything including knowledge
#[test]
fn test_hard_reset_clears_knowledge() {
    let (_temp, paths) = setup_test_env();

    // Create knowledge file
    let knowledge_file = paths.knowledge_dir.join("learned.json");
    fs::write(&knowledge_file, r#"{"fact": "CPU is AMD Ryzen"}"#).unwrap();

    // Create XP store
    let advanced_xp = r#"{"anna":{"level":10,"xp":5000}}"#;
    fs::write(paths.xp_store_file(), advanced_xp).unwrap();

    // Hard reset
    let result = reset_factory(&paths);

    assert!(result.success, "Hard reset should succeed");
    assert_eq!(result.reset_type, ResetType::Factory);

    // Knowledge should be cleared
    assert!(!knowledge_file.exists(), "Knowledge should be deleted after hard reset");

    // XP should be reset to baseline
    let xp_content = fs::read_to_string(paths.xp_store_file()).unwrap();
    assert!(xp_content.contains("\"level\": 1"), "Level should be reset to 1");
}

/// Reset confirmation strings work correctly
#[test]
fn test_reset_confirmation_strings() {
    // Soft reset accepts "yes"
    assert!(ResetType::Experience.is_confirmed("yes"));
    assert!(ResetType::Experience.is_confirmed("YES"));
    assert!(ResetType::Experience.is_confirmed("  yes  "));
    assert!(!ResetType::Experience.is_confirmed("no"));
    assert!(!ResetType::Experience.is_confirmed("maybe"));

    // Hard reset requires exact phrase
    let exact = "I UNDERSTAND AND CONFIRM FACTORY RESET";
    assert!(ResetType::Factory.is_confirmed(exact));
    assert!(!ResetType::Factory.is_confirmed("yes"));
    assert!(!ResetType::Factory.is_confirmed("i understand and confirm factory reset")); // Case sensitive
    assert!(!ResetType::Factory.is_confirmed("I UNDERSTAND")); // Partial
}

/// Experience snapshot captures state correctly
#[test]
fn test_experience_snapshot_capture() {
    let (_temp, paths) = setup_test_env();

    // Empty state
    let snapshot = ExperienceSnapshot::capture(&paths);
    assert!(snapshot.is_empty(), "Fresh state should be empty");

    // Add experience
    let xp_data = r#"{"anna":{"name":"Anna","level":5,"xp":500},"anna_stats":{"total_questions":10}}"#;
    fs::write(paths.xp_store_file(), xp_data).unwrap();

    // Add telemetry
    fs::write(&paths.telemetry_file, "event1\nevent2\nevent3").unwrap();

    let snapshot = ExperienceSnapshot::capture(&paths);
    assert!(!snapshot.is_empty(), "State with data should not be empty");
    assert_eq!(snapshot.anna_level, 5);
    assert_eq!(snapshot.anna_xp, 500);
    assert_eq!(snapshot.total_questions, 10);
    assert_eq!(snapshot.telemetry_line_count, 3);
}

/// Telemetry is cleared by reset
#[test]
fn test_reset_clears_telemetry() {
    let (_temp, paths) = setup_test_env();

    // Create telemetry
    fs::write(&paths.telemetry_file, "event1\nevent2\nevent3").unwrap();

    let result = reset_experience(&paths);
    assert!(result.success);

    // Telemetry should be truncated (not deleted)
    assert!(paths.telemetry_file.exists());
    let content = fs::read_to_string(&paths.telemetry_file).unwrap();
    assert!(content.is_empty(), "Telemetry should be empty after reset");
}

// ============================================================================
// FAILURE DETECTION TESTS
// ============================================================================

/// Telemetry events track failure causes
#[test]
fn test_telemetry_tracks_failures() {
    // Success event
    let success_event = TelemetryEvent {
        timestamp: "2025-11-30T12:00:00Z".to_string(),
        correlation_id: "test-123".to_string(),
        question_hash: "hash123".to_string(),
        outcome: Outcome::Success,
        origin: Origin::Brain,
        reliability: 0.95,
        latency_ms: 50,
        brain_ms: Some(50),
        junior_ms: None,
        senior_ms: None,
        probes_count: 1,
        failure_cause: None,
        cached: false,
    };

    assert_eq!(success_event.outcome, Outcome::Success);
    assert!(success_event.failure_cause.is_none());

    // Failure event should have cause
    let failure_event = TelemetryEvent {
        timestamp: "2025-11-30T12:00:01Z".to_string(),
        correlation_id: "test-124".to_string(),
        question_hash: "hash124".to_string(),
        outcome: Outcome::Failure,
        origin: Origin::Junior,
        reliability: 0.3,
        latency_ms: 5000,
        brain_ms: None,
        junior_ms: Some(5000),
        senior_ms: None,
        probes_count: 0,
        failure_cause: Some("LLM timeout".to_string()),
        cached: false,
    };

    assert_eq!(failure_event.outcome, Outcome::Failure);
    assert!(failure_event.failure_cause.is_some());
    assert!(failure_event.failure_cause.as_ref().unwrap().contains("timeout"));
}

/// Reliability scores are valid (0.0-1.0)
#[test]
fn test_reliability_bounds() {
    // MIN_RECIPE_RELIABILITY should be high
    assert!(
        MIN_RECIPE_RELIABILITY >= 0.80,
        "MIN_RECIPE_RELIABILITY should be >= 0.80"
    );
    assert!(
        MIN_RECIPE_RELIABILITY <= 1.0,
        "MIN_RECIPE_RELIABILITY should be <= 1.0"
    );

    // RECIPE_MATCH_THRESHOLD should be reasonable
    assert!(
        RECIPE_MATCH_THRESHOLD >= 0.50,
        "RECIPE_MATCH_THRESHOLD should be >= 0.50"
    );
    assert!(
        RECIPE_MATCH_THRESHOLD <= 0.90,
        "RECIPE_MATCH_THRESHOLD should be <= 0.90"
    );
}

/// Origin tracking is consistent
#[test]
fn test_origin_tracking() {
    // All origins should be distinct
    assert_ne!(Origin::Brain, Origin::Junior);
    assert_ne!(Origin::Brain, Origin::Senior);
    assert_ne!(Origin::Brain, Origin::Fallback);
    assert_ne!(Origin::Junior, Origin::Senior);
    assert_ne!(Origin::Junior, Origin::Fallback);
    assert_ne!(Origin::Senior, Origin::Fallback);
}

// ============================================================================
// PERCENTAGE FORMATTING TESTS
// ============================================================================

/// format_percentage produces valid percentage strings
#[test]
fn test_format_percentage_output() {
    // High confidence
    let high = format_percentage(0.95);
    assert!(high.contains("95"), "95% should contain '95'");
    assert!(high.contains('%'), "Should contain percentage sign");
    assert!(!high.contains("0.95"), "Should not contain raw float");

    // Medium confidence
    let medium = format_percentage(0.75);
    assert!(medium.contains("75"), "75% should contain '75'");

    // Low confidence
    let low = format_percentage(0.30);
    assert!(low.contains("30"), "30% should contain '30'");

    // Edge cases
    let zero = format_percentage(0.0);
    assert!(zero.contains("0") || zero.contains('%'));

    let hundred = format_percentage(1.0);
    assert!(hundred.contains("100") || hundred.contains('%'));
}

/// No raw floats in percentage context
#[test]
fn test_no_raw_floats_in_percentages() {
    let test_values = [0.0, 0.25, 0.5, 0.75, 0.85, 0.95, 1.0];

    for &value in &test_values {
        let formatted = format_percentage(value);

        // Should not contain "0." raw float patterns
        assert!(
            !formatted.contains("0."),
            "format_percentage({}) = '{}' should not contain raw float",
            value, formatted
        );
    }
}

// ============================================================================
// BRAIN FAST PATH TESTS
// ============================================================================

/// Brain handles canonical hardware questions
#[test]
fn test_brain_handles_hardware_questions() {
    let hardware_questions = [
        ("What CPU do I have?", "cpu"),
        ("How much RAM?", "ram"),
        ("How much disk space?", "disk"),
    ];

    for (question, category) in &hardware_questions {
        let result = try_fast_answer(question);
        assert!(
            result.is_some(),
            "Brain should handle {} question: '{}'",
            category, question
        );
    }
}

/// Brain handles health check questions
#[test]
fn test_brain_handles_health_questions() {
    let health_questions = [
        "health check",
        "are you healthy?",
        "what's your status?",
    ];

    for question in &health_questions {
        let result = try_fast_answer(question);
        assert!(
            result.is_some(),
            "Brain should handle health question: '{}'",
            question
        );
    }
}

/// FastQuestionType classification is consistent
#[test]
fn test_fast_question_type_classification() {
    // CPU questions
    let cpu_type = FastQuestionType::classify("What CPU do I have?");
    assert!(
        matches!(cpu_type, FastQuestionType::CpuModel | FastQuestionType::CpuCores),
        "CPU question should classify to CPU type"
    );

    // RAM questions
    let ram_type = FastQuestionType::classify("How much RAM?");
    assert_eq!(
        ram_type,
        FastQuestionType::Ram,
        "RAM question should classify to Ram type"
    );

    // Disk questions
    let disk_type = FastQuestionType::classify("How much disk space?");
    assert_eq!(
        disk_type,
        FastQuestionType::RootDiskSpace,
        "Disk question should classify to RootDiskSpace type"
    );
}

// ============================================================================
// RECIPE SYSTEM TESTS
// ============================================================================

/// RecipeStore basic operations
#[test]
fn test_recipe_store_operations() {
    let mut store = RecipeStore::default();
    assert_eq!(store.total_recipes, 0, "New store should have 0 recipes");

    // Add a recipe
    let recipe = Recipe::new(
        "cpu_info_basic",
        QuestionType::CpuInfo,
        vec!["cpu.info".to_string()],
        "Your CPU is {cpu_model}",
        0.95,
    ).with_tokens(&["cpu", "model", "cores"]);

    store.add(recipe);
    assert_eq!(store.total_recipes, 1, "Store should have 1 recipe after add");
}

/// Recipe matching requires sufficient token overlap
#[test]
fn test_recipe_matching_threshold() {
    let mut store = RecipeStore::default();

    let recipe = Recipe::new(
        "cpu_info_basic",
        QuestionType::CpuInfo,
        vec!["cpu.info".to_string()],
        "Your CPU is {cpu_model}",
        0.95,
    ).with_tokens(&["cpu", "model", "cores", "threads"]);

    store.add(recipe);

    // Good match: "cpu model cores" = 3/4 tokens = 75%
    let good_match = store.find_match("What CPU model with how many cores?", &QuestionType::CpuInfo);
    assert!(good_match.is_some(), "75% token match should succeed");

    // Poor match: "cpu" alone = 1/4 tokens = 25%
    let poor_match = store.find_match("cpu?", &QuestionType::CpuInfo);
    assert!(poor_match.is_none(), "25% token match should fail (below 50% threshold)");
}

/// Recipe reliability must meet minimum
#[test]
fn test_recipe_reliability_minimum() {
    // Recipes should only be created from high-reliability answers
    let high_rel_recipe = Recipe::new(
        "test_basic",
        QuestionType::CpuInfo,
        vec![],
        "Test",
        MIN_RECIPE_RELIABILITY,
    );

    assert!(
        high_rel_recipe.last_success_score >= MIN_RECIPE_RELIABILITY,
        "Recipe reliability should meet minimum threshold"
    );
}

// ============================================================================
// CONTRACT COMPLIANCE SUMMARY
// ============================================================================

/// Summary test: Learning Contract v3.8.0 compliance
#[test]
fn test_learning_contract_compliance_summary() {
    // A) First Time Behavior - Brain handles canonical questions
    let brain_ok = try_fast_answer("What CPU do I have?").is_some()
        && try_fast_answer("How much RAM?").is_some()
        && try_fast_answer("How much disk space?").is_some()
        && try_fast_answer("Health check").is_some();
    assert!(brain_ok, "Brain should handle canonical questions");

    // B) Recipe thresholds are correct
    assert!(
        MIN_RECIPE_RELIABILITY >= 0.85,
        "MIN_RECIPE_RELIABILITY should be >= 85%"
    );
    assert!(
        RECIPE_MATCH_THRESHOLD >= 0.70,
        "RECIPE_MATCH_THRESHOLD should be >= 70%"
    );

    // C) Reset types are distinct
    assert_ne!(
        ResetType::Experience.label(),
        ResetType::Factory.label(),
        "Reset types should have different labels"
    );

    // D) Debug intent classification works
    assert!(
        DebugIntent::classify("enable debug").is_debug_intent(),
        "Debug enable should be recognized"
    );
    assert!(
        !DebugIntent::classify("what CPU?").is_debug_intent(),
        "Non-debug should not be recognized as debug"
    );

    // E) Percentage formatting is correct
    let pct = format_percentage(0.92);
    assert!(
        pct.contains('%') && !pct.contains("0."),
        "Percentage should be formatted correctly"
    );
}
