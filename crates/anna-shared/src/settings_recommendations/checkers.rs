// v0.0.578: Settings Recommendations - Checkers (Phase 154)
// Functions to check settings and generate recommendations

use crate::unified_settings::{SettingsCategory, UnifiedSettings};

use super::types::{Recommendation, RecommendationPriority, RecommendationType};

/// Check security settings and generate recommendations
pub fn check_security(recommendations: &mut Vec<Recommendation>, next_id: &mut u64, settings: &UnifiedSettings) {
    // Check require_root_confirmation
    if !settings.risk.require_root_confirmation {
        recommendations.push(
            Recommendation::new(
                *next_id,
                RecommendationType::Security,
                SettingsCategory::Risk,
                "require_root_confirmation",
            )
            .priority(RecommendationPriority::High)
            .current("Disabled")
            .recommended("Enabled")
            .reason("Root confirmation helps prevent accidental system changes"),
        );
        *next_id += 1;
    }

    // Check confirmation settings
    if !settings.confirmation.always_confirm_delete {
        recommendations.push(
            Recommendation::new(
                *next_id,
                RecommendationType::Security,
                SettingsCategory::Confirmation,
                "always_confirm_delete",
            )
            .priority(RecommendationPriority::Critical)
            .current("Disabled")
            .recommended("Enabled")
            .reason("Confirmation for delete actions prevents data loss"),
        );
        *next_id += 1;
    }
}

/// Check privacy settings and generate recommendations
pub fn check_privacy(recommendations: &mut Vec<Recommendation>, next_id: &mut u64, settings: &UnifiedSettings) {
    // Check telemetry
    if settings.privacy.allow_telemetry {
        recommendations.push(
            Recommendation::new(
                *next_id,
                RecommendationType::Privacy,
                SettingsCategory::Privacy,
                "allow_telemetry",
            )
            .priority(RecommendationPriority::Low)
            .current("Enabled")
            .recommended("Disabled")
            .reason("Disabling telemetry improves privacy"),
        );
        *next_id += 1;
    }
}

/// Check usability settings and generate recommendations
pub fn check_usability(recommendations: &mut Vec<Recommendation>, next_id: &mut u64, settings: &UnifiedSettings) {
    // Check verbosity
    if !settings.verbosity.show_progress {
        recommendations.push(
            Recommendation::new(
                *next_id,
                RecommendationType::Usability,
                SettingsCategory::Verbosity,
                "show_progress",
            )
            .priority(RecommendationPriority::Low)
            .current("Disabled")
            .recommended("Enabled")
            .reason("Progress indicators help track long-running operations"),
        );
        *next_id += 1;
    }
}

/// Check performance settings and generate recommendations
pub fn check_performance(recommendations: &mut Vec<Recommendation>, next_id: &mut u64, settings: &UnifiedSettings) {
    // Check timeout
    if settings.timeout.command_timeout_ms > 60000 {
        recommendations.push(
            Recommendation::new(
                *next_id,
                RecommendationType::Performance,
                SettingsCategory::Timeout,
                "command_timeout_ms",
            )
            .priority(RecommendationPriority::Medium)
            .current(format!("{}ms", settings.timeout.command_timeout_ms))
            .recommended("60000ms or less")
            .reason("Lower timeout prevents hanging on slow operations"),
        );
        *next_id += 1;
    }
}
