//! Package-related intent handlers.

use crate::strict_contract::StrictSpecialistResponse;
use serde_json::json;
use std::collections::HashMap;

use super::types::HandlerResult;

/// Handle check_package_count intent
pub fn handle_check_package_count(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let package_count = match probes.get("package_count") {
        Some(p) if !p.trim().is_empty() => p.trim(),
        _ => {
            return HandlerResult::MissingProbe {
                probe_name: "package_count".to_string(),
                reason: "Need 'pacman -Qq | wc -l' output".to_string(),
            }
        }
    };

    // Parse the count
    match package_count.parse::<u32>() {
        Ok(count) => {
            let summary = format!("You have {} packages installed", count);
            HandlerResult::Success(
                StrictSpecialistResponse::ok(ticket_id, "check_package_count", &summary, 0.95)
                    .with_evidence("package_count", &format!("{} packages", count))
                    .with_metrics(json!({ "package_count": count })),
            )
        }
        Err(_) => {
            // Might be multi-line, try first line
            if let Ok(count) = package_count
                .lines()
                .next()
                .unwrap_or("0")
                .trim()
                .parse::<u32>()
            {
                let summary = format!("You have {} packages installed", count);
                HandlerResult::Success(
                    StrictSpecialistResponse::ok(ticket_id, "check_package_count", &summary, 0.95)
                        .with_evidence("package_count", &format!("{} packages", count))
                        .with_metrics(json!({ "package_count": count })),
                )
            } else {
                HandlerResult::NeedsSpecialist {
                    reason: format!("Could not parse package count: {}", package_count),
                }
            }
        }
    }
}

/// Handle check_package_installed intent
pub fn handle_check_package_installed(
    ticket_id: &str,
    package_name: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    // Look for package_check_<name> probe
    let probe_key = format!("package_check_{}", package_name.to_lowercase());

    let probe_output = match probes.get(&probe_key) {
        Some(p) => p.trim(),
        None => {
            // Also try installed_packages probe
            if let Some(installed) = probes.get("installed_packages") {
                if installed
                    .lines()
                    .any(|l| l.trim().starts_with(package_name))
                {
                    return HandlerResult::Success(
                        StrictSpecialistResponse::ok(
                            ticket_id,
                            "check_package_installed",
                            &format!("Yes, {} is installed", package_name),
                            0.9,
                        )
                        .with_evidence("installed_packages", "Found in package list"),
                    );
                }
            }
            return HandlerResult::MissingProbe {
                probe_name: probe_key,
                reason: format!("Need 'pacman -Q {}' output", package_name),
            };
        }
    };

    if probe_output.is_empty() {
        // Empty output means not installed
        let summary = format!("No, {} is not installed", package_name);
        HandlerResult::Success(
            StrictSpecialistResponse::ok(ticket_id, "check_package_installed", &summary, 0.95)
                .with_evidence(&probe_key, "Package not found")
                .with_metrics(json!({ "installed": false })),
        )
    } else {
        // Has output - parse version
        let version = probe_output
            .split_whitespace()
            .nth(1)
            .unwrap_or("installed");
        let summary = format!("Yes, {} {} is installed", package_name, version);
        HandlerResult::Success(
            StrictSpecialistResponse::ok(ticket_id, "check_package_installed", &summary, 0.95)
                .with_evidence(&probe_key, probe_output)
                .with_metrics(json!({ "installed": true, "version": version })),
        )
    }
}
