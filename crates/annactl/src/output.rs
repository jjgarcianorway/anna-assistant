//! Output formatting - clean, beautiful terminal output v0.3.0
#![allow(dead_code)]

use anna_common::AnnaResponse;
use owo_colors::OwoColorize;

/// Display a response to the user
pub fn display_response(response: &AnnaResponse) {
    // Confidence color and threshold for v0.3.0
    let conf_pct = (response.confidence * 100.0) as u8;
    let conf_str = format!("{}%", conf_pct);

    // v0.3.0: < 70% is red (insufficient evidence threshold)
    let conf_colored = if conf_pct >= 90 {
        conf_str.bright_green().to_string()
    } else if conf_pct >= 70 {
        conf_str.yellow().to_string()
    } else {
        conf_str.bright_red().to_string()
    };

    // Header with reliability indicator
    println!();
    let reliability_icon = if conf_pct >= 90 {
        "✓".bright_green().to_string()
    } else if conf_pct >= 70 {
        "⚠".yellow().to_string()
    } else {
        "✗".bright_red().to_string()
    };

    println!(
        "{}  {}  Reliability: {}",
        "🤖".bright_magenta(),
        reliability_icon,
        conf_colored
    );
    println!();

    // Answer - check if it's an insufficient evidence response
    if response.confidence < 0.70 {
        // Low reliability - format as warning
        println!("{}", response.answer.bright_red());
    } else {
        println!("{}", response.answer);
    }

    // Sources
    if !response.sources.is_empty() {
        println!();
        println!("{}  Evidence:", "📋".dimmed());
        for source in &response.sources {
            println!("   {}  [source: {}]", "•".bright_blue(), source.cyan());
        }
    }

    // Warning
    if let Some(warning) = &response.warning {
        println!();
        if response.confidence < 0.70 {
            println!("{}  {}", "🚫".bright_red(), warning.bright_red());
        } else {
            println!("{}  {}", "⚠".yellow(), warning.yellow());
        }
    }

    // v0.3.0: No hallucination guarantee footer
    println!();
    if response.confidence >= 0.70 {
        println!(
            "{}",
            "─────────────────────────────────────────".dimmed()
        );
        println!(
            "{}  {}",
            "🛡️".dimmed(),
            "Evidence-based • No hallucinations • No guesses".dimmed()
        );
    }
    println!();
}

/// Display an error
pub fn display_error(message: &str) {
    eprintln!();
    eprintln!("{}  {}", "✗".bright_red(), message.red());
    eprintln!();
}

/// Display a success message
pub fn display_success(message: &str) {
    println!();
    println!("{}  {}", "✓".bright_green(), message.green());
    println!();
}

/// Display an info message
pub fn display_info(message: &str) {
    println!("{}  {}", "ℹ".bright_blue(), message);
}

/// Display a warning
pub fn display_warning(message: &str) {
    println!("{}  {}", "⚠".yellow(), message.yellow());
}

/// Display insufficient evidence (v0.3.0)
pub fn display_insufficient_evidence(domain: &str, missing_probes: &[&str]) {
    eprintln!();
    eprintln!(
        "{}  {}",
        "🚫".bright_red(),
        "Insufficient evidence".bright_red().bold()
    );
    eprintln!();
    eprintln!("{}  Cannot answer questions about: {}", "❌".red(), domain.red());
    eprintln!();
    eprintln!("{}  Missing probes:", "📋".dimmed());
    for probe in missing_probes {
        eprintln!("   {}  {}", "•".red(), probe.red());
    }
    eprintln!();
    eprintln!(
        "{}  Available probes: cpu.info, mem.info, disk.lsblk",
        "🔧".dimmed()
    );
    eprintln!();
}
