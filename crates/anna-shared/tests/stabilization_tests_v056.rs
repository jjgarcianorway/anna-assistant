//! Stabilization tests for v0.0.56.

fn golden_v056_audio_grep_exit_1_is_valid_negative_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // grep returns exit code 1 when no audio devices match
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep: no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    // Must still be Audio variant (not Error!) with empty devices
    assert!(
        matches!(parsed, ParsedProbeData::Audio(_)),
        "grep exit 1 must produce Audio variant (empty), got {:?}",
        parsed
    );

    if let ParsedProbeData::Audio(ref audio) = parsed {
        assert!(audio.devices.is_empty(), "No match = empty devices list");
        assert_eq!(audio.source, "lspci");
    }

    // Must be valid evidence for evidence enforcement
    assert!(
        parsed.is_valid_evidence(),
        "Empty audio evidence is VALID evidence (negative)"
    );
}

/// v0.0.56: Audio devices found = valid evidence for HardwareAudio route.
#[test]
fn golden_v056_audio_device_found_is_valid_evidence() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel Corporation HD Audio Controller\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);
    let parsed_vec = vec![parsed.clone()];

    // Must be valid evidence
    assert!(
        parsed.is_valid_evidence(),
        "Audio with device is valid evidence"
    );

    // find_audio_evidence must find it
    let audio = find_audio_evidence(&parsed_vec);
    assert!(audio.is_some(), "Must find audio evidence");
    assert!(!audio.unwrap().devices.is_empty(), "Must have devices");
}

// === v0.0.56 Goal 3: ConfigureEditor evidence-based tests ===

/// v0.0.56: get_installed_tools extracts ToolExists from parsed probes.
#[test]
fn golden_v056_get_installed_tools_from_probes() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // vim exists
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    // nano does not exist
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1, // Not found
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let probes = vec![vim_probe, nano_probe];
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    // Should have 2 tools (vim exists=true, nano exists=false)
    assert_eq!(tools.len(), 2, "Should have tool evidence for both");

    let vim = tools.iter().find(|t| t.name == "vim");
    assert!(vim.is_some(), "Must have vim evidence");
    assert!(vim.unwrap().exists, "vim must exist");

    let nano = tools.iter().find(|t| t.name == "nano");
    assert!(nano.is_some(), "Must have nano evidence (negative)");
    assert!(!nano.unwrap().exists, "nano must NOT exist");
}

/// v0.0.56: ConfigureEditor must use only editors from current probe evidence.
/// If only vim exists in probes, vim should be auto-picked.
#[test]
fn golden_v056_configure_editor_single_editor_from_probes() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // Only vim exists
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    // nano, emacs, code don't exist
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let probes = vec![vim_probe, nano_probe];
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    // Filter to installed editors only
    let editor_names = ["vim", "nvim", "nano", "emacs", "code", "micro", "vi"];
    let installed_editors: Vec<_> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    assert_eq!(installed_editors.len(), 1, "Only vim should be found");
    assert_eq!(installed_editors[0], "vim");
}

/// v0.0.56: ConfigureEditor with multiple editors returns choices from probes.
#[test]
fn golden_v056_configure_editor_multiple_editors_from_probes() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // vim and code exist
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
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

    // nano doesn't exist (negative evidence)
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let probes = vec![vim_probe, code_probe, nano_probe];
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    // Filter to installed editors only
    let editor_names = ["vim", "nvim", "nano", "emacs", "code", "micro", "vi"];
    let installed_editors: Vec<_> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    assert_eq!(installed_editors.len(), 2, "vim and code should be found");
    assert!(installed_editors.contains(&"vim"), "Must contain vim");
    assert!(installed_editors.contains(&"code"), "Must contain code");
    assert!(
        !installed_editors.contains(&"nano"),
        "Must NOT contain nano (not installed)"
    );
}

/// v0.0.56: Never offer an editor not probed in current request.
#[test]
fn golden_v056_configure_editor_only_probed_editors() {
    use anna_shared::parsers::{get_installed_tools, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // Only vim was probed (exists)
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    // emacs was never probed - shouldn't appear even if it exists on system
    let probes = vec![vim_probe];
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let tools = get_installed_tools(&parsed);

    let editor_names = ["vim", "nvim", "nano", "emacs", "code", "micro", "vi"];
    let installed_editors: Vec<_> = tools
        .iter()
        .filter(|t| t.exists && editor_names.contains(&t.name.as_str()))
        .map(|t| t.name.as_str())
        .collect();

    // Only vim should appear (emacs not probed, so not in list)
    assert_eq!(installed_editors.len(), 1);
    assert_eq!(installed_editors[0], "vim");
    // emacs is NOT in the list even if it exists on the system
    assert!(
        !installed_editors.contains(&"emacs"),
        "emacs was not probed, must not appear"
    );
}

// === v0.0.56 Goal 4: Reliability/trace and "probe failed" messaging ===

/// v0.0.56: Tool check exit_code=1 must count as valid evidence, NOT failed probe.
#[test]
fn golden_v056_tool_exit_1_is_valid_evidence_for_counting() {
    use anna_shared::parsers::{count_valid_evidence_probes, is_probe_valid_evidence};
    use anna_shared::rpc::ProbeResult;

    // Tool exists (exit_code=0)
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    // Tool NOT found (exit_code=1) - this is VALID negative evidence!
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    // Both probes should count as valid evidence
    assert!(
        is_probe_valid_evidence(&vim_probe),
        "exit_code=0 is valid evidence"
    );
    assert!(
        is_probe_valid_evidence(&nano_probe),
        "exit_code=1 for command -v is valid negative evidence"
    );

    let probes = vec![vim_probe, nano_probe];
    let count = count_valid_evidence_probes(&probes);

    // BOTH probes produce valid evidence (1 positive, 1 negative)
    assert_eq!(
        count, 2,
        "Both probes (exit 0 and exit 1) are valid evidence"
    );
}

/// v0.0.56: Audio probe exit_code=1 (grep no match) must count as valid evidence.
#[test]
fn golden_v056_audio_exit_1_is_valid_evidence_for_counting() {
    use anna_shared::parsers::{count_valid_evidence_probes, is_probe_valid_evidence};
    use anna_shared::rpc::ProbeResult;

    // Audio device found
    let audio_found = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // Audio grep returns 1 (no match) - this is VALID empty evidence!
    let audio_empty = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 1, // grep: no match
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 10,
    };

    assert!(
        is_probe_valid_evidence(&audio_found),
        "Audio with device is valid evidence"
    );
    assert!(
        is_probe_valid_evidence(&audio_empty),
        "Audio with exit_code=1 (no match) is valid evidence"
    );

    let probes = vec![audio_found, audio_empty];
    assert_eq!(
        count_valid_evidence_probes(&probes),
        2,
        "Both audio probes are valid evidence"
    );
}

/// v0.0.56: Reliability should NOT penalize when all probes produce valid evidence.
#[test]
fn golden_v056_no_probe_failed_penalty_for_valid_evidence() {
    use anna_shared::reliability::{compute_reliability, ReliabilityInput, ReliabilityReason};

    // Scenario: 2 probes planned, 2 probes returned valid evidence
    // (even though one has exit_code=1, it's still valid negative evidence)
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(2)
        .with_succeeded_probes(2) // Both produced valid evidence
        .with_total_claims(1)
        .with_verified_claims(1)
        .with_answer_grounded(true)
        .with_no_invention(true)
        .with_translator_confidence(90);

    let output = compute_reliability(&input);

    // Should NOT have ProbeFailed reason since all probes succeeded
    assert!(
        !output.reasons.contains(&ReliabilityReason::ProbeFailed),
        "Should NOT have ProbeFailed when all probes produce valid evidence"
    );

    // Score should be high
    assert!(
        output.score >= 80,
        "Score should be >= 80 when all probes produce valid evidence, got {}",
        output.score
    );
}

// ============================================================================
// v0.0.57: Evidence Validity Semantics Tests (Goal 2)
// ============================================================================

// /// v0.0.57: exit_code=127 ("command not found") is NOT valid evidence.
// #[test] // TODO: Incomplete test stub
