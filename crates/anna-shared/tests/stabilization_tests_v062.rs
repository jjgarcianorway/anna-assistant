//! Stabilization tests for v0.0.62.

fn golden_v062_configure_editor_tool_evidence_extraction() {
    use anna_shared::parsers::{installed_editors_from_parsed, parse_probe_result};
    use anna_shared::rpc::ProbeResult;

    // vim installed (exit 0, has path)
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    // nano not installed (exit 1, no path)
    let nano_probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 8,
    };

    // emacs not installed (exit 1)
    let emacs_probe = ProbeResult {
        command: "sh -lc 'command -v emacs'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 9,
    };

    let probes = vec![vim_probe, nano_probe, emacs_probe];
    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();

    let installed = installed_editors_from_parsed(&parsed);

    // Only vim should be in the list
    assert_eq!(installed.len(), 1, "Should have exactly 1 installed editor");
    assert!(
        installed.contains(&"vim".to_string()),
        "vim should be installed"
    );
    assert!(
        !installed.contains(&"nano".to_string()),
        "nano should NOT be installed"
    );
    assert!(
        !installed.contains(&"emacs".to_string()),
        "emacs should NOT be installed"
    );
}

/// v0.0.62: ToolExists evidence counts as valid for grounding purposes.
/// Both positive (exists=true) and negative (exists=false) are valid evidence.
#[test]
fn golden_v062_tool_exists_is_valid_evidence() {
    use anna_shared::parsers::parse_probe_result;
    use anna_shared::rpc::ProbeResult;

    // Tool found - valid positive evidence
    let found_probe = ProbeResult {
        command: "sh -lc 'command -v code'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/code\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };
    let parsed = parse_probe_result(&found_probe);
    assert!(
        parsed.is_valid_evidence(),
        "Tool found (exit 0) should be valid evidence"
    );

    // Tool not found - valid negative evidence
    let not_found_probe = ProbeResult {
        command: "sh -lc 'command -v nonexistent'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 8,
    };
    let parsed = parse_probe_result(&not_found_probe);
    assert!(
        parsed.is_valid_evidence(),
        "Tool not found (exit 1) should be valid negative evidence"
    );
}

/// v0.0.62: ConfigureEditor multiple editors derive valid probe count.
/// When multiple editor probes run, the valid_evidence_count should match.
#[test]
fn golden_v062_configure_editor_valid_evidence_count() {
    use anna_shared::parsers::parse_probe_result;
    use anna_shared::rpc::ProbeResult;

    let probes = vec![
        ProbeResult {
            command: "sh -lc 'command -v vim'".to_string(),
            exit_code: 0,
            stdout: "/usr/bin/vim\n".to_string(),
            stderr: String::new(),
            timing_ms: 10,
        },
        ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 0,
            stdout: "/usr/bin/nano\n".to_string(),
            stderr: String::new(),
            timing_ms: 8,
        },
        ProbeResult {
            command: "sh -lc 'command -v code'".to_string(),
            exit_code: 1, // Not found but still valid evidence
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 9,
        },
    ];

    let parsed: Vec<_> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let valid_count = parsed.iter().filter(|p| p.is_valid_evidence()).count();

    // All 3 should be valid evidence (2 found + 1 not-found)
    assert_eq!(
        valid_count, 3,
        "All tool probes should be valid evidence, got {}",
        valid_count
    );
}

// ===== v0.0.63: Service Desk Theatre Tests =====

// /// v0.0.63: Transcript EvidenceSummary event serialization.
// #[test] // TODO: Incomplete test stub
