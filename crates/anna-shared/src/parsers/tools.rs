//! Tool and package existence parsing (v0.0.173, v0.0.409 robustness fixes).

use crate::rpc::ProbeResult;

use super::atoms::{ParseError, ParseErrorReason};
use super::evidence::{PackageInstalled, ToolExists, ToolExistsMethod};
use super::parsed_data::ParsedProbeData;

/// Try to parse a tool existence probe (v0.45.7, v0.0.57 hardening, v0.0.409 robustness).
/// Handles `command -v`, `which`, and `type` commands.
/// Returns Some if this is a tool check probe, None otherwise.
///
/// v0.0.57: exit_code=127 ("command not found") is an ERROR, not valid evidence.
/// Only exit_code=0 (found) and exit_code=1 (not found) are valid evidence.
///
/// v0.0.409: Never return "unknown" as a tool name. Return error instead.
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

        // v0.0.409: Return error if we can't extract the tool name
        let name = match extract_tool_name_from_command_v(&probe.command) {
            Some(n) => n,
            None => {
                return Some(ParsedProbeData::Error(ParseError::new(
                    &probe.command,
                    ParseErrorReason::MissingSection(
                        "could not extract tool name from command".to_string(),
                    ),
                    &probe.stderr,
                )));
            }
        };
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

        // v0.0.409: Return error if we can't extract the tool name
        let name = match probe.command.split_whitespace().nth(1) {
            Some(n) if !n.is_empty() && n != "2>/dev/null" => n.to_string(),
            _ => {
                return Some(ParsedProbeData::Error(ParseError::new(
                    &probe.command,
                    ParseErrorReason::MissingSection(
                        "could not extract tool name from which command".to_string(),
                    ),
                    &probe.stderr,
                )));
            }
        };
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

/// Try to parse a package installation probe (v0.45.7, v0.0.409 robustness).
/// Handles `pacman -Q` commands.
/// v0.0.409: Never returns "unknown" as a package name. Returns error instead.
pub fn try_parse_package_installed(
    probe: &ProbeResult,
    cmd_lower: &str,
) -> Option<ParsedProbeData> {
    // Pattern: "pacman -Q <name>" or "pacman -Q <name> 2>/dev/null"
    // Note: cmd_lower is already lowercase, so we check for lowercase -q
    if cmd_lower.contains("pacman -q") {
        // v0.0.409: Return error if we can't extract the package name
        let name = match extract_package_name_from_pacman(&probe.command) {
            Some(n) => n,
            None => {
                return Some(ParsedProbeData::Error(ParseError::new(
                    &probe.command,
                    ParseErrorReason::MissingSection(
                        "could not extract package name from pacman command".to_string(),
                    ),
                    &probe.stderr,
                )));
            }
        };
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
/// v0.0.409: Returns None instead of "unknown" fallback
pub fn extract_tool_name_from_command_v(cmd: &str) -> Option<String> {
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
            return Some(name);
        }
    }
    None
}

/// Extract package name from "pacman -Q <name>" command
/// v0.0.409: Returns None instead of "unknown" fallback
/// v0.0.797: Fixed to reject file descriptor redirections (e.g., "2" from "pacman -Q 2>/dev/null")
pub fn extract_package_name_from_pacman(cmd: &str) -> Option<String> {
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
            // v0.0.797: Reject single-digit names followed by redirection (e.g., "2" from "2>/dev/null")
            // Valid package names must be at least 2 characters long
            if !name.is_empty() && name.len() >= 2 {
                // Also reject if the name is followed by '>' (file descriptor redirection)
                let after_name = &trimmed[name.len()..];
                if !after_name.starts_with('>') {
                    return Some(name);
                }
            }
        }
    }
    None
}
