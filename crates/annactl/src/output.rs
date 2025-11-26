//! Output formatting - clean, beautiful terminal output

use anna_common::AnnaResponse;
use owo_colors::OwoColorize;

/// Display a response to the user
pub fn display_response(response: &AnnaResponse) {
    // Confidence color
    let conf_pct = (response.confidence * 100.0) as u8;
    let conf_str = format!("{}%", conf_pct);
    let conf_colored = if conf_pct >= 90 {
        conf_str.bright_green().to_string()
    } else if conf_pct >= 70 {
        conf_str.yellow().to_string()
    } else {
        conf_str.bright_red().to_string()
    };

    // Header
    println!();
    println!(
        "{}  Confidence: {}",
        "".bright_magenta(),
        conf_colored
    );
    println!();

    // Answer
    println!("{}", response.answer);

    // Sources
    if !response.sources.is_empty() {
        println!();
        println!("{}  Sources:", "".dimmed());
        for source in &response.sources {
            println!("   {}  {}", "".bright_blue(), source.dimmed());
        }
    }

    // Warning
    if let Some(warning) = &response.warning {
        println!();
        println!("{}  {}", "".yellow(), warning.yellow());
    }

    println!();
}

/// Display an error
pub fn display_error(message: &str) {
    eprintln!();
    eprintln!("{}  {}", "".bright_red(), message.red());
    eprintln!();
}

/// Display a success message
pub fn display_success(message: &str) {
    println!();
    println!("{}  {}", "".bright_green(), message.green());
    println!();
}

/// Display an info message
pub fn display_info(message: &str) {
    println!("{}  {}", "".bright_blue(), message);
}

/// Display a warning
pub fn display_warning(message: &str) {
    println!("{}  {}", "".yellow(), message.yellow());
}
