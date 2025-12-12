//! Tests for parsers module (v0.0.173).

use super::*;
use crate::rpc::ProbeResult;

#[test]
fn test_parse_probe_output_free() {
    let output = r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:         4.0Gi       256Mi       3.8Gi
"#;
    let result = parse_probe_output("free -h", output);
    assert!(matches!(result, ParsedProbeData::Memory(_)));
}

#[test]
fn test_parse_probe_output_df() {
    let output = r#"Filesystem      Size  Used Avail Use% Mounted on
/dev/sda1        50G   35G   12G  75% /
"#;
    let result = parse_probe_output("df -h", output);
    assert!(matches!(result, ParsedProbeData::Disk(_)));
}

#[test]
fn test_parse_probe_output_systemctl_failed() {
    let output = r#"  UNIT LOAD ACTIVE SUB DESCRIPTION
0 loaded units listed.
"#;
    let result = parse_probe_output("systemctl --failed", output);
    assert!(matches!(result, ParsedProbeData::Services(_)));
}

#[test]
fn test_parse_probe_output_systemctl_is_active() {
    let result = parse_probe_output("systemctl is-active nginx", "active\n");
    assert!(matches!(result, ParsedProbeData::Service(_)));
    if let ParsedProbeData::Service(s) = result {
        assert_eq!(s.name, "nginx.service");
        assert_eq!(s.state, ServiceState::Active);
    }
}

#[test]
fn test_parse_probe_output_raw_text() {
    // v0.0.308: ps aux is now RawText (valid evidence) not Unsupported
    let result = parse_probe_output("ps aux --sort=-%mem", "some output");
    assert!(matches!(result, ParsedProbeData::RawText(_)));
    assert!(result.is_valid_evidence());
}

#[test]
fn test_parse_probe_output_unsupported() {
    // Only truly unknown commands should be Unsupported
    let result = parse_probe_output("some-unknown-command --flag", "some output");
    assert!(matches!(result, ParsedProbeData::Unsupported));
}

#[test]
fn test_parse_probe_result() {
    let probe = ProbeResult {
        command: "free -h".to_string(),
        exit_code: 0,
        stdout: r#"              total        used        free      shared  buff/cache   available
Mem:           15Gi       8.2Gi       1.5Gi       512Mi       5.8Gi       6.5Gi
Swap:         4.0Gi       256Mi       3.8Gi
"#
        .to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };
    let result = parse_probe_result(&probe);
    assert!(matches!(result, ParsedProbeData::Memory(_)));
}

#[test]
fn test_parse_probe_result_failed() {
    let probe = ProbeResult {
        command: "free -h".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: "command not found".to_string(),
        timing_ms: 10,
    };
    let result = parse_probe_result(&probe);
    assert!(result.is_error());
}

#[test]
fn test_tool_exists_positive_evidence() {
    let probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 0,
        stdout: "/usr/bin/nano\n".to_string(),
        stderr: String::new(),
        timing_ms: 5,
    };
    let result = parse_probe_result(&probe);
    assert!(matches!(result, ParsedProbeData::Tool(_)));
    if let ParsedProbeData::Tool(ref t) = result {
        assert_eq!(t.name, "nano");
        assert!(t.exists);
        assert_eq!(t.method, ToolExistsMethod::CommandV);
        assert_eq!(t.path, Some("/usr/bin/nano".to_string()));
    }
}

#[test]
fn test_tool_exists_negative_evidence() {
    // v0.45.7: exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
    let probe = ProbeResult {
        command: "sh -lc 'command -v nano'".to_string(),
        exit_code: 1,
        stdout: String::new(),
        stderr: String::new(),
        timing_ms: 5,
    };
    let result = parse_probe_result(&probe);
    assert!(matches!(result, ParsedProbeData::Tool(_)));
    if let ParsedProbeData::Tool(ref t) = result {
        assert_eq!(t.name, "nano");
        assert!(!t.exists); // Negative evidence!
        assert!(t.path.is_none());
    }
    // Must NOT be an error
    assert!(!result.is_error());
    assert!(result.is_valid_evidence());
}

#[test]
fn test_package_installed_positive_evidence() {
    let probe = ProbeResult {
        command: "pacman -Q nano 2>/dev/null".to_string(),
        exit_code: 0,
        stdout: "nano 7.2-1\n".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };
    let result = parse_probe_result(&probe);
    assert!(matches!(result, ParsedProbeData::Package(_)));
    if let ParsedProbeData::Package(ref p) = result {
        assert_eq!(p.name, "nano");
        assert!(p.installed);
        assert_eq!(p.version, Some("7.2-1".to_string()));
    }
}

#[test]
fn test_find_tool_evidence() {
    let parsed = vec![
        ParsedProbeData::Tool(ToolExists {
            name: "nano".to_string(),
            exists: true,
            method: ToolExistsMethod::CommandV,
            path: Some("/usr/bin/nano".to_string()),
        }),
        ParsedProbeData::Tool(ToolExists {
            name: "vim".to_string(),
            exists: false,
            method: ToolExistsMethod::CommandV,
            path: None,
        }),
    ];

    let nano = find_tool_evidence(&parsed, "nano");
    assert!(nano.is_some());
    assert!(nano.unwrap().exists);

    let vim = find_tool_evidence(&parsed, "vim");
    assert!(vim.is_some());
    assert!(!vim.unwrap().exists);

    let emacs = find_tool_evidence(&parsed, "emacs");
    assert!(emacs.is_none());
}

#[test]
fn test_v055_lspci_audio_with_class_code() {
    // Real-world lspci output with PCI class code [XXXX]
    let output = "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS (rev 10)";
    let devices = parse_lspci_audio_output(output);
    assert_eq!(devices.len(), 1, "Should find one audio device");
    assert!(
        devices[0].description.contains("Intel"),
        "Description should contain Intel"
    );
    assert_eq!(devices[0].pci_slot, Some("00:1f.3".to_string()));
}

#[test]
fn test_v066_audio_evidence_from_lspci_probe() {
    // Simulate lspci | grep -i audio probe with exit_code=0
    let probe = ProbeResult {
        command: "lspci | grep -i audio".to_string(),
        exit_code: 0,
        stdout: "00:1f.3 Multimedia audio controller [0403]: Intel Corporation Cannon Lake PCH cAVS (rev 10)".to_string(),
        stderr: String::new(),
        timing_ms: 10,
    };

    let parsed = parse_probe_result(&probe);
    assert!(
        parsed.as_audio().is_some(),
        "Should parse as Audio evidence"
    );

    let audio = parsed.as_audio().unwrap();
    assert_eq!(audio.devices.len(), 1, "Should have one device");
    assert!(
        audio.devices[0].description.contains("Intel"),
        "Description should contain Intel"
    );
}

#[test]
fn test_find_audio_evidence_prefers_lspci() {
    // When both lspci and pactl have devices, lspci should be preferred
    let lspci = ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "Intel Cannon Lake".to_string(),
            pci_slot: Some("00:1f.3".to_string()),
            vendor: Some("Intel".to_string()),
        }],
        source: "lspci".to_string(),
    });

    let pactl = ParsedProbeData::Audio(AudioDevices {
        devices: vec![AudioDevice {
            description: "alsa_card.pci-0000_00_1f.3".to_string(),
            pci_slot: None,
            vendor: None,
        }],
        source: "pactl".to_string(),
    });

    let parsed = vec![lspci, pactl];
    let merged = find_audio_evidence(&parsed);
    assert!(merged.is_some());

    let audio = merged.unwrap();
    assert!(!audio.devices.is_empty(), "Should have devices");
    // lspci device should be present (has PCI slot)
    assert!(audio.devices.iter().any(|d| d.pci_slot.is_some()));
}
