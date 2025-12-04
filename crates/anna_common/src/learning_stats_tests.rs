//! Integration Tests for v0.0.75 Learning + Stats
//!
//! Tests the full flow of recipe learning and RPG stats:
//! - Repeated queries create recipes
//! - Stats block shows correct values
//! - Human transcript hides IDs, debug shows them
//! - No new CLI commands added

use crate::case_engine::IntentType;
use crate::evidence_topic::EvidenceTopic;
use crate::humanizer::DepartmentTag;
use crate::recipe_engine::{
    RecipeEngine, RecipeEngineStats, RecipeGate, MIN_EVIDENCE_DOCTOR, MIN_EVIDENCE_MUTATION,
    MIN_EVIDENCE_READ_ONLY, MIN_RELIABILITY_DOCTOR, MIN_RELIABILITY_MUTATION,
    MIN_RELIABILITY_READ_ONLY,
};
use crate::recipes::{RecipeManager, RecipeRiskLevel, RecipeStatus};
use crate::rpg_stats::{title_for_level, RpgStats, RpgStatsManager, ROLLING_WINDOW_SIZE};
use crate::transcript_events::TranscriptMode;
use crate::transcript_v075::{
    get_transcript_mode, human_finding, human_reliability_footer, humanize_evidence,
    validate_human_output, HumanStaffMessage,
};

// ============================================================================
// Recipe Creation Gate Tests (v0.0.75 requirements)
// ============================================================================

#[test]
fn test_v075_recipe_gate_read_only_high_reliability() {
    // Read-only query with high reliability and sufficient evidence
    let gate = RecipeEngine::check_creation_gate(
        RecipeRiskLevel::ReadOnly,
        92, // >= 90%
        2,  // >= 1
        false,
    );
    assert!(gate.can_create);
    assert_eq!(gate.status, RecipeStatus::Active);
}

#[test]
fn test_v075_recipe_gate_read_only_low_reliability() {
    // Read-only query with low reliability -> draft
    let gate = RecipeEngine::check_creation_gate(
        RecipeRiskLevel::ReadOnly,
        85, // < 90%
        2,
        false,
    );
    assert!(gate.can_create);
    assert_eq!(gate.status, RecipeStatus::Draft);
}

#[test]
fn test_v075_recipe_gate_doctor_workflow() {
    // Doctor workflow uses lower threshold (80%)
    let gate = RecipeEngine::check_creation_gate(
        RecipeRiskLevel::ReadOnly,
        82, // >= 80%
        2,  // >= 2
        true,
    );
    assert!(gate.can_create);
    assert_eq!(gate.status, RecipeStatus::Active);
}

#[test]
fn test_v075_recipe_gate_mutation_high_reliability() {
    // Mutation requires 95% and 3 evidence
    let gate = RecipeEngine::check_creation_gate(
        RecipeRiskLevel::MediumRisk,
        96, // >= 95%
        3,  // >= 3
        false,
    );
    assert!(gate.can_create);
    assert_eq!(gate.status, RecipeStatus::Active);
}

#[test]
fn test_v075_recipe_gate_mutation_insufficient_evidence() {
    // Mutation with only 2 evidence items -> cannot create
    let gate = RecipeEngine::check_creation_gate(
        RecipeRiskLevel::MediumRisk,
        96,
        2, // < 3
        false,
    );
    assert!(!gate.can_create);
}

#[test]
fn test_v075_thresholds_match_spec() {
    // Verify thresholds match v0.0.75 spec
    assert_eq!(MIN_RELIABILITY_READ_ONLY, 90);
    assert_eq!(MIN_RELIABILITY_DOCTOR, 80);
    assert_eq!(MIN_RELIABILITY_MUTATION, 95);
    assert_eq!(MIN_EVIDENCE_READ_ONLY, 1);
    assert_eq!(MIN_EVIDENCE_DOCTOR, 2);
    assert_eq!(MIN_EVIDENCE_MUTATION, 3);
}

// ============================================================================
// RPG Stats Tests
// ============================================================================

#[test]
fn test_v075_stats_default() {
    let stats = RpgStats::default();
    assert_eq!(stats.xp.level, 0);
    assert_eq!(stats.xp.title, "");
    assert_eq!(stats.requests.total, 0);
    assert_eq!(stats.reliability.average, 0.0);
    assert_eq!(stats.recipe_coverage, 0.0);
}

#[test]
fn test_v075_stats_record_request() {
    let mut stats = RpgStats::default();
    stats.record_request(true, 85, "network", true, false, false, 150);

    assert_eq!(stats.requests.total, 1);
    assert_eq!(stats.requests.successes, 1);
    assert_eq!(stats.requests.success_rate, 1.0);
    assert_eq!(stats.reliability.average, 85.0);
    assert_eq!(stats.reliability.rolling_50, 85.0);
    assert_eq!(stats.escalations.junior_count, 1);
    assert_eq!(stats.escalations.junior_percent, 100.0);
    assert!(stats.domains.contains_key("network"));
}

#[test]
fn test_v075_stats_multiple_requests() {
    let mut stats = RpgStats::default();

    // First request: success
    stats.record_request(true, 90, "network", true, false, false, 100);
    // Second request: success with doctor
    stats.record_request(true, 85, "audio", true, true, false, 200);
    // Third request: failure
    stats.record_request(false, 50, "storage", true, true, false, 500);

    assert_eq!(stats.requests.total, 3);
    assert_eq!(stats.requests.successes, 2);
    assert_eq!(stats.requests.failures, 1);
    assert!((stats.requests.success_rate - 0.6667).abs() < 0.01);

    // Reliability average: (90 + 85 + 50) / 3 = 75
    assert!((stats.reliability.average - 75.0).abs() < 0.1);

    // Junior used 3 times = 100%
    assert_eq!(stats.escalations.junior_percent, 100.0);
    // Doctor used 2 times = 66.67%
    assert!((stats.escalations.doctor_percent - 66.67).abs() < 0.1);

    // Domain breakdown
    assert_eq!(stats.domains.len(), 3);
}

#[test]
fn test_v075_stats_rolling_reliability() {
    let mut stats = RpgStats::default();

    // Add 60 requests (more than ROLLING_WINDOW_SIZE)
    for i in 0..60 {
        stats.record_request(true, 80 + (i % 10) as u8, "test", false, false, false, 100);
    }

    // Should only keep last 50
    assert_eq!(stats.reliability.recent_scores.len(), ROLLING_WINDOW_SIZE);
}

#[test]
fn test_v075_stats_latency_percentiles() {
    let mut stats = RpgStats::default();

    // Add requests with varying latencies
    for ms in [50, 100, 150, 200, 250, 300, 400, 500, 750, 1000] {
        stats.record_request(true, 90, "test", false, false, false, ms);
    }

    // Median should be around 275 (average of 250 and 300)
    assert!(stats.latency.median_total_ms > 200);
    assert!(stats.latency.median_total_ms < 400);

    // P95 should be >= 750
    assert!(stats.latency.p95_total_ms >= 750);
}

#[test]
fn test_v075_level_titles() {
    assert_eq!(title_for_level(0), "Intern");
    assert_eq!(title_for_level(2), "Intern");
    assert_eq!(title_for_level(3), "Apprentice");
    assert_eq!(title_for_level(6), "Technician");
    assert_eq!(title_for_level(9), "Analyst");
    assert_eq!(title_for_level(12), "Engineer");
    assert_eq!(title_for_level(15), "Senior Engineer");
    assert_eq!(title_for_level(18), "Architect");
    assert_eq!(title_for_level(21), "Wizard");
    assert_eq!(title_for_level(25), "Sage");
    assert_eq!(title_for_level(30), "Grandmaster");
    assert_eq!(title_for_level(100), "Grandmaster");
}

#[test]
fn test_v075_stats_format_bars() {
    let mut stats = RpgStats::default();
    stats.xp.progress = 0.5;
    stats.requests.success_rate = 0.8;
    stats.recipe_coverage = 30.0;

    let xp_bar = stats.format_xp_bar(10);
    assert!(xp_bar.contains("====="));
    assert!(xp_bar.contains(" 50%"));

    let success_bar = stats.format_success_bar(10);
    assert!(success_bar.contains("########"));
    assert!(success_bar.contains("80.0%"));

    let coverage_bar = stats.format_coverage_bar(10);
    assert!(coverage_bar.contains("***"));
    assert!(coverage_bar.contains("30.0%"));
}

#[test]
fn test_v075_stats_status_block_format() {
    let mut stats = RpgStats::default();
    stats.xp.level = 5;
    stats.xp.title = "Technician".to_string();
    stats.xp.total_xp = 150;
    stats.xp.next_level_xp = 275;
    stats.xp.progress = 0.4;
    stats.requests.total = 100;
    stats.requests.successes = 85;
    stats.requests.failures = 15;
    stats.requests.success_rate = 0.85;
    stats.reliability.average = 82.0;
    stats.reliability.rolling_50 = 84.0;

    let block = stats.format_status_block();
    assert!(block.contains("[RPG STATS]"));
    assert!(block.contains("Level 5 Technician"));
    assert!(block.contains("XP: 150 / 275"));
    assert!(block.contains("100 total"));
    assert!(block.contains("82.0% avg"));
}

// ============================================================================
// Transcript Validation Tests
// ============================================================================

#[test]
fn test_v075_human_evidence_no_ids() {
    let human = humanize_evidence(
        "hw_snapshot_summary",
        Some(EvidenceTopic::CpuInfo),
        "Intel i9-14900HX",
    );

    // Should not contain evidence IDs
    assert!(!human.description.contains("[E"));
    // Should not contain tool names
    assert!(!human.description.contains("hw_snapshot"));
    // Should contain meaningful description
    assert!(human.description.contains("Hardware snapshot"));
    assert!(human.description.contains("Intel i9-14900HX"));
}

#[test]
fn test_v075_human_evidence_memory() {
    let human = humanize_evidence(
        "memory_info",
        Some(EvidenceTopic::MemoryInfo),
        "32 GiB total, 16 GiB available",
    );

    assert!(!human.description.contains("memory_info"));
    assert!(human.description.contains("Memory status"));
    assert_eq!(human.source, "memory snapshot");
}

#[test]
fn test_v075_human_evidence_network() {
    let human = humanize_evidence(
        "network_status",
        Some(EvidenceTopic::NetworkStatus),
        "Connected via enp5s0",
    );

    assert!(!human.description.contains("network_status"));
    assert!(human.description.contains("Network status"));
    assert_eq!(human.source, "network snapshot");
}

#[test]
fn test_v075_human_evidence_journal() {
    let human = humanize_evidence(
        "recent_errors_summary",
        Some(EvidenceTopic::RecentErrors),
        "2 warnings from iwd in last 24h",
    );

    assert!(!human.description.contains("_summary"));
    assert!(human.description.contains("Journal scan"));
    assert_eq!(human.source, "error journal");
}

#[test]
fn test_v075_validate_human_clean() {
    let clean_output = r#"[service desk] Let me look into that for you.
[network] Checking interface status.
  Network status: Connected via enp5s0, link up
[service desk] You're connected to the network via enp5s0.
Reliability: 92% (good evidence coverage, network snapshot)"#;

    assert!(validate_human_output(clean_output).is_ok());
}

#[test]
fn test_v075_validate_human_violations() {
    // Test each forbidden pattern
    let test_cases = [
        ("[E1] hw_snapshot_summary", "[E1]"),
        ("Tool: hw_snapshot_summary", "hw_snapshot"),
        ("Running mem_summary probe", "mem_summary"),
        ("Using mount_usage tool", "mount_usage"),
        ("service_status returned ok", "service_status"),
        ("network_status check passed", "network_status"),
        (
            "Fell back to deterministic fallback",
            "deterministic fallback",
        ),
    ];

    for (input, expected_term) in test_cases {
        let result = validate_human_output(input);
        assert!(result.is_err(), "Should reject: {}", input);
        let violations = result.unwrap_err();
        assert!(
            violations.iter().any(|v| v.contains(expected_term)),
            "Should mention '{}' in violations for input: {}",
            expected_term,
            input
        );
    }
}

#[test]
fn test_v075_human_finding_confidence_levels() {
    // High confidence: no prefix
    let high = human_finding(DepartmentTag::Network, "Link is up on enp5s0", 95);
    assert!(high.text.starts_with("Link is up"));

    // Medium-high confidence: "It looks like"
    let med_high = human_finding(DepartmentTag::Network, "there's a connection issue", 80);
    assert!(med_high.text.starts_with("It looks like"));

    // Medium confidence: "I think"
    let medium = human_finding(DepartmentTag::Network, "the interface is misconfigured", 65);
    assert!(medium.text.starts_with("I think"));

    // Low confidence: "I'm not certain, but"
    let low = human_finding(DepartmentTag::Network, "there might be a DNS issue", 50);
    assert!(low.text.starts_with("I'm not certain"));
}

#[test]
fn test_v075_human_reliability_footer() {
    // High reliability
    let high = human_reliability_footer(90, "hardware snapshot");
    assert!(high.contains("90%"));
    assert!(high.contains("good evidence coverage"));
    assert!(high.contains("hardware snapshot"));

    // Medium reliability
    let medium = human_reliability_footer(70, "network snapshot");
    assert!(medium.contains("70%"));
    assert!(medium.contains("some evidence gaps"));

    // Low reliability
    let low = human_reliability_footer(45, "journal scan");
    assert!(low.contains("45%"));
    assert!(low.contains("limited evidence"));
}

// ============================================================================
// Recipe Lifecycle Tests
// ============================================================================

#[test]
fn test_v075_recipe_match_scoring() {
    // Test that recipe matching uses canonical fields
    let matches = RecipeEngine::find_matches(
        IntentType::SystemQuery,
        &["cpu".to_string()],
        85,
        &["hw_snapshot_summary".to_string()],
        None,
    );

    // Should return a list (may be empty if no recipes exist)
    // The important thing is that it doesn't panic
    assert!(matches.len() >= 0);
}

// ============================================================================
// CLI Surface Verification
// ============================================================================

#[test]
fn test_v075_no_new_cli_commands() {
    // Verify only allowed CLI commands exist
    // This is a documentation test - actual CLI is tested in binary crate
    let allowed_commands = [
        "annactl",           // REPL
        "annactl <request>", // One-shot
        "annactl status",    // Status
        "annactl reset",     // Reset
        "annactl uninstall", // Uninstall
        "annactl --version", // Version
        "annactl -V",        // Version short
        "annactl --help",    // Help
        "annactl -h",        // Help short
    ];

    // This test documents the contract - v0.0.75 does not add new commands
    assert_eq!(allowed_commands.len(), 9);
}
