//! Service-related intent handlers.

use crate::strict_contract::StrictSpecialistResponse;
use serde_json::json;
use std::collections::HashMap;

use super::types::HandlerResult;

/// Handle check_failed_services intent
pub fn handle_check_failed_services(
    ticket_id: &str,
    probes: &HashMap<String, String>,
) -> HandlerResult {
    let failed_services = match probes.get("failed_services") {
        Some(f) => f,
        None => {
            return HandlerResult::MissingProbe {
                probe_name: "failed_services".to_string(),
                reason: "Need 'systemctl --failed' output".to_string(),
            }
        }
    };

    // Parse failed services - count lines that look like unit entries
    let failed_units: Vec<&str> = failed_services
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Look for unit names (ends with .service, .socket, etc.)
            trimmed.contains(".service")
                || trimmed.contains(".socket")
                || trimmed.contains(".timer")
                || trimmed.contains(".mount")
        })
        .filter(|line| {
            // Skip header/footer lines
            !line.contains("UNIT") && !line.contains("LOAD") && !line.contains("loaded units")
        })
        .collect();

    let count = failed_units.len();
    let summary = if count == 0 {
        "No failed systemd services".to_string()
    } else {
        let names: Vec<String> = failed_units
            .iter()
            .map(|line| {
                line.split_whitespace()
                    .next()
                    .unwrap_or("unknown")
                    .to_string()
            })
            .take(3)
            .collect();
        if count <= 3 {
            format!("{} failed service(s): {}", count, names.join(", "))
        } else {
            format!(
                "{} failed services: {} and {} more",
                count,
                names.join(", "),
                count - 3
            )
        }
    };

    let mut response =
        StrictSpecialistResponse::ok(ticket_id, "check_failed_services", &summary, 0.95)
            .with_evidence(
                "failed_services",
                &format!("{} failed units detected", count),
            )
            .with_metrics(json!({ "failed_count": count }));

    // Add action if there are failures
    if count > 0 {
        response
            .actions
            .push(crate::strict_contract::SuggestedAction {
                kind: crate::strict_contract::ActionKind::Investigate,
                description: "Check service status for details".to_string(),
                command: Some("systemctl status <service-name>".to_string()),
                risk: crate::strict_contract::RiskLevel::Low,
            });
    }

    HandlerResult::Success(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_failed_services_none() {
        let mut probes = HashMap::new();
        probes.insert("failed_services".to_string(), "".to_string());

        match handle_check_failed_services("TEST-001", &probes) {
            HandlerResult::Success(r) => {
                assert!(r.summary.contains("No failed"));
            }
            _ => panic!("Expected success"),
        }
    }
}
