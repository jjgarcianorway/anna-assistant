//! Probe bundling and merging utilities (v0.0.410).

use anna_shared::evidence_engine::{EvidenceBundle, ProbeEvidence};
use anna_shared::rpc::ProbeResult;

/// Build minimal bundle from existing probes
pub fn bundle_from_existing_probes(ticket_id: &str, probes: &[ProbeResult]) -> EvidenceBundle {
    let mut bundle = EvidenceBundle::new(ticket_id);

    for probe in probes {
        if probe.exit_code == 0 && !probe.stdout.is_empty() {
            let evidence = ProbeEvidence::new(
                &format!("probe:{}", sanitize_probe_name(&probe.command)),
                &probe.command,
                &summarize_probe_output(&probe.stdout),
                &truncate(&probe.stdout, 400),
            )
            .with_exit_code(probe.exit_code);

            bundle.add_probe(evidence);
        }
    }

    bundle
}

/// Merge evidence bundle with existing probes
pub fn merge_with_existing_probes(
    mut bundle: EvidenceBundle,
    probes: &[ProbeResult],
) -> EvidenceBundle {
    // Add existing probes that aren't already in the bundle
    for probe in probes {
        let probe_id = format!("probe:{}", sanitize_probe_name(&probe.command));

        if !bundle.probes.iter().any(|p| p.id == probe_id) {
            if probe.exit_code == 0 && !probe.stdout.is_empty() {
                let evidence = ProbeEvidence::new(
                    &probe_id,
                    &probe.command,
                    &summarize_probe_output(&probe.stdout),
                    &truncate(&probe.stdout, 400),
                )
                .with_exit_code(probe.exit_code);

                bundle.add_probe(evidence);
            }
        }
    }

    bundle
}

/// Sanitize probe name for use as ID
pub fn sanitize_probe_name(command: &str) -> String {
    command
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .replace(['/', '-'], "_")
}

/// Generate summary from probe output
fn summarize_probe_output(output: &str) -> String {
    // Take first non-empty line as summary
    output
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| truncate(l, 80))
        .unwrap_or_else(|| "Output available".to_string())
}

/// Truncate string
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_probe_name() {
        assert_eq!(sanitize_probe_name("df -h /"), "df");
        assert_eq!(sanitize_probe_name("systemctl status sshd"), "systemctl");
    }
}
