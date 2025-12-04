//! NetworkingDoctor v2 - Doctor Lifecycle Implementation v0.0.49
//!
//! Implements the Doctor trait for network diagnosis, providing:
//! - Structured diagnostic plan (link -> IP -> route -> DNS -> connectivity)
//! - Evidence-backed findings with confidence scores
//! - Safe next steps and proposed actions with rollback

use std::collections::HashMap;

use crate::doctor_lifecycle::{
    ActionRisk, CollectedEvidence, DiagnosisFinding, DiagnosisResult, DiagnosticCheck, Doctor,
    ProposedAction, SafeNextStep,
};
use crate::doctor_registry::{DoctorDomain, FindingSeverity};
use crate::networking_doctor::{detect_manager_conflicts, detect_network_manager, NetworkManager};

/// NetworkingDoctor v2 - implements Doctor trait
pub struct NetworkingDoctorV2 {
    /// Whether to include wireless checks
    include_wireless: bool,
}

impl Default for NetworkingDoctorV2 {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkingDoctorV2 {
    pub fn new() -> Self {
        Self {
            include_wireless: true,
        }
    }

    /// Check if any evidence indicates wireless interface
    fn has_wireless_interface(evidence: &[CollectedEvidence]) -> bool {
        evidence.iter().any(|e| {
            e.data
                .get("interfaces")
                .and_then(|i| i.as_array())
                .map(|arr| {
                    arr.iter().any(|iface| {
                        iface
                            .get("is_wireless")
                            .and_then(|w| w.as_bool())
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false)
        })
    }

    /// Analyze link layer evidence
    fn analyze_link(&self, evidence: &[CollectedEvidence]) -> Vec<DiagnosisFinding> {
        let mut findings = Vec::new();

        if let Some(iface_ev) = evidence
            .iter()
            .find(|e| e.tool_name == "net_interfaces_summary")
        {
            if let Some(interfaces) = iface_ev.data.get("interfaces").and_then(|i| i.as_array()) {
                let up_count = interfaces
                    .iter()
                    .filter(|i| i.get("state").and_then(|s| s.as_str()) == Some("UP"))
                    .count();

                if up_count == 0 {
                    findings.push(DiagnosisFinding {
                        id: "no_interfaces_up".to_string(),
                        description: "No network interfaces are UP".to_string(),
                        severity: FindingSeverity::Critical,
                        evidence_ids: vec![iface_ev.evidence_id.clone()],
                        confidence: 95,
                        tags: vec!["network".into(), "link".into(), "interface".into()],
                    });
                }

                // Check for carrier but no IP
                for iface in interfaces {
                    let name = iface.get("name").and_then(|n| n.as_str()).unwrap_or("?");
                    let carrier = iface
                        .get("carrier")
                        .and_then(|c| c.as_bool())
                        .unwrap_or(false);
                    let ip4 = iface.get("ip4").and_then(|i| i.as_array());
                    let has_ip = ip4.map(|arr| !arr.is_empty()).unwrap_or(false);

                    if carrier && !has_ip {
                        findings.push(DiagnosisFinding {
                            id: format!("carrier_no_ip_{}", name),
                            description: format!(
                                "Interface {} has carrier but no IP address",
                                name
                            ),
                            severity: FindingSeverity::Warning,
                            evidence_ids: vec![iface_ev.evidence_id.clone()],
                            confidence: 85,
                            tags: vec!["network".into(), "dhcp".into(), name.to_string()],
                        });
                    }
                }
            }
        }

        findings
    }

    /// Analyze routing evidence
    fn analyze_routes(&self, evidence: &[CollectedEvidence]) -> Vec<DiagnosisFinding> {
        let mut findings = Vec::new();

        if let Some(route_ev) = evidence
            .iter()
            .find(|e| e.tool_name == "net_routes_summary")
        {
            let has_default = route_ev
                .data
                .get("has_default_route")
                .and_then(|d| d.as_bool())
                .unwrap_or(false);

            if !has_default {
                findings.push(DiagnosisFinding {
                    id: "no_default_route".to_string(),
                    description: "No default route configured".to_string(),
                    severity: FindingSeverity::Critical,
                    evidence_ids: vec![route_ev.evidence_id.clone()],
                    confidence: 95,
                    tags: vec!["network".into(), "routing".into(), "gateway".into()],
                });
            }
        }

        findings
    }

    /// Analyze DNS evidence
    fn analyze_dns(&self, evidence: &[CollectedEvidence]) -> Vec<DiagnosisFinding> {
        let mut findings = Vec::new();

        if let Some(dns_ev) = evidence.iter().find(|e| e.tool_name == "dns_summary") {
            let servers = dns_ev
                .data
                .get("servers")
                .and_then(|s| s.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0);

            if servers == 0 {
                findings.push(DiagnosisFinding {
                    id: "no_dns_servers".to_string(),
                    description: "No DNS servers configured".to_string(),
                    severity: FindingSeverity::Critical,
                    evidence_ids: vec![dns_ev.evidence_id.clone()],
                    confidence: 90,
                    tags: vec!["network".into(), "dns".into(), "resolution".into()],
                });
            }
        }

        findings
    }

    /// Analyze connectivity evidence
    fn analyze_connectivity(&self, evidence: &[CollectedEvidence]) -> Vec<DiagnosisFinding> {
        let mut findings = Vec::new();

        if let Some(ping_ev) = evidence.iter().find(|e| e.tool_name == "ping_check") {
            let success = ping_ev
                .data
                .get("success")
                .and_then(|s| s.as_bool())
                .unwrap_or(false);

            if !success {
                findings.push(DiagnosisFinding {
                    id: "ping_failed".to_string(),
                    description: "Cannot reach external hosts (ping failed)".to_string(),
                    severity: FindingSeverity::Critical,
                    evidence_ids: vec![ping_ev.evidence_id.clone()],
                    confidence: 90,
                    tags: vec!["network".into(), "connectivity".into(), "internet".into()],
                });
            }
        }

        findings
    }

    /// Analyze wireless evidence
    fn analyze_wireless(&self, evidence: &[CollectedEvidence]) -> Vec<DiagnosisFinding> {
        let mut findings = Vec::new();

        if let Some(wifi_ev) = evidence.iter().find(|e| e.tool_name == "iw_summary") {
            let connected = wifi_ev
                .data
                .get("connected")
                .and_then(|c| c.as_bool())
                .unwrap_or(false);

            if !connected && Self::has_wireless_interface(evidence) {
                findings.push(DiagnosisFinding {
                    id: "wifi_disconnected".to_string(),
                    description: "Wireless interface present but not connected".to_string(),
                    severity: FindingSeverity::Warning,
                    evidence_ids: vec![wifi_ev.evidence_id.clone()],
                    confidence: 85,
                    tags: vec!["network".into(), "wifi".into(), "wireless".into()],
                });
            }

            // Check signal quality
            if let Some(quality) = wifi_ev.data.get("signal_quality").and_then(|q| q.as_str()) {
                if quality == "poor" {
                    findings.push(DiagnosisFinding {
                        id: "wifi_poor_signal".to_string(),
                        description: "WiFi signal quality is poor".to_string(),
                        severity: FindingSeverity::Warning,
                        evidence_ids: vec![wifi_ev.evidence_id.clone()],
                        confidence: 80,
                        tags: vec!["network".into(), "wifi".into(), "signal".into()],
                    });
                }
            }
        }

        findings
    }

    /// Generate proposed actions based on findings
    fn generate_actions(&self, findings: &[DiagnosisFinding]) -> Vec<ProposedAction> {
        let mut actions = Vec::new();

        for finding in findings {
            match finding.id.as_str() {
                "no_interfaces_up" => {
                    // Suggest bringing up interface
                    let manager = detect_network_manager();
                    if manager.manager == NetworkManager::NetworkManager {
                        actions.push(ProposedAction {
                            id: "restart_nm".to_string(),
                            description: "Restart NetworkManager service".to_string(),
                            commands: vec!["sudo systemctl restart NetworkManager".to_string()],
                            risk: ActionRisk::Medium,
                            confirmation_required: true,
                            confirmation_phrase: Some("restart NetworkManager".to_string()),
                            evidence_ids: finding.evidence_ids.clone(),
                            rollback: Some(vec![
                                "sudo systemctl restart NetworkManager".to_string()
                            ]),
                        });
                    }
                }
                "no_default_route" => {
                    actions.push(ProposedAction {
                        id: "check_dhcp".to_string(),
                        description: "Check DHCP client and renew lease".to_string(),
                        commands: vec!["sudo dhclient -v".to_string()],
                        risk: ActionRisk::Low,
                        confirmation_required: true,
                        confirmation_phrase: Some("renew DHCP".to_string()),
                        evidence_ids: finding.evidence_ids.clone(),
                        rollback: None,
                    });
                }
                "wifi_disconnected" => {
                    actions.push(ProposedAction {
                        id: "scan_networks".to_string(),
                        description: "Scan for available WiFi networks".to_string(),
                        commands: vec!["nmcli device wifi list".to_string()],
                        risk: ActionRisk::Low,
                        confirmation_required: false,
                        confirmation_phrase: None,
                        evidence_ids: finding.evidence_ids.clone(),
                        rollback: None,
                    });
                }
                _ => {}
            }
        }

        actions
    }

    /// Determine most likely cause from findings
    fn determine_cause(&self, findings: &[DiagnosisFinding]) -> Option<String> {
        // Find highest severity/confidence finding
        findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .max_by_key(|f| f.confidence)
            .map(|f| f.description.clone())
    }
}

impl Doctor for NetworkingDoctorV2 {
    fn id(&self) -> &str {
        "networking_doctor_v2"
    }

    fn name(&self) -> &str {
        "Networking Doctor"
    }

    fn domain(&self) -> DoctorDomain {
        DoctorDomain::Network
    }

    fn domains(&self) -> Vec<&str> {
        vec![
            "network",
            "networking",
            "wifi",
            "ethernet",
            "internet",
            "dns",
            "ip",
        ]
    }

    fn matches(&self, intent: &str, targets: &[String], raw_text: &str) -> u32 {
        let raw_lower = raw_text.to_lowercase();

        // High match for network-related intents
        if intent.contains("network") || intent.contains("diagnose") {
            if targets
                .iter()
                .any(|t| t.contains("network") || t.contains("internet") || t.contains("wifi"))
            {
                return 95;
            }
        }

        // Check raw text for network keywords
        let keywords = [
            "network",
            "internet",
            "wifi",
            "ethernet",
            "connected",
            "ip",
            "dns",
        ];
        let matches = keywords.iter().filter(|k| raw_lower.contains(*k)).count();

        if matches >= 2 {
            return 80;
        } else if matches == 1 {
            return 60;
        }

        0
    }

    fn plan(&self) -> Vec<DiagnosticCheck> {
        let mut checks = Vec::new();

        // 1. Interface status
        checks.push(DiagnosticCheck {
            id: "interfaces".to_string(),
            description: "Check network interface states".to_string(),
            tool_name: "net_interfaces_summary".to_string(),
            tool_params: HashMap::new(),
            required: true,
            order: 1,
        });

        // 2. Routing
        checks.push(DiagnosticCheck {
            id: "routes".to_string(),
            description: "Check routing table and default gateway".to_string(),
            tool_name: "net_routes_summary".to_string(),
            tool_params: HashMap::new(),
            required: true,
            order: 2,
        });

        // 3. DNS
        checks.push(DiagnosticCheck {
            id: "dns".to_string(),
            description: "Check DNS configuration".to_string(),
            tool_name: "dns_summary".to_string(),
            tool_params: HashMap::new(),
            required: true,
            order: 3,
        });

        // 4. NetworkManager status
        checks.push(DiagnosticCheck {
            id: "nm_status".to_string(),
            description: "Check NetworkManager status".to_string(),
            tool_name: "nm_summary".to_string(),
            tool_params: HashMap::new(),
            required: false,
            order: 4,
        });

        // 5. Wireless (if applicable)
        if self.include_wireless {
            checks.push(DiagnosticCheck {
                id: "wireless".to_string(),
                description: "Check wireless connection".to_string(),
                tool_name: "iw_summary".to_string(),
                tool_params: HashMap::new(),
                required: false,
                order: 5,
            });
        }

        // 6. Connectivity test
        checks.push(DiagnosticCheck {
            id: "connectivity".to_string(),
            description: "Test external connectivity".to_string(),
            tool_name: "ping_check".to_string(),
            tool_params: HashMap::new(),
            required: true,
            order: 6,
        });

        // 7. Recent errors
        checks.push(DiagnosticCheck {
            id: "errors".to_string(),
            description: "Check for recent network errors".to_string(),
            tool_name: "recent_network_errors".to_string(),
            tool_params: HashMap::new(),
            required: false,
            order: 7,
        });

        checks
    }

    fn diagnose(&self, evidence: &[CollectedEvidence]) -> DiagnosisResult {
        let mut all_findings = Vec::new();

        // Analyze each layer
        all_findings.extend(self.analyze_link(evidence));
        all_findings.extend(self.analyze_routes(evidence));
        all_findings.extend(self.analyze_dns(evidence));
        all_findings.extend(self.analyze_connectivity(evidence));
        all_findings.extend(self.analyze_wireless(evidence));

        // Check for manager conflicts
        let conflicts = detect_manager_conflicts();
        for conflict in conflicts {
            all_findings.push(DiagnosisFinding {
                id: "manager_conflict".to_string(),
                description: conflict,
                severity: FindingSeverity::Warning,
                evidence_ids: vec![],
                confidence: 75,
                tags: vec!["network".into(), "service".into(), "conflict".into()],
            });
        }

        // Sort by severity and confidence
        all_findings.sort_by(|a, b| {
            let sev_cmp = severity_rank(&b.severity).cmp(&severity_rank(&a.severity));
            if sev_cmp == std::cmp::Ordering::Equal {
                b.confidence.cmp(&a.confidence)
            } else {
                sev_cmp
            }
        });

        // Determine overall confidence
        let confidence = if all_findings.is_empty() {
            50
        } else {
            all_findings
                .iter()
                .map(|f| f.confidence)
                .max()
                .unwrap_or(50)
        };

        // Determine if issue appears resolved
        let critical_findings = all_findings
            .iter()
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count();
        let issue_resolved = critical_findings == 0;

        // Generate safe next steps
        let mut next_steps = Vec::new();
        if !issue_resolved {
            next_steps.push(SafeNextStep {
                description: "Run 'ip link show' to check interface states".to_string(),
                rationale: "Manual verification of interface status".to_string(),
                evidence_ids: vec![],
            });
            next_steps.push(SafeNextStep {
                description: "Check 'journalctl -u NetworkManager' for errors".to_string(),
                rationale: "Review network manager logs for clues".to_string(),
                evidence_ids: vec![],
            });
        }

        // Generate proposed actions
        let proposed_actions = self.generate_actions(&all_findings);

        // Build summary
        let summary = if issue_resolved {
            "Network appears healthy - all checks passed".to_string()
        } else {
            format!(
                "Found {} issue(s): {} critical, {} warnings",
                all_findings.len(),
                critical_findings,
                all_findings.len() - critical_findings
            )
        };

        // Collect symptom keywords
        let symptom_keywords: Vec<String> = all_findings
            .iter()
            .flat_map(|f| f.tags.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        DiagnosisResult {
            summary,
            most_likely_cause: self.determine_cause(&all_findings),
            findings: all_findings,
            confidence,
            next_steps,
            proposed_actions,
            issue_resolved,
            symptom_keywords,
        }
    }
}

/// Rank severity for sorting
fn severity_rank(severity: &FindingSeverity) -> u8 {
    match severity {
        FindingSeverity::Critical => 4,
        FindingSeverity::Error => 3,
        FindingSeverity::Warning => 2,
        FindingSeverity::Info => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctor_basics() {
        let doctor = NetworkingDoctorV2::new();
        assert_eq!(doctor.id(), "networking_doctor_v2");
        assert_eq!(doctor.domain(), DoctorDomain::Network);
        assert!(!doctor.domains().is_empty());
    }

    #[test]
    fn test_plan_not_empty() {
        let doctor = NetworkingDoctorV2::new();
        let plan = doctor.plan();
        assert!(!plan.is_empty());
        assert!(plan.iter().any(|c| c.required));
    }

    #[test]
    fn test_matches_network_keywords() {
        let doctor = NetworkingDoctorV2::new();

        let score = doctor.matches("diagnose", &[], "my network is not working");
        assert!(score >= 60);

        let score = doctor.matches("diagnose", &[], "wifi and internet broken");
        assert!(score >= 80);
    }

    #[test]
    fn test_diagnose_empty_evidence() {
        let doctor = NetworkingDoctorV2::new();
        let result = doctor.diagnose(&[]);

        // Should handle empty evidence gracefully
        assert!(!result.summary.is_empty());
        assert!(result.confidence <= 50);
    }
}
