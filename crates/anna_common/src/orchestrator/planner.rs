//! Planner core - synthesizes plans from telemetry + wiki knowledge

use super::knowledge::{KnowledgeSourceRef, KnowledgeSourceKind, WikiSummary};
use super::telemetry::TelemetrySummary;
use crate::ipc::{KnowledgeSourceData, SuggestedFixData, SuggestedStepData};

/// Plan step kind
#[derive(Debug, Clone, PartialEq)]
pub enum PlanStepKind {
    Inspect,
    Change,
}

/// A single step in an execution plan
#[derive(Debug, Clone)]
pub struct PlanStep {
    pub kind: PlanStepKind,
    pub command: String,
    pub requires_confirmation: bool,
    pub backup_command: Option<String>,
    pub rollback_command: Option<String>,
    pub knowledge_sources: Vec<KnowledgeSourceRef>,
}

/// Execution plan
#[derive(Debug, Clone)]
pub struct Plan {
    pub steps: Vec<PlanStep>,
}

impl Plan {
    /// Convert to IPC SuggestedFixData
    pub fn to_suggested_fix(&self, description: String) -> SuggestedFixData {
        let steps: Vec<SuggestedStepData> = self.steps.iter().map(|step| {
            SuggestedStepData {
                kind: match step.kind {
                    PlanStepKind::Inspect => "inspect".to_string(),
                    PlanStepKind::Change => "change".to_string(),
                },
                command: step.command.clone(),
                requires_confirmation: step.requires_confirmation,
                rollback_command: step.rollback_command.clone(),
            }
        }).collect();

        let knowledge_sources: Vec<KnowledgeSourceData> = self.steps.iter()
            .flat_map(|step| &step.knowledge_sources)
            .map(|ksref| KnowledgeSourceData {
                url: ksref.url.clone(),
                kind: match ksref.kind {
                    KnowledgeSourceKind::ArchWiki => "ArchWiki".to_string(),
                    KnowledgeSourceKind::OfficialProjectDoc => "OfficialProjectDoc".to_string(),
                },
            })
            .collect();

        // Deduplicate knowledge sources by URL
        let mut unique_sources = Vec::new();
        for source in knowledge_sources {
            if !unique_sources.iter().any(|s: &KnowledgeSourceData| s.url == source.url) {
                unique_sources.push(source);
            }
        }

        SuggestedFixData {
            description,
            steps,
            knowledge_sources: unique_sources,
        }
    }
}

/// Plan DNS fix based on telemetry and wiki guidance
///
/// # Arguments
/// * `user_goal` - User's stated goal (e.g., "fix DNS")
/// * `telemetry` - Current system state
/// * `wiki` - Arch Wiki guidance for DNS
///
/// # Returns
/// A plan with inspect steps followed by change steps if DNS issue detected
pub fn plan_dns_fix(
    _user_goal: &str,
    telemetry: &TelemetrySummary,
    wiki: &WikiSummary,
) -> Plan {
    let mut steps = Vec::new();

    // Only proceed if we have a DNS issue with working network
    if !telemetry.dns_suspected_broken || !telemetry.network_reachable {
        // No DNS issue or network is down - return empty plan
        return Plan { steps };
    }

    // Step 1: Inspect systemd-resolved status
    steps.push(PlanStep {
        kind: PlanStepKind::Inspect,
        command: "systemctl status systemd-resolved.service".to_string(),
        requires_confirmation: false,
        backup_command: None,
        rollback_command: None,
        knowledge_sources: wiki.sources.clone(),
    });

    // Step 2: Inspect recent logs
    steps.push(PlanStep {
        kind: PlanStepKind::Inspect,
        command: "journalctl -u systemd-resolved.service -n 50".to_string(),
        requires_confirmation: false,
        backup_command: None,
        rollback_command: None,
        knowledge_sources: wiki.sources.clone(),
    });

    // Step 3: Check resolver config
    steps.push(PlanStep {
        kind: PlanStepKind::Inspect,
        command: "cat /etc/resolv.conf".to_string(),
        requires_confirmation: false,
        backup_command: None,
        rollback_command: None,
        knowledge_sources: wiki.sources.clone(),
    });

    // Step 4: Test resolution
    steps.push(PlanStep {
        kind: PlanStepKind::Inspect,
        command: "resolvectl query archlinux.org".to_string(),
        requires_confirmation: false,
        backup_command: None,
        rollback_command: None,
        knowledge_sources: wiki.sources.clone(),
    });

    // Step 5: Restart systemd-resolved (CHANGE step - requires confirmation)
    steps.push(PlanStep {
        kind: PlanStepKind::Change,
        command: "sudo systemctl restart systemd-resolved.service".to_string(),
        requires_confirmation: true,
        backup_command: None, // Restarting resolver is reversible
        rollback_command: Some("sudo systemctl restart systemd-resolved.service".to_string()),
        knowledge_sources: wiki.sources.clone(),
    });

    Plan { steps }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::knowledge::get_arch_help_dns;

    #[test]
    fn test_dns_plan_has_inspect_before_change() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        assert!(!plan.steps.is_empty(), "Plan should have steps");

        // Find first change step
        let first_change_idx = plan.steps.iter()
            .position(|s| s.kind == PlanStepKind::Change);

        if let Some(change_idx) = first_change_idx {
            // Verify at least one inspect before first change
            let has_inspect_before = plan.steps[..change_idx].iter()
                .any(|s| s.kind == PlanStepKind::Inspect);

            assert!(has_inspect_before, "Must have Inspect step before Change");
        }
    }

    #[test]
    fn test_dns_plan_change_requires_confirmation() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        // All Change steps must require confirmation
        for step in &plan.steps {
            if step.kind == PlanStepKind::Change {
                assert!(step.requires_confirmation,
                    "Change step must require confirmation: {}", step.command);
            }
        }
    }

    #[test]
    fn test_dns_plan_has_arch_wiki_sources() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        // Every step should reference Arch Wiki
        for step in &plan.steps {
            assert!(!step.knowledge_sources.is_empty(),
                "Step should have knowledge sources: {}", step.command);

            for source in &step.knowledge_sources {
                assert!(source.url.starts_with("https://wiki.archlinux.org/"),
                    "Source should be Arch Wiki: {}", source.url);
            }
        }
    }

    #[test]
    fn test_dns_plan_uses_wiki_commands() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        // All commands should be from wiki or trivial variations
        for step in &plan.steps {
            let is_from_wiki = wiki.recommended_commands.iter()
                .any(|wiki_cmd| {
                    // Allow exact match or with sudo prefix
                    step.command == *wiki_cmd ||
                    step.command == format!("sudo {}", wiki_cmd) ||
                    wiki_cmd.contains(&step.command)
                });

            assert!(is_from_wiki,
                "Command should be from wiki recommendations: {}", step.command);
        }
    }

    #[test]
    fn test_dns_plan_change_has_rollback() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        // All Change steps should have rollback
        for step in &plan.steps {
            if step.kind == PlanStepKind::Change {
                assert!(step.rollback_command.is_some(),
                    "Change step should have rollback: {}", step.command);
            }
        }
    }

    #[test]
    fn test_no_plan_if_network_down() {
        let telemetry = TelemetrySummary {
            dns_suspected_broken: true,
            network_reachable: false, // Network is down
        };
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        assert!(plan.steps.is_empty(),
            "Should not plan DNS fix when network is unreachable");
    }

    #[test]
    fn test_no_plan_if_dns_healthy() {
        let telemetry = TelemetrySummary::healthy();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);

        assert!(plan.steps.is_empty(),
            "Should not plan DNS fix when DNS is healthy");
    }

    #[test]
    fn test_dns_plan_converts_to_suggested_fix() {
        let telemetry = TelemetrySummary::dns_issue();
        let wiki = get_arch_help_dns();

        let plan = plan_dns_fix("fix dns", &telemetry, &wiki);
        let fix = plan.to_suggested_fix("DNS resolution fix based on Arch Wiki".to_string());

        assert!(!fix.steps.is_empty(), "Should have steps");
        assert!(!fix.knowledge_sources.is_empty(), "Should have knowledge sources");

        // Verify all sources are Arch Wiki
        for source in &fix.knowledge_sources {
            assert_eq!(source.kind, "ArchWiki");
            assert!(source.url.starts_with("https://wiki.archlinux.org/"));
        }

        // Verify steps have correct kinds
        let has_inspect = fix.steps.iter().any(|s| s.kind == "inspect");
        let has_change = fix.steps.iter().any(|s| s.kind == "change");

        assert!(has_inspect, "Should have inspect steps");
        assert!(has_change, "Should have change steps");

        // Verify change steps require confirmation
        for step in &fix.steps {
            if step.kind == "change" {
                assert!(step.requires_confirmation, "Change steps must require confirmation");
            }
        }
    }
}
