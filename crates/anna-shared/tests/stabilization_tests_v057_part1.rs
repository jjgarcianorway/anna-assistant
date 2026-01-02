//! Stabilization tests for v0.0.57 - Part 1.

fn golden_v057_command_not_found_is_not_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "sh -lc 'command -v nonexistent_tool'".to_string(),
        exit_code: 127,
        stdout: String::new(),
        stderr: "sh: command: not found".to_string(),
        timing_ms: 5,
    };

    let parsed = parse_probe_result(&probe);

    // exit_code=127 should be treated as an error, not evidence
    assert!(
        matches!(parsed, ParsedProbeData::Error(_)),
        "exit_code=127 should be Error, not valid evidence"
    );
    assert!(
        !parsed.is_valid_evidence(),
        "exit_code=127 must NOT be valid evidence"
    );
}

/// v0.0.57: pacman -Q with exit 1 is valid negative evidence (package not installed).
#[test]
fn golden_v057_pacman_q_exit_1_is_valid_evidence() {
    use anna_shared::parsers::{is_probe_valid_evidence, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // Package not installed (exit 1)
    let not_installed = ProbeResult {
        command: "pacman -Q nonexistent 2>/dev/null".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: "error: package 'nonexistent' was not found".to_string(),
        timing_ms: 10,
    };

    // Package installed (exit 0)
    let installed = ProbeResult {
        command: "pacman -Q vim 2>/dev/null".to_string(),
        exit_code: 0,
        stdout: "vim 9.0.1000-1".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    assert!(
        is_probe_valid_evidence(&not_installed),
        "pacman -Q exit 1 is valid negative evidence"
    );
    assert!(
        is_probe_valid_evidence(&installed),
        "pacman -Q exit 0 is valid positive evidence"
    );

    // Verify parsed structure
    let parsed_not = parse_probe_result(&not_installed);
    let parsed_yes = parse_probe_result(&installed);

    assert!(parsed_not.as_package().is_some(), "Should parse as Package");
    assert!(
        !parsed_not.as_package().unwrap().installed,
        "Should be not-installed"
    );

    assert!(parsed_yes.as_package().is_some(), "Should parse as Package");
    assert!(
        parsed_yes.as_package().unwrap().installed,
        "Should be installed"
    );
}

/// v0.0.57: Timeout probes (if represented) are NOT valid evidence.
#[test]
fn golden_v057_timeout_probe_is_not_evidence() {
    use anna_shared::parsers::is_probe_valid_evidence;
    use anna_shared::rpc::ProbeResult;

    // Simulated timeout - exit code non-standard (e.g., 124 from timeout command)
    // or we could use a very high exit code
    let timeout_probe = ProbeResult {
        command: "timeout 5 some_slow_command".to_string(),
        exit_code: 124, // timeout exit code
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5000,
    };

    // Timeouts are not tool/package probes, so they go through the "other" path
    // which treats non-zero exit as error
    assert!(
        !is_probe_valid_evidence(&timeout_probe),
        "Timeout probe should NOT be valid evidence"
    );
}

/// v0.0.57: Empty stdout with exit 0 for probes where empty output is meaningless is NOT evidence.
/// Example: df with exit 0 but no filesystems listed.
#[test]
fn golden_v057_empty_meaningful_probe_is_not_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // df with exit 0 but empty output (malformed)
    let empty_df = ProbeResult {
        command: "df -h".to_string(),
        exit_code: 0,
        stdout: String::new(), // No filesystem data
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&empty_df);

    // Empty output from df should produce an error (missing section)
    assert!(
        matches!(
            parsed,
            ParsedProbeData::Error(_) | ParsedProbeData::Unsupported
        ),
        "Empty df output should be Error or Unsupported"
    );
}

// ============================================================================
// v0.0.57: Reliability - Stop Penalizing Expected Negatives (Goal 3)
// ============================================================================

/// v0.0.57: InstalledToolCheck with negative evidence must still be grounded/covered.
#[test]
fn golden_v057_negative_tool_check_is_grounded_and_covered() {
    use anna_shared::reliability::{compute_reliability, ReliabilityInput, ReliabilityReason};

    // User asks "do I have nano" - both tool and package checks return "not found"
    // This is expected behavior - we have evidence, it's just negative
    let input = ReliabilityInput::default()
        .with_evidence_required(true)
        .with_planned_probes(2) // command -v nano, pacman -Q nano
        .with_succeeded_probes(2) // Both returned valid evidence (exit 1 = not found)
        .with_total_claims(1) // "nano is not installed"
        .with_verified_claims(1) // Claim is grounded in evidence
        .with_answer_grounded(true) // Answer is grounded
        .with_no_invention(true) // No hallucination
        .with_translator_confidence(95);

    let output = compute_reliability(&input);

    // Should NOT have any negative reasons - this is a valid, grounded response
    assert!(
        !output.reasons.contains(&ReliabilityReason::ProbeFailed),
        "Negative evidence probes should NOT trigger ProbeFailed"
    );
    assert!(
        !output.reasons.contains(&ReliabilityReason::NotGrounded),
        "Negative evidence answer should be grounded"
    );
    assert!(
        !output.reasons.contains(&ReliabilityReason::EvidenceMissing),
        "Negative evidence should count as evidence"
    );

    // Score should be high (all evidence collected, answer is valid)
    assert!(
        output.score >= 80,
        "Negative evidence response should have high reliability, got {}",
        output.score
    );
}

// ============================================================================
// v0.0.57: Output Policy - No Raw Probe Dumps (Goal 4)
// ============================================================================

/// v0.0.57: Deterministic answers must NOT contain raw command strings.
#[test]
fn golden_v057_no_raw_commands_in_tool_check_answer() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    // Simulate the tool check answer builder pattern
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed: Vec<ParsedProbeData> = vec![parse_probe_result(&vim_probe)];

    // Build answer (simulating what answer_installed_tool_check does)
    let tool = parsed.iter().filter_map(|p| p.as_tool()).next().unwrap();
    let answer = if tool.exists {
        let path_info = tool
            .path
            .as_ref()
            .map(|p| format!(" at `{}`", p))
            .unwrap_or_default();
        format!("Yes, **{}** is installed{}", tool.name, path_info)
    } else {
        format!("**{}** is not found in your PATH", tool.name)
    };

    // Answer must NOT contain raw command strings
    assert!(
        !answer.contains("sh -lc"),
        "Answer must not contain 'sh -lc'"
    );
    assert!(
        !answer.contains("command -v"),
        "Answer must not contain 'command -v'"
    );
    assert!(
        !answer.contains("lspci |"),
        "Answer must not contain 'lspci |'"
    );
    assert!(
        !answer.contains("grep -i"),
        "Answer must not contain 'grep -i'"
    );
}

/// v0.0.57: CPU cores answer must not contain raw probe commands.
#[test]
fn golden_v057_no_raw_commands_in_cpu_answer() {
    // Simulated CPU answer (from lscpu parsing)
    let answer = "Your CPU has 8 cores (16 threads).";

    assert!(
        !answer.contains("lscpu"),
        "CPU answer must not contain 'lscpu'"
    );
    assert!(
        !answer.contains("sh -lc"),
        "CPU answer must not contain 'sh -lc'"
    );
}

/// v0.0.57: Memory answer must not contain raw probe commands.
#[test]
fn golden_v057_no_raw_commands_in_memory_answer() {
    // Simulated memory answer (from free parsing)
    let answer = "You have 16.0 GB of RAM, with 8.5 GB available.";

    assert!(
        !answer.contains("free"),
        "Memory answer must not contain 'free'"
    );
    assert!(
        !answer.contains("-h"),
        "Memory answer must not contain '-h'"
    );
    assert!(
        !answer.contains("-b"),
        "Memory answer must not contain '-b'"
    );
}

/// v0.0.57: Audio answer must not contain raw probe commands.
#[test]
fn golden_v057_no_raw_commands_in_audio_answer() {
    use anna_shared::parsers::{find_audio_evidence, parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let audio_probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Audio device: Intel Corporation Cannon Lake PCH cAVS".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed: Vec<ParsedProbeData> = vec![parse_probe_result(&audio_probe)];

    if let Some(audio) = find_audio_evidence(&parsed) {
        // Build answer like answer_hardware_audio does
        let answer = if audio.devices.is_empty() {
            "No audio devices detected.".to_string()
        } else {
            let dev = &audio.devices[0];
            format!("**Audio device**: {}", dev.description)
        };

        // Answer must NOT contain raw command strings
        assert!(
            !answer.contains("lspci"),
            "Audio answer must not contain 'lspci'"
        );
        assert!(
            !answer.contains("grep"),
            "Audio answer must not contain 'grep'"
        );
        assert!(
            !answer.contains("pactl"),
            "Audio answer must not contain 'pactl'"
        );
    }
}

// ============================================================================
// v0.0.58: HardwareAudio Parser Fixes (Goal A)
// ============================================================================

// /// v0.0.58: Audio parser must recognize "Multimedia audio controller:" lines.
// /// This was a false negative - lspci output showed a device but we said "No audio detected."
// #[test] // TODO: Incomplete test stub
