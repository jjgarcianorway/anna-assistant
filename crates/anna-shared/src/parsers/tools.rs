//! Tool and package existence parsing (v0.0.173).

use crate::rpc::ProbeResult;

use super::atoms::{ParseError, ParseErrorReason};
use super::evidence::{PackageInstalled, ToolExists, ToolExistsMethod};
use super::parsed_data::ParsedProbeData;

/// Try to parse a tool existence probe (v0.45.7, v0.0.57 hardening).
/// Handles `command -v`, `which`, and `type` commands.
/// Returns Some if this is a tool check probe, None otherwise.
///
/// v0.0.57: exit_code=127 ("command not found") is an ERROR, not valid evidence.
/// Only exit_code=0 (found) and exit_code=1 (not found) are valid evidence.
pub fn try_parse_tool_exists(probe: &ProbeResult, cmd_lower: &str) -> Option<ParsedProbeData> {
    // Pattern: "command -v <name>" or "sh -lc 'command -v <name>'"
    if cmd_lower.contains("command -v") {
        // v0.0.57: exit_code=127 means the shell itself failed - this is an error
        if probe.exit_code == 127 {
            return Some(ParsedProbeData::Error(ParseError::new(
                &probe.command,
                ParseErrorReason::MissingSection("shell error: command not found".to_string()),
                &probe.stderr,
            )));
        }

        let name = extract_tool_name_from_command_v(&probe.command);
        let exists = probe.exit_code == 0;
        let path = if exists && !probe.stdout.trim().is_empty() {
            Some(probe.stdout.trim().to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Tool(ToolExists {
            name,
            exists,
            method: ToolExistsMethod::CommandV,
            path,
        }));
    }

    // Pattern: "which <name>"
    if cmd_lower.starts_with("which ") {
        // v0.0.57: exit_code=127 means the shell itself failed - this is an error
        if probe.exit_code == 127 {
            return Some(ParsedProbeData::Error(ParseError::new(
                &probe.command,
                ParseErrorReason::MissingSection("shell error: command not found".to_string()),
                &probe.stderr,
            )));
        }

        let name = probe
            .command
            .split_whitespace()
            .nth(1)
            .unwrap_or("unknown")
            .to_string();
        let exists = probe.exit_code == 0;
        let path = if exists && !probe.stdout.trim().is_empty() {
            Some(probe.stdout.trim().to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Tool(ToolExists {
            name,
            exists,
            method: ToolExistsMethod::Which,
            path,
        }));
    }

    None
}

/// Try to parse a package installation probe (v0.45.7).
/// Handles `pacman -Q` commands.
pub fn try_parse_package_installed(
    probe: &ProbeResult,
    cmd_lower: &str,
) -> Option<ParsedProbeData> {
    // Pattern: "pacman -Q <name>" or "pacman -Q <name> 2>/dev/null"
    // Note: cmd_lower is already lowercase, so we check for lowercase -q
    if cmd_lower.contains("pacman -q") {
        let name = extract_package_name_from_pacman(&probe.command);
        let installed = probe.exit_code == 0;
        let version = if installed {
            // pacman -Q outputs: "<name> <version>"
            probe
                .stdout
                .split_whitespace()
                .nth(1)
                .map(|v| v.to_string())
        } else {
            None
        };
        return Some(ParsedProbeData::Package(PackageInstalled {
            name,
            installed,
            version,
        }));
    }

    None
}

/// Extract tool name from "command -v <name>" or "sh -lc 'command -v <name>'"
pub fn extract_tool_name_from_command_v(cmd: &str) -> String {
    // Handle: sh -lc 'command -v nano'
    if let Some(pos) = cmd.find("command -v") {
        let rest = &cmd[pos + "command -v".len()..];
        let trimmed = rest.trim();
        // Extract the tool name (first alphanumeric word, stop at quotes/pipes)
        let name: String = trimmed
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !name.is_empty() {
            return name;
        }
    }
    "unknown".to_string()
}

/// Extract package name from "pacman -Q <name>" command
pub fn extract_package_name_from_pacman(cmd: &str) -> String {
    // Find -Q or -Qi and take the next word
    let cmd_lower = cmd.to_lowercase();
    for pattern in ["-q ", "-qi "] {
        if let Some(pos) = cmd_lower.find(pattern) {
            let rest = &cmd[pos + pattern.len()..];
            let trimmed = rest.trim();
            // Extract package name (stop at whitespace or redirection)
            let name: String = trimmed
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
                .collect();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "unknown".to_string()
}
