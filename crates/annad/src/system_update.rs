//! SystemUpdate query handler.
//! v0.0.311: Deterministic fast-path for "update my system" requests.
//! v0.0.312: Returns RunCommand for user-approved execution.

use anna_shared::change::{plan_run_command, ChangeRisk};
use anna_shared::rpc::{ProbeResult, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::service_desk::{self, FallbackContext};

/// Result of SystemUpdate handling
pub enum SystemUpdateResult {
    /// Handled completely - return this response
    Handled(ServiceDeskResult),
    /// Not applicable - continue with normal pipeline
    NotApplicable,
}

/// Detect package manager from OS
fn detect_package_manager() -> (&'static str, &'static str) {
    // Check for common package managers
    if std::path::Path::new("/usr/bin/pacman").exists() {
        ("pacman", "sudo pacman -Syu")
    } else if std::path::Path::new("/usr/bin/apt").exists() {
        ("apt", "sudo apt update && sudo apt upgrade")
    } else if std::path::Path::new("/usr/bin/dnf").exists() {
        ("dnf", "sudo dnf upgrade")
    } else if std::path::Path::new("/usr/bin/zypper").exists() {
        ("zypper", "sudo zypper update")
    } else if std::path::Path::new("/usr/bin/emerge").exists() {
        ("portage", "sudo emerge --sync && sudo emerge -uDN @world")
    } else {
        ("unknown", "")
    }
}

/// Count available updates from probe results
fn count_updates_from_probe(probe_results: &[ProbeResult]) -> Option<usize> {
    for result in probe_results {
        // Check stdout for package_updates probe
        if result.command.contains("checkupdates") || result.command.contains("pacman -Qu") {
            if result.exit_code == 0 {
                let count = result.stdout.lines().filter(|l| !l.trim().is_empty()).count();
                return Some(count);
            }
        }
    }
    None
}

/// Handle SystemUpdate query class.
/// Proposes the system update command with confirmation.
pub fn handle_system_update(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> SystemUpdateResult {
    let (pkg_manager, update_cmd) = detect_package_manager();

    info!(
        "v0.0.311: SystemUpdate - package_manager={}, command={}",
        pkg_manager, update_cmd
    );

    if pkg_manager == "unknown" || update_cmd.is_empty() {
        // Can't determine package manager
        return SystemUpdateResult::NotApplicable;
    }

    // Build execution trace
    let probe_stats = ProbeStats::from_results(ticket.needs_probes.len(), probe_results);
    let evidence_kinds = vec![EvidenceKind::Packages];

    let fallback_ctx = FallbackContext {
        used_deterministic_fallback: false,
        fallback_route_class: "system_update".to_string(),
        evidence_kinds: vec!["packages".to_string()],
        specialist_outcome: Some(SpecialistOutcome::Skipped),
        fallback_used: Some(anna_shared::trace::FallbackUsed::None),
        evidence_required: Some(false),
    };

    // Check if we have update count from probe
    let update_count = count_updates_from_probe(probe_results);

    // v0.0.312: Build answer text and proposed command
    let (answer, command_plan) = match update_count {
        Some(0) => (
            "Your system is already up to date! No packages need updating.".to_string(),
            None,
        ),
        Some(count) => {
            let desc = format!("Update {} packages using {}", count, pkg_manager);
            (
                format!(
                    "I can update your system ({} packages available).\n\n\
                     Command: `{}`\n\n\
                     This will download and install all available updates using {}.\n\n\
                     **Risk: Medium** - System packages will be modified.",
                    count, update_cmd, pkg_manager
                ),
                Some(plan_run_command(update_cmd, &desc, ChangeRisk::Medium)),
            )
        }
        None => {
            let desc = format!("Update system packages using {}", pkg_manager);
            (
                format!(
                    "I can update your system.\n\n\
                     Command: `{}`\n\n\
                     This will download and install all available updates using {}.\n\n\
                     **Risk: Medium** - System packages will be modified.",
                    update_cmd, pkg_manager
                ),
                Some(plan_run_command(update_cmd, &desc, ChangeRisk::Medium)),
            )
        }
    };

    let mut result = service_desk::build_result_with_flags(
        request_id,
        answer,
        query,
        ticket,
        probe_results.to_vec(),
        transcript,
        classified_domain,
        false,
        true,
        probe_results.len(),
        false,
        fallback_ctx,
    );

    // v0.0.312: Add proposed command for user approval
    if let Some(plan) = command_plan {
        result.proposed_changes.push(plan);
    }

    result.execution_trace = Some(ExecutionTrace::deterministic_route(
        "system_update",
        probe_stats,
        evidence_kinds,
    ));

    SystemUpdateResult::Handled(result)
}
