//! Mutation Transcript Rendering for Anna v0.0.80
//!
//! Fly-on-the-wall dialogue layer for mutation operations:
//! - Human mode: "Change manager" dialogue without internal details
//!   - Preview shows readable diff summaries
//!   - Risk level shown as colloquial phrases
//!   - No evidence IDs or internal state names
//! - Debug mode: Full mutation plan details
//!   - Plan IDs, step IDs, internal state names
//!   - Full diff output
//!   - Timing and privilege details

use crate::mutation_engine_v1::{
    ConfigEditOp, MutationCategory, MutationDetail, MutationPlanState, MutationPlanV1,
    MutationPreview, MutationRiskLevel, StepPreview,
};
use crate::transcript_events::TranscriptMode;
use serde::{Deserialize, Serialize};

// =============================================================================
// Transcript Mode Detection
// =============================================================================

/// Get current transcript mode (human vs debug)
pub fn get_transcript_mode() -> TranscriptMode {
    // Check env vars first
    if std::env::var("ANNA_DEBUG_TRANSCRIPT").is_ok() {
        return TranscriptMode::Debug;
    }
    if std::env::var("ANNA_UI_TRANSCRIPT_MODE")
        .map(|v| v == "debug")
        .unwrap_or(false)
    {
        return TranscriptMode::Debug;
    }
    TranscriptMode::Human
}

/// Check if we're in debug mode
pub fn is_debug_mode() -> bool {
    matches!(get_transcript_mode(), TranscriptMode::Debug)
}

// =============================================================================
// Human Mode Rendering
// =============================================================================

/// Render mutation plan header for human mode (Change Manager dialogue)
pub fn human_plan_header(plan: &MutationPlanV1) -> Vec<String> {
    let mut lines = Vec::new();

    // Change Manager announces intent
    let risk_phrase = human_risk_phrase(plan.risk);
    let category = match &plan.steps.first().map(|s| &s.category) {
        Some(MutationCategory::ServiceControl) => "service control",
        Some(MutationCategory::PackageManagement) => "package management",
        Some(MutationCategory::ConfigEdit) => "configuration edit",
        None => "system change",
    };

    lines.push("[change_manager] to [anna]:".to_string());
    lines.push(format!(
        "  I have a {} {}. Let me show you what will change.",
        risk_phrase, category
    ));

    lines
}

/// Render step preview for human mode
pub fn human_step_preview(step: &StepPreview) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("  {}:", step.description));
    lines.push(format!("    Before: {}", step.current_state));
    lines.push(format!("    After:  {}", step.intended_state));

    if let Some(ref diff) = step.diff {
        // Show simplified diff (first few lines)
        lines.push("    Changes:".to_string());
        for line in diff.lines().take(8) {
            if line.starts_with('+') && !line.starts_with("+++") {
                lines.push(format!("      [add] {}", &line[1..]));
            } else if line.starts_with('-') && !line.starts_with("---") {
                lines.push(format!("      [remove] {}", &line[1..]));
            }
        }
    }

    lines
}

/// Render mutation preview for human mode
pub fn human_mutation_preview(preview: &MutationPreview) -> Vec<String> {
    let mut lines = Vec::new();

    // Risk level and summary
    let risk_phrase = human_risk_phrase(preview.risk);
    lines.push("[change_manager] to [anna]:".to_string());
    lines.push(format!("  This is a {} change.", risk_phrase));
    lines.push("".to_string());

    // Summarize what will happen
    lines.push("  What will happen:".to_string());
    for step in &preview.step_previews {
        for line in human_step_preview(step) {
            lines.push(line);
        }
    }

    lines.push("".to_string());

    // Privilege status
    if !preview.privilege_available {
        lines.push(
            "  [note] I don't have permission to make these changes directly.".to_string(),
        );
        lines.push("  You can run these commands yourself:".to_string());
        if let Some(ref commands) = preview.manual_commands {
            for cmd in commands {
                lines.push(format!("    $ {}", cmd));
            }
        }
    }

    // Confirmation prompt
    lines.push("".to_string());
    lines.push(format!(
        "  To proceed, type: {}",
        preview.confirmation_phrase
    ));

    lines
}

/// Render confirmation received for human mode
pub fn human_confirmation_received(risk: MutationRiskLevel) -> Vec<String> {
    let risk_phrase = human_risk_phrase(risk);
    vec![
        "[anna] to [change_manager]:".to_string(),
        format!("  Confirmed. Proceeding with {} change.", risk_phrase),
    ]
}

/// Render execution progress for human mode
pub fn human_execution_step(step_desc: &str, success: bool) -> Vec<String> {
    let status = if success { "done" } else { "failed" };
    vec![format!(
        "[change_manager] to [anna]: {} ... {}",
        step_desc, status
    )]
}

/// Render verification result for human mode
pub fn human_verification_result(passed: bool, checks_passed: usize, total: usize) -> Vec<String> {
    let mut lines = Vec::new();

    if passed {
        lines.push("[change_manager] to [anna]:".to_string());
        lines.push(format!(
            "  Verification complete: {} of {} checks passed.",
            checks_passed, total
        ));
        lines.push("  Changes applied successfully.".to_string());
    } else {
        lines.push("[change_manager] to [anna]:".to_string());
        lines.push(format!(
            "  Verification issues: {} of {} checks failed.",
            total - checks_passed,
            total
        ));
        lines.push("  I can roll back the changes if needed.".to_string());
        lines.push(format!(
            "  To rollback, type: {}",
            MutationRiskLevel::rollback_phrase()
        ));
    }

    lines
}

/// Render rollback result for human mode
pub fn human_rollback_result(success: bool) -> Vec<String> {
    if success {
        vec![
            "[change_manager] to [anna]:".to_string(),
            "  Rollback complete. System restored to previous state.".to_string(),
        ]
    } else {
        vec![
            "[change_manager] to [anna]:".to_string(),
            "  Rollback encountered issues. Manual intervention may be needed.".to_string(),
        ]
    }
}

/// Render privilege blocked message for human mode
pub fn human_privilege_blocked(manual_commands: &[String]) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[change_manager] to [anna]:".to_string());
    lines.push("  I don't have permission to make these changes.".to_string());
    lines.push("  Please run these commands yourself:".to_string());

    for cmd in manual_commands {
        lines.push(format!("    $ {}", cmd));
    }

    lines.push("".to_string());
    lines.push("  After running, I can verify the changes took effect.".to_string());

    lines
}

/// Convert risk level to human-friendly phrase
fn human_risk_phrase(risk: MutationRiskLevel) -> &'static str {
    match risk {
        MutationRiskLevel::Low => "straightforward, low-risk",
        MutationRiskLevel::Medium => "moderate-risk",
        MutationRiskLevel::High => "significant, high-risk",
    }
}

// =============================================================================
// Debug Mode Rendering
// =============================================================================

/// Render mutation plan for debug mode
pub fn debug_plan_header(plan: &MutationPlanV1) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[mutation_engine] Plan details:".to_string());
    lines.push(format!("  plan_id: {}", plan.plan_id));
    if let Some(ref case_id) = plan.case_id {
        lines.push(format!("  case_id: {}", case_id));
    }
    lines.push(format!("  risk: {:?}", plan.risk));
    lines.push(format!("  state: {}", plan.state));
    lines.push(format!("  steps: {}", plan.steps.len()));
    lines.push(format!("  verification_checks: {}", plan.verification_checks.len()));
    lines.push(format!("  rollback_steps: {}", plan.rollback_steps.len()));

    lines
}

/// Render step preview for debug mode
pub fn debug_step_preview(step: &StepPreview) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("  Step [{}]:", step.step_id));
    lines.push(format!("    description: {}", step.description));
    lines.push(format!("    current: {}", step.current_state));
    lines.push(format!("    intended: {}", step.intended_state));

    if let Some(ref diff) = step.diff {
        lines.push("    diff:".to_string());
        for line in diff.lines() {
            lines.push(format!("      {}", line));
        }
    }

    lines
}

/// Render mutation preview for debug mode
pub fn debug_mutation_preview(preview: &MutationPreview) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[mutation_engine] Preview:".to_string());
    lines.push(format!("  plan_id: {}", preview.plan_id));
    lines.push(format!("  risk: {:?}", preview.risk));
    lines.push(format!("  confirmation: {}", preview.confirmation_phrase));
    lines.push(format!("  privilege_available: {}", preview.privilege_available));

    if let Some(ref commands) = preview.manual_commands {
        lines.push("  manual_commands:".to_string());
        for cmd in commands {
            lines.push(format!("    {}", cmd));
        }
    }

    lines.push("  step_previews:".to_string());
    for step in &preview.step_previews {
        for line in debug_step_preview(step) {
            lines.push(line);
        }
    }

    lines
}

/// Render execution step for debug mode
pub fn debug_execution_step(
    step_id: &str,
    step_desc: &str,
    success: bool,
    exit_code: Option<i32>,
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push(format!("[mutation_engine] Executing step [{}]:", step_id));
    lines.push(format!("  description: {}", step_desc));
    lines.push(format!("  success: {}", success));
    if let Some(code) = exit_code {
        lines.push(format!("  exit_code: {}", code));
    }

    lines
}

/// Render verification for debug mode
pub fn debug_verification_result(
    passed: bool,
    checks: &[(String, bool, String)],
) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[mutation_engine] Verification:".to_string());
    lines.push(format!("  overall_passed: {}", passed));
    lines.push("  checks:".to_string());

    for (desc, check_passed, actual) in checks {
        lines.push(format!("    {} [{}]: {}", desc, if *check_passed { "PASS" } else { "FAIL" }, actual));
    }

    lines
}

/// Render rollback for debug mode
pub fn debug_rollback_result(success: bool, steps: &[String], message: &str) -> Vec<String> {
    let mut lines = Vec::new();

    lines.push("[mutation_engine] Rollback:".to_string());
    lines.push(format!("  success: {}", success));
    lines.push(format!("  message: {}", message));
    lines.push("  steps_rolled_back:".to_string());
    for step in steps {
        lines.push(format!("    {}", step));
    }

    lines
}

/// Render state transition for debug mode
pub fn debug_state_transition(from: MutationPlanState, to: MutationPlanState) -> Vec<String> {
    vec![format!(
        "[mutation_engine] State: {} -> {}",
        from, to
    )]
}

// =============================================================================
// Combined Render (auto-selects mode)
// =============================================================================

/// Render plan header (auto-selects human/debug mode)
pub fn render_plan_header(plan: &MutationPlanV1) -> Vec<String> {
    if is_debug_mode() {
        debug_plan_header(plan)
    } else {
        human_plan_header(plan)
    }
}

/// Render mutation preview (auto-selects human/debug mode)
pub fn render_mutation_preview(preview: &MutationPreview) -> Vec<String> {
    if is_debug_mode() {
        debug_mutation_preview(preview)
    } else {
        human_mutation_preview(preview)
    }
}

/// Render execution step (auto-selects human/debug mode)
pub fn render_execution_step(
    step_id: &str,
    step_desc: &str,
    success: bool,
    exit_code: Option<i32>,
) -> Vec<String> {
    if is_debug_mode() {
        debug_execution_step(step_id, step_desc, success, exit_code)
    } else {
        human_execution_step(step_desc, success)
    }
}

/// Render verification result (auto-selects human/debug mode)
pub fn render_verification(
    passed: bool,
    checks: &[(String, bool, String)],
) -> Vec<String> {
    if is_debug_mode() {
        debug_verification_result(passed, checks)
    } else {
        let checks_passed = checks.iter().filter(|(_, p, _)| *p).count();
        human_verification_result(passed, checks_passed, checks.len())
    }
}

/// Render rollback result (auto-selects human/debug mode)
pub fn render_rollback(success: bool, steps: &[String], message: &str) -> Vec<String> {
    if is_debug_mode() {
        debug_rollback_result(success, steps, message)
    } else {
        human_rollback_result(success)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_risk_phrase() {
        assert_eq!(human_risk_phrase(MutationRiskLevel::Low), "straightforward, low-risk");
        assert_eq!(human_risk_phrase(MutationRiskLevel::Medium), "moderate-risk");
        assert_eq!(human_risk_phrase(MutationRiskLevel::High), "significant, high-risk");
    }

    #[test]
    fn test_human_confirmation_received() {
        let lines = human_confirmation_received(MutationRiskLevel::Medium);
        assert!(lines.len() >= 2);
        assert!(lines[0].contains("anna"));
        assert!(lines[1].contains("moderate-risk"));
    }

    #[test]
    fn test_human_execution_step() {
        let lines = human_execution_step("Install package vim", true);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("done"));

        let lines = human_execution_step("Install package vim", false);
        assert!(lines[0].contains("failed"));
    }

    #[test]
    fn test_human_verification_result() {
        let lines = human_verification_result(true, 3, 3);
        assert!(lines.iter().any(|l| l.contains("3 of 3")));

        let lines = human_verification_result(false, 1, 3);
        assert!(lines.iter().any(|l| l.contains("2 of 3")));
    }

    #[test]
    fn test_human_rollback_result() {
        let lines = human_rollback_result(true);
        assert!(lines.iter().any(|l| l.contains("restored")));

        let lines = human_rollback_result(false);
        assert!(lines.iter().any(|l| l.contains("Manual intervention")));
    }

    #[test]
    fn test_human_privilege_blocked() {
        let commands = vec!["sudo systemctl restart docker".to_string()];
        let lines = human_privilege_blocked(&commands);
        assert!(lines.iter().any(|l| l.contains("permission")));
        assert!(lines.iter().any(|l| l.contains("sudo systemctl")));
    }
}
