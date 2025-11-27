//! Output formatting - clean, ASCII-only terminal output v0.10.0
//!
//! v0.6.0: Sysadmin style - no emojis, ASCII only, professional
//! v0.10.0: Evidence-based answers with citations and confidence scores
#![allow(dead_code)]

use anna_common::{AnnaResponse, ConfidenceLevel, FinalAnswer, THIN_SEPARATOR};
use owo_colors::OwoColorize;

/// Display a response to the user
pub fn display_response(response: &AnnaResponse) {
    // Confidence color and threshold
    let conf_pct = (response.confidence * 100.0) as u8;
    let conf_str = format!("{:.2}", response.confidence);

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
    println!(
        "{}  Reliability: {} ({})",
        reliability_indicator,
        conf_colored,
        if conf_pct >= 90 {
            "green"
        } else if conf_pct >= 70 {
            "yellow"
        } else {
            "red"
        }
    );
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
    println!("  {}  Anna Answer", "📋".bright_cyan());
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
                anna_common::EvidenceStatus::Timeout => "⧖".yellow().to_string(),
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

    println!(
        "{}  [{}] {:.2} (evidence {:.2}, reasoning {:.2}, coverage {:.2})",
        "Confidence:".bold().bright_white(),
        level_colored,
        overall,
        evidence,
        reasoning,
        coverage
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
