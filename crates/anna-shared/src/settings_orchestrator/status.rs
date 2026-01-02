// v0.0.574: Orchestrator Status Types
// Status reporting and formatting for the orchestrator

use serde::{Deserialize, Serialize};

use super::core::SettingsOrchestrator;
use super::state::OrchestratorState;

/// Orchestrator status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    /// Current state
    pub state: OrchestratorState,
    /// Number of profiles
    pub profiles_count: usize,
    /// Number of templates
    pub templates_count: usize,
    /// Number of schedules
    pub schedules_count: usize,
    /// Number of constraints
    pub constraints_count: usize,
    /// Number of hooks
    pub hooks_count: usize,
    /// Unread notifications
    pub unread_notifications: usize,
    /// Audit entries
    pub audit_entries: usize,
    /// Security events
    pub security_events: usize,
}

/// Format orchestrator status
pub fn format_orchestrator_status(orchestrator: &SettingsOrchestrator) -> String {
    let status = orchestrator.status_summary();
    let mut output = String::new();

    output.push_str("=== Settings Orchestrator ===\n\n");
    output.push_str(&format!("State: {}\n\n", status.state));

    output.push_str("Subsystems:\n");
    output.push_str(&format!("  • Profiles: {}\n", status.profiles_count));
    output.push_str(&format!("  • Templates: {}\n", status.templates_count));
    output.push_str(&format!("  • Schedules: {}\n", status.schedules_count));
    output.push_str(&format!("  • Constraints: {}\n", status.constraints_count));
    output.push_str(&format!("  • Hooks: {}\n", status.hooks_count));
    output.push_str(&format!("  • Notifications: {} unread\n", status.unread_notifications));
    output.push_str(&format!("  • Audit: {} entries ({} security)\n",
        status.audit_entries, status.security_events));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_orchestrator_status() {
        let orchestrator = SettingsOrchestrator::with_defaults();
        let output = format_orchestrator_status(&orchestrator);
        assert!(output.contains("Orchestrator"));
    }
}
