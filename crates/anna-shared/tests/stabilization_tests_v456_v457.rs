//! Stabilization tests for v0.45.6 and v0.45.7.

use anna_shared::reliability::{
    compute_reliability, ReliabilityInput, NO_EVIDENCE_RELIABILITY_CAP,
};

// === v0.45.6 Golden Tests: Probe Contract Fix ===

/// v0.45.6: "do I have nano" must enforce CommandV probe.
#[test]
fn golden_v456_tool_check_enforces_command_v() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("do I have nano", &[]);
    assert!(decision.enforced, "Tool check must enforce probes");

    // Must include CommandV probe
    let has_command_v = decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::CommandV(_)));
    assert!(has_command_v, "Tool check must include CommandV probe");

    // When converted to command, should produce executable command
    let command_v_probe = decision
        .probes
        .iter()
        .find(|p| matches!(p, ProbeId::CommandV(_)))
        .unwrap();
    let cmd = probe_to_command(command_v_probe);
    assert!(
        cmd.contains("command -v"),
        "CommandV probe must use 'command -v'"
    );
    assert!(
        cmd.contains("nano"),
        "CommandV probe must include package name"
    );
}

/// v0.45.6: "how many cores" must enforce Lscpu probe.
#[test]
fn golden_v456_cpu_cores_enforces_lscpu() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("how many cores has my cpu", &[]);
    assert!(decision.enforced, "CPU cores query must enforce probes");

    // Must include Lscpu probe
    let has_lscpu = decision.probes.iter().any(|p| matches!(p, ProbeId::Lscpu));
    assert!(has_lscpu, "CPU cores query must include Lscpu probe");

    // When converted to command, should be "lscpu"
    let cmd = probe_to_command(&ProbeId::Lscpu);
    assert_eq!(cmd, "lscpu", "Lscpu probe must produce 'lscpu' command");
}

/// v0.45.6: "what is my sound card" must enforce audio probes.
#[test]
fn golden_v456_sound_card_enforces_audio_probes() {
    use anna_shared::probe_spine::{enforce_minimum_probes, probe_to_command, ProbeId};

    let decision = enforce_minimum_probes("what is my sound card", &[]);
    assert!(decision.enforced, "Sound card query must enforce probes");

    // Must include LspciAudio probe
    let has_lspci_audio = decision
        .probes
        .iter()
        .any(|p| matches!(p, ProbeId::LspciAudio));
    assert!(
        has_lspci_audio,
        "Sound card query must include LspciAudio probe"
    );

    // LspciAudio command should contain lspci and audio
    let cmd = probe_to_command(&ProbeId::LspciAudio);
    assert!(cmd.contains("lspci"), "LspciAudio probe must use lspci");
    assert!(
        cmd.to_lowercase().contains("audio"),
        "LspciAudio probe must filter for audio"
    );
}

/// v0.45.6: Probe commands from probe_spine can be resolved for execution.
#[test]
fn golden_v456_probe_spine_commands_resolvable() {
    use anna_shared::probe_spine::{probe_to_command, ProbeId};

    // All probe_spine commands should start with known executables
    let known_executables = [
        "lscpu",
        "sensors",
        "free",
        "df",
        "lsblk",
        "lspci",
        "pactl",
        "ip",
        "ps",
        "systemctl",
        "journalctl",
        "pacman",
        "sh",
        "uname",
        "systemd-analyze",
    ];

    let probes = [
        ProbeId::Lscpu,
        ProbeId::Sensors,
        ProbeId::Free,
        ProbeId::Df,
        ProbeId::Lsblk,
        ProbeId::LspciAudio,
        ProbeId::PactlCards,
        ProbeId::IpAddr,
        ProbeId::TopMemory,
        ProbeId::TopCpu,
        ProbeId::FailedUnits,
        ProbeId::JournalErrors,
        ProbeId::JournalWarnings,
        ProbeId::PacmanCount,
        ProbeId::CommandV("test".to_string()),
        ProbeId::SystemdAnalyze,
        ProbeId::Uname,
    ];

    for probe in probes {
        let cmd = probe_to_command(&probe);
        let first_word = cmd.split_whitespace().next().unwrap_or("");

        let is_known = known_executables.iter().any(|&exe| first_word == exe);
        assert!(
            is_known,
            "Probe {:?} produces command '{}' with unknown executable '{}'",
            probe, cmd, first_word
        );
    }
}

/// v0.45.6: Evidence kinds are properly bound to probes.
#[test]
fn golden_v456_evidence_binding() {
    use anna_shared::probe_spine::{probes_for_evidence, EvidenceKind};

    // Audio evidence must include audio probes
    let audio_probes = probes_for_evidence(EvidenceKind::Audio);
    assert!(!audio_probes.is_empty(), "Audio evidence must have probes");

    // CPU evidence must include lscpu
    let cpu_probes = probes_for_evidence(EvidenceKind::Cpu);
    assert!(!cpu_probes.is_empty(), "CPU evidence must have probes");

    // Memory evidence must include free
    let mem_probes = probes_for_evidence(EvidenceKind::Memory);
    assert!(!mem_probes.is_empty(), "Memory evidence must have probes");

    // Journal evidence must include journal probes
    let journal_probes = probes_for_evidence(EvidenceKind::Journal);
    assert!(
        !journal_probes.is_empty(),
        "Journal evidence must have probes"
    );
}

// === v0.45.7 Golden Tests: Negative Evidence ===

/// v0.45.7: Tool check with exit_code=1 is VALID NEGATIVE EVIDENCE.
#[test]
fn golden_v457_tool_not_found_is_valid_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData, ToolExistsMethod};
    use anna_shared::rpc::ProbeResult;

    // Exit code 1 = tool not found (VALID negative evidence!)
    let probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed = parse_probe_result(&probe);

    // Must parse as Tool variant (not Error!)
    assert!(
        matches!(parsed, ParsedProbeData::Tool(_)),
        "exit_code=1 from command -v must parse as Tool, got {:?}",
        parsed
    );

    if let ParsedProbeData::Tool(ref t) = parsed {
        assert_eq!(t.name, "nano");
        assert!(!t.exists, "Tool with exit_code=1 must have exists=false");
        assert_eq!(t.method, ToolExistsMethod::CommandV);
        assert!(t.path.is_none(), "Non-existent tool must have no path");
    }

    // Must be valid evidence (not error or unsupported)
    assert!(
        parsed.is_valid_evidence(),
        "exit_code=1 from command -v must be valid evidence"
    );
}

/// v0.45.7: Tool found (exit_code=0) is VALID POSITIVE EVIDENCE.
#[test]
fn golden_v457_tool_found_is_valid_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "sh -lc 'command -v vim'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/vim\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };

    let parsed = parse_probe_result(&probe);

    assert!(matches!(parsed, ParsedProbeData::Tool(_)));
    if let ParsedProbeData::Tool(ref t) = parsed {
        assert_eq!(t.name, "vim");
        assert!(t.exists);
        assert_eq!(t.path, Some("/usr/bin/vim".to_string()));
    }
    assert!(parsed.is_valid_evidence());
}

/// v0.45.7: Package not installed (exit_code=1) is VALID NEGATIVE EVIDENCE.
#[test]
fn golden_v457_package_not_installed_is_valid_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "pacman -Q nano 2>/dev/null".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: "error: package 'nano' was not found".to_string(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    assert!(
        matches!(parsed, ParsedProbeData::Package(_)),
        "exit_code=1 from pacman -Q must parse as Package, got {:?}",
        parsed
    );

    if let ParsedProbeData::Package(ref p) = parsed {
        assert_eq!(p.name, "nano");
        assert!(
            !p.installed,
            "Package with exit_code=1 must have installed=false"
        );
        assert!(p.version.is_none());
    }

    assert!(
        parsed.is_valid_evidence(),
        "exit_code=1 from pacman -Q must be valid evidence"
    );
}

/// v0.45.7: Package installed (exit_code=0) is VALID POSITIVE EVIDENCE.
#[test]
fn golden_v457_package_installed_is_valid_evidence() {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};
    use anna_shared::rpc::ProbeResult;

    let probe = ProbeResult {
        command: "pacman -Q vim 2>/dev/null".to_string(),
        exit_code: 0,
        stdout: "vim 9.0.1897-1\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);

    assert!(matches!(parsed, ParsedProbeData::Package(_)));
    if let ParsedProbeData::Package(ref p) = parsed {
        assert_eq!(p.name, "vim");
        assert!(p.installed);
        assert_eq!(p.version, Some("9.0.1897-1".to_string()));
    }
    assert!(parsed.is_valid_evidence());
}

/// v0.45.7: find_tool_evidence helper works correctly.
#[test]
fn golden_v457_find_tool_evidence() {
    use anna_shared::parsers::{find_tool_evidence, ParsedProbeData, ToolExists, ToolExistsMethod};

    let parsed = vec![
        ParsedProbeData::Tool(ToolExists {
            name: "vim".to_string(),
            exists: true,
            method: ToolExistsMethod::CommandV,
            path: Some("/usr/bin/vim".to_string()),
        }),
        ParsedProbeData::Tool(ToolExists {
            name: "nano".to_string(),
            exists: false,
            method: ToolExistsMethod::CommandV,
            path: None,
        }),
    ];

    // Can find existing tool
    let vim = find_tool_evidence(&parsed, "vim");
    assert!(vim.is_some());
    assert!(vim.unwrap().exists);

    // Can find non-existing tool (negative evidence)
    let nano = find_tool_evidence(&parsed, "nano");
    assert!(nano.is_some());
    assert!(!nano.unwrap().exists);

    // Returns None for unknown tool
    let emacs = find_tool_evidence(&parsed, "emacs");
    assert!(emacs.is_none());
}

/// v0.45.7: "enable syntax highlighting" must enforce editor tool probes.
#[test]
fn golden_v457_editor_config_enforces_tool_probes() {
    use anna_shared::probe_spine::{enforce_minimum_probes, ProbeId};

    let decision = enforce_minimum_probes("enable syntax highlighting", &[]);
    assert!(decision.enforced, "Editor config query must enforce probes");

    // Must include CommandV probes for common editors
    let has_editor_probes = decision
        .probes
        .iter()
        .filter(|p| matches!(p, ProbeId::CommandV(_)))
        .count();
    assert!(
        has_editor_probes >= 3,
        "Editor config must check for at least 3 common editors, got {}",
        has_editor_probes
    );
}

// === v0.45.8 Golden Tests: Audio Evidence + Editor Config Flow ===

// /// v0.45.8: lspci audio output parses to AudioDevices variant.
// #[test] // TODO: Incomplete test stub
