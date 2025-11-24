//! TLP Fix Planner - Wiki-Backed TLP Enablement (6.13.0)
//!
//! Generates safe, wiki-cited plans to enable TLP when it's installed but not enabled.

use crate::arch_wiki_corpus::WikiSnippet;

/// TLP status from annactl detection
#[derive(Debug, Clone)]
pub struct TlpStatusSummary {
    pub installed: bool,
    pub service_exists: bool,
    pub enabled: bool,
    pub active: bool,
    pub warned_in_logs: bool,
}

/// A single step in the TLP fix plan
#[derive(Debug, Clone)]
pub struct TlpActionStep {
    /// Human-readable description
    pub description: String,
    /// Shell command to execute
    pub command: String,
    /// True if this requires confirmation
    pub requires_confirmation: bool,
    /// True if this is a change (vs inspect)
    pub is_change: bool,
    /// Rollback command (if any)
    pub rollback_command: Option<String>,
}

/// Complete TLP fix plan ready for execution
#[derive(Debug, Clone)]
pub struct TlpFixPlan {
    pub recap: String,
    pub explanation: String,
    pub wiki_url: String,
    pub steps: Vec<TlpActionStep>,
}

/// Build wiki-backed TLP fix plan
///
/// Creates a 2-step plan:
/// 1. INSPECT: Check service status (harmless, auto-run)
/// 2. CHANGE: Enable service (requires confirmation, has rollback)
pub fn build_tlp_fix_plan(status: &TlpStatusSummary, wiki: &WikiSnippet) -> TlpFixPlan {
    // Recap: What we detected
    let recap = format!(
        "TLP is installed on your system, but tlp.service is {}enabled{}.{}",
        if status.enabled { "" } else { "not " },
        if status.active { " and running" } else { "" },
        if status.warned_in_logs {
            " Recent logs show TLP warning that power saving will not apply on boot."
        } else {
            ""
        }
    );

    // Explanation: What the wiki says
    let explanation = format!(
        "{}\n\nAccording to the Arch Wiki, TLP must be enabled as a systemd service \
         to apply its power saving settings automatically on boot.",
        wiki.summary
    );

    // Step 1: Inspect (harmless)
    let inspect_step = TlpActionStep {
        description: "Check the current state of tlp.service and confirm it is not enabled."
            .to_string(),
        command: "systemctl status tlp.service".to_string(),
        requires_confirmation: false,
        is_change: false,
        rollback_command: None,
    };

    // Step 2: Enable (requires confirmation)
    let enable_step = TlpActionStep {
        description:
            "Enable TLP so power saving settings apply on boot and start it immediately."
                .to_string(),
        command: "systemctl enable --now tlp.service".to_string(),
        requires_confirmation: true,
        is_change: true,
        rollback_command: Some("systemctl disable --now tlp.service".to_string()),
    };

    TlpFixPlan {
        recap,
        explanation,
        wiki_url: wiki.url.to_string(),
        steps: vec![inspect_step, enable_step],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arch_wiki_corpus::{WikiTopic, get_wiki_snippet};

    #[test]
    fn test_build_tlp_fix_plan_structure() {
        let status = TlpStatusSummary {
            installed: true,
            service_exists: true,
            enabled: false,
            active: false,
            warned_in_logs: true,
        };

        let wiki = get_wiki_snippet(WikiTopic::TlpPowerSaving);
        let plan = build_tlp_fix_plan(&status, &wiki);

        // Should have exactly 2 steps
        assert_eq!(plan.steps.len(), 2);

        // Step 1: Inspect
        assert!(plan.steps[0].command.contains("systemctl"));
        assert!(plan.steps[0].command.contains("status"));
        assert!(plan.steps[0].command.contains("tlp.service"));
        assert!(!plan.steps[0].is_change);
        assert!(!plan.steps[0].requires_confirmation);

        // Step 2: Enable with confirmation
        assert!(plan.steps[1].command.contains("systemctl"));
        assert!(plan.steps[1].command.contains("enable"));
        assert!(plan.steps[1].command.contains("tlp.service"));
        assert!(plan.steps[1].is_change);
        assert!(plan.steps[1].requires_confirmation);

        // Step 2 has rollback
        assert!(plan.steps[1].rollback_command.is_some());
        let rollback = plan.steps[1].rollback_command.as_ref().unwrap();
        assert!(rollback.contains("systemctl"));
        assert!(rollback.contains("disable"));
    }

    #[test]
    fn test_plan_includes_wiki_url() {
        let status = TlpStatusSummary {
            installed: true,
            service_exists: true,
            enabled: false,
            active: false,
            warned_in_logs: false,
        };

        let wiki = get_wiki_snippet(WikiTopic::TlpPowerSaving);
        let plan = build_tlp_fix_plan(&status, &wiki);

        assert!(plan.wiki_url.contains("archlinux.org"));
        assert!(plan.wiki_url.contains("TLP"));
    }

    #[test]
    fn test_plan_recap_mentions_logs() {
        let status = TlpStatusSummary {
            installed: true,
            service_exists: true,
            enabled: false,
            active: false,
            warned_in_logs: true,
        };

        let wiki = get_wiki_snippet(WikiTopic::TlpPowerSaving);
        let plan = build_tlp_fix_plan(&status, &wiki);

        assert!(plan.recap.to_lowercase().contains("logs"));
        assert!(plan.recap.to_lowercase().contains("warning"));
    }
}
