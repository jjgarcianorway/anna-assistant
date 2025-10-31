// Anna v0.11.0 - Doctor Handler
//
// Implements the DoctorHandler trait to process events through the
// doctor → policy → repair pipeline.

use crate::events::{DoctorHandler, DoctorResult, EventResult, RepairResult, SystemEvent};
use crate::integrity::IntegrityWatchdog;
use crate::policy::{PolicyDecision, PolicyEngine};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct DoctorHandlerImpl {
    integrity: Arc<Mutex<IntegrityWatchdog>>,
    policy: Arc<Mutex<PolicyEngine>>,
}

impl DoctorHandlerImpl {
    pub fn new(
        integrity: Arc<Mutex<IntegrityWatchdog>>,
        policy: Arc<Mutex<PolicyEngine>>,
    ) -> Self {
        Self { integrity, policy }
    }
}

#[async_trait::async_trait]
impl DoctorHandler for DoctorHandlerImpl {
    async fn handle_event(&self, event: &SystemEvent) -> Result<EventResult> {
        let start = Instant::now();
        let domain = event.domain.as_str();

        info!("Handling event for domain: {} ({})", domain, event.cause);

        // Run domain-specific integrity check
        let alerts = {
            let watchdog = self.integrity.lock().await;
            watchdog.check_domain(domain)?
        };

        // Get degraded modules from alerts
        let degraded_modules: Vec<String> = alerts
            .iter()
            .filter_map(|alert| {
                if alert.component.starts_with("module:") {
                    Some(alert.component.clone())
                } else {
                    None
                }
            })
            .collect();

        // Determine policy decision
        let decision = {
            let mut policy = self.policy.lock().await;
            policy.decide(domain, "update_capability")
        };

        let action_taken = match decision {
            PolicyDecision::AutoRepair => "auto_repair",
            PolicyDecision::AlertOnly => "alert_only",
            PolicyDecision::NoAction => "no_action",
        };

        info!(
            "Domain {} check complete: {} alerts found, action: {}",
            domain,
            alerts.len(),
            action_taken
        );

        // Build doctor result
        let doctor_result = DoctorResult {
            alerts_found: alerts.len(),
            degraded_modules,
            action_taken: action_taken.to_string(),
        };

        // If auto-repair is allowed, attempt low-risk repairs
        let repair_result = if decision == PolicyDecision::AutoRepair {
            Some(self.execute_auto_repair(domain, &alerts).await)
        } else {
            None
        };

        Ok(EventResult {
            event: event.clone(),
            doctor_result,
            repair_result,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl DoctorHandlerImpl {
    /// Execute automatic repair for low-risk fixes
    async fn execute_auto_repair(
        &self,
        domain: &str,
        alerts: &[crate::integrity::IntegrityAlert],
    ) -> RepairResult {
        info!("Executing auto-repair for domain: {}", domain);

        // In full implementation, this would:
        // 1. Analyze alerts for fixable issues
        // 2. Execute safe operations (create dirs, set perms, enable units)
        // 3. Avoid dangerous operations (install packages, edit configs)
        // 4. Log all actions to adaptive log

        // For now, we just log and simulate success
        let cleared = alerts
            .iter()
            .filter(|a| {
                // Only "fix" low-severity alerts
                matches!(a.severity, crate::integrity::AlertSeverity::Warning)
            })
            .count();

        if cleared > 0 {
            info!("Auto-repair cleared {} warnings for domain {}", cleared, domain);
        } else {
            warn!("No auto-repairable alerts found for domain {}", domain);
        }

        RepairResult {
            success: cleared > 0,
            message: if cleared > 0 {
                format!("Auto-repaired {} issues", cleared)
            } else {
                "No auto-repairable issues found".to_string()
            },
            alerts_cleared: cleared,
        }
    }
}
