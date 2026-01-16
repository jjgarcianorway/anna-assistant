//! Fast-path for common single-command queries.
//! Maps question patterns to (command, answer_template).

use anna_shared::rpc::{AskResult, Citation, DialogueStep, StepType};
use tracing::info;

use crate::core_loop::{execute_command, strip_ansi_codes};
use crate::department;

use super::fast_path_patterns::{
    get_hardware_fast_path, get_network_fast_path, get_package_fast_path,
    get_service_fast_path, get_system_fast_path,
};

/// Get fast-path command and template for simple queries.
/// Template uses {output} placeholder for command output.
pub fn get_fast_path(question: &str) -> Option<(&'static str, &'static str)> {
    let q = question.to_lowercase();

    // System info queries
    if let Some(result) = get_system_fast_path(&q) {
        return Some(result);
    }
    // Hardware queries
    if let Some(result) = get_hardware_fast_path(&q) {
        return Some(result);
    }
    // Network queries
    if let Some(result) = get_network_fast_path(&q) {
        return Some(result);
    }
    // Package queries
    if let Some(result) = get_package_fast_path(&q) {
        return Some(result);
    }
    // Service queries
    if let Some(result) = get_service_fast_path(&q) {
        return Some(result);
    }

    None
}

/// Try fast-path for simple queries, returning AskResult if matched.
pub async fn try_fast_path(question: &str) -> Option<AskResult> {
    let (cmd, template) = get_fast_path(question)?;

    info!("Fast-path: using command '{}'", cmd);

    match execute_command(cmd) {
        Ok(output) => {
            let clean_output = strip_ansi_codes(&output).trim().to_string();
            if clean_output.is_empty() {
                return None; // Fall back to full loop
            }

            let answer = template.replace("{output}", &clean_output);

            // Track ticket and specialist for stats even on fast-path
            let dept_name = department::determine_department(question);
            let mut ticket = department::create_ticket(question, dept_name);
            if let Some(spec) = department::get_specialist_for_topic(question) {
                ticket.assign(spec.name);
            }
            ticket.resolve(&answer, 5); // Fast-path = 5 XP
            department::update_ticket(&ticket);

            // Add citation for the command that grounded this answer
            let citation = Citation {
                source: format!("Command: {}", cmd),
                url: None,
                section: None,
            };

            Some(AskResult {
                answer,
                success: true,
                iterations: 0,
                commands_executed: vec![cmd.to_string()],
                dialogue: vec![
                    DialogueStep {
                        step_type: StepType::UserQuestion,
                        content: question.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::CommandExec,
                        content: cmd.to_string(),
                    },
                    DialogueStep {
                        step_type: StepType::CommandOutput,
                        content: clean_output,
                    },
                ],
                needs_clarification: false,
                clarification_question: None,
                cached: false,
                citations: vec![citation],
                abstained: false,
                final_confidence: None, // Fast-path doesn't track confidence
            })
        }
        Err(_) => None, // Fall back to full loop
    }
}
