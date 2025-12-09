//! Package and tool check handlers (v0.0.176).

use anna_shared::rpc::ProbeResult;

use super::DeterministicResult;
use crate::parsers::find_probe;

/// Answer package count query using pacman_count probe
pub fn answer_package_count(probes: &[ProbeResult], route_class: &str) -> Option<DeterministicResult> {
    let probe = find_probe(probes, "pacman")?;
    if probe.exit_code != 0 {
        return None;
    }

    // Parse package count from output
    let count: usize = probe.stdout.trim().parse().ok()?;

    let answer = format!("You have {} packages installed.", count);

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: 1,
        route_class: route_class.to_string(),
    })
}

/// Answer installed tool check query using typed evidence (v0.45.7)
/// Handles both command_v probes AND pacman -Q probes.
/// Exit code 1 is VALID NEGATIVE EVIDENCE, not an error!
pub fn answer_installed_tool_check(
    probes: &[ProbeResult],
    route_class: &str,
) -> Option<DeterministicResult> {
    use anna_shared::parsers::{parse_probe_result, ParsedProbeData};

    // v0.45.7: Parse all probes to typed evidence
    let parsed: Vec<ParsedProbeData> = probes.iter().map(parse_probe_result).collect();

    // Find tool existence evidence (from command -v, which, etc.)
    let tool_evidence: Vec<_> = parsed.iter().filter_map(|p| p.as_tool()).collect();

    // Find package evidence (from pacman -Q, etc.)
    let package_evidence: Vec<_> = parsed.iter().filter_map(|p| p.as_package()).collect();

    // If we have any tool or package evidence, we can answer
    if tool_evidence.is_empty() && package_evidence.is_empty() {
        return None;
    }

    // Build answer from evidence
    let mut answer_parts: Vec<String> = Vec::new();

    // Report tool evidence first (prefer this over package evidence)
    for tool in &tool_evidence {
        if tool.exists {
            let path_info = tool
                .path
                .as_ref()
                .map(|p| format!(" at `{}`", p))
                .unwrap_or_default();
            answer_parts.push(format!("Yes, **{}** is installed{}", tool.name, path_info));
        } else {
            answer_parts.push(format!("**{}** is not found in your PATH", tool.name));
        }
    }

    // Report package evidence
    for pkg in &package_evidence {
        if pkg.installed {
            let version_info = pkg
                .version
                .as_ref()
                .map(|v| format!(" (version {})", v))
                .unwrap_or_default();
            answer_parts.push(format!("**{}** is installed{}", pkg.name, version_info));
        } else {
            answer_parts.push(format!("**{}** package is not installed", pkg.name));
        }
    }

    // Combine evidence (may have both tool and package for same name)
    let answer = if answer_parts.len() == 1 {
        answer_parts[0].clone()
    } else {
        answer_parts.join("\n")
    };

    Some(DeterministicResult {
        answer,
        grounded: true,
        parsed_data_count: tool_evidence.len() + package_evidence.len(),
        route_class: route_class.to_string(),
    })
}
