//! Probe output parsers for STRUCT-lite phase.
//!
//! This module provides deterministic parsers for common Linux command outputs.
//! All parsing uses integer arithmetic (no floats) and produces typed structs.
//!
//! # Supported Probes
//!
//! - `free -h`: Memory and swap information → `MemoryInfo`
//! - `df -h`: Disk usage per mount → `Vec<DiskUsage>`
//! - `systemctl --failed` / `systemctl is-active`: Service status → `Vec<ServiceStatus>` / `ServiceStatus`
//!
//! # Design Principles
//!
//! - **No floats**: All sizes are stored as `u64` bytes.
//! - **Exact rounding**: Size parsing uses rational arithmetic with deterministic tie-breaking.
//! - **Binary prefixes**: K/M/G/T are treated as base-2 (1024), matching Linux tool defaults.
//! - **Explicit errors**: Parse failures return `ParsedProbeData::Error` with context.

pub mod atoms;
pub mod df;
pub mod free;
pub mod systemctl;

// Re-export main types
pub use atoms::{
    normalize_service_name, parse_display_size, parse_percent, parse_size, ParseError,
    ParseErrorReason,
};
pub use df::{find_by_mount, parse_df, resolve_mount_alias, DiskUsage};
pub use free::{parse_free, MemoryInfo};
pub use systemctl::{
    parse_failed_units, parse_is_active, parse_status_verbose, ServiceState, ServiceStatus,
};

use crate::rpc::ProbeResult;
use serde::{Deserialize, Serialize};

/// Parsed probe data or error.
/// Used internally for enrichment; not serialized over the wire.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParsedProbeData {
    /// Memory info from `free -h`
    Memory(MemoryInfo),
    /// Disk usage from `df -h`
    Disk(Vec<DiskUsage>),
    /// Service status list (from `systemctl --failed` or similar)
    Services(Vec<ServiceStatus>),
    /// Single service status (from `systemctl is-active`)
    Service(ServiceStatus),
    /// Parse error with diagnostic context
    Error(ParseError),
    /// Probe type not supported for structured parsing
    Unsupported,
}

impl ParsedProbeData {
    /// Check if this is an error variant.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Get the error if this is an error variant.
    pub fn as_error(&self) -> Option<&ParseError> {
        match self {
            Self::Error(e) => Some(e),
            _ => None,
        }
    }

    /// Get memory info if this is the Memory variant.
    pub fn as_memory(&self) -> Option<&MemoryInfo> {
        match self {
            Self::Memory(m) => Some(m),
            _ => None,
        }
    }

    /// Get disk usage if this is the Disk variant.
    pub fn as_disk(&self) -> Option<&Vec<DiskUsage>> {
        match self {
            Self::Disk(d) => Some(d),
            _ => None,
        }
    }

    /// Get services if this is the Services variant.
    pub fn as_services(&self) -> Option<&Vec<ServiceStatus>> {
        match self {
            Self::Services(s) => Some(s),
            _ => None,
        }
    }

    /// Get single service if this is the Service variant.
    pub fn as_service(&self) -> Option<&ServiceStatus> {
        match self {
            Self::Service(s) => Some(s),
            _ => None,
        }
    }
}

/// Probe ID constants for matching.
pub mod probe_ids {
    pub const FREE: &str = "free -h";
    pub const DF: &str = "df -h";
    pub const SYSTEMCTL_FAILED: &str = "systemctl --failed";
    pub const TOP_MEMORY: &str = "ps aux --sort=-%mem";
    pub const TOP_CPU: &str = "ps aux --sort=-%cpu";
}

/// Parse a ProbeResult into structured data.
/// Returns `ParsedProbeData::Unsupported` for probes we don't have parsers for.
/// Returns `ParsedProbeData::Error` if the probe failed (non-zero exit code).
pub fn parse_probe_result(probe: &ProbeResult) -> ParsedProbeData {
    // Failed probes return an error with the stderr
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
mod tests {
    use super::*;

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
    fn test_parse_probe_output_unsupported() {
        let result = parse_probe_output("ps aux --sort=-%mem", "some output");
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
    fn test_parsed_probe_data_accessors() {
        let mem = ParsedProbeData::Memory(MemoryInfo {
            total_bytes: 1024,
            used_bytes: 512,
            free_bytes: 256,
            shared_bytes: 0,
            buff_cache_bytes: 256,
            available_bytes: 512,
            swap_total_bytes: None,
            swap_used_bytes: None,
            swap_free_bytes: None,
        });
        assert!(mem.as_memory().is_some());
        assert!(mem.as_disk().is_none());
        assert!(!mem.is_error());

        let err = ParsedProbeData::Error(ParseError::new(
            "test",
            ParseErrorReason::MalformedRow,
            "bad",
        ));
        assert!(err.is_error());
        assert!(err.as_error().is_some());
    }
}
