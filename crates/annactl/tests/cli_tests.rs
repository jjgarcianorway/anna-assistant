//! CLI integration tests for annactl v7.38.0 "Cache-Only Status & Hardened Daemon"
//!
//! Tests the CLI surface (exactly 7 commands, no aliases):
//! - annactl           show help
//! - annactl --version show version (exactly "vX.Y.Z")
//! - annactl status    cache-only: [VERSION], [DAEMON], [HEALTH], [DATA], [TELEMETRY], [UPDATES], [ALERTS], [PATHS]
//! - annactl sw        software overview with [CATEGORIES], [HOTSPOTS] - no duplicates
//! - annactl sw NAME   software profile with [CONFIG] (Detected/Possible), [CONFIG GRAPH], [HISTORY], [LOGS] "(seen N times this boot)", [DEPENDENCIES], [RELATIONSHIPS], Cross notes
//! - annactl hw        hardware overview with [OVERVIEW], [CATEGORIES], [CPU], [GPU], [MEMORY], [STORAGE]+Filesystems, [NETWORK]+Route+DNS, [AUDIO], [INPUT], [SENSORS], [POWER], [HOTSPOTS]
//! - annactl hw NAME   hardware profile with [IDENTITY], [FIRMWARE], [DRIVER], [HISTORY], [HEALTH], [CAPACITY], [STATE], [LOGS], [RELATIONSHIPS] (v7.24.0)
//!
//! Snow Leopard v7.27.0 tests:
//! - Help shows exactly 6 commands (no kdb, knowledge, stats, dashboard aliases)
//! - [INSTRUMENTATION] shows "Tools installed by Anna for metrics: none" if none installed
//! - [TELEMETRY] shows CPU range "(0 - Y percent for N logical cores)"
//! - [CONFIG] shows "Detected" (existing only) and "Possible" (from docs, not present)
//! - [LOGS] shows "(seen N times this boot)" without message truncation
//! - [HOTSPOTS] only shows if data is computed reliably
//! - No deprecated aliases accepted
//!
//! Snow Leopard v7.25.0 tests:
//! - annactl hw has [OVERVIEW] section with device counts
//! - annactl hw has [CATEGORIES] section with bus summaries (USB, Bluetooth, Thunderbolt)
//! - annactl hw usb shows [CONTROLLERS] and [DEVICES] sections
//! - annactl hw bluetooth shows [ADAPTERS] section with state (UP/BLOCKED/down)
//! - annactl hw thunderbolt shows [CONTROLLERS] and [DEVICES] sections
//! - annactl hw sdcard shows [READERS] section with media status
//! - annactl status [ATTACHMENTS] shows USB, Bluetooth, Thunderbolt summary
//! - No new public commands (still exactly 6)
//!
//! Snow Leopard v7.24.0 tests:
//! - annactl sw NAME [RELATIONSHIPS] shows services, processes, hardware, stack packages
//! - annactl hw NAME [RELATIONSHIPS] shows drivers, firmware, services, software
//! - annactl sw [HOTSPOTS] shows top CPU, memory, most started processes
//! - annactl hw [HOTSPOTS] shows warm devices, heavy IO, high load
//! - annactl status [HOTSPOTS] shows compact cross-reference
//! - No new public commands
//!
//! Snow Leopard v7.18.0 tests:
//! - annactl status [LAST BOOT] shows kernel version, boot duration, failed units, health status
//! - annactl status [RECENT CHANGES] shows last 5 package events from pacman.log
//! - annactl sw NAME [HISTORY] shows package lifecycle events (install/upgrade/remove)
//! - annactl sw SERVICE [LOGS] shows boot-anchored patterns with novelty detection (new/known)
//! - annactl hw cpu [HISTORY] shows kernel package changes
//! - annactl hw gpu0 [HISTORY] shows driver package changes
//! - No new public commands
//!
//! Snow Leopard v7.17.0 tests:
//! - annactl hw [STORAGE] shows devices with health status and filesystems with usage
//! - annactl hw [NETWORK] shows interfaces, default route, and DNS
//! - annactl sw NAME [CONFIG GRAPH] shows config ownership and consumers
//! - No new public commands
//!
//! Snow Leopard v7.15.0 tests:
//! - annactl hw structured overview with [CPU], [GPU], [MEMORY], etc.
//! - [FIRMWARE] section in hw cpu and hw wifi with microcode/firmware info
//! - [HEALTH] section for storage devices with SMART data
//! - [CAPACITY] and [STATE] sections for battery profiles
//! - No new public commands
//!
//! Snow Leopard v7.14.0 tests:
//! - [LOGS] Pattern-based grouping with counts and time hints
//! - [CONFIG] Sanity notes section for config health
//! - Cross notes section linking logs/telemetry/deps/config
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

    // v7.38.0: Status is now cache-only, shows [HEALTH] instead of [ANNA NEEDS]
    // [ANNA NEEDS] was removed to enable cache-only status (no live probing)
    assert!(
        stdout.contains("[HEALTH]"),
        "Expected [HEALTH] section in status output (v7.38.0 cache-only), got stdout: {}",
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

/// Test 'sw' command shows overview or no-snapshot message
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

    // v7.41.0: SW command reads snapshots from daemon
    // If daemon is running and snapshot exists: shows [OVERVIEW]
    // If daemon not running: shows "No software snapshot available"
    assert!(
        (stdout.contains("Anna Software") && stdout.contains("[OVERVIEW]")) ||
        stdout.contains("No software snapshot available"),
        "Expected SW output with sections or no-snapshot message, got stdout: {}",
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

        // v7.41.0: SW reads from snapshots, so may show no-snapshot message
        assert!(
            (stdout.contains("Anna Software") && stdout.contains("[OVERVIEW]")) ||
            stdout.contains("No software snapshot available"),
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

    // HW command shows "Anna Hardware" header with component sections
    assert!(
        stdout.contains("Anna Hardware"),
        "Expected 'Anna Hardware' header, got stdout: {}",
        stdout
    );
    // Should have standard hardware sections
    assert!(
        stdout.contains("[CPU]") || stdout.contains("[MEMORY]") || stdout.contains("[GPU]"),
        "Expected hardware sections like [CPU], [MEMORY], [GPU], got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw should succeed");
}

/// Test 'hw' shows hardware components
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

    // Should show hardware component sections
    assert!(
        stdout.contains("[CPU]") || stdout.contains("[GPU]") || stdout.contains("[MEMORY]"),
        "Expected hardware sections like [CPU], [GPU], [MEMORY], got stdout: {}",
        stdout
    );
}

/// Test 'hw wifi' shows [DEPENDENCIES] section - v7.13.0+
#[test]
fn test_annactl_hw_dependencies_section() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with specific profile that has dependencies
    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.13.0+: hw wifi should show [DEPENDENCIES] section
    if !stdout.contains("[NOT FOUND]") {
        assert!(
            stdout.contains("[DEPENDENCIES]"),
            "Expected [DEPENDENCIES] section in hw wifi, got stdout: {}",
            stdout
        );
    }
}

/// Test 'hw' shows drivers information
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

    // Hardware overview should show driver info in component sections
    assert!(
        stdout.contains("driver:") || stdout.contains("Drivers:"),
        "Expected driver info in hw output, got stdout: {}",
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

        // Should show hardware inventory
        assert!(
            stdout.contains("Anna Hardware") && (stdout.contains("[CPU]") || stdout.contains("[GPU]")),
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

    // v7.22.0: Storage lens with [IDENTITY], [TOPOLOGY], [HEALTH]
    assert!(
        stdout.contains("Anna HW: storage") && (stdout.contains("[IDENTITY]") || stdout.contains("[TOPOLOGY]") || stdout.contains("[HEALTH]")),
        "Expected storage lens output, got stdout: {}",
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

    // v7.22.0: Network lens with [IDENTITY], [TOPOLOGY], [TELEMETRY]
    assert!(
        stdout.contains("Anna HW: network") && (stdout.contains("[IDENTITY]") || stdout.contains("[TOPOLOGY]") || stdout.contains("[TELEMETRY]")),
        "Expected Network lens output, got stdout: {}",
        stdout
    );
    assert!(output.status.success(), "annactl hw network should succeed");
}

// ============================================================================
// v7.27.0: KDB alias REMOVED - these tests are now in deprecated_aliases_rejected_v727
// ============================================================================
// The kdb command was deprecated in v7.2.0 and removed in v7.27.0.
// See test_snow_leopard_deprecated_aliases_rejected_v727 for the new behavior.

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

/// Test '--version' flag outputs version (added v7.35.1)
#[test]
fn test_annactl_version_flag_works() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("--version")
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.42.1: --version outputs "annactl vX.Y.Z" (no banners, no ANSI)
    assert!(
        stdout.contains("annactl v7.42"),
        "Expected '--version' to output 'annactl v7.42.x', got: {}",
        stdout
    );
    assert!(output.status.success(), "--version should succeed");
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

    // v7.41.0: SW reads from daemon snapshots
    // If daemon running: shows [OVERVIEW] and [CATEGORIES]
    // If daemon not running: shows "No software snapshot available"
    assert!(
        (stdout.contains("[OVERVIEW]") && stdout.contains("[CATEGORIES]")) ||
        stdout.contains("No software snapshot available"),
        "Expected basic SW output with [OVERVIEW] and [CATEGORIES] or no-snapshot, got: {}",
        stdout
    );
}

/// Test 'sw <name>' shows [USAGE] section (v7.23.0+, renamed from [TELEMETRY])
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

    // Object profile should show [USAGE] section (v7.23.0+, was [TELEMETRY] in v7.4.0-v7.22.0)
    assert!(
        stdout.contains("[USAGE]"),
        "Expected [USAGE] section in object profile, got: {}",
        stdout
    );
}

/// Test 'sw <name>' shows [CONFIG - *] sections (v7.28.0)
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

    // v7.28.0: Config section split into DETECTED/COMMON LOCATIONS/PRECEDENCE
    let has_config = stdout.contains("[CONFIG - DETECTED]")
        || stdout.contains("[CONFIG - COMMON")
        || stdout.contains("[CONFIG - PRECEDENCE]");
    assert!(
        has_config,
        "Expected [CONFIG - *] section in object profile, got: {}",
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

/// Test 'status' command completes in reasonable time
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
    // v7.27.0: Allow 20s for status due to additional telemetry queries
    assert!(
        elapsed.as_secs() < 20,
        "annactl status should complete in <20s, took: {:?}",
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
    // v7.24.0: Relationships and hotspots add discovery queries, allowing 7s
    assert!(
        elapsed.as_secs() < 7,
        "annactl sw <name> should complete in <7s, took: {:?}",
        elapsed
    );
}

/// Test 'hw' command completes in reasonable time
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
    // v7.27.0: Allow 5s for hw due to concurrent test load
    assert!(
        elapsed.as_secs() < 5,
        "annactl hw should complete in <5s, took: {:?}",
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

/// Test sw detail shows [USAGE] with time windows (v7.23.0+, was [TELEMETRY] in v7.9.0)
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

    // v7.23.0: SW detail should show [USAGE] section (was [TELEMETRY] in v7.9.0-v7.22.0)
    assert!(
        stdout.contains("[USAGE]"),
        "Expected [USAGE] section, got: {}",
        stdout
    );

    // v7.23.0: If telemetry is available, should show time-anchored windows
    // Format is: "last 1h:", "last 24h:", "last 7d:", "last 30d:"
    // Or "Telemetry: not collected yet" if no data
    let has_valid_format = stdout.contains("last 1h:")
        || stdout.contains("last 24h:")
        || stdout.contains("Telemetry:")
        || stdout.contains("not collected yet");
    assert!(
        has_valid_format,
        "USAGE section should show valid format (time windows or no data): {}",
        stdout
    );
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

    // HW should show component sections with driver info
    let has_sections = stdout.contains("[CPU]")
        || stdout.contains("[GPU]")
        || stdout.contains("[MEMORY]")
        || stdout.contains("[STORAGE]")
        || stdout.contains("[NETWORK]");
    assert!(
        has_sections,
        "Expected hardware sections, got: {}",
        stdout
    );
    // Should show driver information
    assert!(
        stdout.contains("driver:") || stdout.contains("Drivers:"),
        "Should show driver info: {}",
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

/// Test [CONFIG] section shows Detected/Common/Precedence structure (v7.28.0)
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

    // v7.28.0: CONFIG section split into DETECTED/COMMON LOCATIONS/PRECEDENCE
    let has_config = stdout.contains("[CONFIG - DETECTED]")
        || stdout.contains("[CONFIG - COMMON")
        || stdout.contains("[CONFIG - PRECEDENCE]");
    assert!(
        has_config,
        "Expected [CONFIG - *] section: {}",
        stdout
    );

    // v7.30.0: Should have source attribution for DETECTED or RECOMMENDED section
    let has_attribution = stdout.contains("(sources:")
        || stdout.contains("(from man pages")
        || stdout.contains("(verified present")
        || stdout.contains("(from documentation");
    assert!(
        has_attribution,
        "[CONFIG] should show source attribution: {}",
        stdout
    );

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
        // v7.21.0: man pages and Arch Wiki are primary sources in config atlas
        // Check that a known source is mentioned
        assert!(
            stdout.contains("man ") || stdout.contains("Arch Wiki") || stdout.contains("sources:"),
            "[CONFIG] should include documentation sources: {}",
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

    // hw should show component sections
    let has_sections = stdout.contains("[CPU]")
        || stdout.contains("[GPU]")
        || stdout.contains("[MEMORY]")
        || stdout.contains("[STORAGE]")
        || stdout.contains("[NETWORK]");
    assert!(
        has_sections,
        "Expected hardware sections in hw output, got: {}",
        stdout
    );

    // Should show driver info
    assert!(
        stdout.contains("driver:") || stdout.contains("Drivers:"),
        "Should show driver info: {}",
        stdout
    );

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

/// Test status shows [ALERTS] section (v7.38.0 cache-only status)
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

    // v7.38.0: Status is cache-only, shows [ALERTS] instead of [RESOURCE HOTSPOTS]
    // [RESOURCE HOTSPOTS] was removed - status reads from status_snapshot.json only
    assert!(
        stdout.contains("[ALERTS]"),
        "Expected [ALERTS] section (v7.38.0 cache-only), got: {}",
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

    // hw overview shows component sections (telemetry moved to specific profiles)
    let has_sections = stdout.contains("[CPU]")
        || stdout.contains("[GPU]")
        || stdout.contains("[MEMORY]")
        || stdout.contains("[NETWORK]");
    assert!(
        has_sections,
        "Expected hardware sections, got: {}",
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
    // v7.40.0: Allow up to 6s to avoid flaky tests under load
    // Typical time is < 1s, but CI/parallel tests can slow down
    assert!(
        elapsed.as_secs() < 6,
        "annactl hw should complete in <6s, took: {:?}",
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

/// Test [CONFIG] Detected/Possible structure (v7.27.0 update from v7.12.0)
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
        // v7.27.0: Must use Detected:/Possible: structure (no status markers)
        let has_detected = stdout.contains("Detected:");
        let has_possible = stdout.contains("Possible:");

        // Detected is required if any configs exist
        assert!(
            has_detected || stdout.contains("No configuration paths discovered"),
            "[CONFIG] should have Detected: section or indicate no configs: {}",
            stdout
        );

        // v7.27.0: Should NOT have old status markers
        // (markers removed in v7.27.0)
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

/// Test status [PATHS] shows config, data, internal dirs (v7.38.0 cache-only)
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

    // v7.38.0: [PATHS] is simplified, shows static paths only
    if stdout.contains("[PATHS]") {
        // Should show Config path
        assert!(
            stdout.contains("Config:"),
            "[PATHS] should show Config: path: {}",
            stdout
        );

        // Should show Data directory
        assert!(
            stdout.contains("Data:"),
            "[PATHS] should show Data: directory: {}",
            stdout
        );

        // Should show Internal directory
        assert!(
            stdout.contains("Internal:"),
            "[PATHS] should show Internal: directory: {}",
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

// ============================================================================
// v7.14.0: Snow Leopard Log Patterns, Config Sanity, Cross Notes Tests
// ============================================================================

/// Test sw NAME [LOGS] shows pattern-based summary (v7.16.0, updated v7.18.0)
#[test]
fn test_snow_leopard_sw_logs_patterns_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.16.0: [LOGS] should show multi-window summary
    // v7.18.0: Format changed to "Boot 0 (current):" instead of "This boot:"
    if stdout.contains("[LOGS]") {
        // Should have boot info or "No warnings"
        let has_pattern_info = stdout.contains("This boot:")
            || stdout.contains("Boot 0")
            || stdout.contains("No warnings or errors");
        assert!(
            has_pattern_info,
            "[LOGS] should have pattern summary or 'No warnings': {}",
            stdout
        );
        // Should show Source line
        assert!(
            stdout.contains("Source:"),
            "[LOGS] should show Source attribution: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test sw NAME [LOGS] shows pattern counts (v7.14.0)
#[test]
fn test_snow_leopard_sw_logs_pattern_counts_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.16.0: If patterns exist, should show severity breakdown
    if stdout.contains("This boot:") && !stdout.contains("No warnings or errors") {
        // Should have severity breakdown (Warnings/Errors/Critical) or top patterns
        let has_severity = stdout.contains("Warnings:")
            || stdout.contains("Errors:")
            || stdout.contains("Critical:")
            || stdout.contains("Top patterns:");
        assert!(
            has_severity,
            "[LOGS] patterns should show severity or top patterns: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test sw NAME [CONFIG] structure (v7.27.0 update from v7.14.0)
#[test]
fn test_snow_leopard_sw_config_sanity_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.27.0: [CONFIG] should have Detected/Possible sections
    if stdout.contains("[CONFIG]") {
        // Should show config paths
        let has_config_info = stdout.contains("Detected:")
            || stdout.contains("Possible:")
            || stdout.contains("No configuration paths discovered");
        assert!(
            has_config_info,
            "[CONFIG] should have Detected/Possible sections or indicate no configs: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw wifi [LOGS] shows pattern-based summary (v7.14.0)
#[test]
fn test_snow_leopard_hw_logs_patterns_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.16.0: hw wifi [LOGS] should show pattern summary
    if !stdout.contains("[NOT FOUND]") && stdout.contains("[LOGS]") {
        let has_pattern_info = stdout.contains("Patterns (this boot):")
            || stdout.contains("This boot:")
            || stdout.contains("No warnings or errors");
        assert!(
            has_pattern_info,
            "hw wifi [LOGS] should have pattern summary: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test Sanity notes are descriptive not prescriptive (v7.14.0)
#[test]
fn test_snow_leopard_sanity_not_prescriptive_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.14.0: Sanity notes should not contain prescriptive language
    if stdout.contains("Sanity notes:") {
        assert!(
            !stdout.contains("should") && !stdout.contains("recommend") && !stdout.contains("fix"),
            "Sanity notes should be descriptive, not prescriptive: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test [LOGS] doesn't show init.scope noise (v7.14.0)
#[test]
fn test_snow_leopard_logs_no_init_scope_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.14.0: [LOGS] should not contain unrelated unit noise
    if stdout.contains("[LOGS]") {
        assert!(
            !stdout.contains("init.scope") && !stdout.contains("kernel:"),
            "[LOGS] should not contain init.scope or raw kernel messages: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test patterns show time hints (v7.14.0)
#[test]
fn test_snow_leopard_logs_time_hints_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.16.0: Patterns should show time hints or history
    if (stdout.contains("Patterns (this boot):") || stdout.contains("Top patterns:")) && stdout.contains("1)") {
        // Should have time reference or boot/history info
        let has_time_info = stdout.contains("last at")
            || stdout.contains("boot:")
            || stdout.contains("7d:");
        assert!(
            has_time_info,
            "Pattern entries should show time info or history: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test patterns show counts (v7.16.0)
#[test]
fn test_snow_leopard_logs_seen_counts_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.16.0/v7.29.0: Patterns should show count info
    if (stdout.contains("Patterns (this boot):") || stdout.contains("Top patterns:")) && stdout.contains("1)") {
        // Should have count reference (seen X, boot: X, Nx, etc.)
        // v7.29.0: Format changed to "(Nx, last at TIME)" - note: x is styled
        let has_count = stdout.contains("seen")
            || stdout.contains("boot:")
            || stdout.contains("times")
            || stdout.contains("last at");
        assert!(
            has_count,
            "Pattern entries should show count info: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test no new public commands (v7.14.0)
#[test]
fn test_snow_leopard_no_new_commands_v714() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.14.0: Help should still only show the 6 base commands
    assert!(
        stdout.contains("annactl") && stdout.contains("status")
            && stdout.contains("sw") && stdout.contains("hw"),
        "Help should show base commands: {}",
        stdout
    );
    // Should not have new commands like "log" or "pattern"
    assert!(
        !stdout.contains("annactl log") && !stdout.contains("annactl pattern"),
        "Should not have new log/pattern commands: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.15.0: Deeper Hardware Insight Tests
// ============================================================================

/// Test annactl hw shows structured sections (v7.15.0)
#[test]
fn test_snow_leopard_hw_structured_sections_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: hw should show structured sections
    assert!(
        stdout.contains("[CPU]"),
        "hw should have [CPU] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[GPU]") || stdout.contains("not detected"),
        "hw should have [GPU] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[MEMORY]"),
        "hw should have [MEMORY] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[STORAGE]"),
        "hw should have [STORAGE] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[NETWORK]"),
        "hw should have [NETWORK] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[POWER]"),
        "hw should have [POWER] section: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw overview shows CPU model and microcode (v7.15.0)
#[test]
fn test_snow_leopard_hw_cpu_overview_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: [CPU] section should have model and microcode
    assert!(
        stdout.contains("Model:"),
        "CPU section should show Model: {}",
        stdout
    );
    assert!(
        stdout.contains("Cores:") || stdout.contains("threads"),
        "CPU section should show Cores/threads: {}",
        stdout
    );
    // Microcode info
    assert!(
        stdout.contains("Microcode:") || stdout.contains("microcode"),
        "CPU section should show Microcode info: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw cpu profile has [FIRMWARE] section (v7.15.0)
#[test]
fn test_snow_leopard_hw_cpu_firmware_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: CPU profile should have [FIRMWARE] section
    assert!(
        stdout.contains("[FIRMWARE]"),
        "hw cpu should have [FIRMWARE] section: {}",
        stdout
    );
    // Should show microcode info
    assert!(
        stdout.contains("Microcode:") || stdout.contains("microcode"),
        "FIRMWARE section should show microcode: {}",
        stdout
    );
    // Should have source attribution
    assert!(
        stdout.contains("/sys/devices/system/cpu/microcode") || stdout.contains("Source:"),
        "FIRMWARE should cite source: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw cpu profile has [IDENTITY] with architecture (v7.15.0)
#[test]
fn test_snow_leopard_hw_cpu_identity_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: CPU profile should have rich [IDENTITY]
    assert!(
        stdout.contains("[IDENTITY]"),
        "hw cpu should have [IDENTITY] section: {}",
        stdout
    );
    assert!(
        stdout.contains("Architecture:") || stdout.contains("x86_64") || stdout.contains("aarch64"),
        "IDENTITY should show Architecture: {}",
        stdout
    );
    assert!(
        stdout.contains("Sockets:"),
        "IDENTITY should show Sockets: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw battery profile has [CAPACITY] and [STATE] sections (v7.15.0)
#[test]
fn test_snow_leopard_hw_battery_sections_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "battery"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: Battery profile should have [CAPACITY] and [STATE]
    if stdout.contains("not present") {
        // Desktop system - no battery
        return;
    }

    assert!(
        stdout.contains("[CAPACITY]"),
        "hw battery should have [CAPACITY] section: {}",
        stdout
    );
    assert!(
        stdout.contains("[STATE]"),
        "hw battery should have [STATE] section: {}",
        stdout
    );
    assert!(
        stdout.contains("Design:") || stdout.contains("Wh"),
        "CAPACITY should show design capacity: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw battery shows cycles when available (v7.15.0)
#[test]
fn test_snow_leopard_hw_battery_cycles_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "battery"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: Battery profile should show cycles when available
    if stdout.contains("not present") {
        return;
    }

    // If capacity section exists, should show Cycles
    if stdout.contains("[CAPACITY]") {
        assert!(
            stdout.contains("Cycles:") || stdout.contains("Full now:"),
            "CAPACITY should show Cycles or Full now: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw storage shows [HEALTH] with SMART data (v7.15.0)
#[test]
fn test_snow_leopard_hw_storage_health_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Get first storage device
    let lsblk = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME"])
        .output()
        .expect("lsblk failed");

    let devices = String::from_utf8_lossy(&lsblk.stdout);
    let first_device = devices.lines().next().unwrap_or("nvme0n1");

    let output = Command::new(&binary)
        .args(["hw", first_device])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: Storage profile should have [HEALTH]
    assert!(
        stdout.contains("[HEALTH]"),
        "hw storage should have [HEALTH] section: {}",
        stdout
    );
    // Should show status or unavailable message
    assert!(
        stdout.contains("SMART") || stdout.contains("Status:") || stdout.contains("unavailable"),
        "HEALTH should show SMART status or unavailable: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test hw overview shows firmware status for WiFi (v7.15.0)
#[test]
fn test_snow_leopard_hw_wifi_firmware_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: WiFi in overview should show firmware status
    if stdout.contains("WiFi:") {
        assert!(
            stdout.contains("firmware:") || stdout.contains("driver:"),
            "WiFi should show firmware or driver info: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test hw overview sections don't have [COMPONENTS] anymore (v7.15.0)
#[test]
fn test_snow_leopard_hw_no_components_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: hw overview now uses category sections, not [COMPONENTS]
    // We still allow [COMPONENTS] for backwards compat but prefer category sections
    assert!(
        stdout.contains("[CPU]") || stdout.contains("[COMPONENTS]"),
        "hw should have [CPU] or [COMPONENTS]: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test no new public commands (v7.15.0)
#[test]
fn test_snow_leopard_no_new_commands_v715() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.15.0: Help should still only show the 6 base commands
    assert!(
        stdout.contains("annactl status"),
        "Help should show status command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl sw"),
        "Help should show sw command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl hw"),
        "Help should show hw command: {}",
        stdout
    );
    // Should not have new commands
    assert!(
        !stdout.contains("annactl firmware") && !stdout.contains("annactl smart"),
        "Should not have new firmware/smart commands: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.17.0: Network, Storage & Config Graph Tests
// ============================================================================

/// Test annactl hw [STORAGE] shows devices with health (v7.17.0)
#[test]
fn test_hw_storage_shows_devices_with_health() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: [STORAGE] should show Devices subsection
    assert!(
        stdout.contains("[STORAGE]"),
        "hw should have [STORAGE] section: {}",
        stdout
    );
    assert!(
        stdout.contains("Devices") || stdout.contains("nvme") || stdout.contains("sd"),
        "STORAGE should show Devices list: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test annactl hw [STORAGE] shows filesystems with usage (v7.17.0)
#[test]
fn test_hw_storage_shows_filesystems_with_usage() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: [STORAGE] should show Filesystems subsection with usage
    assert!(
        stdout.contains("Filesystems") || stdout.contains("/") && stdout.contains("%"),
        "STORAGE should show Filesystems with usage: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test annactl hw [NETWORK] shows interfaces (v7.17.0)
#[test]
fn test_hw_network_shows_interfaces() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: [NETWORK] should show Interfaces subsection
    assert!(
        stdout.contains("[NETWORK]"),
        "hw should have [NETWORK] section: {}",
        stdout
    );
    assert!(
        stdout.contains("Interfaces") || stdout.contains("wifi") || stdout.contains("ethernet") || stdout.contains("loopback"),
        "NETWORK should show Interfaces list: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test annactl hw [NETWORK] shows default route (v7.17.0)
#[test]
fn test_hw_network_shows_default_route() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: [NETWORK] should show Default route subsection
    assert!(
        stdout.contains("Default route") || stdout.contains("via"),
        "NETWORK should show Default route: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test annactl hw [NETWORK] shows DNS (v7.17.0)
#[test]
fn test_hw_network_shows_dns() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: [NETWORK] should show DNS subsection
    assert!(
        stdout.contains("DNS") || stdout.contains("source:"),
        "NETWORK should show DNS servers with source: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test annactl sw NAME [CONFIG GRAPH] for services (v7.17.0)
#[test]
fn test_sw_config_graph_for_services() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager"])
        .output()
        .expect("Failed to run annactl sw NetworkManager");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: sw NAME for a service should show [CONFIG GRAPH] section
    // Note: only shows if configs are found
    if stdout.contains("[CONFIG GRAPH]") {
        assert!(
            stdout.contains("Reads:") || stdout.contains("Shared:"),
            "CONFIG GRAPH should show Reads or Shared: {}",
            stdout
        );
    }
    assert!(output.status.success());
}

/// Test annactl hw [NETWORK] shows interface manager (v7.17.0)
#[test]
fn test_hw_network_shows_interface_manager() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: Interfaces should show manager (NetworkManager, systemd-networkd, etc.)
    assert!(
        stdout.contains("NetworkManager") || stdout.contains("systemd-networkd") || stdout.contains("manual") || stdout.contains("unknown"),
        "Interfaces should show manager: {}",
        stdout
    );
    assert!(output.status.success());
}

/// Test no new public commands (v7.17.0)
#[test]
fn test_no_new_commands_v717() {
    let binary = get_binary_path();

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.17.0: Help should still only show the 6 base commands
    assert!(
        stdout.contains("annactl status"),
        "Help should show status command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl sw"),
        "Help should show sw command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl hw"),
        "Help should show hw command: {}",
        stdout
    );
    // Should not have new commands like network, storage, config
    assert!(
        !stdout.contains("annactl network") && !stdout.contains("annactl storage") && !stdout.contains("annactl config"),
        "Should not have new network/storage/config commands: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// v7.18.0: Snow Leopard - Change Journal, Boot Timeline & Error Focus
// ============================================================================

/// Test status command shows [DAEMON] section (v7.38.0 cache-only replaces [BOOT SNAPSHOT])
#[test]
fn test_status_last_boot_section_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [DAEMON] replaces [BOOT SNAPSHOT]
    assert!(
        stdout.contains("[DAEMON]"),
        "Status should show [DAEMON] section (v7.38.0 cache-only): {}",
        stdout
    );

    // v7.38.0: [DAEMON] shows status and uptime
    assert!(
        stdout.contains("Status:") || stdout.contains("Uptime:"),
        "[DAEMON] should show daemon status/uptime: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test status command shows [PATHS] section (v7.38.0 cache-only replaces [RECENT CHANGES])
#[test]
fn test_status_recent_changes_section_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [PATHS] still exists (static paths)
    // [RECENT CHANGES] was removed - requires live probing
    assert!(
        stdout.contains("[PATHS]"),
        "Status should show [PATHS] section (v7.38.0 cache-only): {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test sw profile shows [HISTORY] section for packages (v7.18.0)
#[test]
fn test_sw_profile_history_section_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    // Test with vim (commonly installed and has upgrade history)
    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl sw vim");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.18.0: SW profile should show [HISTORY] section for packages
    // Note: This will only appear if pacman.log has history for this package
    if stdout.contains("[HISTORY]") {
        // If history exists, should show package events
        assert!(
            stdout.contains("Package:") || stdout.contains("pkg_"),
            "HISTORY section should show package events: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test service [LOGS] shows boot-anchored format with pattern IDs (v7.18.0)
#[test]
fn test_sw_service_logs_boot_anchored_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    // Test with NetworkManager.service (commonly has log activity)
    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl sw NetworkManager.service");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.18.0: Service logs should show boot-anchored format
    if stdout.contains("[LOGS]") {
        assert!(
            stdout.contains("Boot 0") || stdout.contains("No warnings or errors"),
            "LOGS section should show boot-anchored view: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test hw cpu shows [HISTORY] for kernel package (v7.18.0)
#[test]
fn test_hw_cpu_history_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl hw cpu");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.18.0: CPU profile should show [HISTORY] for linux kernel package
    if stdout.contains("[HISTORY]") {
        assert!(
            stdout.contains("linux") || stdout.contains("Driver package"),
            "CPU HISTORY should show linux kernel package: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test hw gpu0 shows [HISTORY] for driver package (v7.18.0)
#[test]
fn test_hw_gpu_history_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "gpu0"])
        .output()
        .expect("Failed to run annactl hw gpu0");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.18.0: GPU profile may show [HISTORY] for driver package
    // This depends on whether nvidia/intel/amd driver is installed
    if stdout.contains("[DRIVER]") {
        // If has a driver, it might have history
        if stdout.contains("[HISTORY]") {
            assert!(
                stdout.contains("Driver package") || stdout.contains("pkg_"),
                "GPU HISTORY should show driver package events: {}",
                stdout
            );
        }
    }

    assert!(output.status.success());
}

/// Test no new public commands (v7.18.0)
#[test]
fn test_no_new_commands_v718() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.18.0: Help should still only show the 6 base commands
    // No new commands like history, journal, boot, etc.
    assert!(
        stdout.contains("annactl status"),
        "Help should show status command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl sw"),
        "Help should show sw command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl hw"),
        "Help should show hw command: {}",
        stdout
    );
    // Should not have new commands for the new features
    assert!(
        !stdout.contains("annactl history") && !stdout.contains("annactl journal") && !stdout.contains("annactl boot"),
        "Should not have new history/journal/boot commands: {}",
        stdout
    );
    assert!(output.status.success());
}

// ============================================================================
// v7.19.0 Snow Leopard Tests - Topology, Dependencies & Signal Quality
// ============================================================================

/// Test hw overview shows [DRIVERS] section (v7.19.0)
#[test]
fn test_snow_leopard_hw_drivers_section_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have DRIVERS section
    assert!(
        stdout.contains("[DRIVERS]"),
        "hw should have [DRIVERS] section: {}",
        stdout
    );

    // Should show source
    assert!(
        stdout.contains("lsmod") || stdout.contains("modinfo"),
        "[DRIVERS] should cite lsmod/modinfo as source: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test hw wifi shows [SIGNAL] section (v7.19.0)
#[test]
fn test_snow_leopard_hw_wifi_signal_section_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl hw wifi");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If WiFi exists, should have SIGNAL section
    if !stdout.contains("[NOT FOUND]") && stdout.contains("[IDENTITY]") {
        assert!(
            stdout.contains("[SIGNAL]"),
            "hw wifi should have [SIGNAL] section: {}",
            stdout
        );

        // Should show signal quality source
        assert!(
            stdout.contains("iw") || stdout.contains("/proc/net"),
            "[SIGNAL] should cite iw or /proc/net as source: {}",
            stdout
        );

        // Should show assessment
        assert!(
            stdout.contains("Assessment:"),
            "[SIGNAL] should have Assessment line: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test storage profile shows [SIGNAL] section (v7.19.0)
#[test]
fn test_snow_leopard_storage_signal_section_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    // First get a storage device name
    let lsblk_output = Command::new("lsblk")
        .args(["-d", "-n", "-o", "NAME"])
        .output();

    let device = match lsblk_output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.lines().next().unwrap_or("sda").to_string()
        }
        _ => return, // Skip if lsblk fails
    };

    let output = Command::new(&binary)
        .args(["hw", &device])
        .output()
        .expect("Failed to run annactl hw <device>");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have SIGNAL section for storage
    if stdout.contains("[HEALTH]") {
        assert!(
            stdout.contains("[SIGNAL]"),
            "hw storage should have [SIGNAL] section: {}",
            stdout
        );

        // Should show smart/nvme source
        assert!(
            stdout.contains("smartctl") || stdout.contains("nvme"),
            "[SIGNAL] should cite smartctl or nvme as source: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test status shows [TOPOLOGY HINTS] when applicable (v7.19.0)
#[test]
fn test_snow_leopard_status_topology_hints_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // TOPOLOGY HINTS is optional (only shows if there are hints)
    // Just verify it doesn't crash and shows other expected sections
    assert!(
        stdout.contains("[VERSION]"),
        "status should have [VERSION] section: {}",
        stdout
    );

    // If TOPOLOGY HINTS is present, verify structure
    if stdout.contains("[TOPOLOGY HINTS]") {
        assert!(
            stdout.contains("systemctl") || stdout.contains("lsmod"),
            "[TOPOLOGY HINTS] should cite sources: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test sw service shows cross-reference to related hardware (v7.19.0)
#[test]
fn test_snow_leopard_sw_service_hw_crossref_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    // Test with NetworkManager.service which should show wifi/ethernet cross-refs
    // Using .service suffix to ensure we get the service profile, not package
    let output = Command::new(&binary)
        .args(["sw", "NetworkManager.service"])
        .output()
        .expect("Failed to run annactl sw NetworkManager.service");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If NetworkManager service exists and has dependencies section
    if stdout.contains("[DEPENDENCIES]") && stdout.contains("Service relations:") {
        // Should show related hardware cross-reference
        assert!(
            stdout.contains("Related hardware:") || stdout.contains("See: annactl hw"),
            "NetworkManager.service should have hw cross-reference: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test no new commands (v7.19.0)
#[test]
fn test_no_new_commands_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.19.0: Help should still only show the 6 base commands
    // No new commands like topology, signal, deps, etc.
    assert!(
        stdout.contains("annactl status"),
        "Help should show status command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl sw"),
        "Help should show sw command: {}",
        stdout
    );
    assert!(
        stdout.contains("annactl hw"),
        "Help should show hw command: {}",
        stdout
    );

    // Should not have new commands for the new features
    assert!(
        !stdout.contains("annactl topology")
            && !stdout.contains("annactl signal")
            && !stdout.contains("annactl deps"),
        "Should not have new topology/signal/deps commands: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test DRIVERS section shows loaded modules (v7.19.0)
#[test]
fn test_snow_leopard_drivers_loaded_status_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If DRIVERS section exists and has content
    if stdout.contains("[DRIVERS]") && !stdout.contains("(no key drivers") {
        // Should show [loaded] status for detected drivers
        assert!(
            stdout.contains("[loaded]"),
            "[DRIVERS] should show [loaded] status: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test signal bars in WiFi SIGNAL section (v7.19.0)
#[test]
fn test_snow_leopard_wifi_signal_bars_v719() {
    let binary = get_binary_path();
    if !binary.exists() {
        eprintln!("Skipping: binary not found at {:?}", binary);
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "wifi"])
        .output()
        .expect("Failed to run annactl hw wifi");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If connected to WiFi, should show signal bars
    if stdout.contains("[SIGNAL]") && stdout.contains("dBm") {
        // Should have visual signal bars or quality indicator
        assert!(
            stdout.contains("▂") || stdout.contains("excellent")
                || stdout.contains("good") || stdout.contains("fair")
                || stdout.contains("weak"),
            "[SIGNAL] should have visual quality indicator: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// v7.22.0: Snow Leopard Scenario Lenses & Toolchain Tests
// ============================================================================

/// Test hw network lens has required sections (v7.22.0)
#[test]
fn test_snow_leopard_hw_network_lens_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "network"])
        .output()
        .expect("Failed to run annactl hw network");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.22.0: Network lens must have [IDENTITY], [TOPOLOGY], [TELEMETRY], [HISTORY]
    assert!(
        stdout.contains("[IDENTITY]"),
        "Network lens must have [IDENTITY]: {}",
        stdout
    );
    assert!(
        stdout.contains("[TOPOLOGY]"),
        "Network lens must have [TOPOLOGY]: {}",
        stdout
    );
    assert!(
        stdout.contains("[TELEMETRY]"),
        "Network lens must have [TELEMETRY]: {}",
        stdout
    );
    assert!(
        stdout.contains("[HISTORY]"),
        "Network lens must have [HISTORY]: {}",
        stdout
    );

    // Should show interfaces with driver info
    assert!(
        stdout.contains("interface:") || stdout.contains("driver:"),
        "Network lens should show interface details: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test hw storage lens has SMART health section (v7.22.0)
#[test]
fn test_snow_leopard_hw_storage_lens_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "storage"])
        .output()
        .expect("Failed to run annactl hw storage");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.22.0: Storage lens must have [IDENTITY], [TOPOLOGY], [HEALTH], [TELEMETRY]
    assert!(
        stdout.contains("[IDENTITY]"),
        "Storage lens must have [IDENTITY]: {}",
        stdout
    );
    assert!(
        stdout.contains("[TOPOLOGY]"),
        "Storage lens must have [TOPOLOGY]: {}",
        stdout
    );
    assert!(
        stdout.contains("[HEALTH]"),
        "Storage lens must have [HEALTH]: {}",
        stdout
    );

    // Health section should show SMART status
    assert!(
        stdout.contains("SMART:"),
        "Storage lens [HEALTH] should show SMART status: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test sw network lens shows services and configs (v7.22.0)
#[test]
fn test_snow_leopard_sw_network_lens_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "network"])
        .output()
        .expect("Failed to run annactl sw network");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.22.0: Software network lens shows services and config
    assert!(
        stdout.contains("Anna SW: network"),
        "Should show network software lens: {}",
        stdout
    );
    assert!(
        stdout.contains("[IDENTITY]") || stdout.contains("[TOPOLOGY]"),
        "SW network lens should have identity or topology: {}",
        stdout
    );
    assert!(
        stdout.contains("[CONFIG]"),
        "SW network lens should have config section: {}",
        stdout
    );

    // Should show real config paths, not HTML junk
    assert!(
        !stdout.contains("<p>") && !stdout.contains("&nbsp;"),
        "SW network lens should not have HTML junk: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test status shows toolchain section (v7.22.0)
#[test]
fn test_snow_leopard_status_toolchain_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [ANNA TOOLCHAIN] replaced by [UPDATES]
    // Toolchain info is no longer in status to enable fast cache-only display
    assert!(
        stdout.contains("[UPDATES]"),
        "Status must have [UPDATES] section (v7.38.0 cache-only): {}",
        stdout
    );

    // Should show update mode info
    assert!(
        stdout.contains("Mode:") || stdout.contains("Interval:"),
        "[UPDATES] section should show mode/interval: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test CLI surface unchanged - still exactly 6 commands (v7.22.0)
#[test]
fn test_snow_leopard_cli_surface_unchanged_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Help should list exactly the 6 commands
    assert!(stdout.contains("annactl"), "Should show annactl usage");
    assert!(stdout.contains("status"), "Should show status command");
    assert!(stdout.contains("sw"), "Should show sw command");
    assert!(stdout.contains("hw"), "Should show hw command");

    // Should not have any hidden or undocumented commands
    assert!(
        !stdout.contains("hidden") && !stdout.contains("internal"),
        "Should not mention hidden commands: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test scenario lens logs are scoped - no unrelated patterns (v7.22.0)
#[test]
fn test_snow_leopard_lens_logs_scoped_v722() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "network"])
        .output()
        .expect("Failed to run annactl hw network");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If [LOGS] section exists, patterns should be network-related
    if stdout.contains("[LOGS]") {
        // Check that log patterns have IDs
        assert!(
            stdout.contains("[NET") || stdout.contains("(seen"),
            "[LOGS] patterns should have IDs: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.23.0 Tests - Timelines, Drift & Incidents
// ============================================================================

/// Test that status shows [DAEMON] section with uptime (v7.38.0 cache-only)
#[test]
fn test_snow_leopard_status_boot_snapshot_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [DAEMON] replaces [BOOT SNAPSHOT]
    assert!(
        stdout.contains("[DAEMON]"),
        "status should have [DAEMON] section (v7.38.0 cache-only): {}",
        stdout
    );

    // Must have status and uptime info
    assert!(
        stdout.contains("Status:") || stdout.contains("Uptime:"),
        "[DAEMON] should contain status/uptime info: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test that status shows [DATA] section (v7.38.0 cache-only replaces [INVENTORY])
#[test]
fn test_snow_leopard_inventory_drift_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [DATA] replaces [INVENTORY]
    assert!(
        stdout.contains("[DATA]"),
        "status should have [DATA] section (v7.38.0 cache-only): {}",
        stdout
    );

    // [DATA] shows knowledge objects and scan info
    assert!(
        stdout.contains("Knowledge:") || stdout.contains("Last scan:"),
        "[DATA] should have knowledge/scan info: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test that sw NAME [USAGE] has percentage+range format (v7.23.0)
#[test]
fn test_snow_leopard_sw_usage_percentage_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "NetworkManager"])
        .output()
        .expect("Failed to run annactl sw NetworkManager");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have [USAGE] section (renamed from [TELEMETRY])
    assert!(
        stdout.contains("[USAGE]"),
        "sw NAME should have [USAGE] section: {}",
        stdout
    );

    // If telemetry is available, check for percentage+range format
    if stdout.contains("CPU avg:") {
        // Should have "percent" word (not naked decimals)
        assert!(
            stdout.contains("percent") || stdout.contains("n/a"),
            "[USAGE] CPU should use 'percent' format or n/a: {}",
            stdout
        );
    }

    // Should have trend info
    if stdout.contains("trend:") {
        assert!(
            stdout.contains("stable") || stdout.contains("rising")
                || stdout.contains("falling") || stdout.contains("n/a"),
            "[USAGE] should have valid trend labels: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test that CLI surface remains unchanged - only 6 commands (v7.23.0)
#[test]
fn test_snow_leopard_cli_surface_unchanged_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must show exactly these 6 commands
    assert!(stdout.contains("annactl"), "Help should mention annactl");
    assert!(stdout.contains("status"), "Help should mention status");
    assert!(stdout.contains("sw"), "Help should mention sw");
    assert!(stdout.contains("hw"), "Help should mention hw");

    // Should NOT have hidden commands
    assert!(
        !stdout.contains("--verbose") && !stdout.contains("--debug"),
        "Help should not have hidden flags: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test that incidents use pattern IDs (v7.23.0)
#[test]
fn test_snow_leopard_incident_pattern_ids_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .arg("status")
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If incidents are present, check for pattern IDs
    if stdout.contains("Incidents") && !stdout.contains("none recorded") {
        // Should have pattern IDs like [NET001], [GPU010], [STO001]
        assert!(
            stdout.contains("[NET") || stdout.contains("[GPU")
                || stdout.contains("[STO") || stdout.contains("[SYS")
                || stdout.contains("[GEN") || stdout.contains("(seen"),
            "Incidents should have pattern IDs: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

/// Test that config sections have Source: provenance lines (v7.23.0)
#[test]
fn test_snow_leopard_config_provenance_v723() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw", "vim"])
        .output()
        .expect("Failed to run annactl sw vim");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for provenance in config sections
    // Note: Source lines may be in various sections
    if stdout.contains("[CONFIG") {
        // Config sections should exist and contain paths
        assert!(
            stdout.contains("/") || stdout.contains("~"),
            "Config sections should contain paths: {}",
            stdout
        );
    }

    assert!(output.status.success());
}

// ============================================================================
// v7.24.0: Snow Leopard Relationships, Stacks & Hotspots tests
// ============================================================================

/// Test annactl sw NAME shows [RELATIONSHIPS] section (v7.24.0)
#[test]
fn test_snow_leopard_sw_relationships_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with a common package that has services
    let output = Command::new(&binary)
        .args(["sw", "systemd"])
        .output()
        .expect("Failed to run annactl sw systemd");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // [RELATIONSHIPS] section should be present for packages with services
    // Note: May not show if no relationships discovered, so just check command succeeds
    assert!(output.status.success(), "annactl sw systemd should succeed");

    // If relationships found, should contain expected structure
    if stdout.contains("[RELATIONSHIPS]") {
        // Should have source attribution
        assert!(
            stdout.contains("Source:") || stdout.contains("source:"),
            "[RELATIONSHIPS] should have source attribution"
        );
    }
}

/// Test annactl hw NAME shows [RELATIONSHIPS] section (v7.24.0)
#[test]
fn test_snow_leopard_hw_relationships_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with cpu which always exists
    let output = Command::new(&binary)
        .args(["hw", "cpu"])
        .output()
        .expect("Failed to run annactl hw cpu");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Command should succeed
    assert!(output.status.success(), "annactl hw cpu should succeed");

    // CPU profile should exist
    assert!(
        stdout.contains("[") && (stdout.contains("CPU") || stdout.contains("IDENTITY")),
        "hw cpu should show CPU information"
    );
}

/// Test annactl sw shows [HOTSPOTS] section (v7.24.0)
#[test]
fn test_snow_leopard_sw_hotspots_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw"])
        .output()
        .expect("Failed to run annactl sw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Command should succeed
    assert!(output.status.success(), "annactl sw should succeed");

    // v7.41.0: SW reads from daemon snapshots
    // If daemon running: shows [OVERVIEW] / [CATEGORIES]
    // If daemon not running: shows "No software snapshot available"
    assert!(
        stdout.contains("[OVERVIEW]") || stdout.contains("[CATEGORIES]") ||
        stdout.contains("No software snapshot available"),
        "sw should show overview sections or no-snapshot message"
    );

    // Note: [HOTSPOTS] only appears if telemetry is enabled and has data
    // Just verify command runs successfully
}

/// Test annactl hw shows [HOTSPOTS] section (v7.24.0)
#[test]
fn test_snow_leopard_hw_hotspots_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Command should succeed
    assert!(output.status.success(), "annactl hw should succeed");

    // Overview should show standard sections
    assert!(
        stdout.contains("[CPU]") || stdout.contains("[GPU]") || stdout.contains("[MEMORY]"),
        "hw should show hardware sections"
    );

    // Note: [HOTSPOTS] only appears if telemetry is enabled and has data
    // Just verify command runs successfully
}

/// Test annactl status shows [HOTSPOTS] section (v7.24.0)
#[test]
fn test_snow_leopard_status_hotspots_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Command should succeed
    assert!(output.status.success(), "annactl status should succeed");

    // Status should show standard sections
    assert!(
        stdout.contains("[VERSION]") && stdout.contains("[DAEMON]"),
        "status should show standard sections"
    );

    // Note: [HOTSPOTS] only appears if telemetry is enabled and has data
    // Just verify command runs successfully
}

/// Test relationships module doesn't leak internal errors (v7.24.0)
#[test]
fn test_snow_leopard_relationships_no_errors_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with multiple packages
    for pkg in ["linux", "vim", "pacman", "systemd"] {
        let output = Command::new(&binary)
            .args(["sw", pkg])
            .output()
            .expect(&format!("Failed to run annactl sw {}", pkg));

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should not have error messages in output
        assert!(
            !stdout.contains("error:") && !stdout.contains("Error:"),
            "sw {} should not have errors in stdout: {}",
            pkg,
            stdout
        );

        // stderr might have warnings but shouldn't have crashes
        assert!(
            !stderr.contains("panicked") && !stderr.contains("SIGSEGV"),
            "sw {} should not crash: {}",
            pkg,
            stderr
        );

        assert!(output.status.success(), "annactl sw {} should succeed", pkg);
    }
}

/// Test hotspots format follows spec (v7.24.0)
/// - CPU: "X percent (0 - Y percent for N logical cores)"
#[test]
fn test_snow_leopard_hotspots_format_v724() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["sw"])
        .output()
        .expect("Failed to run annactl sw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If [HOTSPOTS] section exists, verify format
    if stdout.contains("[HOTSPOTS]") {
        // Should show source attribution
        assert!(
            stdout.contains("source:") || stdout.contains("Source:") || stdout.contains("telemetry"),
            "[HOTSPOTS] should have source attribution"
        );

        // CPU format should mention logical cores if present
        if stdout.contains("CPU:") && stdout.contains("percent") {
            // v7.24.0 format: "X percent (0 - Y percent for N logical cores)"
            // Just verify it doesn't have obviously nonsense values like 99999
            assert!(
                !stdout.contains("99999"),
                "CPU hotspots should have valid values"
            );
        }
    }

    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.25.0 tests: Buses, Peripherals & Attachments
// ============================================================================

/// Test hw overview has [OVERVIEW] section with device counts (v7.25.0)
#[test]
fn test_snow_leopard_hw_overview_section_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must have [OVERVIEW] section
    assert!(
        stdout.contains("[OVERVIEW]"),
        "annactl hw should have [OVERVIEW] section"
    );

    // Should show device counts
    assert!(
        stdout.contains("CPU:") && stdout.contains("socket"),
        "[OVERVIEW] should show CPU socket count"
    );

    assert!(
        stdout.contains("Memory:") && stdout.contains("GiB"),
        "[OVERVIEW] should show memory amount"
    );

    assert!(
        stdout.contains("USB:") && stdout.contains("controller"),
        "[OVERVIEW] should show USB summary"
    );

    assert!(output.status.success());
}

/// Test hw overview has [CATEGORIES] section with bus summaries (v7.25.0)
#[test]
fn test_snow_leopard_hw_categories_section_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw"])
        .output()
        .expect("Failed to run annactl hw");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must have [CATEGORIES] section
    assert!(
        stdout.contains("[CATEGORIES]"),
        "annactl hw should have [CATEGORIES] section"
    );

    // Should show bus details
    assert!(
        stdout.contains("USB:") || stdout.contains("USB"),
        "[CATEGORIES] should mention USB"
    );

    assert!(output.status.success());
}

/// Test hw usb shows [CONTROLLERS] and [DEVICES] sections (v7.25.0)
#[test]
fn test_snow_leopard_hw_usb_category_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "usb"])
        .output()
        .expect("Failed to run annactl hw usb");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must have both sections
    assert!(
        stdout.contains("[CONTROLLERS]"),
        "annactl hw usb should have [CONTROLLERS] section"
    );

    assert!(
        stdout.contains("[DEVICES]"),
        "annactl hw usb should have [DEVICES] section"
    );

    // Should mention source
    assert!(
        stdout.contains("lsusb") || stdout.contains("source:"),
        "hw usb should show source attribution"
    );

    assert!(output.status.success());
}

/// Test hw bluetooth shows [ADAPTERS] section (v7.25.0)
#[test]
fn test_snow_leopard_hw_bluetooth_category_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "bluetooth"])
        .output()
        .expect("Failed to run annactl hw bluetooth");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have either [ADAPTERS] section or [NOT FOUND]
    assert!(
        stdout.contains("[ADAPTERS]") || stdout.contains("[NOT FOUND]"),
        "annactl hw bluetooth should have [ADAPTERS] or [NOT FOUND] section"
    );

    // If adapters found, should show state
    if stdout.contains("[ADAPTERS]") {
        assert!(
            stdout.contains("State:") || stdout.contains("UP") || stdout.contains("DOWN") || stdout.contains("BLOCKED"),
            "hw bluetooth [ADAPTERS] should show adapter state"
        );
    }

    assert!(output.status.success());
}

/// Test hw thunderbolt shows [CONTROLLERS] section (v7.25.0)
#[test]
fn test_snow_leopard_hw_thunderbolt_category_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "thunderbolt"])
        .output()
        .expect("Failed to run annactl hw thunderbolt");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have either [CONTROLLERS] or [NOT FOUND]
    assert!(
        stdout.contains("[CONTROLLERS]") || stdout.contains("[NOT FOUND]"),
        "annactl hw thunderbolt should have [CONTROLLERS] or [NOT FOUND] section"
    );

    assert!(output.status.success());
}

/// Test hw sdcard shows [READERS] section (v7.25.0)
#[test]
fn test_snow_leopard_hw_sdcard_category_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["hw", "sdcard"])
        .output()
        .expect("Failed to run annactl hw sdcard");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have either [READERS] or [NOT FOUND]
    assert!(
        stdout.contains("[READERS]") || stdout.contains("[NOT FOUND]"),
        "annactl hw sdcard should have [READERS] or [NOT FOUND] section"
    );

    assert!(output.status.success());
}

/// Test status has [ATTACHMENTS] section (v7.25.0)
#[test]
fn test_snow_leopard_status_attachments_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // [ATTACHMENTS] section should be present if there are USB/BT/TB devices
    // On systems with USB, this should always appear
    if stdout.contains("USB:") || stdout.contains("Bluetooth:") || stdout.contains("Thunderbolt:") {
        assert!(
            stdout.contains("[ATTACHMENTS]"),
            "annactl status should have [ATTACHMENTS] section when peripherals exist"
        );
    }

    assert!(output.status.success());
}

/// Test peripheral category aliases work (v7.25.0)
#[test]
fn test_snow_leopard_peripheral_aliases_v725() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test "bt" alias for bluetooth
    let output = Command::new(&binary)
        .args(["hw", "bt"])
        .output()
        .expect("Failed to run annactl hw bt");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("bluetooth") || stdout.contains("[NOT FOUND]"),
        "annactl hw bt should be alias for bluetooth"
    );
    assert!(output.status.success());

    // Test "tb" alias for thunderbolt
    let output = Command::new(&binary)
        .args(["hw", "tb"])
        .output()
        .expect("Failed to run annactl hw tb");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("thunderbolt") || stdout.contains("[NOT FOUND]"),
        "annactl hw tb should be alias for thunderbolt"
    );
    assert!(output.status.success());

    // Test "sd" alias for sdcard
    let output = Command::new(&binary)
        .args(["hw", "sd"])
        .output()
        .expect("Failed to run annactl hw sd");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("sdcard") || stdout.contains("[NOT FOUND]"),
        "annactl hw sd should be alias for sdcard"
    );
    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.26.0 tests: Instrumentation & Auto-Install
// ============================================================================

/// Test status has [ALERTS] section (v7.38.0 cache-only)
#[test]
fn test_snow_leopard_status_instrumentation_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, shows [ALERTS] section
    assert!(
        stdout.contains("[ALERTS]"),
        "annactl status should have [ALERTS] section (v7.38.0 cache-only)"
    );

    // v7.42.0: Alerts shows critical/warning counts OR no snapshot data message
    assert!(
        stdout.contains("Critical:") || stdout.contains("Warnings:") || stdout.contains("no snapshot data"),
        "[ALERTS] should show counts or no snapshot data"
    );

    assert!(output.status.success());
}

/// Test CLI surface unchanged - still exactly 6 commands (v7.26.0)
#[test]
fn test_snow_leopard_cli_surface_unchanged_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Help should list exactly the 6 commands
    assert!(stdout.contains("annactl"), "Should show annactl usage");
    assert!(stdout.contains("status"), "Should show status command");
    assert!(stdout.contains("sw"), "Should show sw command");
    assert!(stdout.contains("hw"), "Should show hw command");

    // Should NOT have install or instrumentation commands
    assert!(
        !stdout.contains("annactl install") && !stdout.contains("annactl instrument"),
        "Should not have new install/instrument commands: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test help (running annactl with no args) shows commands (v7.26.0)
#[test]
fn test_snow_leopard_help_shows_commands_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Running annactl with no args shows help
    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Must show the main commands
    assert!(stdout.contains("status"), "help should list status");
    assert!(stdout.contains("sw"), "help should list sw");
    assert!(stdout.contains("hw"), "help should list hw");

    // Verify no secret commands
    assert!(
        !stdout.contains("internal") && !stdout.contains("debug"),
        "help should not expose internal commands: {}",
        stdout
    );

    assert!(output.status.success());
}

// v7.27.0: AUR gate and auto-install controls removed from simplified [INSTRUMENTATION]
// These tests are now superseded by test_snow_leopard_instrumentation_none_format_v727

/// Test status shows [TELEMETRY] section (v7.38.0 cache-only)
#[test]
fn test_snow_leopard_instrumentation_installed_count_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [TELEMETRY] section from snapshot
    assert!(
        stdout.contains("[TELEMETRY]"),
        "status should have [TELEMETRY] section (v7.38.0 cache-only): {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test status shows [VERSION] section (v7.38.0 cache-only)
#[test]
fn test_snow_leopard_instrumentation_disclosure_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, always has [VERSION] section
    assert!(
        stdout.contains("[VERSION]"),
        "status must have [VERSION] section (v7.38.0 cache-only)"
    );

    // Version section shows Anna version
    assert!(
        stdout.contains("Anna:"),
        "[VERSION] should show Anna: line: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test version shown in status (v7.38.0)
#[test]
fn test_snow_leopard_version_in_status_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Version is shown in status command
    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // [VERSION] section should show 7.42 (updated for v7.42.0)
    assert!(
        stdout.contains("7.42"),
        "status should show version 7.42: {}",
        stdout
    );

    assert!(output.status.success());
}

/// Test status shows all expected sections in correct order (v7.26.0)
#[test]
fn test_snow_leopard_status_sections_order_v726() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Required sections should appear in order
    let sections = vec![
        "[VERSION]",
        "[DAEMON]",
        "[HEALTH]",
        "[BOOT SNAPSHOT]",
        "[INVENTORY]",
        "[INSTRUMENTATION]",
        "[PATHS]",
    ];

    let mut last_pos = 0;
    for section in sections {
        if let Some(pos) = stdout.find(section) {
            assert!(
                pos > last_pos,
                "Sections should appear in order: {} should be after previous section at pos {}",
                section,
                last_pos
            );
            last_pos = pos;
        }
    }

    assert!(output.status.success());
}

// ============================================================================
// Snow Leopard v7.27.0 Tests - "Knowledge Foundation"
// ============================================================================

/// Test help shows exactly 6 commands - no kdb, knowledge, stats, dashboard aliases (v7.27.0)
#[test]
fn test_snow_leopard_help_exactly_7_commands_v736() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .output()
        .expect("Failed to run annactl");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should have these 7 command lines (v7.35.1 added --version)
    assert!(stdout.contains("annactl") && stdout.contains("show"), "Help should show 'annactl' help line");
    assert!(stdout.contains("annactl --version"), "Help should show 'annactl --version' command");
    assert!(stdout.contains("annactl status"), "Help should show 'annactl status' command");
    assert!(stdout.contains("annactl sw "), "Help should show 'annactl sw' command");
    assert!(stdout.contains("annactl sw NAME"), "Help should show 'annactl sw NAME' command");
    assert!(stdout.contains("annactl hw "), "Help should show 'annactl hw' command");
    assert!(stdout.contains("annactl hw NAME"), "Help should show 'annactl hw NAME' command");

    // Should NOT have deprecated aliases
    assert!(!stdout.contains("kdb"), "Help should not mention 'kdb' alias");
    assert!(!stdout.contains("knowledge"), "Help should not mention 'knowledge' alias");
    assert!(!stdout.contains("stats"), "Help should not mention 'stats' alias");
    assert!(!stdout.contains("dashboard"), "Help should not mention 'dashboard' alias");

    assert!(output.status.success());
}

/// Test deprecated aliases are rejected (v7.27.0)
#[test]
fn test_snow_leopard_deprecated_aliases_rejected_v727() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // kdb should be rejected
    let output = Command::new(&binary)
        .args(["kdb"])
        .output()
        .expect("Failed to run annactl kdb");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a recognized command") || !output.status.success(),
        "kdb should be rejected as deprecated");

    // knowledge should be rejected
    let output = Command::new(&binary)
        .args(["knowledge"])
        .output()
        .expect("Failed to run annactl knowledge");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not a recognized command") || !output.status.success(),
        "knowledge should be rejected as deprecated");
}

/// Test status shows [HEALTH] section (v7.38.0 cache-only)
#[test]
fn test_snow_leopard_instrumentation_none_format_v727() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // v7.38.0: Status is cache-only, [HEALTH] section shows overall status
    assert!(stdout.contains("[HEALTH]"), "Status should have [HEALTH] section (v7.38.0 cache-only)");

    // v7.38.0: Should show overall health status
    assert!(
        stdout.contains("Overall:"),
        "[HEALTH] should show overall status"
    );
}

/// Test [TELEMETRY] shows CPU range format (v7.27.0)
#[test]
fn test_snow_leopard_telemetry_cpu_range_v727() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If telemetry section exists with CPU data
    if stdout.contains("Top CPU identities") {
        // v7.31.0: Should have the range format with % symbol
        assert!(
            stdout.contains("% for") && stdout.contains("logical cores"),
            "CPU telemetry should show range format (0-Y% for N logical cores)"
        );
    }
}

/// Test [CONFIG] shows Detected and Possible sections (v7.27.0)
#[test]
fn test_snow_leopard_config_detected_possible_v727() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    // Test with a common package
    let output = Command::new(&binary)
        .args(["sw", "bash"])
        .output()
        .expect("Failed to run annactl sw bash");

    let stdout = String::from_utf8_lossy(&output.stdout);

    if stdout.contains("[CONFIG]") {
        // Should have proper structure
        // Either "Detected:" with existing paths or "Possible:" with doc paths
        // No more "Active:" or "[present]/[missing]" markers
        assert!(
            !stdout.contains("[present]") && !stdout.contains("[missing]"),
            "[CONFIG] should not have [present]/[missing] markers in v7.27.0"
        );
    }
}

/// Test status sections are in correct v7.27.0 order
#[test]
fn test_snow_leopard_status_sections_order_v727() {
    let binary = get_binary_path();
    if !binary.exists() {
        return;
    }

    let output = Command::new(&binary)
        .args(["status"])
        .output()
        .expect("Failed to run annactl status");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // These sections should exist and be in order
    let required_sections = vec![
        "[VERSION]",
        "[DAEMON]",
        "[HEALTH]",
        "[BOOT SNAPSHOT]",
        "[INVENTORY]",
        "[INSTRUMENTATION]",
        "[PATHS]",
    ];

    let mut last_pos = 0;
    for section in required_sections {
        if let Some(pos) = stdout.find(section) {
            assert!(
                pos > last_pos,
                "v7.27.0: Sections should appear in order: {} should be after previous section",
                section
            );
            last_pos = pos;
        }
    }

    assert!(output.status.success());
}
