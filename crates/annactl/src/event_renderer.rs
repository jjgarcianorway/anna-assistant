//! Unified event renderer for consistent CLI output.
//! v0.3.21: Truth-first rendering - no fake streaming, honest progress.
//!
//! This module provides consistent rendering across all CLI modes:
//! - One-shot queries
//! - REPL mode
//! - Status command
//! - Stats command
//! - Reset command
//!
//! Design principles:
//! - If we have real token streaming, show it
//! - If we don't, show honest progress (spinner + step updates)
//! - Never fake word-by-word output for non-streaming responses

use anna_shared::event_bus::{Event, LlmPurpose, StepType};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::display::{
    print_colored, println_colored, BOLD, CYAN, DIM, GREEN, MAGENTA, RED, RESET, WHITE, YELLOW,
};
use crate::spinner::Spinner;

/// Renderer mode determines output behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RenderMode {
    /// One-shot query - show progress then final answer
    OneShot,
    /// REPL mode - more verbose internal comms
    Repl,
    /// Status display - structured sections
    Status,
    /// Stats display - tables and metrics
    Stats,
    /// Reset operation - progress and confirmation
    Reset,
}

/// Unified event renderer
pub struct EventRenderer {
    mode: RenderMode,
    debug: bool,
    spinner: Option<Spinner>,
    current_step: Option<String>,
    step_start: Option<Instant>,
    streaming_answer: bool,
    answer_buffer: String,
}

impl EventRenderer {
    /// Create a new renderer
    pub fn new(mode: RenderMode, debug: bool) -> Self {
        Self {
            mode,
            debug,
            spinner: None,
            current_step: None,
            step_start: None,
            streaming_answer: false,
            answer_buffer: String::new(),
        }
    }

    /// Start a spinner with message
    pub fn start_spinner(&mut self, message: &str) {
        self.stop_spinner();
        self.spinner = Some(Spinner::new(message));
    }

    /// Stop any running spinner
    pub fn stop_spinner(&mut self) {
        if let Some(spinner) = self.spinner.take() {
            drop(spinner);
        }
    }

    /// Render an event from the EventBus
    pub fn render_event(&mut self, event: &Event) {
        match event {
            Event::StepStarted { step_id, step_type, description } => {
                self.render_step_started(step_type, description);
            }
            Event::StepFinished { step_id, step_type, duration_ms, success } => {
                self.render_step_finished(step_type, *duration_ms, *success);
            }
            Event::ProbeStarted { probe_id, command, display_command } => {
                self.render_probe_started(display_command);
            }
            Event::ProbeFinished { probe_id, exit_code, output_summary, duration_ms } => {
                self.render_probe_finished(*exit_code, output_summary, *duration_ms);
            }
            Event::LlmStarted { request_id, purpose, model } => {
                self.render_llm_started(purpose, model);
            }
            Event::LlmToken { request_id, token } => {
                self.render_llm_token(token);
            }
            Event::LlmFinished { request_id, duration_ms, tokens_used, success } => {
                self.render_llm_finished(*duration_ms, *tokens_used, *success);
            }
            Event::Warning { code, message, source } => {
                self.render_warning(code, message, source.as_deref());
            }
            Event::Error { code, message, source, recoverable } => {
                self.render_error(code, message, source.as_deref(), *recoverable);
            }
            Event::Progress { operation, current, total, message } => {
                self.render_progress(operation, *current, *total, message.as_deref());
            }
            Event::AnswerReady { answer, confidence, citations } => {
                self.render_answer(answer, *confidence, citations);
            }
            Event::InvestigationNeeded { reason, suggested_probes } => {
                self.render_investigation_needed(reason, suggested_probes);
            }
            _ => {
                // Skill events - only show in debug mode
                if self.debug {
                    self.render_debug_event(event);
                }
            }
        }
    }

    fn render_step_started(&mut self, step_type: &StepType, description: &str) {
        self.stop_spinner();
        self.current_step = Some(description.to_string());
        self.step_start = Some(Instant::now());

        // Update spinner with step description
        let display = step_type.display_name();
        self.start_spinner(display);

        if self.debug {
            print_colored(&format!("[step] {} - {}", display, description), DIM);
            println!();
        }
    }

    fn render_step_finished(&mut self, step_type: &StepType, duration_ms: u64, success: bool) {
        self.stop_spinner();

        if self.debug {
            let status = if success { "done" } else { "failed" };
            let color = if success { DIM } else { RED };
            print_colored(&format!("[step] {} {} ({}ms)", step_type.display_name(), status, duration_ms), color);
            println!();
        }

        self.current_step = None;
        self.step_start = None;
    }

    fn render_probe_started(&mut self, display_command: &str) {
        if self.debug {
            print_colored("$ ", DIM);
            println_colored(display_command, DIM);
        }
    }

    fn render_probe_finished(&mut self, exit_code: i32, output_summary: &str, duration_ms: u64) {
        if self.debug {
            let status = if exit_code == 0 { "ok" } else { &format!("exit {}", exit_code) };
            print_colored(&format!("  ({}, {}ms)", status, duration_ms), DIM);
            println!();
            if !output_summary.is_empty() {
                for line in output_summary.lines().take(3) {
                    print_colored("  ", DIM);
                    println_colored(line, DIM);
                }
            }
        }
    }

    fn render_llm_started(&mut self, purpose: &LlmPurpose, model: &str) {
        let display = purpose.display_name();
        self.start_spinner(display);

        if self.debug {
            print_colored(&format!("[llm] {} using {}", display, model), DIM);
            println!();
        }
    }

    fn render_llm_token(&mut self, token: &str) {
        // Only render tokens if we're in streaming mode
        // This is REAL streaming from Ollama API
        if !self.streaming_answer {
            self.stop_spinner();
            self.streaming_answer = true;
            // Print Anna prefix before first token
            print_colored("Anna: ", GREEN);
            io::stdout().flush().ok();
        }

        // Print token directly - this is real streaming
        print!("{}", token);
        io::stdout().flush().ok();
        self.answer_buffer.push_str(token);
    }

    fn render_llm_finished(&mut self, duration_ms: u64, tokens_used: Option<u32>, success: bool) {
        self.stop_spinner();

        if self.streaming_answer {
            // End the streaming answer line
            println!();
            self.streaming_answer = false;
        }

        if self.debug {
            let tokens = tokens_used.map(|t| format!("{} tokens", t)).unwrap_or_default();
            let status = if success { "done" } else { "failed" };
            print_colored(&format!("[llm] {} {} {}", status, duration_ms, tokens), DIM);
            println!();
        }
    }

    fn render_warning(&mut self, code: &str, message: &str, source: Option<&str>) {
        self.stop_spinner();
        print_colored("[!] ", YELLOW);
        print_colored(code, YELLOW);
        print!(": ");
        println!("{}", message);
        if let Some(src) = source {
            print_colored("    source: ", DIM);
            println_colored(src, DIM);
        }
    }

    fn render_error(&mut self, code: &str, message: &str, source: Option<&str>, recoverable: bool) {
        self.stop_spinner();
        print_colored("[X] ", RED);
        print_colored(code, RED);
        print!(": ");
        println!("{}", message);
        if let Some(src) = source {
            print_colored("    source: ", DIM);
            println_colored(src, DIM);
        }
        if !recoverable {
            print_colored("    (non-recoverable)", DIM);
            println!();
        }
    }

    fn render_progress(&mut self, operation: &str, current: u64, total: Option<u64>, message: Option<&str>) {
        if let Some(t) = total {
            let pct = (current as f64 / t as f64 * 100.0) as u32;
            let msg = message.unwrap_or(operation);
            self.start_spinner(&format!("{} {}%", msg, pct));
        } else {
            let msg = message.unwrap_or(operation);
            self.start_spinner(&format!("{} ({})", msg, current));
        }
    }

    /// Render a final answer (non-streaming path)
    /// This is the HONEST alternative to fake word-by-word streaming
    fn render_answer(&mut self, answer: &str, confidence: f32, citations: &[String]) {
        self.stop_spinner();

        // Don't re-render if we already streamed this answer
        if self.streaming_answer {
            self.streaming_answer = false;
            return;
        }

        println!();
        print_colored("Anna: ", GREEN);
        println!("{}", answer);

        // Show citations (evidence)
        if !citations.is_empty() {
            println!();
            print_colored("Evidence: ", DIM);
            println_colored(&citations.join(", "), DIM);
        }

        // Show confidence only in debug mode
        if self.debug && confidence < 1.0 {
            print_colored(&format!("  (confidence: {:.0}%)", confidence * 100.0), DIM);
            println!();
        }
    }

    fn render_investigation_needed(&mut self, reason: &str, suggested_probes: &[String]) {
        self.stop_spinner();
        println!();
        print_colored("Anna: ", YELLOW);
        println!("I need to investigate further.");
        println!();
        print_colored("Reason: ", DIM);
        println!("{}", reason);

        if !suggested_probes.is_empty() {
            println!();
            print_colored("Suggested diagnostics:", DIM);
            println!();
            for probe in suggested_probes {
                print_colored("  - ", DIM);
                println!("{}", probe);
            }
        }
    }

    fn render_debug_event(&mut self, event: &Event) {
        print_colored(&format!("[event] {:?}", event), DIM);
        println!();
    }
}

impl Drop for EventRenderer {
    fn drop(&mut self) {
        self.stop_spinner();
    }
}

/// Render a complete answer without fake streaming
/// Use this for non-streaming paths instead of splitting answer word-by-word
pub fn render_answer_honest(answer: &str, citations: &[String]) {
    println!();
    print_colored("Anna: ", GREEN);
    println!("{}", answer);

    if !citations.is_empty() {
        println!();
        print_colored("Evidence: ", DIM);
        println_colored(&citations.join(", "), DIM);
    }
}

/// Check if we should use real streaming or honest batch rendering
/// Returns true only if Ollama is configured for streaming
pub fn should_stream() -> bool {
    // For now, always prefer honest batch rendering
    // Real streaming is only available when directly talking to Ollama
    // Most code paths generate complete answers then fake-stream
    false
}
