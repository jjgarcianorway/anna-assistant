//! Display Scale GDM - Demonstration capability.
//!
//! This is a ReadOnly capability that:
//! - Gathers facts about the current GDM scaling state
//! - Proposes a plan if changes are needed
//! - Abstains from execution (just produces artifacts)
//!
//! This demonstrates the capability contract:
//! - Deterministic routing (input matches "display.scale.gdm")
//! - Total response (always Resolved or Abstained)
//! - Noise containment (only Display warnings)

use super::response::{CapabilityExecutionResult, ResponseArtifact};
use std::path::Path;

/// Facts gathered about GDM scaling state.
#[derive(Debug, Clone)]
pub struct GdmScalingFacts {
    /// Whether GDM is installed.
    pub gdm_installed: bool,
    /// Current scaling factor (if configured).
    pub current_scale: Option<f32>,
    /// Whether HiDPI is detected.
    pub hidpi_detected: bool,
    /// Resolution of primary display.
    pub primary_resolution: Option<(u32, u32)>,
    /// GDM custom.conf exists.
    pub custom_conf_exists: bool,
}

impl GdmScalingFacts {
    /// Whether scaling is properly configured.
    pub fn is_configured(&self) -> bool {
        self.current_scale.is_some() && self.current_scale != Some(1.0)
    }

    /// Whether HiDPI scaling is recommended.
    pub fn scaling_recommended(&self) -> bool {
        self.hidpi_detected && !self.is_configured()
    }
}

/// Plan for GDM scaling changes.
#[derive(Debug, Clone)]
pub struct GdmScalingPlan {
    /// Steps to execute (for display only, not executed).
    pub steps: Vec<String>,
    /// Recommended scale factor.
    pub recommended_scale: f32,
    /// Files that would be modified.
    pub affected_files: Vec<String>,
}

/// Execute the display.scale.gdm capability.
///
/// This gathers facts and proposes a plan, but does NOT execute.
/// Execution would require Mutating mode (currently blocked).
pub fn execute_display_scale_gdm() -> CapabilityExecutionResult {
    // Gather facts
    let facts = gather_gdm_scaling_facts();

    // Build response
    let mut result = CapabilityExecutionResult::empty();

    // Add facts as artifacts
    result.facts.push(ResponseArtifact::fact(
        "GDM Installed",
        if facts.gdm_installed { "Yes" } else { "No" },
    ));

    if let Some(scale) = facts.current_scale {
        result.facts.push(ResponseArtifact::fact(
            "Current Scale",
            &format!("{}x", scale),
        ));
    } else {
        result.facts.push(ResponseArtifact::fact(
            "Current Scale",
            "Not configured (default 1x)",
        ));
    }

    result.facts.push(ResponseArtifact::fact(
        "HiDPI Detected",
        if facts.hidpi_detected { "Yes" } else { "No" },
    ));

    if let Some((w, h)) = facts.primary_resolution {
        result.facts.push(ResponseArtifact::fact(
            "Primary Resolution",
            &format!("{}x{}", w, h),
        ));
    }

    // Add warnings if applicable
    if facts.scaling_recommended() {
        result.warnings.push(ResponseArtifact::warning(
            "HiDPI Not Configured",
            "Your display appears to be HiDPI but GDM scaling is not configured. \
             The login screen may appear small.",
        ));
    }

    if !facts.gdm_installed {
        result.warnings.push(ResponseArtifact::warning(
            "GDM Not Installed",
            "GDM display manager is not installed. This capability only applies to GDM.",
        ));
    }

    // Propose plan if changes recommended
    if facts.scaling_recommended() && facts.gdm_installed {
        let plan = propose_scaling_plan(&facts);

        let steps_text = plan
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {}", i + 1, s))
            .collect::<Vec<_>>()
            .join("\n");

        result.plan = Some(ResponseArtifact::plan(
            "Recommended Changes",
            &format!(
                "Scale factor: {}x\n\nSteps:\n{}\n\nAffected files:\n- {}",
                plan.recommended_scale,
                steps_text,
                plan.affected_files.join("\n- ")
            ),
        ));

        result.explanation = format!(
            "HiDPI display detected. Recommend configuring GDM scaling to {}x. \
             Plan is ready but execution is blocked (Mutating capability).",
            plan.recommended_scale
        );
    } else if facts.is_configured() {
        result.explanation = format!(
            "GDM scaling is already configured at {}x. No changes needed.",
            facts.current_scale.unwrap_or(1.0)
        );
    } else if !facts.hidpi_detected {
        result.explanation = "No HiDPI display detected. Scaling is not required.".to_string();
    } else if !facts.gdm_installed {
        result.explanation = "GDM is not installed. This capability does not apply.".to_string();
    }

    result
}

/// Gather facts about GDM scaling state.
fn gather_gdm_scaling_facts() -> GdmScalingFacts {
    let gdm_installed = check_gdm_installed();
    let custom_conf_exists = Path::new("/etc/gdm/custom.conf").exists();
    let current_scale = read_current_scale();
    let hidpi_detected = detect_hidpi();
    let primary_resolution = detect_primary_resolution();

    GdmScalingFacts {
        gdm_installed,
        current_scale,
        hidpi_detected,
        primary_resolution,
        custom_conf_exists,
    }
}

/// Check if GDM is installed.
fn check_gdm_installed() -> bool {
    Path::new("/usr/bin/gdm").exists() || Path::new("/usr/sbin/gdm").exists()
}

/// Read current GDM scale factor.
fn read_current_scale() -> Option<f32> {
    // Check GDM dconf database
    let dconf_path = "/etc/dconf/db/gdm.d";
    if !Path::new(dconf_path).exists() {
        return None;
    }

    // In a real implementation, we would read the dconf database
    // For now, return None (not configured)
    None
}

/// Detect if HiDPI display is present.
fn detect_hidpi() -> bool {
    // Check common HiDPI indicators
    // In a real implementation, this would query display info
    if let Some((w, h)) = detect_primary_resolution() {
        // HiDPI typically >= 192 DPI (2x at 96 DPI baseline)
        // Common HiDPI resolutions: 2560x1440 on 13-14", 3840x2160 on 15-27"
        // Heuristic: if resolution > 2560 wide, likely HiDPI
        return w >= 2560 && h >= 1440;
    }
    false
}

/// Detect primary display resolution.
fn detect_primary_resolution() -> Option<(u32, u32)> {
    // Try to read from /sys or use xrandr output
    // For demonstration, return None
    // In real implementation, would parse xrandr or drm info
    None
}

/// Propose a scaling plan based on facts.
fn propose_scaling_plan(facts: &GdmScalingFacts) -> GdmScalingPlan {
    // Determine recommended scale
    let recommended_scale = if let Some((w, _)) = facts.primary_resolution {
        if w >= 3840 {
            2.0 // 4K
        } else if w >= 2560 {
            1.5 // QHD
        } else {
            1.0 // HD
        }
    } else {
        // Default to 2x for HiDPI
        2.0
    };

    let steps = vec![
        "Create /etc/dconf/profile/gdm if it doesn't exist".to_string(),
        "Create /etc/dconf/db/gdm.d/00-scaling".to_string(),
        format!(
            "Set org.gnome.desktop.interface scaling-factor to {}",
            recommended_scale as u32
        ),
        "Run dconf update to apply changes".to_string(),
        "Restart GDM service".to_string(),
    ];

    let affected_files = vec![
        "/etc/dconf/profile/gdm".to_string(),
        "/etc/dconf/db/gdm.d/00-scaling".to_string(),
    ];

    GdmScalingPlan {
        steps,
        recommended_scale,
        affected_files,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_produces_result() {
        let result = execute_display_scale_gdm();

        // Should always produce some output
        assert!(!result.facts.is_empty() || !result.explanation.is_empty());
    }

    #[test]
    fn test_facts_include_gdm_status() {
        let result = execute_display_scale_gdm();

        // Should report whether GDM is installed
        let has_gdm_fact = result
            .facts
            .iter()
            .any(|f| f.label == "GDM Installed");
        assert!(has_gdm_fact);
    }

    #[test]
    fn test_scaling_recommended_when_hidpi_unconfigured() {
        let facts = GdmScalingFacts {
            gdm_installed: true,
            current_scale: None,
            hidpi_detected: true,
            primary_resolution: Some((3840, 2160)),
            custom_conf_exists: false,
        };

        assert!(facts.scaling_recommended());
    }

    #[test]
    fn test_scaling_not_recommended_when_configured() {
        let facts = GdmScalingFacts {
            gdm_installed: true,
            current_scale: Some(2.0),
            hidpi_detected: true,
            primary_resolution: Some((3840, 2160)),
            custom_conf_exists: true,
        };

        assert!(!facts.scaling_recommended());
    }

    #[test]
    fn test_scaling_not_recommended_for_non_hidpi() {
        let facts = GdmScalingFacts {
            gdm_installed: true,
            current_scale: None,
            hidpi_detected: false,
            primary_resolution: Some((1920, 1080)),
            custom_conf_exists: false,
        };

        assert!(!facts.scaling_recommended());
    }

    #[test]
    fn test_plan_includes_steps() {
        let facts = GdmScalingFacts {
            gdm_installed: true,
            current_scale: None,
            hidpi_detected: true,
            primary_resolution: Some((3840, 2160)),
            custom_conf_exists: false,
        };

        let plan = propose_scaling_plan(&facts);

        assert!(!plan.steps.is_empty());
        assert!(!plan.affected_files.is_empty());
        assert!(plan.recommended_scale >= 1.0);
    }
}
