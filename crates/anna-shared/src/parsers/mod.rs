//! Probe output parsers for STRUCT-lite phase (v0.0.173).
//!
//! This module provides deterministic parsers for common Linux command outputs.
//! All parsing uses integer arithmetic (no floats) and produces typed structs.
//!
//! # Supported Probes
//!
//! - `free -h`: Memory and swap information → `MemoryInfo`
//! - `df -h`: Disk usage per mount → `Vec<DiskUsage>`
//! - `systemctl --failed` / `systemctl is-active`: Service status
//! - `lsblk`: Block device information → `Vec<BlockDevice>`
//! - `lscpu`: CPU information → `CpuInfo`
//!
//! # Design Principles
//!
//! - **No floats**: All sizes are stored as `u64` bytes.
//! - **Exact rounding**: Size parsing uses rational arithmetic.
//! - **Binary prefixes**: K/M/G/T are treated as base-2 (1024).
//! - **Explicit errors**: Parse failures return `ParsedProbeData::Error`.

pub mod atoms;
pub mod audio;
pub mod df;
pub mod evidence;
pub mod free;
pub mod helpers;
pub mod journalctl;
pub mod lsblk;
pub mod lscpu;
pub mod parsed_data;
pub mod systemctl;
pub mod tools;

// Re-export main types
pub use atoms::{
    normalize_service_name, parse_display_size, parse_percent, parse_size, ParseError,
    ParseErrorReason,
};
pub use audio::{
    extract_pci_slot, extract_vendor_from_description, merge_audio_devices,
    parse_lspci_audio_output, parse_pactl_cards_output, try_parse_audio_devices,
};
pub use df::{find_by_mount, parse_df, resolve_mount_alias, DiskUsage};
pub use evidence::{AudioDevice, AudioDevices, PackageInstalled, ToolExists, ToolExistsMethod};
pub use free::{parse_free, MemoryInfo};
pub use helpers::{
    find_audio_evidence, find_audio_evidence_ref, find_package_evidence, find_tool_evidence,
    get_installed_tools, has_evidence_for, installed_editors_from_parsed,
};
pub use journalctl::{
    parse_boot_time,
    parse_failed_units as parse_journal_failed_units,
    parse_journalctl_priority,
    BootTimeInfo,
    FailedUnit as JournalFailedUnit, // Alias to avoid conflict with systemctl
    JournalSummary,
    JournalTopItem,
};
pub use lsblk::{find_root_device, parse_lsblk, total_disk_size, BlockDevice, BlockDeviceType};
pub use lscpu::{parse_lscpu, CpuInfo};
pub use parsed_data::ParsedProbeData;
pub use systemctl::{
    parse_failed_units, parse_is_active, parse_status_verbose, ServiceState, ServiceStatus,
};
pub use tools::{
    extract_package_name_from_pacman, extract_tool_name_from_command_v, try_parse_package_installed,
    try_parse_tool_exists,
};

use crate::rpc::ProbeResult;

/// Probe ID constants for matching.
pub mod probe_ids {
    pub const FREE: &str = "free -h";
    pub const DF: &str = "df -h";
    pub const SYSTEMCTL_FAILED: &str = "systemctl --failed";
    pub const TOP_MEMORY: &str = "ps aux --sort=-%mem";
    pub const TOP_CPU: &str = "ps aux --sort=-%cpu";
    pub const LSBLK: &str = "lsblk";
    pub const LSCPU: &str = "lscpu";
}

/// Count how many probes produced valid evidence (v0.0.56).
pub fn count_valid_evidence_probes(probes: &[ProbeResult]) -> usize {
    probes
        .iter()
        .filter(|p| parse_probe_result(p).is_valid_evidence())
        .count()
}

/// Check if a probe result produced valid evidence (v0.0.56).
pub fn is_probe_valid_evidence(probe: &ProbeResult) -> bool {
    parse_probe_result(probe).is_valid_evidence()
}

/// Parse a ProbeResult into structured data.
/// Returns `ParsedProbeData::Unsupported` for probes we don't have parsers for.
/// v0.45.7: Tool/package probes with exit code 1 are VALID NEGATIVE EVIDENCE, not errors!
/// v0.45.8: Audio probes from lspci/pactl are now parsed.
pub fn parse_probe_result(probe: &ProbeResult) -> ParsedProbeData {
    let cmd_lower = probe.command.to_lowercase();

    // v0.45.7: Handle tool existence probes - exit 1 = tool not found (valid evidence!)
    if let Some(parsed) = try_parse_tool_exists(probe, &cmd_lower) {
        return parsed;
    }

    // v0.45.7: Handle package probes - exit 1 = package not installed (valid evidence!)
    if let Some(parsed) = try_parse_package_installed(probe, &cmd_lower) {
        return parsed;
    }

    // v0.45.8: Handle audio probes - lspci audio and pactl
    if let Some(parsed) = try_parse_audio_devices(probe, &cmd_lower) {
        return parsed;
    }

    // For other probes, non-zero exit code is an error
    if probe.exit_code != 0 {
        return ParsedProbeData::Error(ParseError::new(
            &probe.command,
            ParseErrorReason::MissingSection(format!("exit code {}", probe.exit_code)),
            &probe.stderr,
        ));
    }

    parse_probe_output(&probe.command, &probe.stdout)
}

/// Parse probe output based on the command.
/// Returns `ParsedProbeData::Unsupported` for probes we don't have parsers for.
pub fn parse_probe_output(command: &str, stdout: &str) -> ParsedProbeData {
    // Normalize command for matching
    let cmd_lower = command.to_lowercase();

    if cmd_lower.starts_with("free") {
        match parse_free(command, stdout) {
            Ok(info) => ParsedProbeData::Memory(info),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("df") {
        match parse_df(command, stdout) {
            Ok(entries) => ParsedProbeData::Disk(entries),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("lsblk") {
        match parse_lsblk(command, stdout) {
            Ok(devices) => ParsedProbeData::BlockDevices(devices),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("lscpu") {
        match parse_lscpu(command, stdout) {
            Ok(info) => ParsedProbeData::Cpu(info),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("journalctl -p 3") {
        // v0.0.35: Journal errors (priority 3 = err)
        ParsedProbeData::JournalErrors(parse_journalctl_priority(stdout))
    } else if cmd_lower.starts_with("journalctl -p 4") {
        // v0.0.35: Journal warnings (priority 4 = warning)
        ParsedProbeData::JournalWarnings(parse_journalctl_priority(stdout))
    } else if cmd_lower.starts_with("systemd-analyze") {
        // v0.0.35: Boot time
        ParsedProbeData::BootTime(parse_boot_time(stdout))
    } else if cmd_lower.contains("systemctl") && cmd_lower.contains("--failed") {
        match parse_failed_units(command, stdout) {
            Ok(units) => ParsedProbeData::Services(units),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.contains("systemctl") && cmd_lower.contains("is-active") {
        // Extract service name from command
        let service_name = extract_service_from_is_active(&cmd_lower);
        match parse_is_active(command, &service_name, stdout) {
            Ok(status) => ParsedProbeData::Service(status),
            Err(e) => ParsedProbeData::Error(e),
        }
    } else if cmd_lower.starts_with("ps aux")
        || cmd_lower.contains("systemctl list-units")
        || cmd_lower.starts_with("ip ")
        || cmd_lower.starts_with("ss ")
        || cmd_lower.starts_with("who")
        || cmd_lower.starts_with("uptime")
        || cmd_lower.starts_with("uname")
        || cmd_lower.starts_with("cat /")
        || cmd_lower.starts_with("sensors")
        // v0.0.390: Package listing commands are valid evidence
        || cmd_lower.starts_with("pacman -q")
        || cmd_lower.starts_with("dpkg -l")
        || cmd_lower.starts_with("rpm -qa")
        || cmd_lower.starts_with("apk info")
        // v0.0.390: Directory size commands for largest folders
        || cmd_lower.starts_with("du ")
    {
        // v0.0.308: Treat common probes as valid raw evidence
        // These don't need structured parsing - the raw text is valid evidence
        ParsedProbeData::RawText(stdout.to_string())
    } else {
        ParsedProbeData::Unsupported
    }
}

/// Extract service name from "systemctl is-active <service>" command.
fn extract_service_from_is_active(cmd: &str) -> String {
    // Find "is-active" and take the next word
    if let Some(pos) = cmd.find("is-active") {
        let rest = &cmd[pos + "is-active".len()..];
        let trimmed = rest.trim();
        if let Some(name) = trimmed.split_whitespace().next() {
            return name.to_string();
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod atoms_tests;
#[cfg(test)]
mod journalctl_tests;
#[cfg(test)]
mod tests;
