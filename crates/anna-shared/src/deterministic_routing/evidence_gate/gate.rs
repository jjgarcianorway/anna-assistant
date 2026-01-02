//! Evidence Gate - v0.0.439.
//!
//! Main evidence gating logic.
//!
//! Rules:
//! - If required_evidence probes are missing or failed: do not call specialist.
//! - If probes already provide a direct factual answer: do not call specialist.
//! - Only call specialists when synthesis is needed (why, root cause, recommendations).

use std::collections::HashMap;

use super::extractors::*;
use super::types::{DirectAnswer, EvidenceStatus, GateDecision, ProbeResult};
use crate::deterministic_routing::intent_map_table::IntentMapTable;
use crate::deterministic_routing::intent_mapping::IntentMapping;
use crate::deterministic_routing::intent_schema::{CanonicalIntent, Department, TicketIntentSchema};

/// Evidence gate that decides whether to call specialist.
pub struct EvidenceGate {
    /// Intent map for probe requirements.
    intent_map: IntentMapTable,
}

impl EvidenceGate {
    /// Create a new evidence gate.
    pub fn new() -> Self {
        Self {
            intent_map: IntentMapTable::build(),
        }
    }

    /// Evaluate whether we can proceed.
    pub fn evaluate(
        &self,
        schema: &TicketIntentSchema,
        probes: &HashMap<String, ProbeResult>,
    ) -> GateDecision {
        // Get mapping for this intent
        let mapping = match self.intent_map.get(schema.intent) {
            Some(m) => m,
            None => {
                return GateDecision::NeedClarification {
                    question: "I couldn't understand your question. Could you rephrase it?"
                        .to_string(),
                };
            }
        };

        // Check if clarification is needed
        if schema.need_clarification {
            return GateDecision::NeedClarification {
                question: schema
                    .clarifying_question
                    .clone()
                    .unwrap_or_else(|| "Could you clarify your question?".to_string()),
            };
        }

        // Build evidence status
        let evidence = EvidenceStatus::from_probes(probes, &mapping.required_probes);

        // If missing required probes, need more data
        if !evidence.missing_required.is_empty() {
            return GateDecision::NeedMoreData {
                missing_probes: evidence.missing_required.clone(),
                reason: format!(
                    "Missing required probes: {}",
                    evidence.missing_required.join(", ")
                ),
            };
        }

        // If required probes failed, need more data (retry)
        if !evidence.failed_required.is_empty() {
            return GateDecision::NeedMoreData {
                missing_probes: evidence.failed_required.clone(),
                reason: format!(
                    "Failed required probes: {}",
                    evidence.failed_required.join(", ")
                ),
            };
        }

        // If we can answer directly from probes, try to build answer
        if mapping.can_answer_from_probes {
            if let Some(answer) = self.try_build_direct_answer(schema.intent, &evidence) {
                return GateDecision::AnswerFromProbes { answer };
            }
        }

        // Otherwise, need specialist for synthesis
        GateDecision::NeedSpecialist {
            evidence,
            reason: format!(
                "Intent '{}' requires specialist synthesis",
                schema.intent.label()
            ),
        }
    }

    /// Try to build a direct answer from probe data.
    fn try_build_direct_answer(
        &self,
        intent: CanonicalIntent,
        evidence: &EvidenceStatus,
    ) -> Option<DirectAnswer> {
        match intent {
            CanonicalIntent::MemStatus => self.build_mem_status_answer(evidence),
            CanonicalIntent::DiskUsage => self.build_disk_usage_answer(evidence),
            CanonicalIntent::BootPerf => self.build_boot_perf_answer(evidence),
            CanonicalIntent::CpuLoad => self.build_cpu_load_answer(evidence),
            CanonicalIntent::SvcFailed => self.build_svc_failed_answer(evidence),
            CanonicalIntent::GpuInfo => self.build_gpu_info_answer(evidence),
            CanonicalIntent::GpuDriver => self.build_gpu_driver_answer(evidence),
            CanonicalIntent::DnsHealth => self.build_dns_health_answer(evidence),
            CanonicalIntent::WifiStatus => self.build_wifi_status_answer(evidence),
            CanonicalIntent::HardwareSensors => self.build_sensors_answer(evidence),
            CanonicalIntent::LogsRecentErrors => self.build_logs_answer(evidence),
            CanonicalIntent::SecurityFirewall => self.build_firewall_answer(evidence),
            CanonicalIntent::PkgUpdates => self.build_pkg_updates_answer(evidence),
            _ => None, // Needs specialist
        }
    }

    fn build_mem_status_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let free_h = evidence.get_output("free_h")?;
        // Parse free -h output to extract available memory
        let answer = extract_memory_summary(free_h);
        Some(DirectAnswer::new(&answer).with_evidence("free_h", free_h))
    }

    fn build_disk_usage_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let df_h = evidence.get_output("df_h")?;
        let answer = extract_disk_summary(df_h);
        Some(DirectAnswer::new(&answer).with_evidence("df_h", df_h))
    }

    fn build_boot_perf_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let analyze = evidence.get_output("systemd_analyze")?;
        let blame = evidence.get_output("systemd_blame");

        let mut answer = extract_boot_time(analyze);
        if let Some(blame_output) = blame {
            let top_offenders = extract_top_blame(blame_output, 5);
            if !top_offenders.is_empty() {
                answer = format!("{}\nTop boot time consumers:\n{}", answer, top_offenders);
            }
        }

        let mut direct = DirectAnswer::new(&answer).with_evidence("systemd_analyze", analyze);
        if let Some(b) = blame {
            direct = direct.with_evidence("systemd_blame", b);
        }
        Some(direct)
    }

    fn build_cpu_load_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let uptime = evidence.get_output("uptime")?;
        let answer = extract_load_average(uptime);
        Some(DirectAnswer::new(&answer).with_evidence("uptime", uptime))
    }

    fn build_svc_failed_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let failed = evidence.get_output("systemctl_failed")?;
        let answer = extract_failed_services(failed);
        Some(DirectAnswer::new(&answer).with_evidence("systemctl_failed", failed))
    }

    fn build_gpu_info_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let lspci = evidence.get_output("lspci_gpu")?;
        let answer = extract_gpu_info(lspci);
        Some(DirectAnswer::new(&answer).with_evidence("lspci_gpu", lspci))
    }

    fn build_gpu_driver_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let lspci_k = evidence.get_output("lspci_k_gpu")?;
        let lsmod = evidence.get_output("lsmod_gpu");

        let answer = extract_gpu_driver(lspci_k, lsmod);
        let mut direct = DirectAnswer::new(&answer).with_evidence("lspci_k_gpu", lspci_k);
        if let Some(l) = lsmod {
            direct = direct.with_evidence("lsmod_gpu", l);
        }
        Some(direct)
    }

    fn build_dns_health_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let resolvectl = evidence.get_output("resolvectl_status")?;
        let answer = extract_dns_status(resolvectl);
        Some(DirectAnswer::new(&answer).with_evidence("resolvectl_status", resolvectl))
    }

    fn build_wifi_status_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let iw = evidence.get_output("iw_link")?;
        let answer = extract_wifi_status(iw);
        Some(DirectAnswer::new(&answer).with_evidence("iw_link", iw))
    }

    fn build_sensors_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let sensors = evidence.get_output("sensors")?;
        let answer = extract_sensors_summary(sensors);
        Some(DirectAnswer::new(&answer).with_evidence("sensors", sensors))
    }

    fn build_logs_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let logs = evidence.get_output("journalctl_errors_20")?;
        let answer = extract_recent_errors(logs);
        Some(DirectAnswer::new(&answer).with_evidence("journalctl_errors_20", logs))
    }

    fn build_firewall_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let fw = evidence.get_output("firewall_status")?;
        let answer = extract_firewall_status(fw);
        Some(DirectAnswer::new(&answer).with_evidence("firewall_status", fw))
    }

    fn build_pkg_updates_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let updates = evidence.get_output("checkupdates")?;
        let answer = extract_updates_summary(updates);
        Some(DirectAnswer::new(&answer).with_evidence("checkupdates", updates))
    }
}

impl Default for EvidenceGate {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_need_more_data() {
        let gate = EvidenceGate::new();
        let schema = TicketIntentSchema::new(
            "how much RAM?",
            CanonicalIntent::MemStatus,
            Department::Performance,
        );
        let probes = HashMap::new(); // Empty - no probes run yet

        let decision = gate.evaluate(&schema, &probes);
        assert!(matches!(decision, GateDecision::NeedMoreData { .. }));
    }

    #[test]
    fn test_gate_answer_from_probes() {
        let gate = EvidenceGate::new();
        let schema = TicketIntentSchema::new(
            "how much RAM?",
            CanonicalIntent::MemStatus,
            Department::Performance,
        );

        let mut probes = HashMap::new();
        probes.insert("free_h".to_string(), ProbeResult::success("free_h", "              total        used        free      shared  buff/cache   available\nMem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi"));

        let decision = gate.evaluate(&schema, &probes);
        assert!(matches!(decision, GateDecision::AnswerFromProbes { .. }));
    }
}
