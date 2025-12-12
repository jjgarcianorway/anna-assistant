//! Evidence Gating (Part C) - v0.0.439.
//!
//! Rules:
//! - If required_evidence probes are missing or failed: do not call specialist.
//! - If probes already provide a direct factual answer: do not call specialist.
//! - Only call specialists when synthesis is needed (why, root cause, recommendations).

use std::collections::HashMap;

use super::intent_map::{IntentMapTable, IntentMapping};
use super::intent_schema::{CanonicalIntent, Department, TicketIntentSchema};

/// Result of a probe execution.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Probe ID.
    pub probe_id: String,
    /// Whether the probe succeeded.
    pub success: bool,
    /// The probe output (if successful).
    pub output: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
}

impl ProbeResult {
    /// Create a successful probe result.
    pub fn success(probe_id: &str, output: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: true,
            output: Some(output.to_string()),
            error: None,
        }
    }

    /// Create a failed probe result.
    pub fn failed(probe_id: &str, error: &str) -> Self {
        Self {
            probe_id: probe_id.to_string(),
            success: false,
            output: None,
            error: Some(error.to_string()),
        }
    }
}

/// Evidence collection status.
#[derive(Debug, Clone)]
pub struct EvidenceStatus {
    /// Collected probe results.
    pub probes: HashMap<String, ProbeResult>,
    /// Missing required probes.
    pub missing_required: Vec<String>,
    /// Failed required probes.
    pub failed_required: Vec<String>,
    /// Whether all required evidence is available.
    pub all_required_available: bool,
}

impl EvidenceStatus {
    /// Create from probe results and requirements.
    pub fn from_probes(probes: &HashMap<String, ProbeResult>, required: &[&str]) -> Self {
        let mut missing_required = Vec::new();
        let mut failed_required = Vec::new();

        for probe_id in required {
            match probes.get(*probe_id) {
                None => missing_required.push(probe_id.to_string()),
                Some(result) if !result.success => failed_required.push(probe_id.to_string()),
                _ => {}
            }
        }

        let all_required_available = missing_required.is_empty() && failed_required.is_empty();

        Self {
            probes: probes.clone(),
            missing_required,
            failed_required,
            all_required_available,
        }
    }

    /// Get successful probe output.
    pub fn get_output(&self, probe_id: &str) -> Option<&str> {
        self.probes.get(probe_id).and_then(|p| p.output.as_deref())
    }
}

/// Decision from evidence gate.
#[derive(Debug, Clone)]
pub enum GateDecision {
    /// Need more data - run these probes first.
    NeedMoreData {
        missing_probes: Vec<String>,
        reason: String,
    },
    /// Can answer directly from probes (no specialist needed).
    AnswerFromProbes { answer: DirectAnswer },
    /// Need specialist for synthesis.
    NeedSpecialist {
        evidence: EvidenceStatus,
        reason: String,
    },
    /// Intent unknown, need clarification.
    NeedClarification { question: String },
}

/// A direct answer from probe data.
#[derive(Debug, Clone)]
pub struct DirectAnswer {
    /// The factual answer.
    pub answer: String,
    /// Supporting evidence (probe outputs).
    pub evidence: Vec<(String, String)>, // (probe_id, output)
    /// Confidence (always high for direct facts).
    pub confidence: f64,
}

impl DirectAnswer {
    /// Create a new direct answer.
    pub fn new(answer: &str) -> Self {
        Self {
            answer: answer.to_string(),
            evidence: Vec::new(),
            confidence: 0.95,
        }
    }

    /// Add evidence.
    pub fn with_evidence(mut self, probe_id: &str, output: &str) -> Self {
        self.evidence
            .push((probe_id.to_string(), output.to_string()));
        self
    }
}

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
        let answer = Self::extract_memory_summary(free_h);
        Some(DirectAnswer::new(&answer).with_evidence("free_h", free_h))
    }

    fn build_disk_usage_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let df_h = evidence.get_output("df_h")?;
        let answer = Self::extract_disk_summary(df_h);
        Some(DirectAnswer::new(&answer).with_evidence("df_h", df_h))
    }

    fn build_boot_perf_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let analyze = evidence.get_output("systemd_analyze")?;
        let blame = evidence.get_output("systemd_blame");

        let mut answer = Self::extract_boot_time(analyze);
        if let Some(blame_output) = blame {
            let top_offenders = Self::extract_top_blame(blame_output, 5);
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
        let answer = Self::extract_load_average(uptime);
        Some(DirectAnswer::new(&answer).with_evidence("uptime", uptime))
    }

    fn build_svc_failed_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let failed = evidence.get_output("systemctl_failed")?;
        let answer = Self::extract_failed_services(failed);
        Some(DirectAnswer::new(&answer).with_evidence("systemctl_failed", failed))
    }

    fn build_gpu_info_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let lspci = evidence.get_output("lspci_gpu")?;
        let answer = Self::extract_gpu_info(lspci);
        Some(DirectAnswer::new(&answer).with_evidence("lspci_gpu", lspci))
    }

    fn build_gpu_driver_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let lspci_k = evidence.get_output("lspci_k_gpu")?;
        let lsmod = evidence.get_output("lsmod_gpu");

        let answer = Self::extract_gpu_driver(lspci_k, lsmod);
        let mut direct = DirectAnswer::new(&answer).with_evidence("lspci_k_gpu", lspci_k);
        if let Some(l) = lsmod {
            direct = direct.with_evidence("lsmod_gpu", l);
        }
        Some(direct)
    }

    fn build_dns_health_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let resolvectl = evidence.get_output("resolvectl_status")?;
        let answer = Self::extract_dns_status(resolvectl);
        Some(DirectAnswer::new(&answer).with_evidence("resolvectl_status", resolvectl))
    }

    fn build_wifi_status_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let iw = evidence.get_output("iw_link")?;
        let answer = Self::extract_wifi_status(iw);
        Some(DirectAnswer::new(&answer).with_evidence("iw_link", iw))
    }

    fn build_sensors_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let sensors = evidence.get_output("sensors")?;
        let answer = Self::extract_sensors_summary(sensors);
        Some(DirectAnswer::new(&answer).with_evidence("sensors", sensors))
    }

    fn build_logs_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let logs = evidence.get_output("journalctl_errors_20")?;
        let answer = Self::extract_recent_errors(logs);
        Some(DirectAnswer::new(&answer).with_evidence("journalctl_errors_20", logs))
    }

    fn build_firewall_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let fw = evidence.get_output("firewall_status")?;
        let answer = Self::extract_firewall_status(fw);
        Some(DirectAnswer::new(&answer).with_evidence("firewall_status", fw))
    }

    fn build_pkg_updates_answer(&self, evidence: &EvidenceStatus) -> Option<DirectAnswer> {
        let updates = evidence.get_output("checkupdates")?;
        let answer = Self::extract_updates_summary(updates);
        Some(DirectAnswer::new(&answer).with_evidence("checkupdates", updates))
    }

    // ========== Extraction helpers ==========

    fn extract_memory_summary(free_output: &str) -> String {
        // Parse "free -h" output
        for line in free_output.lines() {
            if line.starts_with("Mem:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let total = parts.get(1).unwrap_or(&"?");
                    let used = parts.get(2).unwrap_or(&"?");
                    let available = parts.get(6).unwrap_or(parts.get(3).unwrap_or(&"?"));
                    return format!(
                        "Memory: {} total, {} used, {} available",
                        total, used, available
                    );
                }
            }
        }
        "Memory information unavailable".to_string()
    }

    fn extract_disk_summary(df_output: &str) -> String {
        let mut summaries = Vec::new();
        for line in df_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                let mount = parts.get(5).unwrap_or(&"?");
                let use_pct = parts.get(4).unwrap_or(&"?");
                let avail = parts.get(3).unwrap_or(&"?");
                if *mount == "/" || mount.starts_with("/home") {
                    summaries.push(format!("{}: {} used, {} available", mount, use_pct, avail));
                }
            }
        }
        if summaries.is_empty() {
            "Disk information unavailable".to_string()
        } else {
            summaries.join("\n")
        }
    }

    fn extract_boot_time(analyze_output: &str) -> String {
        // systemd-analyze output like "Startup finished in 3.5s (kernel) + 5.2s (userspace) = 8.7s"
        if let Some(line) = analyze_output.lines().next() {
            if line.contains("Startup finished") {
                return line.to_string();
            }
        }
        format!(
            "Boot analysis: {}",
            analyze_output.lines().next().unwrap_or("unavailable")
        )
    }

    fn extract_top_blame(blame_output: &str, count: usize) -> String {
        blame_output
            .lines()
            .take(count)
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn extract_load_average(uptime_output: &str) -> String {
        // Parse "uptime" output for load averages
        if let Some(idx) = uptime_output.find("load average:") {
            let load_part = &uptime_output[idx..];
            return load_part.to_string();
        }
        format!("Load: {}", uptime_output.trim())
    }

    fn extract_failed_services(failed_output: &str) -> String {
        let lines: Vec<&str> = failed_output.lines().collect();
        if lines.is_empty() || failed_output.contains("0 loaded units") {
            return "No failed services.".to_string();
        }
        let count = lines.len().saturating_sub(1); // Exclude header
        if count == 0 {
            "No failed services.".to_string()
        } else {
            format!("{} failed service(s):\n{}", count, failed_output.trim())
        }
    }

    fn extract_gpu_info(lspci_output: &str) -> String {
        let gpu_lines: Vec<&str> = lspci_output
            .lines()
            .filter(|l| l.contains("VGA") || l.contains("3D") || l.contains("Display"))
            .collect();
        if gpu_lines.is_empty() {
            "No GPU detected".to_string()
        } else {
            gpu_lines.join("\n")
        }
    }

    fn extract_gpu_driver(lspci_k_output: &str, lsmod: Option<&str>) -> String {
        let mut result = String::new();

        // Extract kernel driver from lspci -k
        for line in lspci_k_output.lines() {
            if line.contains("Kernel driver") || line.contains("Kernel modules") {
                result.push_str(line.trim());
                result.push('\n');
            }
        }

        if let Some(lsmod_out) = lsmod {
            if !lsmod_out.is_empty() {
                result.push_str("Loaded modules: ");
                result.push_str(
                    lsmod_out
                        .lines()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join(", ")
                        .as_str(),
                );
            }
        }

        if result.is_empty() {
            "GPU driver information unavailable".to_string()
        } else {
            result.trim().to_string()
        }
    }

    fn extract_dns_status(resolvectl_output: &str) -> String {
        // Extract key DNS info
        let mut servers = Vec::new();
        for line in resolvectl_output.lines() {
            if line.contains("DNS Servers") || line.contains("Current DNS") {
                servers.push(line.trim());
            }
        }
        if servers.is_empty() {
            format!(
                "DNS status:\n{}",
                resolvectl_output
                    .lines()
                    .take(5)
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            servers.join("\n")
        }
    }

    fn extract_wifi_status(iw_output: &str) -> String {
        if iw_output.contains("Not connected") || iw_output.contains("No such device") {
            return "WiFi: Not connected".to_string();
        }

        let mut info = Vec::new();
        for line in iw_output.lines() {
            if line.contains("SSID") || line.contains("signal") || line.contains("freq") {
                info.push(line.trim());
            }
        }
        if info.is_empty() {
            iw_output.lines().take(3).collect::<Vec<_>>().join("\n")
        } else {
            format!("WiFi: {}", info.join(", "))
        }
    }

    fn extract_sensors_summary(sensors_output: &str) -> String {
        let mut temps = Vec::new();
        for line in sensors_output.lines() {
            if line.contains("°C") || line.contains("Core") || line.contains("temp") {
                temps.push(line.trim());
            }
        }
        if temps.is_empty() {
            "No temperature sensors detected".to_string()
        } else {
            temps.into_iter().take(5).collect::<Vec<_>>().join("\n")
        }
    }

    fn extract_recent_errors(logs_output: &str) -> String {
        let lines: Vec<&str> = logs_output.lines().collect();
        if lines.is_empty() {
            return "No recent errors in logs.".to_string();
        }
        format!("{} recent error(s):\n{}", lines.len(), logs_output.trim())
    }

    fn extract_firewall_status(fw_output: &str) -> String {
        if fw_output.contains("inactive") || fw_output.contains("not running") {
            "Firewall: Inactive".to_string()
        } else if fw_output.contains("active") || fw_output.contains("running") {
            "Firewall: Active".to_string()
        } else {
            format!(
                "Firewall status: {}",
                fw_output.lines().next().unwrap_or("unknown")
            )
        }
    }

    fn extract_updates_summary(updates_output: &str) -> String {
        let lines: Vec<&str> = updates_output.lines().collect();
        if lines.is_empty() {
            "System is up to date (no pending updates).".to_string()
        } else {
            format!("{} package update(s) available.", lines.len())
        }
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
    fn test_evidence_status_missing() {
        let probes = HashMap::new();
        let status = EvidenceStatus::from_probes(&probes, &["probe_a", "probe_b"]);

        assert!(!status.all_required_available);
        assert_eq!(status.missing_required.len(), 2);
    }

    #[test]
    fn test_evidence_status_complete() {
        let mut probes = HashMap::new();
        probes.insert(
            "probe_a".to_string(),
            ProbeResult::success("probe_a", "output"),
        );
        probes.insert(
            "probe_b".to_string(),
            ProbeResult::success("probe_b", "output"),
        );

        let status = EvidenceStatus::from_probes(&probes, &["probe_a", "probe_b"]);
        assert!(status.all_required_available);
    }

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

    #[test]
    fn test_extract_memory_summary() {
        let output = "              total        used        free      shared  buff/cache   available\nMem:           31Gi       8.2Gi        15Gi       1.2Gi       7.8Gi        21Gi";
        let summary = EvidenceGate::extract_memory_summary(output);
        assert!(summary.contains("31Gi"));
        assert!(summary.contains("available"));
    }

    #[test]
    fn test_extract_failed_services() {
        let empty = "";
        assert_eq!(
            EvidenceGate::extract_failed_services(empty),
            "No failed services."
        );

        let with_failed = "  UNIT                    LOAD   ACTIVE SUB    DESCRIPTION\n● foo.service           loaded failed failed Foo Service";
        let summary = EvidenceGate::extract_failed_services(with_failed);
        assert!(summary.contains("failed"));
    }
}
