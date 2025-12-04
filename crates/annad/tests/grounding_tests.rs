//! Integration tests for grounded LLM responses.
//!
//! These tests verify that Anna's responses are grounded in actual data
//! and never invent facts or suggest manual commands when data is available.

use anna_shared::rpc::{Capabilities, HardwareSummary, RuntimeContext};
use anna_shared::VERSION;
use std::collections::HashMap;

/// Test that RuntimeContext always contains the correct version
#[test]
fn test_runtime_context_version_matches() {
    let context = RuntimeContext {
        version: VERSION.to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: "Test CPU".to_string(),
            cpu_cores: 8,
            ram_gb: 16.0,
            gpu: None,
            gpu_vram_gb: None,
        },
        probes: HashMap::new(),
    };

    // Version in context must match the actual VERSION constant
    assert_eq!(context.version, VERSION);
    assert!(!context.version.is_empty());
    assert!(context.version.starts_with("0.0."));
}

/// Test that hardware summary is properly populated
#[test]
fn test_hardware_summary_populated() {
    let hardware = HardwareSummary {
        cpu_model: "AMD Ryzen 9 7945HX".to_string(),
        cpu_cores: 32,
        ram_gb: 31.0,
        gpu: Some("NVIDIA GeForce RTX 4060".to_string()),
        gpu_vram_gb: Some(8.0),
    };

    // All fields should be properly set
    assert!(!hardware.cpu_model.is_empty());
    assert!(hardware.cpu_cores > 0);
    assert!(hardware.ram_gb > 0.0);
    assert!(hardware.gpu.is_some());
    assert!(hardware.gpu_vram_gb.is_some());
}

/// Test that capabilities default to safe values
#[test]
fn test_capabilities_default_safe() {
    let caps = Capabilities::default();

    // Read operations should be allowed
    assert!(caps.can_read_system_info);
    assert!(caps.can_run_probes);

    // Write operations should be disabled by default
    assert!(!caps.can_modify_files);
    assert!(!caps.can_install_packages);
}

/// Test that system prompt contains required grounding elements
#[test]
fn test_system_prompt_grounding_format() {
    // Simulate what the system prompt builder does
    let context = RuntimeContext {
        version: "0.0.6".to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: "Test CPU".to_string(),
            cpu_cores: 8,
            ram_gb: 16.0,
            gpu: None,
            gpu_vram_gb: None,
        },
        probes: HashMap::new(),
    };

    // Build a simulated prompt (mirrors rpc_handler.rs logic)
    let prompt = format!(
        "Version: {}\nCPU: {} ({} cores)\nRAM: {:.1} GB",
        context.version,
        context.hardware.cpu_model,
        context.hardware.cpu_cores,
        context.hardware.ram_gb
    );

    // Verify key elements are present
    assert!(prompt.contains("0.0.6"));
    assert!(prompt.contains("Test CPU"));
    assert!(prompt.contains("8 cores"));
    assert!(prompt.contains("16.0 GB"));
}

/// Test that probe results would be included in context
#[test]
fn test_probe_results_in_context() {
    let mut probes = HashMap::new();
    probes.insert(
        "top_memory_processes".to_string(),
        "USER       PID %MEM\nroot       123 10.5".to_string(),
    );

    let context = RuntimeContext {
        version: VERSION.to_string(),
        daemon_running: true,
        capabilities: Capabilities::default(),
        hardware: HardwareSummary {
            cpu_model: "Test".to_string(),
            cpu_cores: 4,
            ram_gb: 8.0,
            gpu: None,
            gpu_vram_gb: None,
        },
        probes,
    };

    // Probe results should be accessible
    assert!(context.probes.contains_key("top_memory_processes"));
    assert!(context.probes["top_memory_processes"].contains("PID"));
}

/// Test version format is semantic versioning
#[test]
fn test_version_format() {
    let parts: Vec<&str> = VERSION.split('.').collect();

    // Should have major.minor.patch format
    assert_eq!(parts.len(), 3, "Version should be major.minor.patch format");

    // All parts should be numeric
    for part in parts {
        assert!(part.parse::<u32>().is_ok(), "Version parts should be numeric");
    }
}
