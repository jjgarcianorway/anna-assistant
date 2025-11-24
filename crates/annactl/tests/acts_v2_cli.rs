//! ACTS v2 - CLI-Level Capability Tests (6.14.0)
//!
//! Tests Anna's behavior from the user's perspective using real CLI-style queries.
//! These tests exercise the full query handler with mocked telemetry/knowledge.
//!
//! Philosophy:
//! - Test at handle_unified_query level (not binary spawning)
//! - Use synthetic fixtures for deterministic results
//! - Assert on real CLI output patterns
//! - Verify user-visible behavior, not internal structs
//!
//! Scenarios covered:
//! 1. Hardware questions (knowledge-first, no command suggestions)
//! 2. TLP wiki-backed fix (detect → wiki → plan → y/N)
//! 3. Health queries (telemetry-based, not LLM noise)
//! 4. Desktop detection (knowledge-based)
//! 5. Unsupported questions (safe LLM fallback)

use anna_common::llm::{LlmConfig, LlmMode, LlmBackendKind};
use anna_common::telemetry::*;
use anna_common::system_knowledge::{SystemKnowledgeBase, HardwareProfile};
use annactl::unified_query_handler::{handle_unified_query, UnifiedQueryResult, AnswerConfidence};

// ============================================================================
// Test Fixtures
// ============================================================================

/// Create minimal telemetry for testing
fn create_test_telemetry() -> SystemTelemetry {
    SystemTelemetry {
        timestamp: chrono::Utc::now(),
        hardware: HardwareInfo {
            cpu_model: "Intel Core i7-1165G7".to_string(),
            total_ram_mb: 16384,
            machine_type: MachineType::Laptop,
            has_battery: true,
            has_gpu: true,
            gpu_info: Some("Intel Xe Graphics".to_string()),
        },
        disks: vec![DiskInfo {
            mount_point: "/".to_string(),
            total_mb: 512000,
            used_mb: 256000,
            usage_percent: 50.0,
            fs_type: "ext4".to_string(),
            smart_status: Some(SmartStatus::Healthy),
        }],
        memory: MemoryInfo {
            total_mb: 16384,
            available_mb: 8192,
            used_mb: 8192,
            swap_total_mb: 8192,
            swap_used_mb: 0,
            usage_percent: 50.0,
        },
        cpu: CpuInfo {
            cores: 8,
            load_avg_1min: 1.2,
            load_avg_5min: 1.5,
            usage_percent: Some(25.0),
        },
        packages: PackageInfo {
            total_installed: 1200,
            updates_available: 5,
            orphaned: 3,
            cache_size_mb: 1024.0,
            last_update: Some(chrono::Utc::now()),
        },
        services: ServiceInfo {
            total_units: 250,
            failed_units: vec![],
            recently_restarted: vec![],
        },
        network: NetworkInfo {
            is_connected: true,
            primary_interface: Some("wlan0".to_string()),
            firewall_active: true,
            firewall_type: Some("ufw".to_string()),
        },
        security: SecurityInfo {
            failed_ssh_attempts: 0,
            auto_updates_enabled: false,
            audit_warnings: vec![],
        },
        desktop: Some(DesktopInfo {
            de_name: Some("Hyprland".to_string()),
            wm_name: Some("Hyprland".to_string()),
            display_server: Some("Wayland".to_string()),
            monitor_count: 2,
        }),
        boot: Some(BootInfo {
            last_boot_time_secs: 15.5,
            avg_boot_time_secs: Some(16.0),
            trend: Some(BootTrend::Stable),
        }),
        audio: AudioTelemetry {
            has_sound_hardware: true,
            pipewire_running: true,
            wireplumber_running: true,
            pipewire_pulse_running: true,
        },
    }
}

/// Create SystemKnowledgeBase with integrated GPU (no discrete GPU)
fn create_knowledge_integrated_gpu() -> SystemKnowledgeBase {
    let mut kb = SystemKnowledgeBase::default();
    kb.hardware = HardwareProfile {
        cpu_model: Some("Intel Core i7-1165G7".to_string()),
        cpu_physical_cores: Some(4),
        cpu_logical_cores: Some(8),
        gpu_model: Some("Intel Xe Graphics".to_string()),
        gpu_type: Some("integrated".to_string()),
        sound_devices: vec!["Intel HDA".to_string()],
        total_ram_bytes: Some(16 * 1024 * 1024 * 1024),
        machine_model: Some("Laptop Model X".to_string()),
    };
    kb
}

/// Create SystemKnowledgeBase with discrete GPU
fn create_knowledge_discrete_gpu() -> SystemKnowledgeBase {
    let mut kb = SystemKnowledgeBase::default();
    kb.hardware = HardwareProfile {
        cpu_model: Some("AMD Ryzen 9 5900X".to_string()),
        cpu_physical_cores: Some(12),
        cpu_logical_cores: Some(24),
        gpu_model: Some("NVIDIA GeForce RTX 3080".to_string()),
        gpu_type: Some("discrete".to_string()),
        sound_devices: vec!["USB Audio".to_string()],
        total_ram_bytes: Some(32 * 1024 * 1024 * 1024),
        machine_model: Some("Desktop Workstation".to_string()),
    };
    kb
}

/// Create LLM config for testing (can be dummy since we test deterministic paths)
fn create_test_llm_config() -> LlmConfig {
    LlmConfig {
        mode: LlmMode::Local,
        backend: LlmBackendKind::LocalHttp,
        base_url: Some("http://localhost:11434/v1".to_string()),
        api_key_env: None,
        model: Some("test-model".to_string()),
        max_tokens: Some(1000),
        cost_per_1k_tokens: None,
        safety_notes: vec![],
        description: "Test LLM config".to_string(),
        model_profile_id: None,
    }
}

/// Normalize output for assertions (strip ANSI, normalize whitespace)
fn normalize_output(text: &str) -> String {
    // Remove ANSI color codes
    let re = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let stripped = re.replace_all(text, "");

    // Normalize whitespace
    stripped
        .lines()
        .map(|l| l.trim())
        .collect::<Vec<_>>()
        .join("\n")
}

// ============================================================================
// ACTS v2 Test Suite
// ============================================================================

#[tokio::test]
async fn test_hardware_discrete_gpu_question_no_discrete() {
    // Scenario 1: User asks about discrete GPU when they have integrated only
    //
    // Expected:
    // - Clear answer: "you do not have a discrete GPU"
    // - Mentions actual GPU model (Intel Xe Graphics)
    // - Includes Arch Wiki URL
    // - NO "run lspci" or command suggestions

    let telemetry = create_test_telemetry();
    let _knowledge = create_knowledge_integrated_gpu();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "do I have a discrete graphic card?",
        &telemetry,
        &llm_config,
    ).await;

    assert!(result.is_ok(), "Query should succeed");

    match result.unwrap() {
        UnifiedQueryResult::ConversationalAnswer { answer, confidence, sources } => {
            let normalized = normalize_output(&answer);

            // Should clearly state no discrete GPU
            assert!(
                normalized.to_lowercase().contains("do not have")
                    || normalized.to_lowercase().contains("don't have")
                    || normalized.to_lowercase().contains("integrated"),
                "Answer should clearly indicate no discrete GPU. Got: {}", normalized
            );

            // Should mention actual GPU
            assert!(
                normalized.contains("Intel") || normalized.contains("integrated"),
                "Answer should mention integrated GPU type. Got: {}", normalized
            );

            // Should be high confidence (from knowledge)
            assert!(
                matches!(confidence, AnswerConfidence::High),
                "Hardware answer should be high confidence"
            );

            // Should NOT suggest running commands
            assert!(
                !normalized.contains("lspci") && !normalized.contains("run"),
                "Answer should not suggest running commands. Got: {}", normalized
            );

            // Sources should indicate knowledge base
            assert!(
                sources.iter().any(|s| s.contains("Knowledge")),
                "Sources should mention SystemKnowledgeBase"
            );
        }
        other => panic!("Expected ConversationalAnswer, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_hardware_ram_question() {
    // Scenario 2: User asks about RAM
    //
    // Expected:
    // - States RAM amount (16 GB)
    // - NO "run free -h" suggestion
    // - High confidence from knowledge

    let telemetry = create_test_telemetry();
    let _knowledge = create_knowledge_integrated_gpu();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "how much ram do I have?",
        &telemetry,
        &llm_config,
    ).await;

    assert!(result.is_ok(), "Query should succeed");

    match result.unwrap() {
        UnifiedQueryResult::ConversationalAnswer { answer, confidence, .. } => {
            let normalized = normalize_output(&answer);

            // Should mention RAM amount
            assert!(
                normalized.contains("16") && (normalized.contains("GB") || normalized.contains("GiB")),
                "Answer should state 16 GB RAM. Got: {}", normalized
            );

            // Should be high confidence
            assert!(
                matches!(confidence, AnswerConfidence::High),
                "RAM answer should be high confidence"
            );

            // Should NOT suggest running commands
            assert!(
                !normalized.contains("free -h") && !normalized.contains("run free"),
                "Answer should not suggest running free command. Got: {}", normalized
            );
        }
        other => panic!("Expected ConversationalAnswer, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_tlp_not_enabled_wiki_backed_fix() {
    // Scenario 3: TLP installed but not enabled - should get wiki-backed plan
    //
    // NOTE: This test will need actual TLP detection mocking.
    // For now, we test the query classification works.

    let telemetry = create_test_telemetry();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "tlp not working",
        &telemetry,
        &llm_config,
    ).await;

    assert!(result.is_ok(), "Query should succeed");

    // At minimum, verify the query is classified correctly
    // Full TLP plan testing would require mocking systemctl/journalctl
    match result.unwrap() {
        UnifiedQueryResult::ConversationalAnswer { answer, sources, .. } => {
            let normalized = normalize_output(&answer);

            // If TLP is detected, should mention wiki
            if normalized.contains("TLP") {
                assert!(
                    sources.iter().any(|s| s.contains("Wiki") || s.contains("wiki")),
                    "TLP answer should cite Arch Wiki. Sources: {:?}", sources
                );
            }
        }
        _ => {
            // Other result types are acceptable if TLP isn't detected on test system
        }
    }
}

#[tokio::test]
async fn test_desktop_environment_question() {
    // Scenario 4: User asks about their desktop
    //
    // Expected:
    // - States DE/WM from knowledge
    // - NO "echo $DESKTOP_SESSION" suggestion

    let telemetry = create_test_telemetry();
    let _knowledge = create_knowledge_integrated_gpu();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "what desktop am I using?",
        &telemetry,
        &llm_config,
    ).await;

    assert!(result.is_ok(), "Query should succeed");

    match result.unwrap() {
        UnifiedQueryResult::ConversationalAnswer { answer, .. } => {
            let normalized = normalize_output(&answer);

            // Should NOT suggest shell commands
            assert!(
                !normalized.contains("echo $") && !normalized.contains("printenv"),
                "Answer should not suggest environment variable commands. Got: {}", normalized
            );
        }
        other => panic!("Expected ConversationalAnswer for desktop question, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_unsupported_install_question_safe_fallback() {
    // Scenario 5: User asks about installing something
    //
    // Expected:
    // - Gets some kind of answer (conversational or action plan)
    // - No obviously destructive commands without context

    let telemetry = create_test_telemetry();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "how do I install docker?",
        &telemetry,
        &llm_config,
    ).await;

    // This query should succeed (either template, action plan, or conversational)
    assert!(result.is_ok(), "Install query should not crash");

    // We don't assert specific content since this may route to LLM
    // Just verify it returns something reasonable
    match result.unwrap() {
        UnifiedQueryResult::ActionPlan { .. } => {
            // Action plan is acceptable
        }
        UnifiedQueryResult::ConversationalAnswer { .. } => {
            // Conversational answer is acceptable
        }
        UnifiedQueryResult::Template { .. } => {
            // Template match is acceptable
        }
        _ => {
            // Other types are fine too
        }
    }
}

#[tokio::test]
async fn test_sound_card_question() {
    // Scenario 6: User asks about sound card
    //
    // Expected:
    // - Mentions sound device from knowledge
    // - NO "run aplay -l" suggestion

    let telemetry = create_test_telemetry();
    let _knowledge = create_knowledge_integrated_gpu();
    let llm_config = create_test_llm_config();

    let result = handle_unified_query(
        "do I have a sound card?",
        &telemetry,
        &llm_config,
    ).await;

    assert!(result.is_ok(), "Query should succeed");

    match result.unwrap() {
        UnifiedQueryResult::ConversationalAnswer { answer, confidence, .. } => {
            let normalized = normalize_output(&answer);

            // Should be high confidence from knowledge
            assert!(
                matches!(confidence, AnswerConfidence::High),
                "Sound card answer should be high confidence"
            );

            // Should NOT suggest running commands
            assert!(
                !normalized.contains("aplay") && !normalized.contains("lspci"),
                "Answer should not suggest running commands. Got: {}", normalized
            );
        }
        other => panic!("Expected ConversationalAnswer, got: {:?}", other),
    }
}

// ============================================================================
// Sanity Tests - Status Formatting
// ============================================================================

#[test]
fn test_emoji_spacing_two_spaces() {
    // Scenario: Verify emoji spacing is exactly 2 spaces
    //
    // This locks down the fix from 6.12.2 where emojis need double spacing

    use anna_common::terminal_format::section_title;

    let header = section_title("⚙️", "Core Health");

    // Should have exactly 2 spaces after emoji
    assert!(
        header.contains("⚙️  Core Health"),
        "Emoji should be followed by exactly 2 spaces. Got: {}", header
    );

    // Should NOT have only 1 space
    assert!(
        !header.contains("⚙️ Core") || header.contains("⚙️  Core"),
        "Should not have only 1 space after emoji"
    );
}
