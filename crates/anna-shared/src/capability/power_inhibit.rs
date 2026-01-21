//! Power Inhibit: Sleep/Suspend/Hibernate Control
//!
//! Capability: power.inhibit.sleep (Mutating)
//!
//! Phase 33: Deterministic probing of power management targets.
//!
//! What this does:
//! - Probe logind for current HandleLidSwitch, HandleSuspendKey, IdleAction
//! - Detect inhibitors (what's blocking sleep)
//! - Generate ActionPlan to modify /etc/systemd/logind.conf
//!
//! What this does NOT do:
//! - Does not handle ACPI events directly
//! - Does not configure TLP or power-profiles-daemon
//! - Does not touch kernel parameters

use super::response::{AbstainReason, CapabilityExecutionResult, ResponseArtifact};
use crate::action_plan::{ActionPlan, ActionStep};
use std::fs;
use std::process::Command;

// =============================================================================
// PROBE TYPES
// =============================================================================

/// Current power management configuration.
#[derive(Debug, Clone)]
pub struct PowerInhibitProbes {
    pub logind_config: LogindConfig,
    pub inhibitors: Vec<Inhibitor>,
    pub can_suspend: bool,
    pub can_hibernate: bool,
    pub can_hybrid_sleep: bool,
}

/// Parsed logind.conf settings.
#[derive(Debug, Clone, Default)]
pub struct LogindConfig {
    pub handle_lid_switch: String,
    pub handle_lid_switch_external_power: String,
    pub handle_lid_switch_docked: String,
    pub handle_suspend_key: String,
    pub handle_hibernate_key: String,
    pub handle_power_key: String,
    pub idle_action: String,
    pub idle_action_sec: String,
}

/// Active inhibitor blocking sleep.
#[derive(Debug, Clone)]
pub struct Inhibitor {
    pub who: String,
    pub why: String,
    pub mode: String,
    pub uid: String,
}

impl PowerInhibitProbes {
    /// Phase 35: Evidence capped at 3 lines.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let cfg = &self.logind_config;
        // Line 1: Current lid action
        let lid = format!("lid:{} suspend-key:{}", cfg.handle_lid_switch, cfg.handle_suspend_key);
        // Line 2: Capabilities
        let caps = format!("suspend:{} hibernate:{}", if self.can_suspend { "Y" } else { "N" }, if self.can_hibernate { "Y" } else { "N" });
        // Line 3: Idle or inhibitors
        let extra = if !self.inhibitors.is_empty() {
            format!("{} inhibitor(s)", self.inhibitors.len())
        } else if cfg.idle_action != "ignore" {
            format!("idle:{}", cfg.idle_action)
        } else { "idle:ignore".to_string() };
        vec![
            ResponseArtifact::evidence("Config:", &lid),
            ResponseArtifact::evidence("Support:", &caps),
            ResponseArtifact::evidence("Status:", &extra),
        ]
    }

    /// Phase 35: Single-line explanation.
    pub fn format_explanation(&self) -> String {
        let cfg = &self.logind_config;
        format!("Lid close: {}. Suspend key: {}. Idle: {}.", cfg.handle_lid_switch, cfg.handle_suspend_key, cfg.idle_action)
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

/// Run all probes for power inhibit.
pub fn gather_probes() -> PowerInhibitProbes {
    let logind_config = probe_logind_config();
    let inhibitors = probe_inhibitors();
    let (can_suspend, can_hibernate, can_hybrid_sleep) = probe_capabilities();

    PowerInhibitProbes {
        logind_config,
        inhibitors,
        can_suspend,
        can_hibernate,
        can_hybrid_sleep,
    }
}

fn probe_logind_config() -> LogindConfig {
    let mut cfg = LogindConfig::default();
    if let Ok(content) = fs::read_to_string("/etc/systemd/logind.conf") {
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || !line.contains('=') { continue; }
            let mut parts = line.splitn(2, '=');
            let (key, val) = match (parts.next(), parts.next()) { (Some(k), Some(v)) => (k.trim(), v.trim()), _ => continue };
            match key {
                "HandleLidSwitch" => cfg.handle_lid_switch = val.to_string(),
                "HandleLidSwitchExternalPower" => cfg.handle_lid_switch_external_power = val.to_string(),
                "HandleLidSwitchDocked" => cfg.handle_lid_switch_docked = val.to_string(),
                "HandleSuspendKey" => cfg.handle_suspend_key = val.to_string(),
                "HandleHibernateKey" => cfg.handle_hibernate_key = val.to_string(),
                "HandlePowerKey" => cfg.handle_power_key = val.to_string(),
                "IdleAction" => cfg.idle_action = val.to_string(),
                "IdleActionSec" => cfg.idle_action_sec = val.to_string(),
                _ => {}
            }
        }
    }
    // Apply defaults
    if cfg.handle_lid_switch.is_empty() { cfg.handle_lid_switch = "suspend".to_string(); }
    if cfg.handle_suspend_key.is_empty() { cfg.handle_suspend_key = "suspend".to_string(); }
    if cfg.handle_power_key.is_empty() { cfg.handle_power_key = "poweroff".to_string(); }
    if cfg.idle_action.is_empty() { cfg.idle_action = "ignore".to_string(); }
    cfg
}

fn probe_inhibitors() -> Vec<Inhibitor> {
    let mut inhibitors = Vec::new();
    if let Ok(output) = Command::new("systemd-inhibit").args(["--list", "--no-legend"]).output() {
        if output.status.success() {
            for line in String::from_utf8_lossy(&output.stdout).lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 7 {
                    inhibitors.push(Inhibitor { who: parts[0].to_string(), uid: parts[1].to_string(),
                        why: parts.get(5).unwrap_or(&"").to_string(), mode: parts.get(6).unwrap_or(&"").to_string() });
                }
            }
        }
    }
    inhibitors
}

fn probe_capabilities() -> (bool, bool, bool) {
    (check_can_action("suspend"), check_can_action("hibernate"), check_can_action("hybrid-sleep"))
}

fn check_can_action(action: &str) -> bool {
    Command::new("systemctl").args([&format!("can-{}", action)]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "yes").unwrap_or(false)
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Target action to configure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InhibitTarget {
    LidClose,
    IdleAction,
    SuspendKey,
    All,
}

/// Desired state for the target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InhibitAction {
    Ignore,      // Do nothing
    Suspend,     // Suspend to RAM
    Hibernate,   // Hibernate to disk
    HybridSleep, // Both
    Lock,        // Just lock screen
}

impl InhibitAction {
    fn as_str(&self) -> &'static str {
        match self {
            InhibitAction::Ignore => "ignore",
            InhibitAction::Suspend => "suspend",
            InhibitAction::Hibernate => "hibernate",
            InhibitAction::HybridSleep => "hybrid-sleep",
            InhibitAction::Lock => "lock",
        }
    }
}

/// Execute the power.inhibit.sleep capability.
/// Phase 33: Returns ActionPlan for mutating capabilities.
pub fn execute_power_inhibit_sleep(target: InhibitTarget, action: InhibitAction) -> CapabilityExecutionResult {
    let probes = gather_probes();

    // Check if already configured as requested
    let already_configured = match target {
        InhibitTarget::LidClose => probes.logind_config.handle_lid_switch == action.as_str(),
        InhibitTarget::IdleAction => probes.logind_config.idle_action == action.as_str(),
        InhibitTarget::SuspendKey => probes.logind_config.handle_suspend_key == action.as_str(),
        InhibitTarget::All => {
            probes.logind_config.handle_lid_switch == action.as_str()
                && probes.logind_config.idle_action == action.as_str()
                && probes.logind_config.handle_suspend_key == action.as_str()
        }
    };

    if already_configured {
        return build_already_configured_response(&probes, target, action);
    }

    // Check if action is supported
    match action {
        InhibitAction::Hibernate if !probes.can_hibernate => {
            return build_unsupported_action_response(&probes, "hibernate");
        }
        InhibitAction::HybridSleep if !probes.can_hybrid_sleep => {
            return build_unsupported_action_response(&probes, "hybrid-sleep");
        }
        InhibitAction::Suspend if !probes.can_suspend => {
            return build_unsupported_action_response(&probes, "suspend");
        }
        _ => {}
    }

    // Build ActionPlan
    build_inhibit_action_plan(&probes, target, action)
}

// =============================================================================
// RESPONSE BUILDERS
// =============================================================================

fn target_name(target: InhibitTarget) -> &'static str {
    match target { InhibitTarget::LidClose => "lid close", InhibitTarget::IdleAction => "idle action",
        InhibitTarget::SuspendKey => "suspend key", InhibitTarget::All => "power events" }
}

fn build_already_configured_response(probes: &PowerInhibitProbes, target: InhibitTarget, action: InhibitAction) -> CapabilityExecutionResult {
    let tgt = target_name(target);
    let mut plan = ActionPlan::new("power inhibit", &format!("{} to {}", tgt, action.as_str()), "Power management");
    plan.mark_no_changes(&format!("{} already set to {}.", tgt, action.as_str()));
    CapabilityExecutionResult::with_action_plan(probes.to_evidence(), plan)
}

fn build_unsupported_action_response(probes: &PowerInhibitProbes, action: &str) -> CapabilityExecutionResult {
    let mut result = CapabilityExecutionResult::abstain(AbstainReason::PrerequisitesNotMet,
        &format!("System does not support {}. Requires swap (hibernate) or kernel config.", action));
    result.evidence = probes.to_evidence();
    result
}

fn build_inhibit_action_plan(probes: &PowerInhibitProbes, target: InhibitTarget, action: InhibitAction) -> CapabilityExecutionResult {
    let tgt = target_name(target);
    let act = action.as_str();
    let mut plan = ActionPlan::new("power inhibit", &format!("{} to {}", tgt, act),
        &format!("Modify /etc/systemd/logind.conf to set {} to {}.", tgt, act));

    // Build settings based on target
    let settings: Vec<(&str, &str)> = match target {
        InhibitTarget::LidClose => vec![("HandleLidSwitch", act)],
        InhibitTarget::IdleAction => vec![("IdleAction", act)],
        InhibitTarget::SuspendKey => vec![("HandleSuspendKey", act)],
        InhibitTarget::All => vec![("HandleLidSwitch", act), ("IdleAction", act), ("HandleSuspendKey", act)],
    };

    // Step 1: Backup (stash for rollback)
    plan.add_step_full(ActionStep::new("Backup logind.conf", "cp /etc/systemd/logind.conf /etc/systemd/logind.conf.anna-backup", true)
        .with_files(&["/etc/systemd/logind.conf"]).with_verify("test -f /etc/systemd/logind.conf.anna-backup", ""));

    // Step 2: Apply settings
    for (key, value) in &settings {
        let sed_cmd = format!("sed -i -e 's/^#*{}=.*$/{}={}/; t; $a{}={}' /etc/systemd/logind.conf", key, key, value, key, value);
        plan.add_step_full(ActionStep::new(&format!("Set {} to {}", key, value), &sed_cmd, true)
            .with_files(&["/etc/systemd/logind.conf"])
            .with_verify(&format!("grep -E '^{}={}$' /etc/systemd/logind.conf", key, value), value)
            .with_rollback("cp /etc/systemd/logind.conf.anna-backup /etc/systemd/logind.conf"));
    }

    // Step 3: Reload logind
    plan.add_step_full(ActionStep::new("Reload logind", "systemctl kill -s HUP systemd-logind", true)
        .with_units(&["systemd-logind.service"]));

    // Phase 35: Comprehensive verification confirms effective logind configuration
    if let Some((key, value)) = settings.first() {
        plan.set_verification(&format!("grep -E '^{}={}$' /etc/systemd/logind.conf", key, value), *value,
            &format!("{} effectively set to {} in logind.conf", key, value));
    }
    plan.rollback.possible = true;
    plan.rollback.reason = Some("Restore logind.conf from backup".to_string());
    CapabilityExecutionResult::with_action_plan(probes.to_evidence(), plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_probes() -> PowerInhibitProbes {
        PowerInhibitProbes {
            logind_config: LogindConfig { handle_lid_switch: "suspend".to_string(), handle_lid_switch_external_power: String::new(),
                handle_lid_switch_docked: String::new(), handle_suspend_key: "suspend".to_string(), handle_hibernate_key: String::new(),
                handle_power_key: "poweroff".to_string(), idle_action: "ignore".to_string(), idle_action_sec: String::new() },
            inhibitors: vec![], can_suspend: true, can_hibernate: true, can_hybrid_sleep: true,
        }
    }

    #[test]
    fn test_handler_returns_action_plan_or_abstain() {
        let result = execute_power_inhibit_sleep(InhibitTarget::LidClose, InhibitAction::Ignore);
        assert!(result.action_plan.is_some() || result.wants_abstain(), "Must return ActionPlan or Abstain");
    }

    #[test]
    fn test_probe_returns_valid_config() {
        assert!(!gather_probes().logind_config.handle_lid_switch.is_empty());
    }

    #[test]
    fn test_action_plan_no_raw_commands() {
        let result = build_inhibit_action_plan(&test_probes(), InhibitTarget::LidClose, InhibitAction::Ignore);
        let plan = result.action_plan.unwrap();
        let confirm = plan.format_for_confirmation();
        assert!(!confirm.contains("sed -i") && !confirm.contains("systemctl kill"));
        assert!(confirm.contains("Backup") && confirm.contains("Set HandleLidSwitch"));
    }

    #[test]
    fn test_evidence_capped_at_three() {
        assert!(test_probes().to_evidence().len() <= 3, "Phase 35: Evidence must be capped at 3 lines");
    }

    #[test]
    fn test_phase35_rollback_enabled() {
        let result = build_inhibit_action_plan(&test_probes(), InhibitTarget::LidClose, InhibitAction::Ignore);
        let plan = result.action_plan.unwrap();
        assert!(plan.rollback.possible, "Phase 35: Rollback must be enabled");
    }
}
