//! Anna CLI (annactl) v7.0.0 - Minimal Surface
//!
//! Only 4 commands:
//! - annactl           show help
//! - annactl status    health and runtime of Anna
//! - annactl kdb       overview of knowledge database
//! - annactl kdb NAME  profile for a package, command or category

mod commands;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Known category names for `kdb <category>` queries
const CATEGORY_NAMES: [&str; 7] = [
    "editors", "terminals", "shells", "browsers",
    "compositors", "tools", "services"
];

fn is_category(name: &str) -> bool {
    let lower = name.to_lowercase();
    CATEGORY_NAMES.contains(&lower.as_str())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "annactl=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer().without_time().with_target(false))
        .init();

    let args: Vec<String> = env::args().skip(1).collect();

    // v7.0.0: Only 4 commands
    match args.as_slice() {
        // annactl (no args) - show help
        [] => run_help(),

        // annactl status
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // annactl kdb
        [cmd] if cmd.eq_ignore_ascii_case("kdb") => commands::kdb::run().await,

        // annactl kdb <name-or-category>
        [cmd, name] if cmd.eq_ignore_ascii_case("kdb") => {
            if is_category(name) {
                commands::kdb_detail::run_category(name).await
            } else {
                commands::kdb_detail::run_object(name).await
            }
        }

        // Unknown
        _ => run_unknown(&args),
    }
}

/// Top-level help - concise list of commands
fn run_help() -> Result<()> {
    println!();
    println!("{}", "  Anna CLI".bold());
    println!("{}", THIN_SEP);
    println!("  annactl           show this help");
    println!("  annactl status    health and runtime of Anna");
    println!("  annactl kdb       overview of knowledge database");
    println!("  annactl kdb NAME  profile for a package, command or category");
    println!("{}", THIN_SEP);
    println!();
    Ok(())
}

fn run_unknown(args: &[String]) -> Result<()> {
    eprintln!();
    eprintln!("  {} '{}' is not a recognized command.", "error:".red(), args.join(" "));
    eprintln!();
    eprintln!("  Run 'annactl' for available commands.");
    eprintln!();
    std::process::exit(1);
}
