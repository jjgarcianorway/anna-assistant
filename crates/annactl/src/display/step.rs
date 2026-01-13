//! Step printing for dialogue display.

use anna_shared::rpc::{AskResult, StepType};

use super::colors::*;
use super::formatting::is_debug_mode;

/// Print a single dialogue step
fn print_step_internal(step: &anna_shared::rpc::DialogueStep, force_final_answer: bool) {
    let debug = is_debug_mode();

    match step.step_type {
        // ALWAYS VISIBLE
        StepType::UserQuestion => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::FinalAnswer => {
            if !step.content.is_empty() || force_final_answer {
                println!();
                print_colored("Anna: ", GREEN);
                if force_final_answer {
                    println!();
                }
                println!("{}", step.content);
                println!();
            }
        }
        StepType::ClarificationQuestion => {
            print_colored("Anna: ", YELLOW);
            println!("{}", step.content);
            println!();
        }
        StepType::ClarificationResponse => {
            print_colored("You: ", CYAN);
            println!("{}", step.content);
            println!();
        }
        StepType::IntentClassifying => {
            if debug {
                println_colored("  understanding question...", DIM);
            }
        }
        StepType::UnderstandingCheck => {
            print_colored("Anna: ", CYAN);
            println!("{}", step.content);
        }
        StepType::ConfirmationRequest => {
            println!();
            print_colored("Anna: ", YELLOW);
            println!("Please confirm:");
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::MissingInfo => {
            print_colored("Anna: ", RED);
            println!("Missing information:");
            for line in step.content.lines() {
                println!("  - {}", line);
            }
        }
        StepType::SystemAlert => {
            println!();
            println_colored("SYSTEM ALERT", YELLOW);
            for line in step.content.lines() {
                println!("  {}", line);
            }
            println!();
        }
        StepType::LlmError => {
            if debug {
                print_colored("Error: ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println!("{}", ctx.message);
                } else {
                    println!("{}", step.content);
                }
            } else {
                print_colored("  [X] ", RED);
                if let Ok(ctx) =
                    serde_json::from_str::<anna_shared::rpc::LlmErrorContext>(&step.content)
                {
                    println_colored(&ctx.message, RED);
                } else {
                    println_colored("An error occurred", RED);
                }
            }
            println!();
        }
        // Team dialogue (always visible - fly on the wall)
        StepType::TicketCreated => {
            println!();
            print_colored("Ticket ", CYAN);
            println_colored(&step.content, WHITE);
        }
        StepType::TeamAssignment => {
            print_colored("Anna -> ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::TeamDialogue => {
            println!("  {}", step.content);
        }
        StepType::TeamEscalation => {
            println!();
            print_colored("  [^] Escalating: ", YELLOW);
            println!("{}", step.content);
        }
        StepType::TeamDispatch => {
            print_colored("  ", DIM);
            println!("{}", step.content);
        }
        StepType::SpecialistWorking => {
            print_colored("  ", DIM);
            println_colored(&step.content, CYAN);
        }

        // Investigator mode (always visible - explicit entry/exit)
        StepType::InvestigationStart => {
            println!();
            print_colored("INVESTIGATING: ", CYAN);
            println!("{}", step.content);
        }
        StepType::InvestigationHypothesis => {
            print_colored("  Hypothesis: ", DIM);
            println!("{}", step.content);
        }
        StepType::InvestigationProbe => {
            print_colored("  Probe: ", DIM);
            println_colored(&step.content, CYAN);
        }
        StepType::InvestigationResult => {
            if debug {
                print_colored("    -> ", DIM);
                println!("{}", step.content);
            }
        }
        StepType::InvestigationComplete => {
            println!();
            print_colored("INVESTIGATION COMPLETE: ", GREEN);
            println!("{}", step.content);
        }
        StepType::ExperimentStart => {
            println!();
            print_colored("EXPERIMENT: ", MAGENTA);
            println!("{}", step.content);
        }
        StepType::ExperimentResult => {
            print_colored("  Result: ", DIM);
            println!("{}", step.content);
        }

        // DEBUG ONLY
        StepType::AnnaToLlm => {
            if debug {
                println_colored("  [prompt to LLM]", DIM);
            }
        }
        StepType::LlmCommands => {
            if debug {
                println_colored("  [LLM response]", DIM);
                if step.content != "NONE" && step.content != "DONE" {
                    for line in step.content.lines() {
                        let line = line.trim();
                        if !line.is_empty() {
                            print_colored("    $ ", DIM);
                            println_colored(line, CYAN);
                        }
                    }
                }
            }
        }
        StepType::CommandExec => {
            if debug {
                print_colored("  $ ", DIM);
                println!("{}", step.content);
            }
        }
        StepType::CommandOutput => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
        StepType::ValidationPrompt | StepType::ValidationResponse | StepType::FinalPrompt => {
            if debug {
                println_colored("  [internal]", DIM);
            }
        }
        StepType::WikiSearch => {
            if debug {
                println_colored("  Checking Arch Wiki...", DIM);
            }
        }
        StepType::WikiResults | StepType::WikiCommands => {
            if debug {
                println_colored("  [wiki results]", DIM);
            }
        }
        StepType::IntentResult => {
            if debug {
                println_colored(&format!("  intent: {}", step.content), DIM);
            }
        }
        StepType::SubQuestion | StepType::SubQuestionResult => {
            if debug {
                println_colored(&format!("  {}", step.content), DIM);
            }
        }
    }
}

/// Print a single dialogue step (streaming mode)
pub fn print_step(step: &anna_shared::rpc::DialogueStep) {
    print_step_internal(step, false);
}

/// Print the full dialogue
#[allow(dead_code)]
pub fn print_dialogue(result: &AskResult) {
    for step in &result.dialogue {
        print_step_internal(step, true);
    }
}

/// Print timeout error
pub fn print_timeout_error(timeout_secs: u64) {
    println!();
    println_colored("REQUEST TIMED OUT", RED);
    println!();
    println!("  The request took longer than {}s.", timeout_secs);
    println!();
    println_colored("Possible causes:", YELLOW);
    println!("  - Ollama model is loading (first query is slow)");
    println!("  - Complex question requiring many iterations");
    println!("  - LLM server is overloaded");
    println!();
    println_colored("Try:", GREEN);
    println!("  - Run again - model may be loaded now");
    println!("  - Check: annactl status");
    println!();
}
