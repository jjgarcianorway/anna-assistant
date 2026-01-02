//! Stabilization tests for v0.0.61.

fn golden_v061_lspci_audio_detected_by_output_content() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Simulate a probe where command string is unusual but output is clear lspci audio
    let probe = ProbeResult {
        command: "lspci -nn 2>/dev/null".to_string(),  // No "audio" in command
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS [8086:a348] (rev 10)\n".to_string(),
        stderr: String::new(),
        timing_ms: 20,
    };

    let parsed = parse_probe_result(&probe);

    // v0.0.61: Should detect audio device by output content, not just command
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Should parse as Audio when output contains 'Multimedia audio controller:', got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        assert!(
            !audio.devices.is_empty(),
            "Should have at least 1 audio device"
        );
        assert_eq!(audio.source, "lspci");
    }
}

/// v0.0.61: When lspci has devices and pactl is empty, result has devices.
/// This is the core fix for "No audio devices detected" false negative.
#[test]
fn golden_v061_answer_hardware_audio_prefers_positive_lspci() {
    use anna_shared::parsers::{find_audio_evidence, AudioDevice, AudioDevices, ParsedProbeData};

    // lspci found an audio device
    let lspci_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "Intel Corporation Cannon Lake PCH cAVS".to_string(),
            pci_slot: Some("00:1f.3".to_string()),
            vendor: Some("Intel".to_string()),
        }],
        source: "lspci".to_string(),
    });

    // pactl returned empty (no PulseAudio, or no cards)
    let pactl_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![],
        source: "pactl".to_string(),
    });

    let parsed = vec![lspci_audio, pactl_audio];
    let merged = find_audio_evidence(&parsed);

    // CRITICAL: Result must have devices, NOT be empty
    assert!(merged.is_some(), "Should find audio evidence");
    let audio = merged.unwrap();

    assert!(
        !audio.devices.is_empty(),
        "Result must have devices when lspci has devices, even if pactl is empty"
    );
    assert!(
        audio
            .devices
            .iter()
            .any(|d| d.description.contains("Intel")),
        "Should include the Intel device from lspci"
    );
}

/// v0.0.61: When pactl has devices and lspci is empty, result has devices.
#[test]
fn golden_v061_answer_hardware_audio_prefers_positive_pactl() {
    use anna_shared::parsers::{find_audio_evidence, AudioDevice, AudioDevices, ParsedProbeData};

    // lspci found nothing
    let lspci_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![],
        source: "lspci".to_string(),
    });

    // pactl found a card
    let pactl_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "HDA Intel PCH".to_string(),
            pci_slot: None,
            vendor: Some("Intel".to_string()),
        }],
        source: "pactl".to_string(),
    });

    let parsed = vec![lspci_audio, pactl_audio];
    let merged = find_audio_evidence(&parsed);

    // Result must have devices from pactl
    assert!(merged.is_some(), "Should find audio evidence");
    let audio = merged.unwrap();

    assert!(
        !audio.devices.is_empty(),
        "Result must have devices when pactl has devices, even if lspci is empty"
    );
}

/// v0.0.61: Detect pactl cards by output content (Card # blocks).
#[test]
fn golden_v061_pactl_detected_by_output_content() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Command doesn't have "cards" in name but output is pactl cards output
    let probe = ProbeResult {
        command: "pactl list".to_string(), // No "cards" in command
        exit_code: 0,
        stdout: r#"Card #0
    Name: alsa_card.pci-0000_00_1f.3
    Driver: module-alsa-card.c
    alsa.card_name = "HDA Intel PCH"
"#
        .to_string(),
        stderr: String::new(),
        timing_ms: 30,
    };

    let parsed = parse_probe_result(&probe);

    // v0.0.61: Should detect pactl cards by "Card #" in output
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Should parse as Audio when output contains 'Card #', got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        assert_eq!(audio.source, "pactl");
    }
}

/// v0.0.61: "No audio devices" only when BOTH sources are truly empty.
#[test]
fn golden_v061_no_audio_only_when_both_empty() {
    use anna_shared::parsers::{find_audio_evidence, AudioDevices, ParsedProbeData};

    // Both lspci and pactl return empty
    let lspci_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![],
        source: "lspci".to_string(),
    });

    let pactl_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![],
        source: "pactl".to_string(),
    });

    let parsed = vec![lspci_audio, pactl_audio];
    let merged = find_audio_evidence(&parsed);

    // ONLY then should result be empty
    assert!(merged.is_some(), "Should return audio evidence (empty)");
    let audio = merged.unwrap();

    assert!(
        audio.devices.is_empty(),
        "Should have no devices when both sources are empty"
    );
    assert!(
        audio.source.contains("+"),
        "Source should indicate both were checked"
    );
}

// ===== v0.0.62: ConfigureEditor Probe Accounting Tests =====

// /// v0.0.62: Extract installed editors from ToolExists evidence.
// /// Verifies that installed_editors_from_parsed correctly extracts only installed editors.
// #[test] // TODO: Incomplete test stub
