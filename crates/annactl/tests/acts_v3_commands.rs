//! ACTS v3 - Command Intelligence Layer Test Suite (6.15.0)
//!
//! Tests dynamic command synthesis for operational "how do I" questions.
//!
//! Philosophy:
//! - NO hardcoded command answers
//! - Pattern-based assertions (regex matching)
//! - Tests classification → synthesis → wiki attachment
//! - Validates CIL can handle diverse questions without templates

use anna_common::command_intelligence::{
    classify_command_query, resolve_command_for_intent, CommandIntent, InspectTarget,
};
use anna_common::system_knowledge::SystemKnowledgeBase;
use anna_common::telemetry::*;
use anna_common::llm::{LlmConfig, LlmMode, LlmBackendKind};
use annactl::unified_query_handler::{handle_unified_query, UnifiedQueryResult};
use regex::Regex;

// ============================================================================
// Test Fixtures
// ============================================================================

/// CIL test case with pattern matching
struct CilTest {
    question: &'static str,
    accepted_patterns: &'static [&'static str],
    description: &'static str,
}

/// Test telemetry fixture
fn create_test_telemetry() -> SystemTelemetry {
    SystemTelemetry {
        timestamp: chrono::Utc::now(),
        hardware: HardwareInfo {
            cpu_model: "Intel Core i7-8550U".to_string(),
            total_ram_mb: 16384,
            machine_type: MachineType::Laptop,
            has_battery: true,
            has_gpu: true,
            gpu_info: Some("Intel UHD Graphics 620".to_string()),
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

// ============================================================================
// ACTS v3 Test Cases
// ============================================================================

const ACTS_V3_TESTS: &[CilTest] = &[
    // CPU inspection
    CilTest {
        question: "how do I check my CPU?",
        accepted_patterns: &[r"lscpu", r"cat\s+/proc/cpuinfo"],
        description: "CPU info command",
    },
    CilTest {
        question: "how can I see CPU usage?",
        accepted_patterns: &[r"htop", r"top", r"lscpu"],
        description: "CPU usage monitoring",
    },

    // Memory inspection
    CilTest {
        question: "how do I check memory usage?",
        accepted_patterns: &[r"free\s+-h", r"vmstat"],
        description: "Memory usage",
    },
    CilTest {
        question: "how do I list my RAM?",
        accepted_patterns: &[r"free"],
        description: "RAM listing",
    },

    // Disk inspection
    CilTest {
        question: "how do I list my disks?",
        accepted_patterns: &[r"lsblk", r"fdisk\s+-l", r"df"],
        description: "Disk listing",
    },
    CilTest {
        question: "how can I see disk usage?",
        accepted_patterns: &[r"df\s+-h", r"lsblk"],
        description: "Disk usage",
    },

    // GPU inspection
    CilTest {
        question: "how do I check my GPU?",
        accepted_patterns: &[r"lspci.*vga", r"nvidia-smi"],
        description: "GPU info",
    },

    // Network inspection
    CilTest {
        question: "how do I check my network?",
        accepted_patterns: &[r"ip\s+addr", r"ip\s+link", r"ifconfig"],
        description: "Network interfaces",
    },

    // Services
    CilTest {
        question: "how do I show running services?",
        accepted_patterns: &[r"systemctl\s+list-units.*service", r"systemctl.*failed"],
        description: "Service listing",
    },

    // Processes
    CilTest {
        question: "how do I list processes?",
        accepted_patterns: &[r"htop", r"ps\s+aux", r"top"],
        description: "Process listing",
    },
];

// ============================================================================
// Test Harness
// ============================================================================

#[tokio::test]
async fn test_acts_v3_command_intelligence() {
    let telemetry = create_test_telemetry();
    let llm_config = create_test_llm_config();

    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut failed_tests = Vec::new();

    println!("\n=== ACTS v3 - Command Intelligence Layer Tests ===\n");

    for test in ACTS_V3_TESTS {
        total += 1;

        // Query CIL
        let result = handle_unified_query(test.question, &telemetry, &llm_config).await;

        let test_passed = match result {
            Ok(UnifiedQueryResult::ConversationalAnswer { answer, .. }) => {
                // Normalize output
                let normalized = answer.to_lowercase();

                // Check if any pattern matches
                let matched = test.accepted_patterns.iter().any(|pattern| {
                    if let Ok(re) = Regex::new(pattern) {
                        re.is_match(&normalized)
                    } else {
                        false
                    }
                });

                if matched {
                    println!("✓ {}: {}", test.description, test.question);
                    true
                } else {
                    println!("✗ {}: {}", test.description, test.question);
                    println!("  Expected pattern: {:?}", test.accepted_patterns);
                    println!("  Got: {}\n", answer.lines().take(3).collect::<Vec<_>>().join("\n  "));
                    false
                }
            }
            Ok(other) => {
                println!("✗ {}: {} (got {:?} instead of command)", test.description, test.question, other);
                false
            }
            Err(e) => {
                println!("✗ {}: {} (error: {})", test.description, test.question, e);
                false
            }
        };

        if test_passed {
            passed += 1;
        } else {
            failed += 1;
            failed_tests.push(test.question);
        }
    }

    println!("\n=== ACTS v3 Summary ===");
    println!("Total:  {}", total);
    println!("Passed: {} ({}%)", passed, (passed * 100) / total);
    println!("Failed: {} ({}%)", failed, (failed * 100) / total);

    if !failed_tests.is_empty() {
        println!("\nFailed tests:");
        for q in failed_tests {
            println!("  - {}", q);
        }
    }

    // Require at least 70% pass rate for CIL
    assert!(
        passed >= (total * 7) / 10,
        "CIL must handle at least 70% of test cases. Got {}/{}", passed, total
    );
}

// ============================================================================
// Unit Tests - CIL Classification
// ============================================================================

#[test]
fn test_cil_classify_cpu_check() {
    let intent = classify_command_query("how do I check my CPU?");
    assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Cpu)));
}

#[test]
fn test_cil_classify_memory_check() {
    let intent = classify_command_query("how do I check memory usage?");
    assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Memory)));
}

#[test]
fn test_cil_classify_disk_check() {
    let intent = classify_command_query("how do I list my disks?");
    assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Disk)));
}

#[test]
fn test_cil_classify_gpu_check() {
    let intent = classify_command_query("how do I check my GPU?");
    assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Gpu)));
}

#[test]
fn test_cil_classify_network_check() {
    let intent = classify_command_query("how do I check my network?");
    assert!(matches!(intent, CommandIntent::Inspect(InspectTarget::Network)));
}

// ============================================================================
// Unit Tests - CIL Command Resolution
// ============================================================================

#[test]
fn test_cil_resolve_cpu_commands() {
    let kb = SystemKnowledgeBase::default();
    let commands = resolve_command_for_intent(
        CommandIntent::Inspect(InspectTarget::Cpu),
        &kb,
    );

    assert!(!commands.is_empty(), "Should return CPU inspection commands");
    assert!(
        commands.iter().any(|c| c.command.contains("lscpu")),
        "Should include lscpu command"
    );
}

#[test]
fn test_cil_resolve_memory_commands() {
    let kb = SystemKnowledgeBase::default();
    let commands = resolve_command_for_intent(
        CommandIntent::Inspect(InspectTarget::Memory),
        &kb,
    );

    assert!(!commands.is_empty(), "Should return memory inspection commands");
    assert!(
        commands.iter().any(|c| c.command.contains("free")),
        "Should include free command"
    );
}

#[test]
fn test_cil_resolve_disk_commands() {
    let kb = SystemKnowledgeBase::default();
    let commands = resolve_command_for_intent(
        CommandIntent::Inspect(InspectTarget::Disk),
        &kb,
    );

    assert!(!commands.is_empty(), "Should return disk inspection commands");
    assert!(
        commands.iter().any(|c| c.command.contains("lsblk") || c.command.contains("df")),
        "Should include lsblk or df command"
    );
}
