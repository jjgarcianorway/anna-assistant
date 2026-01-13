//! Structured Output Parsing - Parse JSON from system commands.
//!
//! Many modern Linux tools support JSON output:
//! - `ip -j` for network interfaces
//! - `ss -j` for socket statistics
//! - `lsblk -J` for block devices
//! - `systemctl show --output=json` for service info
//! - `pacman -Qi --json` (future)
//!
//! This module provides type-safe parsing for these outputs.
//!
//! v0.3.15: Initial implementation

mod network;
mod storage;
mod systemd;

pub use network::{parse_ip_output, parse_ss_output, IpAddress, NetworkInterface, SocketInfo};
pub use storage::{parse_lsblk_output, BlockDevice, Partition};
pub use systemd::{parse_systemctl_output, ServiceInfo, ServiceState};

use serde::de::DeserializeOwned;

/// Result of parsing structured output
#[derive(Debug)]
pub enum ParseResult<T> {
    /// Successfully parsed
    Ok(T),
    /// Command doesn't support JSON, got raw text
    RawText(String),
    /// JSON parse error
    ParseError(String),
    /// Command failed
    CommandError(String),
}

impl<T> ParseResult<T> {
    pub fn is_ok(&self) -> bool {
        matches!(self, ParseResult::Ok(_))
    }

    pub fn ok(self) -> Option<T> {
        match self {
            ParseResult::Ok(v) => Some(v),
            _ => None,
        }
    }
}

/// Parse JSON output from a command
pub fn parse_json<T: DeserializeOwned>(output: &str) -> ParseResult<T> {
    // Try to detect if this is JSON
    let trimmed = output.trim();
    if !trimmed.starts_with('[') && !trimmed.starts_with('{') {
        return ParseResult::RawText(output.to_string());
    }

    match serde_json::from_str::<T>(trimmed) {
        Ok(parsed) => ParseResult::Ok(parsed),
        Err(e) => ParseResult::ParseError(e.to_string()),
    }
}

/// Get the JSON flag for a command
pub fn json_flag_for(command: &str) -> Option<&'static str> {
    let base = command.split_whitespace().next()?;

    match base {
        "ip" => Some("-j"),
        "ss" => Some("-j"),
        "lsblk" => Some("-J"),
        "findmnt" => Some("-J"),
        "lscpu" => Some("-J"),
        "hostnamectl" => Some("--json=short"),
        "timedatectl" => Some("--output=json"),
        "loginctl" => Some("--output=json"),
        "journalctl" => Some("-o json"),
        "resolvectl" => Some("--json=short"),
        _ => None,
    }
}

/// Enhance a command with JSON output if supported
pub fn with_json_output(command: &str) -> String {
    if let Some(flag) = json_flag_for(command) {
        // Insert JSON flag after the command name
        let parts: Vec<&str> = command.splitn(2, ' ').collect();
        if parts.len() == 2 {
            format!("{} {} {}", parts[0], flag, parts[1])
        } else {
            format!("{} {}", command, flag)
        }
    } else {
        command.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_flag() {
        assert_eq!(json_flag_for("ip addr"), Some("-j"));
        assert_eq!(json_flag_for("ss -tuln"), Some("-j"));
        assert_eq!(json_flag_for("lsblk"), Some("-J"));
        assert_eq!(json_flag_for("cat /etc/passwd"), None);
    }

    #[test]
    fn test_with_json_output() {
        assert_eq!(with_json_output("ip addr"), "ip -j addr");
        assert_eq!(with_json_output("lsblk"), "lsblk -J");
        assert_eq!(with_json_output("cat /etc/passwd"), "cat /etc/passwd");
    }

    #[test]
    fn test_parse_json() {
        let json = r#"{"name": "test"}"#;
        let result: ParseResult<serde_json::Value> = parse_json(json);
        assert!(result.is_ok());

        let text = "not json";
        let result: ParseResult<serde_json::Value> = parse_json(text);
        assert!(matches!(result, ParseResult::RawText(_)));
    }
}
