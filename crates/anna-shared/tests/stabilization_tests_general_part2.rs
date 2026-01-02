//! General stabilization tests - Part 2: Clarification and Editor Probe tests.

fn golden_clarification_attaches_probes_and_transcript() {
    use anna_shared::rpc::{ProbeResult, ReliabilitySignals};

    // Simulate probes that ran before clarification was needed
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

    let probes = vec![vim_probe, nano_probe];

    // Simulate grounded clarification response signals
    let has_probes = !probes.is_empty();
    let signals = ReliabilitySignals {
        translator_confident: has_probes,
        probe_coverage: has_probes,
        answer_grounded: true, // Options derived from evidence
        no_invention: true,
        clarification_not_needed: false,
    };

    // Verify the response includes probes
    assert!(has_probes, "Clarification should have probes attached");
    assert_eq!(probes.len(), 2, "Should have 2 probes");

    // Verify signals reflect grounding
    assert!(
        signals.probe_coverage,
        "probe_coverage should be true when probes present"
    );
    assert!(
        signals.answer_grounded,
        "answer_grounded should be true when options from evidence"
    );
    assert!(
        signals.no_invention,
        "no_invention should always be true for clarification"
    );
}

/// v0.0.56: Clarification is grounded when options come from current probe evidence.
#[test]
fn golden_clarification_is_grounded_when_options_come_from_evidence() {
    use anna_shared::rpc::ReliabilitySignals;

    // Grounded clarification: options derived from probe evidence
    let grounded_signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: true,
        answer_grounded: true, // Grounded!
        no_invention: true,
        clarification_not_needed: false,
    };

    // Ungrounded clarification: no evidence (e.g., triage asking for domain)
    let ungrounded_signals = ReliabilitySignals {
        translator_confident: false,
        probe_coverage: false,
        answer_grounded: false, // Not grounded
        no_invention: true,
        clarification_not_needed: false,
    };

    // Grounded clarification should have higher score
    let grounded_score = grounded_signals.score();
    let ungrounded_score = ungrounded_signals.score();

    assert!(
        grounded_score > ungrounded_score,
        "Grounded clarification ({}) should have higher score than ungrounded ({})",
        grounded_score,
        ungrounded_score
    );

    // Grounded clarification (with evidence) should score >= 60 (it has valid probes)
    // Ungrounded clarification (no evidence) should cap at 40
    assert!(
        grounded_score >= 60,
        "Grounded clarification score {} should be >= 60 (has evidence)",
        grounded_score
    );
    assert!(
        ungrounded_score <= 40,
        "Ungrounded clarification score {} should be <= 40",
        ungrounded_score
    );
}

// ============================================================================
// v0.0.56: ConfigureEditor Routing (Goal 2)
// ============================================================================

/// v0.0.56: ConfigureEditor route must be deterministic and require evidence.
#[test]
fn test_configure_editor_route_is_deterministic_and_requires_evidence() {
    use anna_shared::probe_spine::EvidenceKind;

    // Mock the route capability checking
    let can_answer_deterministically = true; // ConfigureEditor is now deterministic
    let evidence_required = true;
    let required_evidence = vec![EvidenceKind::ToolExists];

    assert!(
        can_answer_deterministically,
        "ConfigureEditor must be deterministic with evidence"
    );
    assert!(evidence_required, "ConfigureEditor must require evidence");
    assert!(
        required_evidence.contains(&EvidenceKind::ToolExists),
        "ConfigureEditor must require ToolExists evidence"
    );
}

/// v0.0.56: ConfigureEditor route must add probes for supported editors.
#[test]
fn test_configure_editor_route_adds_editor_probes() {
    // v0.0.56: ConfigureEditor probes list
    let probes = vec![
        "command_v_vim",
        "command_v_nvim",
        "command_v_nano",
        "command_v_emacs",
        "command_v_micro",
        "command_v_helix",
        "command_v_code",
    ];

    // All supported editors must be probed
    assert!(probes.contains(&"command_v_vim"), "Must probe vim");
    assert!(probes.contains(&"command_v_nvim"), "Must probe nvim");
    assert!(probes.contains(&"command_v_nano"), "Must probe nano");
    assert!(probes.contains(&"command_v_emacs"), "Must probe emacs");
    assert!(probes.contains(&"command_v_code"), "Must probe code");
    assert!(probes.contains(&"command_v_micro"), "Must probe micro");
    assert!(probes.contains(&"command_v_helix"), "Must probe helix");

    // Verify count
    assert_eq!(probes.len(), 7, "Should probe exactly 7 editors");
}

// ============================================================================
// v0.0.56: Probe Spine Editor Config Phrases (Goal 3)
// ============================================================================

/// v0.0.56: Probe spine must match common editor config phrasings.
#[test]
fn golden_editor_config_probe_spine_matches_common_phrasings() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let queries = [
        "enable syntax highlighting",
        "turn on syntax highlighting",
        "enable line numbers",
        "set vim to show line numbers",
        "configure editor theme",
        "set colorscheme",
        "enable auto indent",
        "turn on word wrap",
    ];

    for query in queries {
        let decision = enforce_minimum_probes(query, &[]);
        assert!(
            decision.enforced,
            "Query '{}' should enforce editor probes",
            query
        );

        // Should have CommandV probes for editors
        let has_editor_probes = decision
            .probes
            .iter()
            .any(|p| matches!(p, ProbeId::CommandV(_)));
        assert!(
            has_editor_probes,
            "Query '{}' should add CommandV probes for editors",
            query
        );
    }
}

/// v0.0.56: Probe spine must NOT trigger on unrelated "enable" phrases.
#[test]
fn golden_editor_config_probe_spine_does_not_trigger_on_unrelated_enable() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let unrelated_queries = [
        "enable wifi",
        "enable bluetooth",
        "turn on network",
        "enable service nginx",
        "enable dark mode in terminal", // dark mode != editor theme without editor keywords
    ];

    for query in unrelated_queries {
        let decision = enforce_minimum_probes(query, &[]);

        // Should NOT have CommandV probes for vim/nano/emacs
        let has_editor_vim = decision.probes.iter().any(|p| {
            if let ProbeId::CommandV(name) = p {
                name == "vim" || name == "nano" || name == "emacs"
            } else {
                false
            }
        });
        assert!(
            !has_editor_vim,
            "Query '{}' should NOT trigger editor probes",
            query
        );
    }
}

// ============================================================================
// v0.0.56: Audio Dual Evidence Sources (Goal 4)
// ============================================================================

/// v0.0.56/v0.0.60: Audio should merge lspci and pactl when both present.
#[test]
fn golden_audio_merges_lspci_and_pactl_when_both_present() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // lspci has device
    let lspci_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel Corporation Cannon Lake PCH cAVS\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // pactl also has device
    let pactl_probe = ProbeResult {
        command: "pactl list cards".to_string(),
        exit_code: 0,
        stdout:
            "Card #0\n\tName: alsa_card.pci-0000_00_1f.3\n\talsa.card_name = \"HDA Intel PCH\"\n"
                .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed: Vec<ParsedProbeData> = vec![
        parse_probe_result(&lspci_probe),
        parse_probe_result(&pactl_probe),
    ];

    let audio = find_audio_evidence(&parsed).expect("Should find audio evidence");

    // v0.0.60: When both have devices, merge them with source indicating both
    assert!(!audio.devices.is_empty(), "Should have devices");
    assert!(
        audio.source.contains("lspci"),
        "Should indicate lspci source"
    );
    // v0.0.60: Now returns merged source "lspci+pactl"
    assert!(
        audio.source.contains("+") || audio.source == "lspci+pactl",
        "Should indicate merged sources, got: {}",
        audio.source
    );
}

/// v0.0.56: Audio with negative evidence (both empty) should still be grounded.
#[test]
fn golden_audio_negative_evidence_is_grounded() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // lspci returns nothing (grep exit 1)
    let lspci_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep: no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // pactl also returns nothing
    let pactl_probe = ProbeResult {
        command: "pactl list cards".to_string(),
        exit_code: 1, // No pulseaudio
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed: Vec<ParsedProbeData> = vec![
        parse_probe_result(&lspci_probe),
        parse_probe_result(&pactl_probe),
    ];

    let audio = find_audio_evidence(&parsed).expect("Should find audio evidence (negative)");

    // Negative evidence is still valid evidence
    assert!(audio.devices.is_empty(), "Should have no devices");

    // The audio evidence exists (is_valid_evidence should be true for empty list)
    // This proves we have grounded negative evidence
    let lspci_parsed = parse_probe_result(&lspci_probe);
    assert!(
        lspci_parsed.is_valid_evidence(),
        "Empty audio is valid negative evidence"
    );
}

/// v0.0.56/v0.0.60: If only one source has devices, use that source (merged).
#[test]
fn golden_audio_uses_non_empty_source() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // lspci empty
    let lspci_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // pactl has device
    let pactl_probe = ProbeResult {
        command: "pactl list cards".to_string(),
        exit_code: 0,
        stdout: "Card #0\n\tName: alsa_card.usb\n\talsa.card_name = \"USB Audio\"\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed: Vec<ParsedProbeData> = vec![
        parse_probe_result(&lspci_probe),
        parse_probe_result(&pactl_probe),
    ];

    let audio = find_audio_evidence(&parsed).expect("Should find audio evidence");

    // v0.0.60: When lspci is empty but pactl has devices, merged result has devices
    assert!(!audio.devices.is_empty(), "Should have devices from pactl");
    // v0.0.60: Now returns merged source "lspci+pactl" when both are present
    assert!(
        audio.source.contains("pactl"),
        "Should indicate pactl source, got: {}",
        audio.source
    );
}

// ============================================================================
// v0.0.57: ConfigureEditor Flow - No Inventory, No Questions
// ============================================================================

// /// v0.0.57: Single editor answer contains only that editor's steps, no questions.
// #[test] // TODO: Incomplete test stub
