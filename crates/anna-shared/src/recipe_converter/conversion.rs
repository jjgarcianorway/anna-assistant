//! Conversion logic for turning tickets and candidates into recipes.

use crate::recipe_engine::{
    EvidenceRequirement, Recipe, RecipeKind, RecipeStep, RecipeStepType,
};
use crate::ticket_log::TicketLog;
use tracing::{debug, info};

use super::types::SpecialistRecipeCandidate;
use super::types::SpecialistStepCandidate;
use super::validation::{is_eligible_for_recipe, validate_candidate};

/// Convert a successful ticket to a recipe
pub fn try_convert_ticket(
    ticket: &TicketLog,
    candidate: Option<&SpecialistRecipeCandidate>,
) -> Option<Recipe> {
    // Check eligibility
    if !is_eligible_for_recipe(ticket) {
        debug!("Ticket {} not eligible for recipe conversion", ticket.id);
        return None;
    }

    // If specialist provided a candidate, validate and use it
    if let Some(cand) = candidate {
        let validation = validate_candidate(cand);
        if !validation.valid {
            for err in &validation.errors {
                tracing::warn!("Recipe candidate validation failed: {}", err);
            }
            return None;
        }
        return Some(convert_candidate(cand, &ticket.id));
    }

    // Auto-generate recipe from ticket data
    auto_generate_recipe(ticket)
}

/// Convert candidate to full recipe
pub fn convert_candidate(candidate: &SpecialistRecipeCandidate, ticket_id: &str) -> Recipe {
    let recipe_id = generate_recipe_id(&candidate.name);

    let kind = match candidate.domain.as_str() {
        "services" | "system" => RecipeKind::Diagnose,
        "storage" => RecipeKind::Inspect,
        "packages" => RecipeKind::Inspect,
        "network" => RecipeKind::Inspect,
        _ => RecipeKind::ProbeOnly,
    };

    let evidence: Vec<EvidenceRequirement> = candidate
        .required_evidence
        .iter()
        .map(|e| parse_evidence_requirement(e))
        .collect();

    let steps: Vec<RecipeStep> = candidate
        .steps
        .iter()
        .enumerate()
        .map(|(i, s)| convert_step(s, i))
        .collect();

    Recipe::new(&recipe_id, &candidate.name, kind, &candidate.domain)
        .with_intent(&candidate.intent_pattern)
        .with_tags(candidate.tags.iter().map(|s| s.as_str()).collect())
        .with_evidence(evidence)
        .with_steps(steps)
        .with_docs(candidate.doc_sources.iter().map(|s| s.as_str()).collect())
        .from_ticket(ticket_id)
}

/// Convert step candidate to full step
fn convert_step(step: &SpecialistStepCandidate, index: usize) -> RecipeStep {
    let step_id = format!("s{}", index + 1);
    let kind = match step.kind.as_str() {
        "run_probe" => RecipeStepType::RunProbe,
        "run_command" => RecipeStepType::RunCommand,
        "check_condition" => RecipeStepType::CheckCondition,
        "edit_file" => RecipeStepType::EditFile,
        "render_answer" => RecipeStepType::RenderAnswer,
        "subrecipe" => RecipeStepType::Subrecipe,
        _ => RecipeStepType::RunCommand,
    };

    RecipeStep {
        id: step_id,
        kind,
        description: step.description.clone(),
        params: step.params.clone(),
        depends_on: if index > 0 {
            vec![format!("s{}", index)]
        } else {
            vec![]
        },
    }
}

/// Auto-generate recipe from ticket (when no specialist candidate)
pub fn auto_generate_recipe(ticket: &TicketLog) -> Option<Recipe> {
    let recipe_id = format!("auto-{}", ticket.id.to_lowercase());
    let kind = RecipeKind::ProbeOnly;

    let mut steps: Vec<RecipeStep> = ticket
        .probes
        .iter()
        .enumerate()
        .map(|(i, p)| {
            RecipeStep::command(
                &format!("s{}", i + 1),
                &p.command,
                &format!("Run: {}", p.id),
            )
        })
        .collect();

    // Add render step
    let render_template = format!(
        "Based on the commands above:\n\n{}",
        ticket.answer.chars().take(500).collect::<String>()
    );
    let render_step = RecipeStep::render(
        &format!("s{}", steps.len() + 1),
        &render_template,
        "Generate answer",
    );
    steps.push(render_step);

    let doc_sources: Vec<&str> = ticket.docs_used.iter().map(|d| d.title.as_str()).collect();

    Some(
        Recipe::new(
            &recipe_id,
            &format!("Auto: {}", ticket.domain),
            kind,
            &ticket.domain,
        )
        .with_intent(&ticket.query)
        .with_steps(steps)
        .with_docs(doc_sources)
        .from_ticket(&ticket.id),
    )
}

/// Parse evidence requirement string
pub fn parse_evidence_requirement(s: &str) -> EvidenceRequirement {
    match s.to_lowercase().as_str() {
        "meminfo" | "memory" => EvidenceRequirement::Meminfo,
        "swaps" | "swap" => EvidenceRequirement::Swaps,
        "df_root" | "disk" => EvidenceRequirement::DfRoot,
        "systemd_failed" | "failed_units" => EvidenceRequirement::SystemdFailed,
        "pacman_list" | "packages" => EvidenceRequirement::PacmanList,
        "journal_errors" | "logs" => EvidenceRequirement::JournalErrors,
        "network_interfaces" | "network" => EvidenceRequirement::NetworkInterfaces,
        "gpu_info" | "gpu" => EvidenceRequirement::GpuInfo,
        "audio_devices" | "audio" => EvidenceRequirement::AudioDevices,
        _ => EvidenceRequirement::Custom(s.to_string()),
    }
}

/// Generate unique recipe ID
pub fn generate_recipe_id(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
        .hash(&mut hasher);

    format!("recipe-{:08x}", hasher.finish() as u32)
}
