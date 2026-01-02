//! Stabilization tests for v0.0.60 - Part 2: Editor configuration and probe spine.

fn golden_v060_configure_editor_never_invents_code() {
    use anna_shared::parsers::{
        installed_editors_from_parsed, parse_probe_result, ParsedProbeData,
    };
    use anna_shared::rpc::ProbeResult;

    // Simulate probes where vim exists, code does not
    let vim_probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let code_probe = ProbeResult {
        command: "sh -lc 'command -v code'".to_string(),
        exit_code: 1, // code not found
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let nvim_probe = ProbeResult {
        command: "sh -lc 'command -v nvim'".to_string(),
        exit_code: 1, // nvim not found
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed: Vec<ParsedProbeData> = vec![
        parse_probe_result(&vim_probe),
        parse_probe_result(&code_probe),
        parse_probe_result(&nvim_probe),
    ];

    let editors = installed_editors_from_parsed(&parsed);

    // CRITICAL: Must only contain vim, not code
    assert!(
        editors.contains(&"vim".to_string()),
        "vim should be detected"
    );
    assert!(
        !editors.contains(&"code".to_string()),
        "code must NOT be in list when probe shows exit_code=1"
    );
    assert!(
        !editors.contains(&"nvim".to_string()),
        "nvim must NOT be in list when probe shows exit_code=1"
    );

    // Single editor -> deterministic path
    assert_eq!(editors.len(), 1, "Only vim should be in the list");
}

/// v0.0.60: When only one editor exists, it should be auto-selected (deterministic path).
#[test]
fn golden_v060_configure_editor_single_editor_autopicks() {
    use anna_shared::parsers::{
        installed_editors_from_parsed, parse_probe_result, ParsedProbeData,
    };
    use anna_shared::rpc::ProbeResult;

    // Only nano exists
    let probes = vec![
        ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 0,
            stdout: "/usr/bin/nano\n".to_string(),
            stderr: String::new(),
            timing_ms: 5,
        },
        ProbeResult {
            command: "sh -lc 'command -v vim'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
        ProbeResult {
            command: "sh -lc 'command -v emacs'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
    ];

    let parsed: Vec<ParsedProbeData> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let editors = installed_editors_from_parsed(&parsed);

    // Exactly one editor -> deterministic path should be taken (single-editor branch)
    assert_eq!(editors.len(), 1, "Only one editor should be detected");
    assert_eq!(editors[0], "nano", "nano should be the detected editor");
}

/// v0.0.60: probe_spine reduce_probes allows 10 probes for configure_editor.
#[test]
fn golden_v060_probe_spine_allows_10_editor_probes() {
    use anna_shared::probe_spine::{reduce_probes, ProbeId, Urgency};

    // Simulate 10 editor probes
    let probes = vec![
        ProbeId::CommandV("code".to_string()),
        ProbeId::CommandV("vim".to_string()),
        ProbeId::CommandV("nvim".to_string()),
        ProbeId::CommandV("nano".to_string()),
        ProbeId::CommandV("emacs".to_string()),
        ProbeId::CommandV("micro".to_string()),
        ProbeId::CommandV("helix".to_string()),
        ProbeId::CommandV("hx".to_string()),
        ProbeId::CommandV("kate".to_string()),
        ProbeId::CommandV("gedit".to_string()),
    ];

    let reduced = reduce_probes(probes.clone(), "configure_editor", Urgency::Normal);

    // v0.0.60: ConfigureEditor should keep all 10 probes
    assert_eq!(
        reduced.len(),
        10,
        "configure_editor should allow 10 probes, got {}",
        reduced.len()
    );
}

/// v0.0.60: Other routes still cap at 3 probes.
#[test]
fn golden_v060_probe_spine_other_routes_cap_at_3() {
    use anna_shared::probe_spine::{reduce_probes, ProbeId, Urgency};

    // 5 probes for a normal route
    let probes = vec![
        ProbeId::Free,
        ProbeId::Df,
        ProbeId::Lscpu,
        ProbeId::Lsblk,
        ProbeId::Uname,
    ];

    let reduced = reduce_probes(probes, "memory_usage", Urgency::Normal);

    // Normal route should cap at 3
    assert_eq!(
        reduced.len(),
        3,
        "Normal routes should cap at 3 probes, got {}",
        reduced.len()
    );
}

/// v0.0.60: Empty probe results (all editors exit 1) should list what was checked.
#[test]
fn golden_v060_no_editors_grounded_negative_evidence() {
    use anna_shared::parsers::{
        get_installed_tools, installed_editors_from_parsed, parse_probe_result, ParsedProbeData,
    };
    use anna_shared::rpc::ProbeResult;

    // All probes return exit 1 (no editors found)
    let probes = vec![
        ProbeResult {
            command: "sh -lc 'command -v vim'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
        ProbeResult {
            command: "sh -lc 'command -v nano'".to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: String::new(),
            timing_ms: 5,
        },
    ];

    let parsed: Vec<ParsedProbeData> = probes.iter().map(|p| parse_probe_result(p)).collect();
    let editors = installed_editors_from_parsed(&parsed);
    let tools = get_installed_tools(&parsed);

    // No editors found
    assert!(editors.is_empty(), "Should have no installed editors");

    // But we still checked tools (grounded negative evidence)
    assert_eq!(tools.len(), 2, "Should have checked 2 tools");
    assert!(
        tools.iter().all(|t| !t.exists),
        "All tools should show exists=false"
    );
}

// ===== v0.0.61: HardwareAudio Parser + Merge Tests =====

// /// v0.0.61: Parser detects lspci audio output even with unknown command.
// /// Covers the case where command string doesn't match but output is clearly audio.
// #[test] // TODO: Incomplete test stub
