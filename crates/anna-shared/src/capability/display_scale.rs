//! Display Scale: GDM Login Screen
//!
//! Capability: display.scale.gdm (Mutating)
//!
//! Phase 31: Returns ActionPlan for confirmation flow.
//! No manual commands shown to user. Uses pkexec execution.
//!
//! Prerequisites:
//! - GDM must be the active display manager
//! - User's monitors.xml must exist with scale configuration
//!
//! What this does NOT do:
//! - Does not scale GNOME session (that's display.scale.session.gnome)
//! - Does not work with SDDM, LightDM, or other display managers

use super::response::{AbstainReason, CapabilityExecutionResult, ResponseArtifact};
use crate::action_plan::{ActionPlan, ActionStep};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// =============================================================================
// DISPLAY MANAGER DETECTION
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayManager {
    Gdm,
    Gdm3,  // Debian/Ubuntu variant
    Sddm,
    Lightdm,
    Greetd,
    Unknown(String),
    None,
}

impl DisplayManager {
    pub fn name(&self) -> &str {
        match self {
            Self::Gdm => "GDM",
            Self::Gdm3 => "GDM3",
            Self::Sddm => "SDDM",
            Self::Lightdm => "LightDM",
            Self::Greetd => "greetd",
            Self::Unknown(s) => s,
            Self::None => "none",
        }
    }

    pub fn is_gdm(&self) -> bool {
        matches!(self, Self::Gdm | Self::Gdm3)
    }
}

/// Detect the active display manager.
fn detect_display_manager() -> DisplayManager {
    // Check which DM service is enabled
    for (service, dm) in [
        ("gdm.service", DisplayManager::Gdm), ("gdm3.service", DisplayManager::Gdm3),
        ("sddm.service", DisplayManager::Sddm), ("lightdm.service", DisplayManager::Lightdm),
    ] {
        if let Ok(o) = Command::new("systemctl").args(["is-enabled", service]).output() {
            if o.status.success() && String::from_utf8_lossy(&o.stdout).trim() == "enabled" {
                return dm;
            }
        }
    }
    // Fallback: check for running DM process
    for (proc, dm) in [("gdm", DisplayManager::Gdm), ("gdm3", DisplayManager::Gdm3), ("sddm", DisplayManager::Sddm)] {
        if Command::new("pgrep").args(["-x", proc]).output().map(|o| o.status.success()).unwrap_or(false) {
            return dm;
        }
    }
    // Binary presence check
    if Path::new("/usr/bin/gdm").exists() || Path::new("/usr/sbin/gdm").exists() {
        return DisplayManager::Gdm;
    }
    DisplayManager::None
}

// =============================================================================
// PROBE TYPES
// =============================================================================

/// Environment probe results.
pub struct GdmScalingProbes {
    pub display_manager: DisplayManager,
    pub session_type: String,
    pub user_monitors: MonitorsXmlStatus,
    pub gdm_monitors: MonitorsXmlStatus,
    pub detected_scale: Option<String>,
}

pub struct MonitorsXmlStatus {
    pub path: PathBuf,
    pub exists: bool,
    pub has_scale: bool,
    pub scale_value: Option<String>,
}

impl GdmScalingProbes {
    /// Phase 35: Evidence capped at 3 lines.
    pub fn to_evidence(&self) -> Vec<ResponseArtifact> {
        let user_status = match (self.user_monitors.exists, self.user_monitors.has_scale) {
            (true, true) => format!("scale {}", self.user_monitors.scale_value.as_deref().unwrap_or("set")),
            (true, false) => "no scale".to_string(), (false, _) => "not found".to_string(),
        };
        let gdm_status = match (self.gdm_monitors.exists, self.gdm_monitors.has_scale) {
            (true, true) => format!("scale {}", self.gdm_monitors.scale_value.as_deref().unwrap_or("set")),
            (true, false) => "no scale".to_string(), (false, _) => "not configured".to_string(),
        };
        vec![
            ResponseArtifact::evidence("DM:", self.display_manager.name()),
            ResponseArtifact::evidence("User:", &user_status),
            ResponseArtifact::evidence("GDM:", &gdm_status),
        ]
    }
}

// =============================================================================
// PROBE IMPLEMENTATION
// =============================================================================

/// Run all probes for GDM scaling.
pub fn gather_probes() -> GdmScalingProbes {
    let display_manager = detect_display_manager();
    let session_type = detect_session_type();
    let user_monitors = probe_monitors_xml(get_user_monitors_path());
    let gdm_monitors = probe_monitors_xml(get_gdm_monitors_path(&display_manager));
    let detected_scale = user_monitors.scale_value.clone();
    GdmScalingProbes { display_manager, session_type, user_monitors, gdm_monitors, detected_scale }
}

fn detect_session_type() -> String {
    if let Ok(session) = std::env::var("XDG_SESSION_TYPE") {
        return session;
    }
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return "wayland".to_string();
    }
    if std::env::var("DISPLAY").is_ok() {
        return "x11".to_string();
    }
    "unknown".to_string()
}

fn probe_monitors_xml(path: PathBuf) -> MonitorsXmlStatus {
    if !path.exists() {
        return MonitorsXmlStatus { path, exists: false, has_scale: false, scale_value: None };
    }
    let (has_scale, scale_value) = fs::read_to_string(&path).ok()
        .map(|c| (c.contains("<scale>"), extract_scale_value(&c)))
        .unwrap_or((false, None));
    MonitorsXmlStatus { path, exists: true, has_scale, scale_value }
}

fn extract_scale_value(content: &str) -> Option<String> {
    content.find("<scale>").and_then(|start| {
        let rest = &content[start + 7..];
        rest.find("</scale>").map(|end| rest[..end].to_string())
    })
}

fn get_user_monitors_path() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/home/user"))
        .join(".config/monitors.xml")
}

fn get_gdm_monitors_path(dm: &DisplayManager) -> PathBuf {
    if *dm == DisplayManager::Gdm3 { PathBuf::from("/var/lib/gdm3/.config/monitors.xml") }
    else { PathBuf::from("/var/lib/gdm/.config/monitors.xml") }
}

// =============================================================================
// CAPABILITY HANDLER
// =============================================================================

/// Execute the display.scale.gdm capability.
/// Phase 31: Returns ActionPlan for mutating capabilities.
pub fn execute_display_scale_gdm() -> CapabilityExecutionResult {
    let probes = gather_probes();

    // Check: Is GDM the display manager?
    if !probes.display_manager.is_gdm() {
        return build_wrong_dm_response(&probes);
    }

    // Check: Does user's monitors.xml exist?
    if !probes.user_monitors.exists {
        return build_no_user_config_response(&probes);
    }

    // Check: Does user config have scale?
    if !probes.user_monitors.has_scale {
        return build_no_scale_in_user_config_response(&probes);
    }

    // Check: Is GDM already configured?
    if probes.gdm_monitors.exists && probes.gdm_monitors.has_scale {
        if probes.user_monitors.scale_value == probes.gdm_monitors.scale_value {
            return build_already_configured_response(&probes);
        }
        // Scale mismatch - still need to propagate
    }

    // Build ActionPlan for propagation
    build_propagate_action_plan(&probes)
}

// =============================================================================
// RESPONSE BUILDERS
// =============================================================================

fn build_wrong_dm_response(probes: &GdmScalingProbes) -> CapabilityExecutionResult {
    let dm = probes.display_manager.name();
    let hint = match &probes.display_manager {
        DisplayManager::Sddm => " SDDM uses ServerArguments DPI in /etc/sddm.conf.",
        DisplayManager::Lightdm => " LightDM uses Xft.dpi in Xresources.",
        _ => "",
    };
    let mut result = CapabilityExecutionResult::abstain(
        AbstainReason::PrerequisitesNotMet,
        &format!("System uses {}. GDM scaling not applicable.{}", dm, hint),
    );
    result.evidence = probes.to_evidence();
    result
}

fn build_prerequisite_abstain(probes: &GdmScalingProbes, msg: &str) -> CapabilityExecutionResult {
    let mut result = CapabilityExecutionResult::abstain(AbstainReason::PrerequisitesNotMet, msg);
    result.evidence = probes.to_evidence();
    result
}

fn build_no_user_config_response(probes: &GdmScalingProbes) -> CapabilityExecutionResult {
    build_prerequisite_abstain(probes, "No monitors.xml found. Open Settings > Displays, set scale, click Apply, then ask Anna again.")
}

fn build_no_scale_in_user_config_response(probes: &GdmScalingProbes) -> CapabilityExecutionResult {
    build_prerequisite_abstain(probes, "monitors.xml has no scale. Open Settings > Displays, change scale, click Apply, then ask Anna again.")
}

fn build_already_configured_response(probes: &GdmScalingProbes) -> CapabilityExecutionResult {
    let scale = probes.detected_scale.as_deref().unwrap_or("configured");
    let mut plan = ActionPlan::new("scale gdm", &format!("GDM scaling to {}", scale), "GDM scaling");
    plan.mark_no_changes(&format!("GDM already configured with scale {}. Log out to verify.", scale));
    CapabilityExecutionResult::with_action_plan(probes.to_evidence(), plan)
}

fn build_propagate_action_plan(probes: &GdmScalingProbes) -> CapabilityExecutionResult {
    let user_path = probes.user_monitors.path.display().to_string();
    let gdm_path = probes.gdm_monitors.path.display().to_string();
    let gdm_dir = probes.gdm_monitors.path.parent().map(|p| p.display().to_string()).unwrap_or_else(|| "/var/lib/gdm/.config".to_string());
    let scale = probes.detected_scale.as_deref().unwrap_or("configured");
    let is_gdm3 = probes.display_manager == DisplayManager::Gdm3;
    let owner = if is_gdm3 { "gdm3:gdm3" } else { "gdm:gdm" };
    let owner_name = if is_gdm3 { "gdm3" } else { "gdm" };

    let mut plan = ActionPlan::new("scale gdm", &format!("GDM login scale {}", scale),
        &format!("Copy monitors.xml to GDM for login screen scaling ({})", scale));

    plan.add_step_full(ActionStep::new("Create GDM config directory", &format!("mkdir -p {}", gdm_dir), true)
        .with_verify(&format!("test -d {}", gdm_dir), ""));
    plan.add_step_full(ActionStep::new("Copy display config to GDM", &format!("cp {} {}", user_path, gdm_path), true)
        .with_files(&[&gdm_path]).with_verify(&format!("test -f {}", gdm_path), "").with_rollback(&format!("rm -f {}", gdm_path)));
    plan.add_step_full(ActionStep::new("Set GDM file ownership", &format!("chown {} {}", owner, gdm_path), true)
        .with_verify(&format!("stat -c %U {}", gdm_path), owner_name));

    // Phase 35: Comprehensive verification with ownership and scale check
    if let Some(expected_scale) = &probes.detected_scale {
        plan.set_verification(&format!("test -f {} && stat -c %U {} && grep -o '<scale>[^<]*</scale>' {} | head -1", gdm_path, gdm_path, gdm_path),
            &format!("{}\n<scale>{}</scale>", owner_name, expected_scale),
            &format!("GDM monitors.xml exists, owned by {}, scale {}", owner_name, expected_scale));
    } else {
        plan.set_verification(&format!("test -f {} && stat -c %U {}", gdm_path, gdm_path), owner_name,
            &format!("GDM monitors.xml exists, owned by {}", owner_name));
    }
    plan.rollback.possible = true;
    plan.rollback.reason = Some("Remove GDM monitors.xml to restore default".to_string());
    CapabilityExecutionResult::with_action_plan(probes.to_evidence(), plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_probes(dm: DisplayManager, user_exists: bool, user_scale: Option<&str>, gdm_exists: bool, gdm_scale: Option<&str>) -> GdmScalingProbes {
        GdmScalingProbes {
            display_manager: dm, session_type: "wayland".to_string(),
            user_monitors: MonitorsXmlStatus {
                path: PathBuf::from("/home/user/.config/monitors.xml"),
                exists: user_exists, has_scale: user_scale.is_some(), scale_value: user_scale.map(String::from),
            },
            gdm_monitors: MonitorsXmlStatus {
                path: PathBuf::from("/var/lib/gdm/.config/monitors.xml"),
                exists: gdm_exists, has_scale: gdm_scale.is_some(), scale_value: gdm_scale.map(String::from),
            },
            detected_scale: user_scale.map(String::from),
        }
    }

    #[test]
    fn test_handler_returns_action_plan_or_abstain() {
        let result = execute_display_scale_gdm();
        assert!(result.action_plan.is_some() || result.wants_abstain(), "Must return ActionPlan or Abstain");
    }

    #[test]
    fn test_wrong_dm_abstains_no_commands() {
        let result = build_wrong_dm_response(&test_probes(DisplayManager::Sddm, true, Some("2"), false, None));
        assert!(result.wants_abstain());
        let msg = result.abstain.as_ref().unwrap().1.clone();
        assert!(!msg.contains("sudo ") && !msg.contains("cp ") && !msg.contains("chown "));
    }

    #[test]
    fn test_propagate_action_plan_no_raw_commands() {
        let result = build_propagate_action_plan(&test_probes(DisplayManager::Gdm, true, Some("2"), false, None));
        assert!(result.action_plan.is_some());
        let plan = result.action_plan.unwrap();
        assert_eq!(plan.steps.len(), 3);
        let confirm = plan.format_for_confirmation();
        assert!(!confirm.contains("mkdir -p") && !confirm.contains("cp /home") && !confirm.contains("chown gdm"));
        assert!(confirm.contains("Create GDM") && confirm.contains("Copy display") && confirm.contains("Set GDM"));
    }

    #[test]
    fn test_already_configured_no_changes() {
        let result = build_already_configured_response(&test_probes(DisplayManager::Gdm, true, Some("2"), true, Some("2")));
        assert!(result.action_plan.is_some());
        let plan = result.action_plan.unwrap();
        assert!(!plan.changes_needed && plan.skip_reason.is_some());
    }

    #[test]
    fn test_missing_user_config_abstains() {
        let result = build_no_user_config_response(&test_probes(DisplayManager::Gdm, false, None, false, None));
        assert!(result.wants_abstain());
        let msg = result.abstain.as_ref().unwrap().1.clone();
        assert!(!msg.contains("sudo ") && msg.contains("Settings") && msg.contains("Displays"));
    }

    #[test]
    fn test_phase35_verification_includes_ownership() {
        let result = build_propagate_action_plan(&test_probes(DisplayManager::Gdm, true, Some("2"), false, None));
        let plan = result.action_plan.unwrap();
        assert!(plan.verification.is_some());
        let verify = plan.verification.as_ref().unwrap();
        assert!(verify.description.contains("owned by gdm"), "Verification must check ownership");
    }
}
