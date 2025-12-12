//! ConfigureShell query handler.
//! v0.0.311: Deterministic fast-path for shell configuration questions.

use anna_shared::rpc::{ProbeResult, ServiceDeskResult, SpecialistDomain, TranslatorTicket};
use anna_shared::shell_recipes::{detect_feature, find_recipe, Shell, ShellFeature};
use anna_shared::trace::{EvidenceKind, ExecutionTrace, ProbeStats, SpecialistOutcome};
use anna_shared::transcript::Transcript;
use tracing::info;

use crate::service_desk::{self, FallbackContext};

/// Result of ConfigureShell handling
pub enum ConfigureShellResult {
    /// Handled completely - return this response
    Handled(ServiceDeskResult),
    /// Not applicable - continue with normal pipeline
    NotApplicable,
}

/// Handle ConfigureShell query class.
/// Detects user's shell and requested feature, returns recipe-based answer.
pub fn handle_configure_shell(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> ConfigureShellResult {
    // Detect user's shell from $SHELL
    let user_shell = Shell::detect();

    // Detect requested feature from query
    let feature = detect_feature(query);

    info!(
        "v0.0.311: ConfigureShell - shell={:?}, feature={:?}",
        user_shell, feature
    );

    // Build execution trace
    let probe_stats = ProbeStats::from_results(ticket.needs_probes.len(), probe_results);
    let evidence_kinds = vec![EvidenceKind::ToolExists];

    let fallback_ctx = FallbackContext {
        used_deterministic_fallback: false,
        fallback_route_class: "configure_shell".to_string(),
        evidence_kinds: vec!["shell_detect".to_string()],
        specialist_outcome: Some(SpecialistOutcome::Skipped),
        fallback_used: Some(anna_shared::trace::FallbackUsed::None),
        evidence_required: Some(false),
    };

    match (user_shell, feature) {
        (Some(shell), Some(feat)) => {
            // Have both shell and feature - try to find recipe
            if let Some(recipe) = find_recipe(shell, feat) {
                let result = build_recipe_response(
                    request_id,
                    query,
                    ticket,
                    probe_results,
                    transcript,
                    classified_domain,
                    &shell,
                    &recipe,
                    probe_stats,
                    evidence_kinds,
                    fallback_ctx,
                );
                ConfigureShellResult::Handled(result)
            } else {
                // No recipe for this combo - ask what they want
                let result = build_no_recipe_response(
                    request_id,
                    query,
                    ticket,
                    probe_results,
                    transcript,
                    classified_domain,
                    &shell,
                    &feat,
                    probe_stats,
                    evidence_kinds,
                    fallback_ctx,
                );
                ConfigureShellResult::Handled(result)
            }
        }
        (Some(shell), None) => {
            // Have shell but unclear what feature - list available options
            let result = build_clarify_feature_response(
                request_id,
                query,
                ticket,
                probe_results,
                transcript,
                &shell,
            );
            ConfigureShellResult::Handled(result)
        }
        (None, Some(feat)) => {
            // Have feature but no shell detected - ask which shell
            let result =
                build_clarify_shell_response(request_id, ticket, probe_results, transcript, &feat);
            ConfigureShellResult::Handled(result)
        }
        (None, None) => {
            // Neither detected - fall back to LLM
            ConfigureShellResult::NotApplicable
        }
    }
}

/// Build response with a specific recipe
fn build_recipe_response(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    shell: &Shell,
    recipe: &anna_shared::shell_recipes::ShellRecipe,
    probe_stats: ProbeStats,
    evidence_kinds: Vec<EvidenceKind>,
    fallback_ctx: FallbackContext,
) -> ServiceDeskResult {
    let config_path = shell.config_path();
    let lines_to_add = recipe.lines.join("\n");

    let answer = format!(
        "To {} in {}, add these lines to {}:\n\n```bash\n{}\n```\n\n{}",
        recipe.description.to_lowercase(),
        shell.display_name(),
        config_path.display(),
        lines_to_add,
        recipe
            .rollback_hint
            .as_ref()
            .map(|h| format!("To undo: {}", h))
            .unwrap_or_default()
    );

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
        0,
        false,
        fallback_ctx,
    );

    result.execution_trace = Some(ExecutionTrace::deterministic_route(
        "configure_shell",
        probe_stats,
        evidence_kinds,
    ));

    result
}

/// Build response when no recipe exists for shell+feature combo
fn build_no_recipe_response(
    request_id: String,
    query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
    shell: &Shell,
    feature: &ShellFeature,
    probe_stats: ProbeStats,
    evidence_kinds: Vec<EvidenceKind>,
    fallback_ctx: FallbackContext,
) -> ServiceDeskResult {
    let answer = format!(
        "I don't have a built-in recipe for {} in {}.\n\n\
         You might need to install additional packages or check the {} documentation.",
        feature.display_name(),
        shell.display_name(),
        shell.display_name()
    );

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
        0,
        false,
        fallback_ctx,
    );

    result.execution_trace = Some(ExecutionTrace::deterministic_route(
        "configure_shell",
        probe_stats,
        evidence_kinds,
    ));

    result
}

/// Build response asking which feature they want
fn build_clarify_feature_response(
    request_id: String,
    _query: &str,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    shell: &Shell,
) -> ServiceDeskResult {
    let features = vec![
        ("colored_prompt", "Colored command prompt"),
        ("git_prompt", "Show git branch in prompt"),
        ("syntax", "Syntax highlighting"),
        ("history", "Improved history settings"),
        ("aliases", "Common aliases"),
    ];

    let feature_list: Vec<String> = features
        .iter()
        .enumerate()
        .map(|(i, (_, desc))| format!("{}) {}", i + 1, desc))
        .collect();

    let answer = format!(
        "I can help configure {}. What would you like to set up?\n{}\nReply with the number.",
        shell.display_name(),
        feature_list.join("\n")
    );

    let options: Vec<(String, String)> = features
        .iter()
        .map(|(id, desc)| (id.to_string(), desc.to_string()))
        .collect();

    let mut result = service_desk::create_clarification_with_options(
        request_id,
        ticket,
        &answer,
        options,
        probe_results.to_vec(),
        transcript,
    );
    result.answer = answer;

    result
}

/// Build response asking which shell they use
fn build_clarify_shell_response(
    request_id: String,
    ticket: TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    feature: &ShellFeature,
) -> ServiceDeskResult {
    let shells = vec![("bash", "Bash"), ("zsh", "Zsh"), ("fish", "Fish")];

    let shell_list: Vec<String> = shells
        .iter()
        .enumerate()
        .map(|(i, (_, name))| format!("{}) {}", i + 1, name))
        .collect();

    let answer = format!(
        "I can help with {}. Which shell do you use?\n{}\nReply with the number.",
        feature.display_name(),
        shell_list.join("\n")
    );

    let options: Vec<(String, String)> = shells
        .iter()
        .map(|(id, name)| (id.to_string(), name.to_string()))
        .collect();

    let mut result = service_desk::create_clarification_with_options(
        request_id,
        ticket,
        &answer,
        options,
        probe_results.to_vec(),
        transcript,
    );
    result.answer = answer;

    result
}
