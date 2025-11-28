//! Output formatting - clean, ASCII-only terminal output v0.81.0
//!
//! v0.6.0: Sysadmin style - no emojis, ASCII only, professional
//! v0.10.0: Evidence-based answers with citations and confidence scores
//! v0.15.8: Timing display for spinner UX
//! v0.16.3: Debug trace display for development troubleshooting
//! v0.16.4: Improved debug trace with [JUNIOR model] and [SENIOR model] labels
//! v0.28.0: Strict ASCII - removed all emojis for hacker aesthetic
//! v0.72.0: Clear answer output block with Anna header, Evidence, Reliability sections
//! v0.81.0: Structured answer format - headline, details[], evidence[], reliability_label
#![allow(dead_code)]

use anna_common::{AnnaResponse, ConfidenceLevel, DebugTrace, FinalAnswer, StructuredAnswer, THIN_SEPARATOR};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Display a response to the user
pub fn display_response(response: &AnnaResponse) {
    // Confidence as percentage (v0.15.21)
    let conf_pct = (response.confidence * 100.0) as u8;
    let conf_str = format!("{}%", conf_pct);

    // v0.6.0: Color categories
    let (conf_colored, reliability_indicator) = if conf_pct >= 90 {
        (
            conf_str.bright_green().to_string(),
            "[OK]".bright_green().to_string(),
        )
    } else if conf_pct >= 70 {
        (
            conf_str.yellow().to_string(),
            "[PARTIAL]".yellow().to_string(),
        )
    } else {
        (
            conf_str.bright_red().to_string(),
            "[LOW]".bright_red().to_string(),
        )
    };

    // Header with reliability indicator (v0.6.0: ASCII only)
    println!();
    println!("{}  Reliability: {}", reliability_indicator, conf_colored);
    println!();

    // Answer - check if it's an insufficient evidence response
    if response.confidence < 0.70 {
        // Low reliability - format as warning
        println!("{}", response.answer.bright_red());
    } else {
        println!("{}", response.answer);
    }

    // Sources (v0.6.0: ASCII-only formatting)
    if !response.sources.is_empty() {
        println!();
        println!("[EVIDENCE]");
        for source in &response.sources {
            println!("  * [source: {}]", source.cyan());
        }
    }

    // Warning (v0.6.0: ASCII-only)
    if let Some(warning) = &response.warning {
        println!();
        if response.confidence < 0.70 {
            println!("[WARNING] {}", warning.bright_red());
        } else {
            println!("[NOTE] {}", warning.yellow());
        }
    }

    // v0.6.0: ASCII-only footer
    println!();
    if response.confidence >= 0.70 {
        println!("{}", THIN_SEPARATOR.dimmed());
        println!(
            "{}",
            "Evidence-based * No hallucinations * No guesses".dimmed()
        );
    }
    println!();
}

/// Display an error (v0.6.0: ASCII-only)
pub fn display_error(message: &str) {
    eprintln!();
    eprintln!("[ERROR] {}", message.red());
    eprintln!();
}

/// Display a success message (v0.6.0: ASCII-only)
pub fn display_success(message: &str) {
    println!();
    println!("[OK] {}", message.green());
    println!();
}

/// Display an info message (v0.6.0: ASCII-only)
pub fn display_info(message: &str) {
    println!("[INFO] {}", message);
}

/// Display a warning (v0.6.0: ASCII-only)
pub fn display_warning(message: &str) {
    println!("[WARNING] {}", message.yellow());
}

/// Display insufficient evidence (v0.6.0: ASCII-only)
pub fn display_insufficient_evidence(domain: &str, missing_probes: &[&str]) {
    eprintln!();
    eprintln!("[ERROR] {}", "Insufficient evidence".bright_red().bold());
    eprintln!();
    eprintln!("Cannot answer questions about: {}", domain.red());
    eprintln!();
    eprintln!("[MISSING PROBES]");
    for probe in missing_probes {
        eprintln!("  * {}", probe.red());
    }
    eprintln!();
    eprintln!(
        "[AVAILABLE PROBES] cpu.info, mem.info, disk.lsblk, hardware.gpu, drivers.gpu, hardware.ram"
    );
    eprintln!();
}

// ============================================================================
// v0.10.0: Evidence-Based Answer Display
// ============================================================================

/// Display an evidence-based answer from the v0.10.0 answer engine
pub fn display_final_answer(answer: &FinalAnswer) {
    // Header
    println!();
    println!(
        "{}",
        "==================================================".cyan()
    );
    println!("  {}  Anna Answer", "[>]".bright_cyan());
    println!(
        "{}",
        "==================================================".cyan()
    );
    println!();

    // Question
    println!("{}  {}", "Q:".bold().bright_white(), answer.question);
    println!();

    // Answer or refusal
    if answer.is_refusal {
        println!(
            "{}  {}",
            "A:".bold().bright_red(),
            answer.answer.bright_red()
        );
    } else {
        println!("{}  {}", "A:".bold().bright_green(), answer.answer);
    }

    // Evidence citations
    if !answer.citations.is_empty() {
        println!();
        println!("{}:", "Evidence".bold().bright_white());
        for citation in &answer.citations {
            let status_icon = match citation.status {
                anna_common::EvidenceStatus::Ok => "✓".bright_green().to_string(),
                anna_common::EvidenceStatus::Error => "✗".bright_red().to_string(),
                anna_common::EvidenceStatus::NotFound => "?".yellow().to_string(),
                anna_common::EvidenceStatus::Timeout => "[T]".yellow().to_string(),
            };
            // Format: [probe_id] → summary
            let summary = citation
                .raw
                .as_ref()
                .map(|r| {
                    let line = r.lines().next().unwrap_or("");
                    if line.len() > 50 {
                        format!("{}...", &line[..47])
                    } else {
                        line.to_string()
                    }
                })
                .unwrap_or_else(|| "no output".to_string());

            println!(
                "  {}  [{}]  →  {}",
                status_icon,
                citation.probe_id.cyan(),
                summary.dimmed()
            );
        }
    }

    // Confidence score with colored level
    println!();
    let level_colored = match answer.confidence_level {
        ConfidenceLevel::Green => "GREEN".bright_green().bold().to_string(),
        ConfidenceLevel::Yellow => "YELLOW".yellow().bold().to_string(),
        ConfidenceLevel::Red => "RED".bright_red().bold().to_string(),
    };

    let overall = answer.scores.overall;
    let evidence = answer.scores.evidence;
    let reasoning = answer.scores.reasoning;
    let coverage = answer.scores.coverage;

    // v0.15.21: Show as percentage
    println!(
        "{}  [{}] {:.0}%  (evidence {:.0}%, reasoning {:.0}%, coverage {:.0}%)",
        "Confidence:".bold().bright_white(),
        level_colored,
        overall * 100.0,
        evidence * 100.0,
        reasoning * 100.0,
        coverage * 100.0
    );

    // Loop iterations info if multiple rounds
    if answer.loop_iterations > 1 {
        println!(
            "{}  {} iterations",
            "Audit loops:".dimmed(),
            answer.loop_iterations
        );
    }

    // Footer
    println!();
    println!(
        "{}",
        "==================================================".cyan()
    );
    // Show model used
    if let Some(ref model) = answer.model_used {
        println!("{}  {}", "Model:".dimmed(), model.bright_blue());
    }
    if !answer.is_refusal {
        println!(
            "{}",
            "Evidence-based  *  LLM-A/LLM-B audited  *  No hallucinations".dimmed()
        );
    } else {
        println!(
            "{}",
            "Refused due to insufficient evidence or low confidence".dimmed()
        );
    }
    println!();
}

/// Display an evidence-based answer with elapsed time (v0.72.0)
/// Clear answer output block with Anna header, Evidence, Reliability sections
pub fn display_final_answer_with_time(answer: &FinalAnswer, elapsed: Duration) {
    // v0.72.0: Clear Anna header block
    println!();
    println!(
        "{}",
        "==============================================================================".cyan()
    );
    println!(
        "  {}",
        "Anna".bright_white().bold()
    );
    println!(
        "{}",
        "==============================================================================".cyan()
    );

    // Final answer text - plain text, the main content
    if answer.is_refusal {
        println!("{}", answer.answer.bright_red());
    } else {
        println!("{}", answer.answer);
    }

    // v0.72.0: Evidence section - show probes and commands with status
    if !answer.citations.is_empty() {
        println!();
        println!("{}", "Evidence");
        println!("{}", THIN_SEPARATOR);
        for citation in &answer.citations {
            let status_str = match citation.status {
                anna_common::EvidenceStatus::Ok => "ok".bright_green().to_string(),
                anna_common::EvidenceStatus::Error => "error".bright_red().to_string(),
                anna_common::EvidenceStatus::NotFound => "not_found".yellow().to_string(),
                anna_common::EvidenceStatus::Timeout => "timeout".yellow().to_string(),
            };
            // v0.72.0: Show probe_id, command, and status
            println!(
                "  [{}]   command: {}   status: {}",
                citation.probe_id.cyan(),
                citation.command.dimmed(),
                status_str
            );
        }
    }

    // v0.72.0: Reliability section with percentage and color
    println!();
    println!("{}", "Reliability");
    println!("{}", THIN_SEPARATOR);

    let overall_pct = (answer.scores.overall * 100.0).round() as u32;
    let (color_str, pct_colored) = match answer.confidence_level {
        ConfidenceLevel::Green => (
            "Green".bright_green().to_string(),
            format!("{}%", overall_pct).bright_green().to_string(),
        ),
        ConfidenceLevel::Yellow => (
            "Yellow".yellow().to_string(),
            format!("{}%", overall_pct).yellow().to_string(),
        ),
        ConfidenceLevel::Red => (
            "Red".bright_red().to_string(),
            format!("{}%", overall_pct).bright_red().to_string(),
        ),
    };

    println!(
        "  Overall: {} ({})",
        pct_colored,
        color_str
    );

    // v0.72.0: Show timing info at bottom
    let elapsed_str = if elapsed.as_millis() < 1000 {
        format!("{}ms", elapsed.as_millis())
    } else {
        format!("{:.2}s", elapsed.as_secs_f64())
    };
    println!();
    println!(
        "{}",
        format!("Duration: {}  |  Iterations: {}", elapsed_str, answer.loop_iterations).dimmed()
    );

    // v0.15.21: Show clarification question if needed
    if let Some(ref clarification) = answer.clarification_needed {
        println!();
        println!("{}", "Clarification Needed");
        println!("{}", THIN_SEPARATOR);
        println!("  {}  {}", "?".yellow().bold(), clarification);
    }

    println!();
}

// ============================================================================
// v0.81.0: Structured Answer Display
// ============================================================================

/// Display a structured answer (v0.81.0)
/// Shows: headline, details[], evidence[], reliability_label
pub fn display_structured_answer(answer: &FinalAnswer, elapsed: std::time::Duration) {
    let structured = answer.to_structured_answer();
    display_structured_answer_direct(&structured, elapsed, answer.loop_iterations as u32);
}

/// Display a structured answer directly from StructuredAnswer struct
pub fn display_structured_answer_direct(
    structured: &StructuredAnswer,
    elapsed: std::time::Duration,
    iterations: u32,
) {
    // Header
    println!();
    println!(
        "{}",
        "==============================================================================".cyan()
    );
    println!("  {}", "Anna".bright_white().bold());
    println!(
        "{}",
        "==============================================================================".cyan()
    );
    println!();

    // Headline - colored by reliability
    let headline_display = match structured.reliability_label.as_str() {
        "Green" => structured.headline.bright_green().bold().to_string(),
        "Yellow" => structured.headline.yellow().bold().to_string(),
        "Red" => structured.headline.bright_red().bold().to_string(),
        _ => structured.headline.bold().to_string(),
    };
    println!("{}", headline_display);
    println!();

    // Details (bullet points)
    if !structured.details.is_empty() {
        println!("{}", "Details".bold());
        println!("{}", THIN_SEPARATOR);
        for detail in &structured.details {
            println!("  *  {}", detail);
        }
        println!();
    }

    // Evidence
    if !structured.evidence.is_empty() {
        println!("{}", "Evidence".bold());
        println!("{}", THIN_SEPARATOR);
        for ev in &structured.evidence {
            println!("  {}", ev.cyan());
        }
        println!();
    }

    // Reliability label
    println!("{}", "Reliability".bold());
    println!("{}", THIN_SEPARATOR);
    let label_colored = match structured.reliability_label.as_str() {
        "Green" => structured.reliability_label.bright_green().bold().to_string(),
        "Yellow" => structured.reliability_label.yellow().bold().to_string(),
        "Red" => structured.reliability_label.bright_red().bold().to_string(),
        _ => structured.reliability_label.bold().to_string(),
    };
    println!("  {}", label_colored);

    // Timing footer
    let elapsed_str = if elapsed.as_millis() < 1000 {
        format!("{}ms", elapsed.as_millis())
    } else {
        format!("{:.2}s", elapsed.as_secs_f64())
    };
    println!();
    println!(
        "{}",
        format!("Duration: {}  |  Iterations: {}", elapsed_str, iterations).dimmed()
    );
    println!();
}

// ============================================================================
// v0.16.3: Debug Trace Display for Development
// ============================================================================

/// Display the full LLM dialog trace for development troubleshooting
/// v0.16.4: Uses [JUNIOR model] and [SENIOR model] labels for clarity
/// Only shown when ANNA_DEBUG environment variable is set
pub fn display_debug_trace(trace: &DebugTrace) {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════════════════════════════╗"
            .bright_magenta()
    );
    println!(
        "{}",
        "║  [?]  DEBUG TRACE SUMMARY (see stderr for real-time output)                ║"
            .bright_magenta()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════════════════════════════╝"
            .bright_magenta()
    );
    println!();

    // Models used with clear labels
    println!(
        "{}  [JUNIOR {}]  │  [SENIOR {}]",
        "Models:".bold().bright_white(),
        trace.junior_model.bright_green(),
        trace.senior_model.bright_blue()
    );
    println!(
        "{}  {:.2}s  │  {} iterations",
        "Duration:".bold().bright_white(),
        trace.duration_secs,
        trace.iterations.len()
    );
    println!();

    // Each iteration
    for iter in &trace.iterations {
        println!(
            "{}",
            "████████████████████████████████████████████████████████████████████████████████"
                .bright_yellow()
        );
        println!(
            "██  ITERATION {}/6  ██",
            iter.iteration.to_string().bold().bright_yellow()
        );
        println!(
            "{}",
            "████████████████████████████████████████████████████████████████████████████████"
                .bright_yellow()
        );
        println!();

        // LLM-A (Junior) Section
        println!(
            "{}",
            format!(
                "┌─ [JUNIOR {}] ──────────────────────────────────────────────────────────────┐",
                trace.junior_model
            )
            .bright_green()
        );
        println!();

        // LLM-A Prompt (truncated)
        println!("{}:", "[>]  PROMPT".bold().bright_white());
        display_wrapped_text(&iter.llm_a_prompt, "    ", 74);
        println!();

        // LLM-A Response
        println!("{}:", "[<]  RESPONSE".bold().bright_white());
        display_wrapped_text(&iter.llm_a_response, "    ", 74);
        println!();

        // LLM-A Parsed Summary
        println!(
            "{}  intent={}, probes={:?}, has_draft={}",
            "[=]  Parsed:".bold().dimmed(),
            iter.llm_a_intent.cyan(),
            iter.llm_a_probes,
            iter.llm_a_has_draft
        );

        // Probes executed
        if !iter.probes_executed.is_empty() {
            println!(
                "{}  {:?}",
                "[P]  Probes Executed:".bold().dimmed(),
                iter.probes_executed
            );
        }
        println!();
        println!(
            "{}",
            "└──────────────────────────────────────────────────────────────────────────────┘"
                .bright_green()
        );
        println!();

        // LLM-B (Senior) Section (if present)
        if iter.llm_b_prompt.is_some() || iter.llm_b_response.is_some() {
            println!(
                "{}",
                format!("┌─ [SENIOR {}] ──────────────────────────────────────────────────────────────┐", trace.senior_model)
                    .bright_blue()
            );
            println!();

            // LLM-B Prompt
            if let Some(ref prompt) = iter.llm_b_prompt {
                println!("{}:", "[>]  PROMPT".bold().bright_white());
                display_wrapped_text(prompt, "    ", 74);
                println!();
            }

            // LLM-B Response
            if let Some(ref response) = iter.llm_b_response {
                println!("{}:", "[<]  RESPONSE".bold().bright_white());
                display_wrapped_text(response, "    ", 74);
                println!();
            }

            // LLM-B Parsed Summary with verdict
            if let Some(ref verdict) = iter.llm_b_verdict {
                let verdict_colored = match verdict.as_str() {
                    "approve" => format!("[+]  {}", verdict).bright_green().to_string(),
                    "fix_and_accept" => format!("[~]  {}", verdict).yellow().to_string(),
                    "needs_more_probes" => format!("[~]  {}", verdict).cyan().to_string(),
                    "refuse" => format!("[-]  {}", verdict).bright_red().to_string(),
                    _ => verdict.to_string(),
                };
                print!(
                    "{}  verdict={}, ",
                    "[=]  Parsed:".bold().dimmed(),
                    verdict_colored
                );
                if let Some(conf) = iter.llm_b_confidence {
                    println!("confidence={:.0}%", conf * 100.0);
                } else {
                    println!();
                }
            }
            println!();
            println!(
                "{}",
                "└──────────────────────────────────────────────────────────────────────────────┘"
                    .bright_blue()
            );
            println!();
        }
    }

    println!(
        "{}",
        "════════════════════════════════════════════════════════════════════════════════"
            .bright_magenta()
    );
    println!(
        "{}",
        "TIP: Run 'journalctl -fu annad' in another terminal to see real-time LLM dialog".dimmed()
    );
    println!(
        "{}",
        "     or set ANNA_DEBUG=1 before starting the daemon.".dimmed()
    );
    println!();
}

/// Display text wrapped to a given width with a prefix
fn display_wrapped_text(text: &str, prefix: &str, width: usize) {
    let lines: Vec<&str> = text.lines().collect();
    let max_lines = 20; // Limit displayed lines

    for (i, line) in lines.iter().enumerate() {
        if i >= max_lines {
            println!("{}... ({} more lines)", prefix, lines.len() - max_lines);
            break;
        }

        // Truncate long lines
        let display_line = if line.len() > width {
            format!("{}...", &line[..width - 3])
        } else {
            line.to_string()
        };
        println!("{}{}", prefix, display_line.dimmed());
    }
}
