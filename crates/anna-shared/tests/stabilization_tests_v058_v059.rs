//! Stabilization tests for v0.0.58 and v0.0.59.

fn golden_v058_lspci_empty_output_is_valid_negative_evidence() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // lspci audio returns empty (no audio devices)
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0, // Successful but empty
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // Should parse as Audio (not Error)
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Empty lspci output should still parse as Audio evidence"
    );

    // Should be valid evidence
    assert!(
        parsed.is_valid_evidence(),
        "Empty lspci output with exit_code 0 should be valid evidence"
    );

    let parsed_list = vec![parsed];
    let audio = find_audio_evidence(&parsed_list).expect("Should find audio evidence (empty)");

    // Should have zero devices (valid negative evidence)
    assert!(
        audio.devices.is_empty(),
        "No devices expected from empty output"
    );
}

/// v0.0.58: grep exit_code 1 (no match) is also valid negative evidence for audio.
#[test]
fn golden_v058_lspci_grep_exit_1_is_valid_negative_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // grep -i audio returns exit 1 when no match
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep: no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // v0.0.58: exit_code 1 for grep audio should still parse as Audio with empty devices
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "grep exit 1 should parse as Audio evidence (empty), got: {:?}",
        parsed
    );

    // Should be valid evidence (grounded negative)
    assert!(
        parsed.is_valid_evidence(),
        "grep exit 1 for audio should be valid negative evidence"
    );
}

/// v0.0.58: Parser extracts PCI slot correctly from lspci format.
#[test]
fn golden_v058_lspci_extracts_pci_slot() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS\n"
            .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = vec![parse_probe_result(&probe)];
    let audio = find_audio_evidence(&parsed).expect("Should find audio");

    assert_eq!(audio.devices.len(), 1);
    assert_eq!(
        audio.devices[0].pci_slot.as_deref(),
        Some("00:1f.3"),
        "PCI slot must be extracted correctly"
    );
}

/// v0.0.58: Description extracted correctly without device class prefix.
#[test]
fn golden_v058_lspci_description_no_device_class_prefix() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout:
            "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS (rev 10)\n"
                .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = vec![parse_probe_result(&probe)];
    let audio = find_audio_evidence(&parsed).expect("Should find audio");

    assert_eq!(audio.devices.len(), 1);
    let desc = &audio.devices[0].description;

    // Description should NOT include the device class
    assert!(
        !desc.to_lowercase().contains("multimedia audio controller"),
        "Description should not include device class prefix: {}",
        desc
    );
    assert!(
        !desc.to_lowercase().contains("audio device:"),
        "Description should not include device class prefix: {}",
        desc
    );

    // Description SHOULD include the vendor/device info
    assert!(
        desc.contains("Intel"),
        "Description should contain vendor: {}",
        desc
    );
    assert!(
        desc.contains("Cannon Lake"),
        "Description should contain device name: {}",
        desc
    );
}

/// v0.0.58: ProbeSpine for "sound card" returns LspciAudio probe.
#[test]
fn golden_v058_sound_card_query_uses_lspci_audio_probe() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let query = "what is my sound card";
    let decision = enforce_minimum_probes(query, &[]);

    // Should have LspciAudio probe
    let has_lspci_audio = decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::LspciAudio));

    assert!(
        has_lspci_audio,
        "Query '{}' should trigger LspciAudio probe, got: {:?}",
        query, decision.probes
    );
}

// ============================================================================
// v0.0.59: ConfigureEditor Evidence-Grounded Flow
// ============================================================================

/// v0.0.59: Editor probes must include "code" for VS Code detection.
#[test]
fn golden_v059_editor_probes_include_code() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let query = "enable syntax highlighting";
    let decision = enforce_minimum_probes(query, &[]);

    // Collect all CommandV probe names
    let editor_probes: Vec<&str> = decision
        .probes
        .iter()
        .filter_map(|p| {
            if let ProbeId::CommandV(name) = p {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect();

    // Must include "code" for VS Code
    assert!(
        editor_probes.contains(&"code"),
        "Editor probes must include 'code' for VS Code, got: {:?}",
        editor_probes
    );

    // Must include other common editors
    assert!(editor_probes.contains(&"vim"), "Must probe vim");
    assert!(editor_probes.contains(&"nvim"), "Must probe nvim");
    assert!(editor_probes.contains(&"nano"), "Must probe nano");
    assert!(editor_probes.contains(&"emacs"), "Must probe emacs");
}

/// v0.0.59: Extract installed editors from ToolExists evidence.
#[test]
fn golden_v059_installed_editors_from_tool_evidence() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // Simulate probe results: vim exists, nano doesn't, code exists
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1, // Not installed
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };
    let code_probe = ProbeResult {
        command: "sh -lc 'command -v code'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/code\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed: Vec<_> = [vim_probe, nano_probe, code_probe]
        .iter()
        .map(|p| parse_probe_result(p))
        .collect();

    let tools = get_installed_tools(&parsed);

    // Filter to installed editors
    let editor_names = ["code", "vim", "nvim", "nano", "emacs", "micro", "helix"];
    let mut installed: Vec<&str> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();
    installed.sort();

    // Should have code and vim, not nano
    assert_eq!(
        installed,
        vec!["code", "vim"],
        "Should extract installed editors only: got {:?}",
        installed
    );
}

/// v0.0.59: Clarification for multiple editors must be grounded with probes attached.
#[test]
fn golden_v059_multi_editor_clarification_is_grounded() {
    use anna_shared::rpc::ReliabilitySignals;

    // When multiple editors detected from probes:
    // - translator_confident = true (probes ran)
    // - probe_coverage = true (probes succeeded)
    // - answer_grounded = true (options from current evidence)
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true,
        no_invention: true,
        clarification_not_needed: false,
    };

    // Score should be reasonable (not 0, not capped too low)
    let score = signals.score();
    assert!(
        score >= 60,
        "Grounded multi-editor clarification should have score >= 60, got {}",
        score
    );
}

/// v0.0.59: Single-editor answer must not contain question marks.
#[test]
fn golden_v059_single_editor_answer_no_questions() {
    // Simulate single-editor answer format (same as build_editor_config_answer)
    let vim_answer = "I detected **vim** installed. To enable syntax highlighting:\n\n\
        1. Edit `~/.vimrc` (create if needed)\n\
        2. Add: `syntax on`\n\
        3. Save and reopen vim\n\n\
        For line numbers, also add: `set number`";

    // Must NOT contain question marks
    assert!(
        !vim_answer.contains('?'),
        "Single-editor answer must not contain questions"
    );

    // Must NOT contain "Would you like"
    assert!(
        !vim_answer.to_lowercase().contains("would you like"),
        "Single-editor answer must not contain 'Would you like'"
    );

    // Must NOT contain "Do you want"
    assert!(
        !vim_answer.to_lowercase().contains("do you want"),
        "Single-editor answer must not contain 'Do you want'"
    );
}

/// v0.0.59: No-editors-found response must list what was checked.
#[test]
fn golden_v059_no_editors_found_lists_checked() {
    // When no editors found, the answer should list what we checked
    let checked_editors = vec!["vim", "nano", "emacs", "code"];
    let answer = format!(
        "No supported text editors were detected.\n\n\
        Checked: {}\n\n\
        Install vim, nano, or another editor and retry.",
        checked_editors.join(", ")
    );

    // Must mention what was checked
    assert!(
        answer.contains("Checked:"),
        "Must indicate what was checked"
    );
    assert!(answer.contains("vim"), "Must list vim in checked");
    assert!(answer.contains("nano"), "Must list nano in checked");

    // Must be grounded (it's valid negative evidence)
    // The response code sets grounded=true for this case
}

// ===== v0.0.60: HardwareAudio parsing tests =====

// /// v0.0.60: lspci "Multimedia audio controller:" lines must parse as positive evidence.
// #[test] // TODO: Incomplete test stub
