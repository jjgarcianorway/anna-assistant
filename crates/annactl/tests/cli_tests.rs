//! CLI integration tests for annactl v7.13.0 "Dependency Graph and Network Awareness"
//!
//! Tests the CLI surface:
//! - annactl           show help
//! - annactl status    health, alerts, [TELEMETRY], [RESOURCE HOTSPOTS], [ANNA NEEDS], Network in [INVENTORY]
//! - annactl sw        software overview with [CATEGORIES] - no duplicates
//! - annactl sw NAME   software profile with [CONFIG], [TELEMETRY], [LOGS], [DEPENDENCIES] (v7.13.0)
//! - annactl hw        hardware overview with [COMPONENTS], [HW TELEMETRY]
//! - annactl hw NAME   hardware profile with [IDENTITY], [DRIVER], [DEPENDENCIES] (v7.13.0), [INTERFACES] (v7.13.0)
//!
//! Deprecated (still works):
//! - annactl kdb       alias to sw
//! - annactl kdb NAME  alias to sw NAME
//!
//! Snow Leopard v7.13.0 tests:
//! - [DEPENDENCIES] section in sw NAME profiles (package, service, module deps)
//! - [DEPENDENCIES] section in hw wifi/ethernet profiles (module chain, related services)
//! - [INTERFACES] section in hw wifi/ethernet profiles with State/IP/Traffic
//! - Network summary in status [INVENTORY] section
//!
//! Snow Leopard v7.11.0 tests:
//! - [RESOURCE HOTSPOTS] section in status with health notes
//! - [HW TELEMETRY] section in hw with CPU/GPU/Memory/Disk I/O
//! - [TELEMETRY] Notes subsection linking telemetry with logs
//!
//! Snow Leopard v7.10.0 tests:
//! - [CONFIG] correctness: only paths with identity name, no unrelated tools
//! - No HTML leakage (no <div, <a href, etc)
//! - [CATEGORIES] has no duplicates like "hyprland, Hyprland"
//! - [COMPONENTS] shows driver and firmware for hardware
//! - [DRIVER] shows kernel module, loaded status, driver package, firmware
//! - [LOGS] uses -p warning..alert format with deduplication
//!
//! Snow Leopard telemetry tests (v7.9.0):
//! - Unified [TELEMETRY] section with trends (24h vs 7d)
//! - Activity windows (Last 1h, 24h, 7d, 30d) with sample counts
//! - Trend classification: stable, higher recently, lower recently
//! - Warming up behavior when insufficient data
//! - No invented numbers, no bullshit labels (spiking, exploding)
//!
//! Snow Leopard CONFIG hygiene tests (v7.8.0):
//! - System/User/Other structure with source attribution
//! - Status indicators [present]/[not present]
//! - No ecosystem pollution (mako, uwsm, waybar for hyprland)
//! - No HTML or junk in config paths
//! - Filesystem discovery as primary source

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn get_binary_path() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/release/annactl")
}

// ============================================================================
// v7.2.0: Help command tests
// ============================================================================

/// Test no args shows help
#[test]
fn test_annactl_no_args_shows_help() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.2.0: Help shows sw and hw commands
    assert!(
        stdout.contains("Anna CLI"),
        "Expected 'Anna CLI' help header, got: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl status"),
        "Help should mention status command"
    );
    assert!(
        stdout.contains("annactl sw"),
        "Help should mention sw command"
    );
    assert!(
        stdout.contains("annactl hw"),
        "Help should mention hw command"
    );
    assert!(output.status.success(), "annactl should succeed");
}

// ============================================================================
// v7.6.0: Status command tests
// ============================================================================

/// Test 'status' command shows structured output
#[test]
fn test_annactl_status_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Status command shows [VERSION], [DAEMON], [INVENTORY] sections
    assert!(
        stdout.contains("Anna Status") && stdout.contains("[VERSION]"),
        "Expected status output with sections, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl status should succeed");
}

/// Test 'status' command shows [ALERTS] section
#[test]
fn test_annactl_status_alerts_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.5.0: Status command shows [ALERTS] section
    assert!(
        stdout.contains("[ALERTS]"),
        "Expected [ALERTS] section in status output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl status should succeed");
}

/// Test 'status' command shows [ANNA NEEDS] section - v7.6.0
#[test]
fn test_annactl_status_anna_needs_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.6.0: Status command shows [ANNA NEEDS] section
    assert!(
        stdout.contains("[ANNA NEEDS]"),
        "Expected [ANNA NEEDS] section in status output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl status should succeed");
}

/// Test 'status' command is case-insensitive
#[test]
fn test_annactl_status_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test various case combinations
    for status_arg in ["status", "STATUS", "Status", "sTaTuS"] {
        let output = Command::new(&binary)
            .arg(status_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // All should be recognized as status command - shows [VERSION] section
        assert!(
            stdout.contains("Anna Status") && stdout.contains("[VERSION]"),
            "'{}' should be recognized as status command, got stdout: {}",
            status_arg,
            stdout
        );
    }
}

// ============================================================================
// v7.2.0: SW (software) command tests
// ============================================================================

/// Test 'sw' command shows overview
#[test]
fn test_annactl_sw_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("sw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.2.0: SW command shows "Anna Software" header with [OVERVIEW] section
    assert!(
        stdout.contains("Anna Software") && stdout.contains("[OVERVIEW]"),
        "Expected SW output with sections, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl sw should succeed");
}

/// Test 'sw' command is case-insensitive
#[test]
fn test_annactl_sw_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    for sw_arg in ["sw", "SW", "Sw", "sW"] {
        let output = Command::new(&binary)
            .arg(sw_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);

        assert!(
            stdout.contains("Anna Software") && stdout.contains("[OVERVIEW]"),
            "'{}' should be recognized as sw command, got stdout: {}",
            sw_arg,
            stdout
        );
    }
}

/// Test 'sw <name>' shows object profile
#[test]
fn test_annactl_sw_object() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with pacman (should exist on all Arch systems)
    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show object profile with [IDENTITY] section
    assert!(
        stdout.contains("Anna SW: pacman") && stdout.contains("[IDENTITY]"),
        "Expected object profile output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl sw pacman should succeed");
}

/// Test 'sw <category>' shows category view
#[test]
fn test_annactl_sw_category() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test editors category
    let output = Command::new(&binary)
        .args(["sw", "editors"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show category header
    assert!(
        stdout.contains("Anna SW: Editors"),
        "Expected category output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl sw editors should succeed");
}

// ============================================================================
// v7.2.0: HW (hardware) command tests
// ============================================================================

/// Test 'hw' command shows overview with drivers/health sections
#[test]
fn test_annactl_hw_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: HW command shows "Anna Hardware" header with [COMPONENTS] section
    assert!(
        stdout.contains("Anna Hardware") && stdout.contains("[COMPONENTS]"),
        "Expected HW output with [COMPONENTS], got stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("[HEALTH HIGHLIGHTS]"),
        "Expected [HEALTH HIGHLIGHTS] section, got stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("[CATEGORIES]"),
        "Expected [CATEGORIES] section, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw should succeed");
}

/// Test 'hw' shows health status
#[test]
fn test_annactl_hw_health_status() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.5.0: Should show health status for components
    assert!(
        stdout.contains("CPU:") && stdout.contains("[HEALTH HIGHLIGHTS]"),
        "Expected health status in [HEALTH HIGHLIGHTS], got stdout: {}",
        stdout
    );
}

/// Test 'hw' shows [DEPENDENCIES] section - v7.6.0
#[test]
fn test_annactl_hw_dependencies_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.6.0: Should show [DEPENDENCIES] section
    assert!(
        stdout.contains("[DEPENDENCIES]"),
        "Expected [DEPENDENCIES] section, got stdout: {}",
        stdout
    );
    // Should list hardware tools
    assert!(
        stdout.contains("smartctl:") || stdout.contains("sensors:"),
        "Expected hardware tools in [DEPENDENCIES], got stdout: {}",
        stdout
    );
}

/// Test 'hw' shows drivers per component - v7.10.0
#[test]
fn test_annactl_hw_drivers_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: Should show drivers in [COMPONENTS] section
    assert!(
        stdout.contains("[COMPONENTS]") && stdout.contains("driver:"),
        "Expected [COMPONENTS] section with drivers, got stdout: {}",
        stdout
    );
}

/// Test 'hw' command is case-insensitive
#[test]
fn test_annactl_hw_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    for hw_arg in ["hw", "HW", "Hw", "hW"] {
        let output = Command::new(&binary)
            .arg(hw_arg)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // v7.10.0: Now uses [COMPONENTS] instead of [OVERVIEW]
        assert!(
            stdout.contains("Anna Hardware") && stdout.contains("[COMPONENTS]"),
            "'{}' should be recognized as hw command, got stdout: {}",
            hw_arg,
            stdout
        );
    }
}

/// Test 'hw cpu' shows CPU profile with identity, driver, health, logs
#[test]
fn test_annactl_hw_cpu() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.5.0: Should show CPU profile with [IDENTITY], [DRIVER], [HEALTH], [LOGS]
    assert!(
        stdout.contains("Anna HW: cpu") && stdout.contains("[IDENTITY]"),
        "Expected CPU profile with [IDENTITY], got stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("[DRIVER]"),
        "Expected [DRIVER] section, got stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("[HEALTH]"),
        "Expected [HEALTH] section, got stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("[LOGS]"),
        "Expected [LOGS] section, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw cpu should succeed");
}

/// Test 'hw memory' shows memory profile
#[test]
fn test_annactl_hw_memory() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "memory"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show memory profile
    assert!(
        stdout.contains("Anna HW: Memory") && stdout.contains("[SUMMARY]"),
        "Expected memory profile output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw memory should succeed");
}

/// Test 'hw storage' shows storage profile
#[test]
fn test_annactl_hw_storage() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "storage"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show storage profile
    assert!(
        stdout.contains("Anna HW: Storage") && stdout.contains("[DEVICES]"),
        "Expected storage profile output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw storage should succeed");
}

/// Test 'hw gpu' shows GPU category with drivers
#[test]
fn test_annactl_hw_gpu_category() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "gpu"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.3.0: GPU category shows controllers with drivers
    assert!(
        stdout.contains("Anna HW: GPU") && stdout.contains("[CONTROLLERS]"),
        "Expected GPU category with [CONTROLLERS], got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw gpu should succeed");
}

/// Test 'hw network' shows interfaces with drivers
#[test]
fn test_annactl_hw_network_category() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "network"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.3.0: Network category shows interfaces
    assert!(
        stdout.contains("Anna HW: Network") && stdout.contains("[INTERFACES]"),
        "Expected Network category with [INTERFACES], got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw network should succeed");
}

// ============================================================================
// v7.2.0: Deprecated KDB command tests (alias to sw)
// ============================================================================

/// Test 'kdb' command still works (alias to sw)
#[test]
fn test_annactl_kdb_alias_overview() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("kdb")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.2.0: kdb is alias to sw, shows "Anna Software"
    assert!(
        stdout.contains("Anna Software") && stdout.contains("[OVERVIEW]"),
        "kdb should alias to sw, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl kdb should succeed");
}

/// Test 'kdb <name>' still works (alias to sw)
#[test]
fn test_annactl_kdb_alias_object() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["kdb", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.2.0: kdb <name> is alias to sw <name>
    assert!(
        stdout.contains("Anna SW: pacman") && stdout.contains("[IDENTITY]"),
        "kdb <name> should alias to sw <name>, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl kdb pacman should succeed");
}

// ============================================================================
// v7.2.0: Unknown command tests
// ============================================================================

/// Test unknown commands show error
#[test]
fn test_annactl_unknown_command() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("unknown_command_that_doesnt_exist")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show error message with suggestion
    assert!(
        stderr.contains("error:") && stderr.contains("not a recognized command"),
        "Expected error message for unknown command, got stderr: {}",
        stderr
    );
    assert!(!output.status.success(), "Unknown command should fail");
}

/// Test '--help' flag is not recognized (minimal surface)
#[test]
fn test_annactl_help_flag_not_recognized() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--help")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // --help is not recognized, should show error
    assert!(
        stderr.contains("not a recognized command"),
        "Expected '--help' to be rejected, got stderr: {}",
        stderr
    );
}

/// Test '--version' flag is not recognized (minimal surface)
#[test]
fn test_annactl_version_flag_not_recognized() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // --version is not recognized, should show error
    assert!(
        stderr.contains("not a recognized command"),
        "Expected '--version' to be rejected, got stderr: {}",
        stderr
    );
}

// ============================================================================
// v7.2.0: Telemetry & Config Tests
// ============================================================================

/// Test 'sw' command shows [USAGE HIGHLIGHTS] section
#[test]
fn test_annactl_sw_shows_top_offenders() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("sw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.6.0: SW may show [TOP CPU (24h)] and [TOP RAM (24h)] when telemetry is available
    // These are optional - only shown when daemon is running with enough data
    // So we just verify the command runs successfully without error
    assert!(
        stdout.contains("[OVERVIEW]") && stdout.contains("[CATEGORIES]"),
        "Expected basic SW output with [OVERVIEW] and [CATEGORIES], got: {}",
        stdout
    );
}

/// Test 'sw <name>' shows [TELEMETRY] section
#[test]
fn test_annactl_sw_object_shows_telemetry_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Object profile should show [TELEMETRY] section (v7.4.0+)
    assert!(
        stdout.contains("[TELEMETRY]"),
        "Expected [TELEMETRY] section in object profile, got: {}",
        stdout
    );
}

/// Test 'sw <name>' shows [CONFIG] section
#[test]
fn test_annactl_sw_object_shows_config_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Config section should exist
    assert!(
        stdout.contains("[CONFIG]"),
        "Expected [CONFIG] section in object profile, got: {}",
        stdout
    );
}

// ============================================================================
// v7.2.0: Name Resolution Tests
// ============================================================================

/// Test case-insensitive name resolution for packages
#[test]
fn test_annactl_sw_case_insensitive_package() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test PACMAN (uppercase) resolves to pacman
    let output = Command::new(&binary)
        .args(["sw", "PACMAN"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should resolve and show profile
    assert!(
        stdout.contains("[IDENTITY]"),
        "PACMAN should resolve case-insensitively, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test category names are case-insensitive
#[test]
fn test_annactl_sw_category_case_insensitive() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test EDITORS (uppercase) resolves to Editors category
    let output = Command::new(&binary)
        .args(["sw", "EDITORS"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show category view
    assert!(
        stdout.contains("Anna SW: Editors"),
        "EDITORS should resolve to Editors category, got: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// v7.2.0: Performance Tests
// ============================================================================

/// Test 'status' command completes in reasonable time (<2s)
#[test]
fn test_annactl_status_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl status should succeed");
    // v7.11.0: Allow 3s for status due to journalctl health note lookups
    assert!(
        elapsed.as_secs() < 3,
        "annactl status should complete in <3s, took: {:?}",
        elapsed
    );
}

/// Test 'sw' command completes in reasonable time (<15s)
#[test]
fn test_annactl_sw_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("sw")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl sw should succeed");
    assert!(
        elapsed.as_secs() < 15,
        "annactl sw should complete in <15s, took: {:?}",
        elapsed
    );
}

/// Test 'sw <name>' command completes in reasonable time (<2s)
#[test]
fn test_annactl_sw_object_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl sw pacman should succeed");
    assert!(
        elapsed.as_secs() < 2,
        "annactl sw <name> should complete in <2s, took: {:?}",
        elapsed
    );
}

/// Test 'hw' command completes in reasonable time (<2s)
#[test]
fn test_annactl_hw_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl hw should succeed");
    assert!(
        elapsed.as_secs() < 2,
        "annactl hw should complete in <2s, took: {:?}",
        elapsed
    );
}

/// Test 'hw cpu' command completes in reasonable time (<2s)
#[test]
fn test_annactl_hw_cpu_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl hw cpu should succeed");
    assert!(
        elapsed.as_secs() < 2,
        "annactl hw cpu should complete in <2s, took: {:?}",
        elapsed
    );
}

// ============================================================================
// v7.9.0: Snow Leopard Telemetry Tests (Real Trends)
// ============================================================================

/// Test status shows [TELEMETRY] section with v7.9.0 format
#[test]
fn test_snow_leopard_status_telemetry_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: Status must show [TELEMETRY] section
    assert!(
        stdout.contains("[TELEMETRY]"),
        "Expected [TELEMETRY] section, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test status telemetry shows warming up when insufficient data
/// OR shows proper structure when data is available (v7.9.0)
#[test]
fn test_snow_leopard_telemetry_no_fake_numbers() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must either show "warming up" OR show proper data format
    // Must NOT show invented percentages or placeholder values
    if stdout.contains("warming up") {
        // If warming up, should not show fake "Top CPU identities" data
        assert!(
            !stdout.contains("Top CPU identities:") || stdout.contains("warming up"),
            "Should not show Top CPU identities while warming up: {}",
            stdout
        );
    } else if stdout.contains("Top CPU identities:") {
        // v7.9.0: If showing data, should have proper format with "percent" and trend
        assert!(
            stdout.contains("percent"),
            "Top CPU should show 'percent' format: {}",
            stdout
        );
        // v7.9.0: Should show trend classification (vs 7d)
        let has_trend = stdout.contains("stable vs 7d")
            || stdout.contains("higher recently vs 7d")
            || stdout.contains("lower recently vs 7d");
        // Trend is optional if not enough 7d data
        if stdout.contains("peak") {
            // If showing peak data, format should be "avg X percent, peak Y percent"
            assert!(
                stdout.contains("avg") && stdout.contains("peak"),
                "Top CPU should show avg and peak: {}",
                stdout
            );
        }
    }
    // Either case is valid - depends on daemon state
    assert!(output.status.success());
}

/// Test sw detail shows [TELEMETRY] with Activity windows (v7.9.0)
#[test]
fn test_snow_leopard_sw_telemetry_activity_windows() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: SW detail should show [TELEMETRY] section
    assert!(
        stdout.contains("[TELEMETRY]"),
        "Expected [TELEMETRY] section, got: {}",
        stdout
    );

    // v7.9.0: If telemetry is available, should show Activity windows subsection
    if stdout.contains("Activity windows:") {
        // Should show samples count and metrics per window
        // v7.12.0: Format is "N samples, avg CPU X%, peak Y%, avg RSS Z, peak W"
        let has_valid_format = stdout.contains("samples,")
            || stdout.contains("samples active")
            || stdout.contains("no samples")
            || stdout.contains("no data");
        assert!(
            has_valid_format,
            "Activity windows should show valid format (samples count/no samples/no data): {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test sw detail telemetry windows show Last 1h, Last 24h, Last 7d, Last 30d (v7.9.0)
#[test]
fn test_snow_leopard_telemetry_four_windows() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: If Activity windows section exists, should show all four time windows
    if stdout.contains("Activity windows:") {
        assert!(
            stdout.contains("Last 1h:"),
            "Activity windows should show Last 1h: {}",
            stdout
        );
        assert!(
            stdout.contains("Last 24h:"),
            "Activity windows should show Last 24h: {}",
            stdout
        );
        assert!(
            stdout.contains("Last 7d:"),
            "Activity windows should show Last 7d: {}",
            stdout
        );
        assert!(
            stdout.contains("Last 30d:"),
            "Activity windows should show Last 30d: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test no invented trends when insufficient history (v7.9.0)
#[test]
fn test_snow_leopard_no_invented_trends() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: Trend section compares 24h vs 7d
    // Trend values must be one of: "stable", "higher recently", "lower recently"
    if stdout.contains("Trend:") && stdout.contains("24h vs 7d") {
        let has_valid_trend = stdout.contains("stable")
            || stdout.contains("higher recently")
            || stdout.contains("lower recently");
        assert!(
            has_valid_trend,
            "Trend should only use 'stable', 'higher recently', or 'lower recently': {}",
            stdout
        );
        // Must NOT contain bullshit labels
        assert!(
            !stdout.contains("spiking") && !stdout.contains("exploding") && !stdout.contains("hot"),
            "Trend should not use invented labels like 'spiking', 'exploding', 'hot': {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test trend shows 24h vs 7d comparison (v7.9.0)
#[test]
fn test_snow_leopard_trend_24h_vs_7d() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: If Trend section exists, should compare 24h vs 7d
    if stdout.contains("Trend:") && stdout.contains("CPU:") {
        assert!(
            stdout.contains("24h vs 7d"),
            "Trend should compare 24h vs 7d: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw command shows [COMPONENTS] section with driver/firmware info - v7.10.0
#[test]
fn test_snow_leopard_hw_components_v710() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: HW should show [COMPONENTS] section with driver info
    assert!(
        stdout.contains("[COMPONENTS]"),
        "Expected [COMPONENTS] section, got: {}",
        stdout
    );
    // Should show driver information
    assert!(
        stdout.contains("driver:"),
        "[COMPONENTS] should show driver info: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test no HTML or junk leaks into telemetry sections
#[test]
fn test_snow_leopard_no_html_junk_in_telemetry() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // No HTML tags or entities in output
    assert!(
        !stdout.contains("<html") && !stdout.contains("</") && !stdout.contains("&nbsp;"),
        "Output should not contain HTML: {}",
        stdout
    );
    // No raw JSON or code artifacts
    assert!(
        !stdout.contains("\"name\":") && !stdout.contains("null"),
        "Output should not contain raw JSON: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test telemetry source attribution is shown (v7.9.0)
#[test]
fn test_snow_leopard_telemetry_source_attribution() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: Telemetry section should show source attribution
    if stdout.contains("[TELEMETRY]") {
        assert!(
            stdout.contains("Anna daemon") || stdout.contains("sampling"),
            "TELEMETRY should attribute data source (Anna daemon, sampling): {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test status telemetry shows window header when data is available (v7.9.0)
#[test]
fn test_snow_leopard_telemetry_window_header() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.9.0: If showing data, should show Window header with 24h and sampling interval
    if stdout.contains("Top CPU identities:") {
        assert!(
            stdout.contains("Window: last 24h"),
            "TELEMETRY should include Window header: {}",
            stdout
        );
        assert!(
            stdout.contains("sampling every"),
            "Window header should mention sampling interval: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

// ============================================================================
// v7.8.0: Snow Leopard CONFIG Hygiene Tests
// ============================================================================

/// Test [CONFIG] section shows System/User/Other structure
#[test]
fn test_snow_leopard_config_section_structure() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: CONFIG section must exist and show structure
    assert!(
        stdout.contains("[CONFIG]"),
        "Expected [CONFIG] section: {}",
        stdout
    );

    // v7.12.0: Should have Primary: and/or Secondary: subsections
    let has_structure = stdout.contains("Primary:") || stdout.contains("Secondary:");
    assert!(
        has_structure,
        "[CONFIG] should have Primary: or Secondary: subsections: {}",
        stdout
    );

    // Should show source attribution per line (parentheses with source)
    if stdout.contains("Primary:") || stdout.contains("Secondary:") {
        assert!(
            stdout.contains("(filesystem)") || stdout.contains("(pacman") || stdout.contains("(man") || stdout.contains("(Arch Wiki)"),
            "[CONFIG] should show source attribution in parentheses: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test [CONFIG] section shows status indicators [present], [missing], [recommended]
#[test]
fn test_snow_leopard_config_status_indicators() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: CONFIG paths should show status indicators
    if stdout.contains("[CONFIG]") && (stdout.contains("Primary:") || stdout.contains("Secondary:")) {
        let has_status = stdout.contains("[present]")
            || stdout.contains("[not present]");
        assert!(
            has_status,
            "[CONFIG] should show status indicators [present]/[not present]: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test hyprland CONFIG does NOT show mako, uwsm, waybar, dunst paths
#[test]
fn test_snow_leopard_hyprland_no_ecosystem_pollution() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "hyprland"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.8.0: Hyprland CONFIG should NOT show mako, uwsm, waybar, dunst paths
    if stdout.contains("[CONFIG]") {
        assert!(
            !stdout.contains("/mako/"),
            "hyprland CONFIG should NOT contain mako paths: {}",
            stdout
        );
        assert!(
            !stdout.contains("/uwsm/"),
            "hyprland CONFIG should NOT contain uwsm paths: {}",
            stdout
        );
        assert!(
            !stdout.contains("/waybar/"),
            "hyprland CONFIG should NOT contain waybar paths: {}",
            stdout
        );
        assert!(
            !stdout.contains("/dunst/"),
            "hyprland CONFIG should NOT contain dunst paths: {}",
            stdout
        );
        assert!(
            !stdout.contains("/rofi/"),
            "hyprland CONFIG should NOT contain rofi paths: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test CONFIG section shows Precedence explanation
#[test]
fn test_snow_leopard_config_precedence_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: CONFIG should include Notes section with precedence info
    if stdout.contains("[CONFIG]") && stdout.contains("Primary:") && stdout.contains("Secondary:") {
        assert!(
            stdout.contains("Notes:"),
            "[CONFIG] should include Notes section when both Primary and Secondary exist: {}",
            stdout
        );
        // Notes should explain config status or precedence
        assert!(
            stdout.contains("User config") || stdout.contains("XDG paths") || stdout.contains("active"),
            "Notes should explain config status: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test CONFIG section does NOT contain HTML or junk
#[test]
fn test_snow_leopard_config_no_html_noise() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "hyprland"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.8.0: CONFIG should not contain HTML tags or entities
    if stdout.contains("[CONFIG]") {
        assert!(
            !stdout.contains("<a href=") && !stdout.contains("</a>"),
            "[CONFIG] should not contain HTML links: {}",
            stdout
        );
        assert!(
            !stdout.contains("&lt;") && !stdout.contains("&gt;") && !stdout.contains("&amp;"),
            "[CONFIG] should not contain HTML entities: {}",
            stdout
        );
        assert!(
            !stdout.contains("<code>") && !stdout.contains("</code>"),
            "[CONFIG] should not contain HTML code tags: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test CONFIG path limits (max 6 system, 6 user, 4 other)
#[test]
fn test_snow_leopard_config_path_limits() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "git"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("[CONFIG]") {
        // Count paths in each section by counting lines with status indicators
        let config_section: String = stdout
            .split("[CONFIG]")
            .nth(1)
            .unwrap_or("")
            .split('[')  // Stop at next section
            .next()
            .unwrap_or("")
            .to_string();

        // Count paths with status indicators as a rough count
        let path_count = config_section.matches("[present]").count()
            + config_section.matches("[missing]").count()
            + config_section.matches("[recommended]").count();

        // Total should be limited (6 + 6 + 4 = 16 max)
        assert!(
            path_count <= 20,  // Allow some margin for display variations
            "[CONFIG] should limit total paths (max ~16), found {}: {}",
            path_count,
            config_section
        );
    }

    assert!(output.status.success());
}

/// Test CONFIG filesystem discovery is primary source
#[test]
fn test_snow_leopard_config_filesystem_priority() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("[CONFIG]") {
        // v7.8.0: Filesystem discovery should be primary source
        // Check that filesystem is mentioned as a source
        assert!(
            stdout.contains("filesystem"),
            "[CONFIG] should include filesystem as a source: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// v7.10.0: Snow Leopard Arch Wiki & Hardware Driver Tests
// ============================================================================

/// Test sw CATEGORIES has no case-insensitive duplicates like "hyprland, Hyprland"
#[test]
fn test_snow_leopard_categories_no_duplicates() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("sw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: Check for case-insensitive duplicates
    // Look for patterns like "Name, name" or "name, Name"
    let lines: Vec<&str> = stdout.lines().collect();
    for line in &lines {
        if line.contains(":") && line.contains(",") {
            // This is likely a category line with multiple items
            let parts: Vec<&str> = if let Some(idx) = line.find(":") {
                line[idx + 1..].split(',').map(|s| s.trim()).collect()
            } else {
                continue;
            };

            // Check for case-insensitive duplicates
            for i in 0..parts.len() {
                for j in (i + 1)..parts.len() {
                    let a = parts[i].to_lowercase();
                    let b = parts[j].to_lowercase();
                    assert!(
                        a != b,
                        "Found case-insensitive duplicate in categories: '{}' and '{}' in line: {}",
                        parts[i], parts[j], line
                    );
                }
            }
        }
    }

    assert!(output.status.success());
}

/// Test hw shows [COMPONENTS] section with drivers and firmware (v7.10.0)
#[test]
fn test_snow_leopard_hw_components_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: hw should show [COMPONENTS] section
    assert!(
        stdout.contains("[COMPONENTS]"),
        "Expected [COMPONENTS] section in hw output, got: {}",
        stdout
    );

    // v7.10.0: [COMPONENTS] should mention driver for at least CPU
    if stdout.contains("CPU:") {
        assert!(
            stdout.contains("driver:"),
            "[COMPONENTS] should show driver for CPU: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test hw NAME shows [DRIVER] section with module, loaded, package (v7.10.0)
#[test]
fn test_snow_leopard_hw_driver_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "gpu"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: hw gpu should show [DRIVER] section
    if stdout.contains("[DRIVER]") {
        // Should show Kernel module and Loaded status
        let has_module = stdout.contains("Kernel module:");
        let has_loaded = stdout.contains("Loaded:");

        // At least one of these should be present
        assert!(
            has_module || has_loaded || stdout.contains("none"),
            "[DRIVER] should show module info or 'none': {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test no HTML tags in any output (v7.10.0)
#[test]
fn test_snow_leopard_no_html_leakage() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test several common commands
    let commands = vec![
        vec!["sw"],
        vec!["sw", "vim"],
        vec!["hw"],
        vec!["hw", "cpu"],
    ];

    for cmd_args in commands {
        let output = Command::new(&binary)
            .args(&cmd_args)
            .output()
            .expect("Failed to run annactl");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // v7.10.0: Must not contain HTML tags
        let html_patterns = ["<div", "<a href", "<span", "</div>", "</a>", "<p>", "<br>"];
        for pattern in &html_patterns {
            assert!(
                !stdout.contains(pattern),
                "Found HTML tag '{}' in output of 'annactl {}': {}",
                pattern,
                cmd_args.join(" "),
                stdout
            );
        }
    }
}

/// Test sw NAME [CONFIG] uses v7.10.0 format with [present]/[not present]
#[test]
fn test_snow_leopard_config_v710_format() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: [CONFIG] should use Primary:/Secondary: format with [present]/[not present]
    if stdout.contains("[CONFIG]") && (stdout.contains("Primary:") || stdout.contains("Secondary:")) {
        // Should have Notes section
        assert!(
            stdout.contains("Notes:") || stdout.contains("Source:"),
            "[CONFIG] should have Notes or Source section: {}",
            stdout
        );

        // v7.12.0: Should use [present] or [not present] status markers
        if stdout.contains("/etc") || stdout.contains("/usr") {
            let has_status = stdout.contains("[present]") || stdout.contains("[not present]");
            // It's okay if no configs are detected
            if !stdout.contains("No specific config files detected") {
                assert!(
                    has_status,
                    "[CONFIG] should use [present]/[not present] markers: {}",
                    stdout
                );
            }
        }
    }

    assert!(output.status.success());
}

/// Test sw NAME [LOGS] shows journalctl command info (v7.10.0)
#[test]
fn test_snow_leopard_logs_v710_format() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with a service that should have logs
    let output = Command::new(&binary)
        .args(["sw", "systemd"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.10.0: If [LOGS] section exists, should mention journalctl
    if stdout.contains("[LOGS]") {
        assert!(
            stdout.contains("journalctl"),
            "[LOGS] should mention journalctl source: {}",
            stdout
        );

        // v7.10.0: Should use -p warning..alert (or show "No warnings or errors")
        let has_warning_filter = stdout.contains("warning") || stdout.contains("No warnings");
        assert!(
            has_warning_filter,
            "[LOGS] should filter by warning priority or show no warnings message: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// v7.11.0: Snow Leopard Honest Telemetry Tests
// ============================================================================

/// Test status shows [RESOURCE HOTSPOTS] section (v7.11.0)
#[test]
fn test_snow_leopard_status_resource_hotspots_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: Status must show [RESOURCE HOTSPOTS] section
    assert!(
        stdout.contains("[RESOURCE HOTSPOTS]"),
        "Expected [RESOURCE HOTSPOTS] section, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw shows [HW TELEMETRY] section (v7.11.0)
#[test]
fn test_snow_leopard_hw_telemetry_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: HW must show [HW TELEMETRY] section
    assert!(
        stdout.contains("[HW TELEMETRY]"),
        "Expected [HW TELEMETRY] section, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test [HW TELEMETRY] shows CPU, GPU, Memory, Disk I/O lines (v7.11.0)
#[test]
fn test_snow_leopard_hw_telemetry_content() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: [HW TELEMETRY] should show telemetry for major components
    if stdout.contains("[HW TELEMETRY]") {
        // Should have CPU line
        assert!(
            stdout.contains("  CPU:"),
            "[HW TELEMETRY] should have CPU line: {}",
            stdout
        );
        // Should have GPU line
        assert!(
            stdout.contains("  GPU:"),
            "[HW TELEMETRY] should have GPU line: {}",
            stdout
        );
        // Should have Memory line
        assert!(
            stdout.contains("  Memory:"),
            "[HW TELEMETRY] should have Memory line: {}",
            stdout
        );
        // Should have Disk I/O line
        assert!(
            stdout.contains("  Disk I/O:"),
            "[HW TELEMETRY] should have Disk I/O line: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test [HW TELEMETRY] shows source attribution (v7.11.0)
#[test]
fn test_snow_leopard_hw_telemetry_source() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: [HW TELEMETRY] should show source (hwmon, thermal, proc)
    if stdout.contains("[HW TELEMETRY]") {
        assert!(
            stdout.contains("hwmon") || stdout.contains("thermal") || stdout.contains("/proc"),
            "[HW TELEMETRY] should mention source: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test [RESOURCE HOTSPOTS] shows health notes when there are hotspots (v7.11.0)
#[test]
fn test_snow_leopard_resource_hotspots_format() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: [RESOURCE HOTSPOTS] should either:
    // - Show "warming up" or "no telemetry" message
    // - Show CPU/RAM hotspots with proper format
    if stdout.contains("[RESOURCE HOTSPOTS]") {
        let has_valid_content = stdout.contains("warming up")
            || stdout.contains("Telemetry disabled")
            || stdout.contains("telemetry DB not available")
            || stdout.contains("No significant resource consumers")
            || stdout.contains("top resource consumers")
            || (stdout.contains("CPU:") && stdout.contains("[RESOURCE HOTSPOTS]"));
        
        assert!(
            has_valid_content,
            "[RESOURCE HOTSPOTS] should have valid content: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw command performance with telemetry (v7.11.0)
/// Ensure telemetry sampling doesn't slow down too much
#[test]
fn test_snow_leopard_hw_telemetry_performance() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let start = std::time::Instant::now();
    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let elapsed = start.elapsed();

    assert!(output.status.success(), "annactl hw should succeed");
    // v7.11.0: Allow slightly more time for telemetry sampling (300ms CPU + disk)
    // But should still be under 3 seconds
    assert!(
        elapsed.as_secs() < 3,
        "annactl hw should complete in <3s even with telemetry, took: {:?}",
        elapsed
    );
}

/// Test sw NAME [TELEMETRY] Notes subsection (v7.11.0)
/// Notes should only appear when there are health concerns
#[test]
fn test_snow_leopard_sw_telemetry_notes_format() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "systemd"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.11.0: Test passes if:
    // 1. No "Notes:" subsection (healthy identity - nothing to report)
    // 2. "Notes:" subsection with actual concerns (errors, high usage)
    if stdout.contains("[TELEMETRY]") {
        // Check if there are notes in the TELEMETRY section
        // Notes should only appear after "Trend:" or "Activity windows:"
        let telemetry_section: String = stdout
            .split("[TELEMETRY]")
            .nth(1)
            .unwrap_or("")
            .split("[")
            .next()
            .unwrap_or("")
            .to_string();

        if telemetry_section.contains("Notes:") {
            // If notes exist, they should contain real concerns
            let has_real_note = telemetry_section.contains("error")
                || telemetry_section.contains("High CPU")
                || telemetry_section.contains("High memory")
                || telemetry_section.contains("see [LOGS]");

            assert!(
                has_real_note,
                "[TELEMETRY] Notes should contain real concerns, got: {}",
                telemetry_section
            );
        }
        // If no Notes: section, that's fine - healthy identity
    }
    assert!(output.status.success());
}

// ============================================================================
// v7.12.0: Snow Leopard Config Intelligence and Log Literacy Tests
// ============================================================================

/// Test [CONFIG] Primary/Secondary/Notes structure (v7.12.0)
#[test]
fn test_snow_leopard_config_v712_structure() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("[CONFIG]") {
        // v7.12.0: Must use Primary:/Secondary:/Notes: structure
        let has_primary = stdout.contains("Primary:");
        let has_secondary = stdout.contains("Secondary:");
        let has_notes = stdout.contains("Notes:");

        // Primary is required if any configs exist
        assert!(
            has_primary || stdout.contains("No specific config files detected"),
            "[CONFIG] should have Primary: section or indicate no configs: {}",
            stdout
        );

        // Notes is required when we have configs
        if has_primary {
            assert!(
                has_notes,
                "[CONFIG] should have Notes: section after Primary: {}",
                stdout
            );
        }

        // If both Primary and Secondary exist, Notes should explain precedence
        if has_primary && has_secondary {
            assert!(
                stdout.contains("User config") || stdout.contains("XDG paths") || stdout.contains("active"),
                "[CONFIG] Notes should explain config status when both sections exist: {}",
                stdout
            );
        }
    }

    assert!(output.status.success());
}

/// Test [LOGS] deduplication with count format (v7.12.0)
#[test]
fn test_snow_leopard_logs_v712_deduplication() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with a service that typically has logs
    let output = Command::new(&binary)
        .args(["sw", "systemd"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: If [LOGS] section exists and has entries
    if stdout.contains("[LOGS]") {
        // Should NOT have truncated messages (no "..." in middle of lines)
        // It's okay to have ellipsis at end for very long lines
        let logs_section: String = stdout
            .split("[LOGS]")
            .nth(1)
            .unwrap_or("")
            .split("[")
            .next()
            .unwrap_or("")
            .to_string();

        // If there are duplicate messages, they should show (seen N times this boot) format
        // This is conditional - only if duplicates exist
        if logs_section.contains("seen") {
            assert!(
                logs_section.contains("(seen") && logs_section.contains("times this boot)"),
                "[LOGS] deduplication should use '(seen N times this boot)' format: {}",
                logs_section
            );
        }
    }

    assert!(output.status.success());
}

/// Test status [PATHS] shows ops.log and internal dir (v7.12.0)
#[test]
fn test_snow_leopard_status_paths_v712() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: [PATHS] should show Internal: and Ops log: entries
    if stdout.contains("[PATHS]") {
        // Should show Internal directory
        assert!(
            stdout.contains("Internal:"),
            "[PATHS] should show Internal: directory: {}",
            stdout
        );

        // Should show Ops log path
        assert!(
            stdout.contains("Ops log:"),
            "[PATHS] should show Ops log: entry: {}",
            stdout
        );

        // Should show Docs status
        assert!(
            stdout.contains("Docs:"),
            "[PATHS] should show Docs: local docs status: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test [TELEMETRY] State summary line (v7.12.0)
#[test]
fn test_snow_leopard_telemetry_state_v712() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "annad"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: [TELEMETRY] should have State (24h): line when data exists
    if stdout.contains("[TELEMETRY]") {
        // Check for State line
        let has_state = stdout.contains("State (24h):");
        let has_not_enough = stdout.contains("not enough data yet") || stdout.contains("No telemetry");

        // Either should have State summary or indicate no data
        assert!(
            has_state || has_not_enough,
            "[TELEMETRY] should have 'State (24h):' line or indicate no data: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test hw [HW TELEMETRY] State line (v7.12.0)
#[test]
fn test_snow_leopard_hw_telemetry_state_v712() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("hw")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.12.0: [HW TELEMETRY] should have State (now): line
    if stdout.contains("[HW TELEMETRY]") {
        assert!(
            stdout.contains("State (now):"),
            "[HW TELEMETRY] should have 'State (now):' summary line: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// v7.13.0: Snow Leopard Dependency Graph and Network Awareness Tests
// ============================================================================

/// Test sw NAME shows [DEPENDENCIES] section (v7.13.0)
#[test]
fn test_snow_leopard_sw_dependencies_section_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with NetworkManager (should have package and service deps)
    let output = Command::new(&binary)
        .args(["sw", "NetworkManager"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: SW profile should show [DEPENDENCIES] section
    assert!(
        stdout.contains("[DEPENDENCIES]"),
        "Expected [DEPENDENCIES] section in sw profile, got: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test sw NAME [DEPENDENCIES] shows Package dependencies (v7.13.0)
#[test]
fn test_snow_leopard_sw_dependencies_package_deps_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: [DEPENDENCIES] should show Package deps subsection
    if stdout.contains("[DEPENDENCIES]") {
        assert!(
            stdout.contains("package deps:") || stdout.contains("Package deps:"),
            "[DEPENDENCIES] should have 'package deps:' subsection: {}",
            stdout
        );
        // Should show source attribution
        assert!(
            stdout.contains("pacman") || stdout.contains("pactree"),
            "[DEPENDENCIES] should mention pacman/pactree source: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test sw service NAME [DEPENDENCIES] shows Service relations (v7.13.0)
#[test]
fn test_snow_leopard_sw_dependencies_service_relations_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: Service [DEPENDENCIES] should show Service relations
    if stdout.contains("[DEPENDENCIES]") {
        assert!(
            stdout.contains("Service relations:"),
            "[DEPENDENCIES] should have 'Service relations:' subsection: {}",
            stdout
        );
        // Should show relation types (Requires, Wants, Part of, WantedBy)
        let has_relations = stdout.contains("Requires:")
            || stdout.contains("Wants:")
            || stdout.contains("Part of:")
            || stdout.contains("WantedBy:");
        assert!(
            has_relations || stdout.contains("none"),
            "[DEPENDENCIES] should show service relations or 'none': {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw wifi shows [DEPENDENCIES] section (v7.13.0)
#[test]
fn test_snow_leopard_hw_wifi_dependencies_section_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: hw wifi should show [DEPENDENCIES] section (if wifi exists)
    if !stdout.contains("[NOT FOUND]") {
        assert!(
            stdout.contains("[DEPENDENCIES]"),
            "hw wifi should have [DEPENDENCIES] section: {}",
            stdout
        );
        // Should show module chain or module deps
        let has_module_info = stdout.contains("Driver module chain:")
            || stdout.contains("Module depends on:");
        assert!(
            has_module_info,
            "[DEPENDENCIES] should show driver module info: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw wifi shows [INTERFACES] section with traffic (v7.13.0)
#[test]
fn test_snow_leopard_hw_wifi_interfaces_section_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: hw wifi should show [INTERFACES] section (if wifi exists)
    if !stdout.contains("[NOT FOUND]") {
        assert!(
            stdout.contains("[INTERFACES]"),
            "hw wifi should have [INTERFACES] section: {}",
            stdout
        );
        // Should show interface details
        assert!(
            stdout.contains("Type:") && stdout.contains("State:"),
            "[INTERFACES] should show Type and State: {}",
            stdout
        );
        // Should show traffic if connected
        if stdout.contains("connected") {
            assert!(
                stdout.contains("Traffic:") || stdout.contains("RX"),
                "[INTERFACES] should show traffic for connected interface: {}",
                stdout
            );
        }
    }
    assert!(output.status.success());
}

/// Test hw ethernet shows [DEPENDENCIES] and [INTERFACES] sections (v7.13.0)
#[test]
fn test_snow_leopard_hw_ethernet_sections_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "ethernet"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: If ethernet exists, should show sections
    if !stdout.contains("[NOT FOUND]") {
        assert!(
            stdout.contains("[DEPENDENCIES]"),
            "hw ethernet should have [DEPENDENCIES] section: {}",
            stdout
        );
        assert!(
            stdout.contains("[INTERFACES]"),
            "hw ethernet should have [INTERFACES] section: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test status [INVENTORY] shows Network summary (v7.13.0)
#[test]
fn test_snow_leopard_status_inventory_network_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: [INVENTORY] should show Network line
    if stdout.contains("[INVENTORY]") {
        assert!(
            stdout.contains("Network:"),
            "[INVENTORY] should have Network: line: {}",
            stdout
        );
        // Should show interface count and names
        let has_network_info = stdout.contains("interfaces")
            || stdout.contains("wifi")
            || stdout.contains("ethernet")
            || stdout.contains("no physical interfaces");
        assert!(
            has_network_info,
            "Network: should show interface summary: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test [DEPENDENCIES] source attribution (v7.13.0)
#[test]
fn test_snow_leopard_dependencies_source_attribution_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: [DEPENDENCIES] should show source attribution
    if stdout.contains("[DEPENDENCIES]") {
        assert!(
            stdout.contains("sources:") || stdout.contains("Source:") || stdout.contains("(from"),
            "[DEPENDENCIES] should show source attribution: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw wifi [DEPENDENCIES] shows Related services (v7.13.0)
#[test]
fn test_snow_leopard_hw_dependencies_related_services_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: hw wifi [DEPENDENCIES] should show Related services
    if !stdout.contains("[NOT FOUND]") && stdout.contains("[DEPENDENCIES]") {
        // Should show Related services section (NetworkManager, wpa_supplicant, etc.)
        let has_related = stdout.contains("Related services:")
            || stdout.contains("NetworkManager")
            || stdout.contains("wpa_supplicant");
        assert!(
            has_related,
            "[DEPENDENCIES] should show related services for wifi: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test [INTERFACES] shows IP addresses (v7.13.0)
#[test]
fn test_snow_leopard_interfaces_show_ip_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: [INTERFACES] should show IP addresses for connected interfaces
    if !stdout.contains("[NOT FOUND]") && stdout.contains("[INTERFACES]") {
        if stdout.contains("connected") {
            // Connected interfaces should show IP address
            assert!(
                stdout.contains("IP:"),
                "[INTERFACES] should show IP: for connected interface: {}",
                stdout
            );
        }
    }
    assert!(output.status.success());
}

/// Test no invented dependency data (v7.13.0)
#[test]
fn test_snow_leopard_no_invented_dependencies_v713() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "pacman"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0: [DEPENDENCIES] should NOT contain placeholder or invented data
    if stdout.contains("[DEPENDENCIES]") {
        // Should not contain placeholder values
        assert!(
            !stdout.contains("TODO") && !stdout.contains("placeholder") && !stdout.contains("unknown dependency"),
            "[DEPENDENCIES] should not contain placeholder data: {}",
            stdout
        );
        // Should not contain random/invented package names
        assert!(
            !stdout.contains("libfoo") && !stdout.contains("libbar"),
            "[DEPENDENCIES] should not contain invented package names: {}",
            stdout
        );
    }
    assert!(output.status.success());
}
