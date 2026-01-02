//! Stabilization tests for v0.45.8.

// === v0.45.8 Golden Tests: Audio Evidence + Editor Config Flow ===

/// v0.45.8: lspci audio output parses to AudioDevices variant.
#[test]
fn golden_v458_lspci_audio_parses_to_audio_devices() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel Corporation Sunrise Point-LP HD Audio (rev 21)\n"
            .to_string(),
        stderr: String::new(),
        timing_ms: 15,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Audio variant (Bug A fix)
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "lspci audio output must parse as Audio variant, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(ref audio) = parsed {
        assert!(
            !audio.devices.is_empty(),
            "Must have at least one audio device"
        );
        assert!(
            audio.devices[0].description.contains("Audio"),
            "Device description must contain 'Audio'"
        );
        assert_eq!(audio.source, "lspci");
    }
}

/// v0.45.8: lspci audio with no output (exit_code=0) is valid empty evidence.
#[test]
fn golden_v458_lspci_audio_empty_is_valid_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep returns 1 when no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // Must still parse as Audio variant with empty devices (valid negative evidence)
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "lspci audio with no output must parse as Audio variant, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(ref audio) = parsed {
        assert!(audio.devices.is_empty(), "Must have empty devices list");
    }

    assert!(
        parsed.is_valid_evidence(),
        "Empty audio output is valid evidence"
    );
}

/// v0.45.8: find_audio_evidence helper works correctly.
#[test]
fn golden_v458_find_audio_evidence() {
    use anna_shared::parsers::{find_audio_evidence, AudioDevice, AudioDevices, ParsedProbeData};

    let parsed = vec![ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "Intel Corporation HD Audio".to_string(),
            pci_slot: Some("00:1f.3".to_string()),
            vendor: Some("Intel".to_string()),
        }],
        source: "lspci".to_string(),
    })];

    let audio = find_audio_evidence(&parsed);
    assert!(audio.is_some(), "Must find audio evidence");
    let audio = audio.unwrap();
    assert_eq!(audio.devices.len(), 1);
    assert!(audio.devices[0].description.contains("Intel"));
}

/// v0.45.8: HardwareAudio route is now deterministically answerable.
#[test]
fn golden_v458_hardware_audio_is_deterministic() {
    // The route configuration for HardwareAudio should have can_answer_deterministically = true
    // This is a compile-time/integration check - the actual route is in annad crate
    // Here we verify the probe spine still enforces audio probes
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("what is my sound card", &[]);
    assert!(decision.enforced, "Audio query must enforce probes");
    assert!(
        decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::LspciAudio)),
        "Audio query must include LspciAudio probe"
    );
}

/// v0.45.8: ConfigureEditor route uses clarification flow, never goes to specialist.
/// (Integration behavior tested in annad - here we verify the pattern classification)
#[test]
fn golden_v458_configure_editor_classifies_correctly() {
    use anna_shared::probe_spine::enforce_minimum_probes;

    // "enable syntax highlighting" should trigger editor config flow
    let decision = enforce_minimum_probes("enable syntax highlighting", &[]);
    assert!(decision.enforced, "Editor config must enforce probes");

    // Should have editor tool probes
    let decision2 = enforce_minimum_probes("enable line numbers", &[]);
    assert!(
        decision2.enforced,
        "Editor config variant must also enforce probes"
    );
}

/// v0.45.8: lspci with "Multimedia audio controller" parses correctly.
#[test]
fn golden_v458_lspci_multimedia_audio_controller() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout:
            "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS (rev 10)\n"
                .to_string(),
        stderr: String::new(),
        timing_ms: 15,
    };

    let parsed = parse_probe_result(&probe);
    println!("Parsed: {:?}", parsed);

    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Multimedia audio controller must parse as Audio variant, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(ref audio) = parsed {
        println!("Devices: {:?}", audio.devices);
        assert!(
            !audio.devices.is_empty(),
            "Must have at least one audio device"
        );
        assert!(
            audio.devices[0].description.contains("Intel"),
            "Device description must contain Intel, got: {}",
            audio.devices[0].description
        );
    }
}

/// v0.0.56: grep exit 1 (no match) produces VALID EMPTY audio evidence, not error.
#[test]
fn golden_v056_grep_exit1_is_valid_empty_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep exit 1 = no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed = parse_probe_result(&probe);

    // Should parse as Audio variant with empty devices (valid negative evidence)
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "grep exit 1 must parse as Audio variant with empty devices, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(ref audio) = parsed {
        assert!(audio.devices.is_empty(), "Must have empty devices list");
    }

    assert!(
        parsed.is_valid_evidence(),
        "grep exit 1 (no match) is valid evidence of no audio devices"
    );
}
