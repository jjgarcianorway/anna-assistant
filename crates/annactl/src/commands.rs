//! Subcommand handlers: reset, capabilities, watch, report.

use anna_shared::declaration::CapabilityDeclaration;
use std::io::Write;

use crate::display::colors::*;
use crate::display::formatting::*;
use crate::rpc;

/// Format for capability output
pub enum CapabilitiesFormat {
    Plain,
    Onboarding,
    Deterministic,
}

/// Show capability declaration
pub fn show_capabilities(format: CapabilitiesFormat) {
    let decl = CapabilityDeclaration::from_ledger();
    let output = match format {
        CapabilitiesFormat::Plain => decl.render_plain_text(),
        CapabilitiesFormat::Onboarding => decl.render_onboarding(),
        CapabilitiesFormat::Deterministic => decl.render_deterministic(),
    };
    println!("{}", output);
}

/// Show capabilities help
pub fn show_capabilities_help() {
    println!();
    println_colored("CAPABILITY DECLARATION", BOLD);
    println!();
    println!("Anna declares her capabilities before acting. This command shows");
    println!("what Anna can do, cannot do automatically, and will never do.");
    println!();
    println!("Usage:");
    println!("  annactl capabilities             Human-readable declaration");
    println!("  annactl capabilities --onboarding   Compact summary");
    println!("  annactl capabilities --deterministic   Diffable format");
    println!();
    println!("Why this matters:");
    println!("  Anna's trust is structural, not promised. This declaration is");
    println!("  derived directly from the capability ledger and cannot diverge");
    println!("  from actual behavior. What you see is what Anna can do.");
    println!();
}

/// Run real-time watch mode.
pub async fn run_watch_mode(compact: bool) {
    use tokio::time::{interval, Duration};

    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })
    .ok();

    let mut tick = interval(Duration::from_secs(2));

    while running.load(std::sync::atomic::Ordering::SeqCst) {
        anna_shared::watch::print_watch_frame(compact);
        tick.tick().await;
    }

    print!("\x1B[2J\x1B[H");
    println!("Watch mode ended.");
}

/// Handle reset command
pub async fn handle_reset(mode: anna_shared::rpc::ResetMode, skip_confirm: bool) {
    println!();
    println_colored("RESET", CYAN);
    println!();

    print!("  mode:          ");
    println_colored(&format!("{:?}", mode).to_lowercase(), YELLOW);
    print!("  will reset:    ");
    println_colored(mode.description(), DIM);
    println!();

    if !skip_confirm && mode == anna_shared::rpc::ResetMode::Everything {
        print_colored("This will delete all Anna data and cannot be undone.", YELLOW);
        println!();
        print!("  Type 'yes' to confirm: ");
        std::io::stdout().flush().ok();

        let mut response = String::new();
        if std::io::stdin().read_line(&mut response).is_err() {
            println_colored("Cancelled.", DIM);
            return;
        }
        if response.trim().to_lowercase() != "yes" {
            println_colored("Reset cancelled.", DIM);
            println!();
            return;
        }
        println!();
    }

    match rpc::reset(mode).await {
        Ok(result) => {
            println_colored("Reset complete:", GREEN);
            for item in &result.cleared {
                println!("  [OK] {}", item);
            }
            if let Some(backup) = &result.backup_path {
                println!();
                print_colored("  backup saved: ", DIM);
                println_colored(backup, CYAN);
            }
            println!();
            println_colored("Anna is ready to start fresh.", DIM);
        }
        Err(e) => {
            print_colored("Error: ", RED);
            println!("{}", e);
        }
    }
    println!();
}

/// Show reset help
pub fn show_reset_help() {
    println!();
    println_colored("RESET MODES", BOLD);
    println!();
    println!("  annactl reset              Reset everything (with confirmation)");
    println!("  annactl reset memory       Reset memory only (experiences, patterns)");
    println!("  annactl reset config       Reset config only (settings to defaults)");
    println!("  annactl reset models       Reset model preferences");
    println!("  annactl reset helpers      Reset helper tracking");
    println!("  annactl reset everything   Full factory reset");
    println!();
    println!("  annactl reset --force      Skip confirmation");
    println!();
}

/// Handle a PDF report request
pub async fn handle_pdf_report_request() {
    println!();
    print_colored("Generating system health report...", CYAN);
    println!();

    match rpc::generate_report().await {
        Ok(path) => {
            println!();
            print_colored("Report generated:", GREEN);
            println!(" {}", path.display());
            println!();
            println_colored("The PDF contains:", DIM);
            println!("  System health overview");
            println!("  7-day performance trends");
            println!("  Predictive alerts and forecasts");
            println!("  Personalized recommendations");
            println!("  Automated maintenance summary");
            println!();

            if std::path::Path::new("/etc/anna/telegram.env").exists() {
                print_colored("Sending to Telegram...", CYAN);
                println!();
                if let Err(e) = rpc::send_report_to_telegram(&path).await {
                    print_colored("Note: ", YELLOW);
                    println!("Could not send to Telegram: {}", e);
                }
            }
        }
        Err(e) => {
            print_colored("Error generating report: ", RED);
            println!("{}", e);
            println!();
            print_colored("Tip: ", YELLOW);
            println!("Make sure the daemon is running and fonts are installed");
        }
    }
    println!();
}
