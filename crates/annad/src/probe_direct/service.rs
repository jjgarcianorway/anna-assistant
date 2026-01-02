//! Service status queries

use anna_shared::rpc::ProbeResult;
use regex::Regex;
use tracing::info;

use super::DirectAnswerResult;

/// Service status answer
pub(crate) fn try_service_answer(query: &str, probes: &[ProbeResult]) -> Option<DirectAnswerResult> {
    // Extract service name from query
    let service = extract_service_name(query)?;

    for probe in probes {
        let cmd = probe.command.to_lowercase();
        if !cmd.contains("systemctl") {
            continue;
        }
        if !cmd.contains(&service) && !cmd.contains("--all") && !cmd.contains("--failed") {
            continue;
        }

        let output = &probe.stdout;
        let stderr = &probe.stderr;

        // Check systemctl status output patterns
        if output.contains("Active: active (running)") {
            info!("v0.0.403: Direct service answer - running");
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **active and running**.", service),
                confidence: 95,
            });
        }

        if output.contains("Active: inactive") || output.contains("inactive (dead)") {
            return Some(DirectAnswerResult {
                answer: format!(
                    "**{}.service** is **inactive** (not running).\n\nTo start: `sudo systemctl start {}`",
                    service, service
                ),
                confidence: 95,
            });
        }

        if output.contains("Active: failed") {
            return Some(DirectAnswerResult {
                answer: format!(
                    "**{}.service** is in **failed** state.\n\nCheck logs: `journalctl -u {} -n 50`",
                    service, service
                ),
                confidence: 95,
            });
        }

        if output.contains("could not be found") || stderr.contains("could not be found") {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** does not exist on this system.", service),
                confidence: 95,
            });
        }

        // Simple is-active output
        let trimmed = output.trim();
        if trimmed == "active" {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **active**.", service),
                confidence: 90,
            });
        }
        if trimmed == "inactive" {
            return Some(DirectAnswerResult {
                answer: format!("**{}.service** is **inactive**.", service),
                confidence: 90,
            });
        }
    }

    None
}

/// Extract service name from query
fn extract_service_name(query: &str) -> Option<String> {
    // Common service name patterns
    let patterns = [
        r"(?:is|check|status)\s+(\w+)(?:\.service)?\s+(?:running|active|started)",
        r"(\w+)(?:\.service)?\s+(?:running|active|status|started)",
        r"(?:running|active|started)\s+(\w+)(?:\.service)?",
    ];

    for pattern in &patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(query) {
                if let Some(m) = caps.get(1) {
                    return Some(m.as_str().to_string());
                }
            }
        }
    }

    // Check for common service keywords
    for svc in [
        "bluetooth",
        "docker",
        "nginx",
        "ssh",
        "sshd",
        "cups",
        "pipewire",
        "pulseaudio",
        "networkmanager",
        "apache",
        "mysql",
        "postgresql",
        "redis",
    ] {
        if query.contains(svc) {
            return Some(svc.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_extraction() {
        assert_eq!(
            extract_service_name("is bluetooth running"),
            Some("bluetooth".to_string())
        );
        assert_eq!(
            extract_service_name("docker status"),
            Some("docker".to_string())
        );
        assert_eq!(
            extract_service_name("check nginx service"),
            Some("nginx".to_string())
        );
    }
}
