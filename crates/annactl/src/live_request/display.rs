//! Display functions for progress events.
//!
//! Handles real-time display of internal comms, streaming tokens,
//! spinners, and stage transitions in fly-on-wall style.

use anna_shared::progress::{ProgressEvent, ProgressEventType};
use anna_shared::roster;
use anna_shared::ui::colors;
use anna_shared::user_profile::UserProfile;
use std::io::{self, Write};

use super::state::StreamingState;

/// Display a progress event in fly-on-wall style
/// v0.0.237: Enhanced conversational format with better styling
/// v0.0.238: Added streaming token support
/// v0.0.253: Enhanced with role titles and internal comms header
/// v0.0.278: Hollywood-style stage transitions with animated spinners
pub fn display_progress_event(event: &ProgressEvent, state: &mut StreamingState) {
    // Check user preference for internal comms
    let profile = UserProfile::load();
    let show_internal = profile.preferences.show_internal_comms;

    match &event.event {
        ProgressEventType::Starting { timeout_secs: _ } => {
            // v0.0.278: Show stage transition with Hollywood flair
            let stage_name = match event.stage {
                anna_shared::progress::RequestStage::Translator => "classifying query",
                anna_shared::progress::RequestStage::Probes => "gathering system data",
                anna_shared::progress::RequestStage::Specialist => "consulting specialist",
                anna_shared::progress::RequestStage::Supervisor => "verifying answer",
            };
            // Clear any previous spinner
            if state.spinner_active {
                print!("\r{}\r", " ".repeat(60));
            }
            state.current_stage = Some(stage_name.to_string());
            state.spinner_active = true;
        }
        ProgressEventType::InternalComms { from, message } => {
            if !show_internal {
                return;
            }
            // Clear spinner if active
            if state.spinner_active {
                print!("\r{}\r", " ".repeat(60));
                state.spinner_active = false;
            }
            // If we were streaming, end the line first
            if state.started_streaming && !state.at_line_start {
                println!();
                state.at_line_start = true;
            }
            // v0.0.253: Show internal comms header on first message
            if !state.shown_internal_header {
                println!("{}--- internal comms ---{}", colors::DIM, colors::RESET);
                state.shown_internal_header = true;
            }
            // v0.0.312: Show internal comms with timestamp and role titles
            display_internal_comms(from, message, event.elapsed_ms);
            let _ = io::stdout().flush();
        }
        ProgressEventType::Generation { tokens } => {
            // Only show generation progress if not streaming tokens
            if !state.started_streaming {
                let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];
                let frame = (*tokens / 5) % spinner.len();
                let stage_desc = state.current_stage.as_deref().unwrap_or("thinking");
                print!(
                    "\r  {}{}{} {}... {} tokens",
                    colors::CYAN,
                    spinner[frame],
                    colors::RESET,
                    stage_desc,
                    tokens
                );
                state.spinner_active = true;
                let _ = io::stdout().flush();
            }
        }
        ProgressEventType::StreamingToken { token, is_final } => {
            // Clear spinner line if this is first token
            if !state.started_streaming {
                print!("\r{}\r", " ".repeat(60));
                state.started_streaming = true;
                state.at_line_start = true;
                state.spinner_active = false;
            }
            // Print the token (word-by-word output)
            print!("{}", token);
            state.at_line_start = token.ends_with('\n');
            let _ = io::stdout().flush();

            // If final, ensure we end on a newline
            if *is_final && !state.at_line_start {
                println!();
                state.at_line_start = true;
            }
        }
        ProgressEventType::Complete => {
            // Clear generation line if needed (but not if we were streaming)
            if state.spinner_active && !state.started_streaming {
                print!("\r{}\r", " ".repeat(60));
                state.spinner_active = false;
                let _ = io::stdout().flush();
            }
        }
        // v0.0.320: Show probes when running
        ProgressEventType::ProbeRunning { probe_id } => {
            if show_internal {
                // Clear spinner if active
                if state.spinner_active {
                    print!("\r{}\r", " ".repeat(60));
                    state.spinner_active = false;
                }
                // Show internal comms header if not shown
                if !state.shown_internal_header {
                    println!("{}--- internal comms ---{}", colors::DIM, colors::RESET);
                    state.shown_internal_header = true;
                }
                // Show probe as it runs
                let ts = format!("{:.1}s", event.elapsed_ms as f64 / 1000.0);
                println!(
                    "  {}[{}]{} {}[probe]{} {}",
                    colors::DIM,
                    ts,
                    colors::RESET,
                    colors::CYAN,
                    colors::RESET,
                    probe_id
                );
                let _ = io::stdout().flush();
            }
        }
        // v0.0.320: Show probe completion with exit code
        ProgressEventType::ProbeComplete {
            probe_id,
            exit_code,
            timing_ms,
        } => {
            if show_internal {
                // Show probe result if it failed or took long
                if *exit_code != 0 || *timing_ms > 1000 {
                    let ts = format!("{:.1}s", event.elapsed_ms as f64 / 1000.0);
                    let status = if *exit_code == 0 {
                        format!("{}ok{}", colors::OK, colors::RESET)
                    } else {
                        format!("{}exit={}{}", colors::ERR, exit_code, colors::RESET)
                    };
                    println!(
                        "  {}[{}]{} {}[probe]{} {} → {} ({}ms)",
                        colors::DIM,
                        ts,
                        colors::RESET,
                        colors::CYAN,
                        colors::RESET,
                        probe_id,
                        status,
                        timing_ms
                    );
                    let _ = io::stdout().flush();
                }
            }
        }
        _ => {
            // Other events are handled silently
        }
    }
}

/// v0.0.312: Display internal comms with role lookups and timestamps
fn display_internal_comms(from: &str, message: &str, elapsed_ms: u64) {
    // Format timestamp as seconds with one decimal
    let ts = format!("{:.1}s", elapsed_ms as f64 / 1000.0);

    if from == "Anna" {
        println!(
            "  {}[{}]{} {}Anna:{} {}",
            colors::DIM,
            ts,
            colors::RESET,
            colors::OK,
            colors::RESET,
            message
        );
    } else {
        // Try to look up the person by display name
        if let Some(person) = roster::person_by_display_name(from) {
            println!(
                "  {}[{}]{} {}{} ({}):{} {}",
                colors::DIM,
                ts,
                colors::RESET,
                colors::WARN,
                person.display_name,
                person.role_title,
                colors::RESET,
                message
            );
        } else {
            // Fallback if name not in roster
            println!(
                "  {}[{}]{} {}{}:{} {}",
                colors::DIM,
                ts,
                colors::RESET,
                colors::WARN,
                from,
                colors::RESET,
                message
            );
        }
    }
}
