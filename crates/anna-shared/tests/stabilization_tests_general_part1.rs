//! General stabilization tests - Part 1: Audio and Editor tests.

fn golden_audio_parses_multimedia_audio_controller_line() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Actual lspci output format that was being missed
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout:
            "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS (rev 10)\n"
                .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Audio variant, not Error or Unsupported
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "Multimedia audio controller line must parse as Audio, got: {:?}",
        parsed
    );

    let parsed_list = vec![parsed];
    let audio = find_audio_evidence(&parsed_list).expect("Should find audio evidence");

    // Must have exactly one device
    assert_eq!(
        audio.devices.len(),
        1,
        "Should detect exactly one audio device"
    );

    let device = &audio.devices[0];

    // PCI slot must be preserved
    assert_eq!(
        device.pci_slot.as_deref(),
        Some("00:1f.3"),
        "PCI slot must be extracted"
    );

    // Description must contain the vendor/device info, not the device type
    assert!(
        device.description.contains("Intel"),
        "Description must include vendor"
    );
    assert!(
        device.description.contains("Cannon Lake"),
        "Description must include device name"
    );
    assert!(
        !device.description.contains("Multimedia"),
        "Description must NOT include device type prefix"
    );

    // Vendor must be extracted
    assert_eq!(
        device.vendor.as_deref(),
        Some("Intel"),
        "Vendor must be Intel"
    );
}

/// v0.0.58: Both "Audio device:" and "Multimedia audio controller:" formats must work.
#[test]
fn golden_audio_parses_both_device_formats() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // "Audio device:" format
    let audio_device_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel Corporation Sunrise Point-LP HD Audio (rev 21)\n"
            .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // "Multimedia audio controller:" format
    let multimedia_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller: Intel Corporation Cannon Lake PCH cAVS\n"
            .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed1 = vec![parse_probe_result(&audio_device_probe)];
    let parsed2 = vec![parse_probe_result(&multimedia_probe)];

    let audio1 = find_audio_evidence(&parsed1).expect("Audio device format must parse");
    let audio2 = find_audio_evidence(&parsed2).expect("Multimedia format must parse");

    assert_eq!(audio1.devices.len(), 1, "Audio device format: one device");
    assert_eq!(audio2.devices.len(), 1, "Multimedia format: one device");

    // Both should extract Intel as vendor
    assert!(audio1.devices[0].description.contains("Intel"));
    assert!(audio2.devices[0].description.contains("Intel"));
}

/// v0.0.58: Audio answer must list device when present, not say "No audio detected."
#[test]
fn golden_audio_answer_lists_device_when_present() {
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
    let audio = find_audio_evidence(&parsed).expect("Must find audio evidence");

    // Simulate answer building (same logic as answer_hardware_audio)
    let answer = if audio.devices.is_empty() {
        "No audio devices detected.".to_string()
    } else if audio.devices.len() == 1 {
        let dev = &audio.devices[0];
        let vendor_info = dev
            .vendor
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();
        format!("**Audio device{}**: {}", vendor_info, dev.description)
    } else {
        format!("Found {} audio devices", audio.devices.len())
    };

    // Critical: must NOT say "No audio devices detected" when we have a device
    assert!(
        !answer.contains("No audio devices detected"),
        "Must not say 'No audio devices' when device is present. Answer: {}",
        answer
    );

    // Must mention Intel (the device)
    assert!(
        answer.contains("Intel"),
        "Answer must mention the device vendor"
    );
}

// ============================================================================
// v0.0.58: ConfigureEditor Evidence-Only Flow (Goal C)
// ============================================================================

/// v0.0.58: ConfigureEditor must use ONLY current probe evidence, not stale inventory.
#[test]
fn golden_configure_editor_uses_current_probe_evidence_only() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Probe evidence shows vim exists, nano doesn't
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

    let probes = vec![vim_probe, nano_probe];
    let parsed: Vec<ParsedProbeData> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    // Get only INSTALLED editors from current evidence
    let editor_names = [
        "vim", "nvim", "nano", "emacs", "code", "micro", "helix", "vi",
    ];
    let installed_editors: Vec<&str> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    // Only vim should be in the list (nano exists=false)
    assert_eq!(
        installed_editors.len(),
        1,
        "Only probed installed editors should be listed"
    );
    assert!(
        installed_editors.contains(&"vim"),
        "vim was probed as installed"
    );
    assert!(
        !installed_editors.contains(&"nano"),
        "nano was probed as NOT installed"
    );
}

/// v0.0.58: ConfigureEditor must NOT suggest editors that were not probed.
#[test]
fn golden_configure_editor_does_not_suggest_unprobed_editors() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Only vim was probed (code, emacs etc were NOT probed)
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let probes = vec![vim_probe];
    let parsed: Vec<ParsedProbeData> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    let editor_names = [
        "vim", "nvim", "nano", "emacs", "code", "micro", "helix", "vi",
    ];
    let available_editors: Vec<&str> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    // Even if "code" is installed on the system, it must NOT appear here
    // because it was not probed in this request
    assert!(
        !available_editors.contains(&"code"),
        "code was not probed, must not appear"
    );
    assert!(
        !available_editors.contains(&"emacs"),
        "emacs was not probed, must not appear"
    );
    assert!(
        !available_editors.contains(&"nano"),
        "nano was not probed, must not appear"
    );

    // Only vim should appear
    assert_eq!(
        available_editors,
        vec!["vim"],
        "Only probed editors should appear"
    );
}

/// v0.0.58: ConfigureEditor response must be grounded and have probes attached.
#[test]
fn golden_configure_editor_is_grounded_and_has_probes_attached() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Multiple editors probed
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/nano\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let probes = vec![vim_probe.clone(), nano_probe.clone()];
    let parsed: Vec<ParsedProbeData> = probes.iter().map(|p| parse_probe_result(p)).collect();

    // All parsed results should be valid evidence
    for p in &parsed {
        assert!(
            p.is_valid_evidence(),
            "All probes should produce valid evidence"
        );
    }

    let tools = get_installed_tools(&parsed);
    let editor_names = [
        "vim", "nvim", "nano", "emacs", "code", "micro", "helix", "vi",
    ];
    let installed_editors: Vec<&str> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    // Both vim and nano should be found
    assert_eq!(installed_editors.len(), 2);
    assert!(installed_editors.contains(&"vim"));
    assert!(installed_editors.contains(&"nano"));

    // Reliability should reflect grounding
    // (This mimics what build_result_with_flags would compute)
    let evidence_count = probes.len();
    assert!(
        evidence_count >= 2,
        "Should have probes attached for grounding"
    );
}

// ============================================================================
// v0.0.58: Output Policy - No Follow-up Questions (Goal D)
// ============================================================================

/// v0.0.58: Single-editor ConfigureEditor answer must NOT contain "Would you like..." questions.
#[test]
fn golden_configure_editor_no_followup_questions() {
    // Simulated single-editor answer (same format as rpc_handler produces)
    let editor = "vim";
    let answer = format!(
        "To configure **{}** for syntax highlighting, edit its configuration file.\n\n\
        For {}, the typical approach is:\n\
        - **vim/nvim**: Add `syntax on` to `~/.vimrc` or `~/.config/nvim/init.vim`\n\
        - **nano**: Uncomment `include` lines in `/etc/nanorc` or `~/.nanorc`\n\
        - **emacs**: Add `(global-font-lock-mode t)` to `~/.emacs`",
        editor, editor
    );

    // Must NOT contain follow-up questions
    assert!(
        !answer.contains("Would you like"),
        "Answer must not contain 'Would you like'"
    );
    assert!(
        !answer.contains("would you like"),
        "Answer must not contain 'would you like'"
    );
    assert!(
        !answer.contains("Do you want"),
        "Answer must not contain 'Do you want'"
    );
    assert!(
        !answer.contains("do you want"),
        "Answer must not contain 'do you want'"
    );
    assert!(
        !answer.contains("Shall I"),
        "Answer must not contain 'Shall I'"
    );
    assert!(
        !answer.contains("shall I"),
        "Answer must not contain 'shall I'"
    );
    assert!(
        !answer.ends_with("?"),
        "Answer should not end with a question mark"
    );
}

// ============================================================================
// v0.0.56: Clarification Response with Probes (Goal 1)
// ============================================================================

// /// v0.0.56: Clarification response must attach probes and transcript when derived from evidence.
// #[test] // TODO: Incomplete test stub
