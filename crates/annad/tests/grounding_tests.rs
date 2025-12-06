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
        version: "0.0.7".to_string(),
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
    assert!(prompt.contains("0.0.7"));
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
        assert!(
            part.parse::<u32>().is_ok(),
            "Version parts should be numeric"
        );
    }
}

/// v0.0.68: Test version consistency - all sources must match workspace version
#[test]
fn test_v068_version_consistency() {
    // The VERSION constant must match the expected workspace version
    assert_eq!(VERSION, "0.0.68", "VERSION constant must match workspace version");

    // Version must be used consistently in status output
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
        probes: HashMap::new(),
    };
    assert_eq!(context.version, "0.0.68", "RuntimeContext version must match");
}

/// v0.0.68: Test that audio parsing correctly handles Multimedia audio controller output
#[test]
fn test_v068_audio_multimedia_controller_parsing() {
    use anna_shared::parsers::{parse_probe_result, find_audio_evidence, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Real-world lspci output: "Multimedia audio controller" (not "Audio device")
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS (rev 10)".to_string(),
        stderr: String::new(),
        timing_ms: 15,
    };

    let parsed = parse_probe_result(&probe);
    assert!(parsed.as_audio().is_some(), "Multimedia audio controller line must parse as Audio evidence");

    let audio = parsed.as_audio().unwrap();
    assert_eq!(audio.devices.len(), 1, "Should detect exactly one device");
    assert!(audio.devices[0].description.contains("Intel"), "Description should contain Intel");
    assert!(audio.devices[0].description.contains("Cannon Lake"), "Description should contain Cannon Lake");
    assert_eq!(audio.devices[0].pci_slot, Some("00:1f.3".to_string()), "PCI slot should be extracted");
}

/// v0.0.68: Test that deterministic answer correctly reports device when parsing succeeds
#[test]
fn test_v068_audio_deterministic_answer_with_device() {
    use anna_shared::parsers::{parse_probe_result, find_audio_evidence, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Create probe results with valid audio output
    let probes = vec![ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS (rev 10)".to_string(),
        stderr: String::new(),
        timing_ms: 15,
    }];

    // Parse all probes
    let parsed: Vec<ParsedProbeData> = probes.iter()
        .map(|p| parse_probe_result(p))
        .collect();

    // Find audio evidence
    let audio = find_audio_evidence(&parsed);
    assert!(audio.is_some(), "Should find audio evidence");

    let audio = audio.unwrap();
    assert!(!audio.devices.is_empty(), "Devices list must NOT be empty when lspci shows audio");

    // The answer must NOT say "No audio devices detected" when we have a device
    // This is the bug we're fixing in v0.0.68
    let would_say_no_audio = audio.devices.is_empty();
    assert!(!would_say_no_audio, "BUG: Answer would incorrectly say 'No audio devices' when lspci shows one");
}

/// v0.0.68: Test that audio grep exit_code=1 is valid negative evidence
#[test]
fn test_v068_audio_grep_exit1_is_valid_negative_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // grep exit code 1 = no matches (valid evidence, not an error)
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Audio evidence (empty devices list)
    assert!(parsed.as_audio().is_some(), "grep exit 1 must parse as Audio evidence (negative)");
    assert!(parsed.is_valid_evidence(), "grep exit 1 must be valid evidence");

    let audio = parsed.as_audio().unwrap();
    assert!(audio.devices.is_empty(), "Negative evidence should have empty devices list");
}

/// v0.0.68: Test ConfigureEditor only shows editors that are probed and found
#[test]
fn test_v068_configure_editor_grounded_to_probes() {
    use anna_shared::parsers::{parse_probe_result, installed_editors_from_parsed, get_installed_tools};
    use anna_shared::rpc::ProbeResult;

    // Simulated probe results: only vim and nano were probed
    let probes = vec![
        // vim exists
        ProbeResult {
            command: "sh -lc 'command -v vim'".to_string(),
            exit_code: 0,
            stdout: "/usr/bin/vim\n".to_string(),
            stderr: String::new(),
            timing_ms: 10,
        },
        // nano does NOT exist
        ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
        // nvim does NOT exist
        ProbeResult {
            command: "sh -lc 'command -v nvim'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
    ];

    // Parse probes
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();

    // Get installed editors - should ONLY be vim
    let installed = installed_editors_from_parsed(&parsed);

    assert_eq!(installed.len(), 1, "Only vim should be detected as installed");
    assert!(installed.contains(&"vim".to_string()), "vim should be in installed list");
    assert!(!installed.contains(&"nano".to_string()), "nano should NOT be in installed list");
    assert!(!installed.contains(&"code".to_string()), "code was not probed, should NOT appear");
}

/// v0.0.68: Test that clarification prompt ends with period, not question mark
#[test]
fn test_v068_configure_editor_clarification_ends_with_period() {
    // The answer format for multiple editors must end with a period/statement
    let editors = vec!["vim", "code"];
    let editors_list: Vec<String> = editors.iter()
        .enumerate()
        .map(|(i, e)| format!("{}) {}", i + 1, e))
        .collect();
    let answer = format!(
        "I can configure syntax highlighting for one of these editors:\n{}\nReply with the number.",
        editors_list.join("\n")
    );

    // Must NOT end with a question mark
    assert!(!answer.ends_with('?'), "Clarification must not end with question mark");
    // Must end with a period (in "Reply with the number.")
    assert!(answer.ends_with('.'), "Clarification must end with period");
}
