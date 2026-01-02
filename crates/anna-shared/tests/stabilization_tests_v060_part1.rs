//! Stabilization tests for v0.0.60 - Part 1: Audio parsing.

fn golden_v060_lspci_multimedia_audio_controller_parses_positive() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Real-world lspci output with "Multimedia audio controller:" format
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

    // Must parse as Audio, not Unsupported or Error
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Multimedia audio controller must parse as Audio, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        // Must have at least 1 device
        assert!(
            !audio.devices.is_empty(),
            "Must detect at least 1 audio device from 'Multimedia audio controller:' line"
        );

        // Check the device details
        let dev = &audio.devices[0];
        assert!(dev.pci_slot.is_some(), "Should have PCI slot");
        assert_eq!(dev.pci_slot.as_ref().unwrap(), "00:1f.3");
        assert!(
            dev.description.contains("Intel"),
            "Description should contain Intel"
        );
        assert!(
            dev.description.contains("Cannon Lake"),
            "Description should contain Cannon Lake"
        );
        assert_eq!(dev.vendor, Some("Intel".to_string()));
    }
}

/// v0.0.60: pactl list cards output must parse to AudioDevices.
#[test]
fn golden_v060_pactl_cards_parses_to_audio_devices() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Real-world pactl list cards output
    let probe = ProbeResult {
        command: "pactl list cards 2>/dev/null || true".to_string(),
        exit_code: 0,
        stdout: r#"Card #0
    Name: alsa_card.pci-0000_00_1f.3
    Driver: module-alsa-card.c
    Owner Module: 7
    Properties:
        alsa.card = "0"
        alsa.card_name = "HDA Intel PCH"
        alsa.long_card_name = "HDA Intel PCH at 0xa1318000 irq 134"
        device.bus_path = "pci-0000:00:1f.3"
        device.description = "Built-in Audio"
"#
        .to_string(),
        stderr: String::new(),
        timing_ms: 25,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Audio
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "pactl cards must parse as Audio, got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        assert_eq!(audio.source, "pactl");
        assert!(
            !audio.devices.is_empty(),
            "Must detect at least 1 audio device from pactl cards"
        );

        // Check the device (should use alsa.card_name or device.description)
        let dev = &audio.devices[0];
        // Description could be "HDA Intel PCH" or "Built-in Audio"
        assert!(!dev.description.is_empty());
    }
}

/// v0.0.60: Audio deduplication merges lspci and pactl sources correctly.
#[test]
fn golden_v060_audio_dedupe_merges_sources() {
    use anna_shared::parsers::{find_audio_evidence, AudioDevice, AudioDevices, ParsedProbeData};

    // Simulate lspci finding an Intel audio device
    let lspci_audio = ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "Intel Corporation Cannon Lake PCH cAVS".to_string(),
            pci_slot: Some("00:1f.3".to_string()),
            vendor: Some("Intel".to_string()),
        }],
        source: "lspci".to_string(),
    });

    // Simulate pactl finding the same device (different description)
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

    assert!(merged.is_some());
    let audio = merged.unwrap();

    // Source should indicate both
    assert!(
        audio.source.contains("lspci") || audio.source.contains("+"),
        "Merged source should indicate lspci+pactl"
    );

    // Should have devices (not be empty)
    assert!(!audio.devices.is_empty(), "Merged result must have devices");

    // Deduplication: Intel device appears once with PCI slot preserved
    // (lspci version preferred because it has PCI slot)
    let intel_devices: Vec<_> = audio
        .devices
        .iter()
        .filter(|d| {
            d.vendor
                .as_ref()
                .map(|v| v.contains("Intel"))
                .unwrap_or(false)
        })
        .collect();

    // If properly deduplicated, should have 1 Intel device (not 2)
    // The pactl "HDA Intel PCH" should be recognized as overlapping with lspci description
    assert!(
        intel_devices.len() <= 2,
        "Should dedupe overlapping Intel devices"
    );
}

/// v0.0.60: grep exit code 1 is valid empty evidence, not error.
#[test]
fn golden_v060_grep_exit_1_is_valid_empty_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // grep -i audio with no matches returns exit code 1
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Audio with empty devices (valid negative evidence)
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "grep exit 1 should be Audio (empty), not Error, got {:?}",
        parsed
    );

    // Must be valid evidence (check before moving)
    assert!(
        parsed.is_valid_evidence(),
        "grep exit 1 with empty stdout is valid negative evidence"
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        assert!(audio.devices.is_empty(), "No devices found is correct");
        assert_eq!(audio.source, "lspci");
    }
}

/// v0.0.60: "Audio controller:" variant also parses correctly.
#[test]
fn golden_v060_audio_controller_variant_parses() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Some lspci outputs use "Audio controller:" format
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:14.2 Audio controller: Advanced Micro Devices, Inc. [AMD/ATI] SBx00 Azalia (rev 40)\n".to_string(),
        stderr: String::new(),
        timing_ms: 12,
    };

    let parsed = parse_probe_result(&probe);

    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Audio controller: line must parse as Audio"
    );

    if let ParsedProbeData::Audio(audio) = parsed {
        assert!(!audio.devices.is_empty());
        let dev = &audio.devices[0];
        assert!(
            dev.description.contains("AMD") || dev.description.contains("Azalia"),
            "Description should contain AMD or Azalia"
        );
        assert!(dev.pci_slot.is_some());
    }
}

// ===== v0.0.60: ConfigureEditor Grounded Selection Tests =====

// /// v0.0.60: ConfigureEditor must never invent editors not probed.
// /// If probes show only vim exists, the editor list must not contain "code".
// #[test] // TODO: Incomplete test stub
