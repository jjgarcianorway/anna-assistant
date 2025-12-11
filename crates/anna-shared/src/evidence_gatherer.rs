//! Evidence Gatherer - Orchestrates evidence collection (v0.0.410).
//!
//! This module ties together:
//! - Probe registry (which probes to run)
//! - Doc fetchers (which docs to search)
//! - Recipe candidates (learned patterns)
//!
//! It produces an EvidenceBundle for specialist consumption.

use crate::doc_fetcher;
use crate::evidence_engine::{
    DocSnippet, EvidenceBundle, EvidenceDomain, EvidenceIntent, EvidenceRequest, ProbeEvidence,
    RecipeMatch,
};
use crate::probe_registry::{ProbeDef, ProbeRegistry};
use crate::recipe_candidate::RecipeCandidateStore;
use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Maximum probes to run per request
const MAX_PROBES: usize = 6;

/// Maximum docs to fetch per request
const MAX_DOCS: usize = 4;

/// Probe execution timeout (seconds)
const PROBE_TIMEOUT_SECS: u64 = 10;

/// Gather evidence for a request
pub fn gather_evidence(request: &EvidenceRequest) -> EvidenceBundle {
    let start = Instant::now();
    let mut bundle = EvidenceBundle::new(&request.ticket_id);

    info!(
        "Gathering evidence for ticket={} domain={} intent={} tags={:?}",
        request.ticket_id, request.domain, request.intent, request.tags
    );

    // 1. Select and run probes
    let registry = ProbeRegistry::new();
    let probes = registry.select(request.domain, request.intent, &request.tags, MAX_PROBES);

    debug!("Selected {} probes for execution", probes.len());

    for probe_def in probes {
        bundle.metadata.probes_run.push(probe_def.id.clone());

        if let Some(evidence) = execute_probe(probe_def) {
            bundle.add_probe(evidence);
        }
    }

    // 2. Fetch relevant documentation
    let docs = doc_fetcher::fetch_docs(&request.tags, MAX_DOCS);
    for source in ["arch_wiki", "man", "help", "doc"] {
        if docs.iter().any(|d| d.source.to_string() == source) {
            bundle.metadata.docs_searched.push(source.to_string());
        }
    }

    for doc in docs {
        bundle.add_doc(doc);
    }

    // 3. Look for matching recipe candidates
    let recipe_store = RecipeCandidateStore::load();
    let similar = recipe_store.find_similar(
        &request.domain.to_string(),
        &request.intent.to_string(),
        &request.tags,
    );

    for candidate in similar.iter().take(3) {
        bundle.add_recipe(RecipeMatch {
            id: candidate.id.clone(),
            title: format!(
                "Learned: {}",
                candidate.pattern_keywords.join(", ")
            ),
            summary: format!(
                "{} ({} confirmations)",
                candidate.intent,
                candidate.confirmations
            ),
            confidence: (candidate.confirmations * 30).min(95) as u8,
            actions: candidate.actions.iter().map(|a| a.description.clone()).collect(),
        });
    }

    // Record timing
    bundle.metadata.gather_time_ms = start.elapsed().as_millis() as u64;

    info!(
        "Gathered {} probes, {} docs, {} recipes in {}ms",
        bundle.probes.len(),
        bundle.docs.len(),
        bundle.recipes.len(),
        bundle.metadata.gather_time_ms
    );

    bundle
}

/// Execute a single probe
fn execute_probe(probe: &ProbeDef) -> Option<ProbeEvidence> {
    debug!("Executing probe: {} ({})", probe.id, probe.command);

    let output = Command::new("sh")
        .args(["-c", &probe.command])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();

            if stdout.is_empty() && stderr.is_empty() {
                debug!("Probe {} produced no output", probe.id);
                return None;
            }

            // Generate summary and excerpt
            let (summary, excerpt) = parse_probe_output(&stdout, &probe);

            Some(
                ProbeEvidence::new(&probe.id, &probe.command, &summary, &excerpt)
                    .with_exit_code(out.status.code().unwrap_or(-1)),
            )
        }
        Err(e) => {
            warn!("Probe {} failed: {}", probe.id, e);
            None
        }
    }
}

/// Parse probe output to summary and excerpt
fn parse_probe_output(output: &str, probe: &ProbeDef) -> (String, String) {
    let trimmed = output.trim();

    // Handle specific probe types
    let summary = match probe.id.as_str() {
        "probe:df_root" | "probe:df_all" => {
            // Extract usage percentage
            if let Some(line) = trimmed.lines().skip(1).next() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    format!("{} {} used", parts.get(0).unwrap_or(&"root"), parts.get(4).unwrap_or(&"?%"))
                } else {
                    "Disk usage data".to_string()
                }
            } else {
                "Disk usage data".to_string()
            }
        }
        "probe:memory" => {
            // Extract used/total
            if let Some(line) = trimmed.lines().find(|l| l.starts_with("Mem:")) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                format!(
                    "Memory: {} used of {}",
                    parts.get(2).unwrap_or(&"?"),
                    parts.get(1).unwrap_or(&"?")
                )
            } else {
                "Memory usage data".to_string()
            }
        }
        "probe:pacman_count" => {
            format!("{} packages installed", trimmed)
        }
        "probe:systemctl_failed" => {
            let failed_count = trimmed.lines().filter(|l| l.contains("failed")).count();
            if failed_count == 0 {
                "No failed units".to_string()
            } else {
                format!("{} failed unit(s)", failed_count)
            }
        }
        "probe:sensors" => {
            // Look for temperature values
            let temps: Vec<&str> = trimmed
                .lines()
                .filter(|l| l.contains("°C") || l.contains("temp"))
                .take(3)
                .collect();
            if temps.is_empty() {
                "Sensor data".to_string()
            } else {
                "Temperature sensors found".to_string()
            }
        }
        "probe:uptime" => {
            // Extract load average
            if let Some(pos) = trimmed.find("load average:") {
                format!("Load: {}", &trimmed[pos + 14..].trim())
            } else {
                "Uptime data".to_string()
            }
        }
        _ => {
            // Generic: first line or type
            trimmed.lines().next().unwrap_or("Output available").to_string()
        }
    };

    // Excerpt: first few lines, cleaned up
    let excerpt = trimmed
        .lines()
        .take(10)
        .collect::<Vec<_>>()
        .join("\n");

    let excerpt = truncate(&excerpt, 400);

    (summary, excerpt)
}

/// Truncate string
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Build evidence request from translator output
pub fn build_evidence_request(
    ticket_id: &str,
    domain_str: &str,
    intent_str: &str,
    question: &str,
    tags: Vec<String>,
) -> EvidenceRequest {
    let domain = EvidenceDomain::from_str(domain_str).unwrap_or(EvidenceDomain::System);
    let intent = EvidenceIntent::from_str(intent_str).unwrap_or(EvidenceIntent::Diagnose);

    EvidenceRequest {
        ticket_id: ticket_id.to_string(),
        domain,
        intent,
        question: question.to_string(),
        tags,
    }
}

/// Quick evidence check (cheap probes only)
pub fn quick_evidence(domain: EvidenceDomain, tags: &[String]) -> Vec<ProbeEvidence> {
    let registry = ProbeRegistry::new();
    let probes = registry.select(domain, EvidenceIntent::Inspect, tags, 3);

    probes
        .into_iter()
        .filter(|p| p.cost == crate::probe_registry::ProbeCost::Cheap)
        .filter_map(execute_probe)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_evidence_request() {
        let request = build_evidence_request(
            "TEST-001",
            "storage",
            "diagnose",
            "Why is my disk full?",
            vec!["disk".to_string(), "space".to_string()],
        );

        assert_eq!(request.ticket_id, "TEST-001");
        assert_eq!(request.domain, EvidenceDomain::Storage);
        assert_eq!(request.intent, EvidenceIntent::Diagnose);
    }

    #[test]
    fn test_parse_probe_output_df() {
        let output = "Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   75G   25G  75% /";
        let probe = crate::probe_registry::ProbeDef {
            id: "probe:df_root".into(),
            command: "df -h /".into(),
            description: "Root usage".into(),
            domains: vec![EvidenceDomain::Storage],
            tags: vec![],
            cost: crate::probe_registry::ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        };

        let (summary, excerpt) = parse_probe_output(output, &probe);
        assert!(summary.contains("75%") || summary.contains("/dev/sda1"));
        assert!(excerpt.contains("/dev/sda1"));
    }

    #[test]
    fn test_parse_probe_output_memory() {
        let output = "              total        used        free\nMem:           16Gi       8.0Gi       8.0Gi\nSwap:         8.0Gi          0B       8.0Gi";
        let probe = crate::probe_registry::ProbeDef {
            id: "probe:memory".into(),
            command: "free -h".into(),
            description: "Memory".into(),
            domains: vec![EvidenceDomain::Performance],
            tags: vec![],
            cost: crate::probe_registry::ProbeCost::Cheap,
            intents: vec![],
            parse_hint: None,
        };

        let (summary, _) = parse_probe_output(output, &probe);
        assert!(summary.contains("Memory"));
    }
}
