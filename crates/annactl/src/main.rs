//! Anna CLI (annactl) v7.2.0 - Snow Leopard
//!
//! Commands:
//! - annactl           show help
//! - annactl status    health and runtime of Anna
//! - annactl sw        software overview (packages, commands, services)
//! - annactl sw NAME   software profile
//! - annactl hw        hardware overview (CPU, memory, GPU, storage, network)
//! - annactl hw NAME   hardware profile
//!
//! Deprecated (still works internally):
//! - annactl kdb       alias to sw
//! - annactl kdb NAME  alias to sw NAME

mod commands;

use anyhow::Result;
use owo_colors::OwoColorize;
use std::env;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use anna_common::grounded::categoriser::is_valid_category;

const THIN_SEP: &str = "------------------------------------------------------------";

/// Check if name is a software category (using rule-based categoriser + services)
fn is_sw_category(name: &str) -> bool {
    // Special case: services is always a category
    if name.eq_ignore_ascii_case("services") || name.eq_ignore_ascii_case("service") {
        return true;
    }
    is_valid_category(name)
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

    // v7.2.0: sw/hw surface
    match args.as_slice() {
        // annactl (no args) - show help
        [] => run_help(),

        // annactl status
        [cmd] if cmd.eq_ignore_ascii_case("status") => commands::status::run().await,

        // annactl sw (software overview)
        [cmd] if cmd.eq_ignore_ascii_case("sw") => commands::sw::run().await,

        // annactl sw <name-or-category>
        [cmd, name] if cmd.eq_ignore_ascii_case("sw") => {
            if is_sw_category(name) {
                commands::sw_detail::run_category(name).await
            } else {
                commands::sw_detail::run_object(name).await
            }
        }

        // annactl hw (hardware overview)
        [cmd] if cmd.eq_ignore_ascii_case("hw") => commands::hw::run().await,

        // annactl hw <name>
        [cmd, name] if cmd.eq_ignore_ascii_case("hw") => {
            commands::hw_detail::run(name).await
        }

        // Deprecated: annactl kdb (alias to sw)
        [cmd] if cmd.eq_ignore_ascii_case("kdb") => commands::sw::run().await,

        // Deprecated: annactl kdb <name-or-category> (alias to sw)
        [cmd, name] if cmd.eq_ignore_ascii_case("kdb") => {
            if is_sw_category(name) {
                commands::sw_detail::run_category(name).await
            } else {
                commands::sw_detail::run_object(name).await
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
    println!("  annactl sw        software overview");
    println!("  annactl sw NAME   software profile (package, command, category)");
    println!("  annactl hw        hardware overview");
    println!("  annactl hw NAME   hardware profile (cpu, memory, gpu, storage, ...)");
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
