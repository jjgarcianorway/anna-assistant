//! Desktop wallpaper query handler (v0.0.309).
//!
//! Fast-path handler for wallpaper queries:
//! - Checks probe for current wallpaper from DE settings
//! - If found, shows the current wallpaper path
//! - If unknown DE, asks user where they keep their wallpapers

use anna_shared::rpc::{
    ProbeResult, ReliabilitySignals, ServiceDeskResult, SpecialistDomain, TranslatorTicket,
};
use anna_shared::trace::{ExecutionTrace, FallbackUsed, ProbeStats, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::parsers::find_probe;
use crate::service_desk::get_relevant_hardware_fields;

/// Result of DesktopWallpaper handling
pub enum DesktopWallpaperResult {
    /// Handled completely - return this response
    Handled(ServiceDeskResult),
    /// Not applicable - continue with normal pipeline
    NotApplicable,
}

/// Handle DesktopWallpaper query class.
/// v0.0.309: Deterministic fast-path for wallpaper queries
pub fn handle_desktop_wallpaper(
    request_id: String,
    _query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
) -> DesktopWallpaperResult {
    let probe = find_probe(probe_results, "desktop_wallpaper");

    let (answer, needs_clarification, clarify_question) = if let Some(p) = probe {
        let output = p.stdout.trim();

        if output.contains("UNKNOWN_DE") || output.is_empty() || p.exit_code != 0 {
            // Desktop environment not detected or no wallpaper setting found
            info!("v0.0.309: Desktop wallpaper - DE not detected or no setting found");
            (
                String::new(),
                true,
                Some("I couldn't detect your desktop environment's wallpaper. Where do you keep your wallpaper files? (e.g., ~/Pictures/Wallpapers)".to_string()),
            )
        } else {
            // Found wallpaper setting - parse it
            let wallpaper_path = parse_wallpaper_output(output);
            info!("v0.0.309: Desktop wallpaper found: {}", wallpaper_path);
            (
                format!("Your current wallpaper is: {}", wallpaper_path),
                false,
                None,
            )
        }
    } else {
        // No probe result - ask user
        info!("v0.0.309: Desktop wallpaper - no probe result");
        (
            String::new(),
            true,
            Some("I couldn't check your wallpaper settings. Where do you keep your wallpaper files? (e.g., ~/Pictures/Wallpapers)".to_string()),
        )
    };

    let probe_stats = ProbeStats::from_results(ticket.needs_probes.len(), probe_results);
    let signals = ReliabilitySignals {
        translator_confident: true,
        probe_coverage: probe.is_some(),
        answer_grounded: !needs_clarification,
        no_invention: true,
        clarification_not_needed: !needs_clarification,
    };

    let execution_trace = Some(ExecutionTrace {
        specialist_outcome: SpecialistOutcome::Skipped,
        fallback_used: FallbackUsed::Deterministic {
            route_class: "desktop_wallpaper".to_string(),
        },
        probe_stats,
        evidence_kinds: vec![],
        answer_is_deterministic: true,
        reviewer_outcome: None,
    });

    let result = ServiceDeskResult {
        request_id,
        case_number: None,
        assigned_staff: None,
        staff_id: None,
        answer,
        validated: !needs_clarification,
        reliability_score: if needs_clarification { 50 } else { 85 },
        reliability_signals: signals,
        reliability_explanation: None,
        domain: SpecialistDomain::System,
        evidence: anna_shared::rpc::EvidenceBlock {
            hardware_fields: get_relevant_hardware_fields(&ticket),
            probes_executed: probe_results.to_vec(),
            translator_ticket: ticket,
            last_error: None,
        },
        needs_clarification,
        clarification_question: clarify_question,
        clarification_request: None,
        transcript,
        execution_trace,
        proposed_change: None,
        proposed_changes: Vec::new(),
        feedback_request: None,
    };

    DesktopWallpaperResult::Handled(result)
}

/// Parse wallpaper output from various DE formats
fn parse_wallpaper_output(output: &str) -> String {
    // GNOME/Cinnamon: 'file:///path/to/image.jpg'
    if output.starts_with("'file://") {
        return output
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .trim_start_matches("file://")
            .to_string();
    }

    // MATE: '/path/to/image.jpg'
    if output.starts_with('\'') && output.ends_with('\'') {
        return output
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .to_string();
    }

    // Hyprland: wallpaper = monitor,/path/to/image.jpg
    if output.contains("wallpaper") && output.contains(',') {
        if let Some(path) = output.split(',').nth(1) {
            return path.trim().to_string();
        }
    }

    // Default: return as-is
    output.to_string()
}
