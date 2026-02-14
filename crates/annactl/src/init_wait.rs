//! Wait for daemon initialization to complete, showing live progress.

use anna_shared::status::DaemonState;
use std::io::Write;

const MORNING_REPORT_PATH: &str = "/var/lib/anna/morning_report.txt";
const MORNING_REPORT_SHOWN: &str = "/var/lib/anna/morning_report.shown";

/// Display the morning report if it's new (generated today and not yet shown).
pub fn show_morning_report_if_new() {
    // Check if report exists
    let report_meta = std::fs::metadata(MORNING_REPORT_PATH);
    let shown_meta = std::fs::metadata(MORNING_REPORT_SHOWN);

    let report_mtime = match report_meta {
        Ok(m) => match m.modified() {
            Ok(t) => t,
            Err(_) => return,
        },
        Err(_) => return, // No report yet
    };

    // If already shown and shown file is newer than report, skip
    if let Ok(sm) = shown_meta {
        if let Ok(shown_mtime) = sm.modified() {
            if shown_mtime >= report_mtime {
                return;
            }
        }
    }

    // Report is newer than last shown marker — display it
    if let Ok(content) = std::fs::read_to_string(MORNING_REPORT_PATH) {
        println!();
        println!("{}", content);
        println!();
        // Mark as shown
        let _ = std::fs::write(MORNING_REPORT_SHOWN, "");
    }
}

/// Poll daemon status until ready.
/// - Never breaks on transient connection failures (retries indefinitely)
/// - Shows live init_status from daemon when reachable
/// - Shows "waiting for daemon" when socket temporarily unavailable
/// - Only exits when daemon is Ready or after 10 minutes with no response
pub async fn wait_for_ready() {
    // Fast path: if daemon is already ready, return immediately
    if let Ok(s) = crate::rpc::get_status().await {
        if s.state == DaemonState::Ready {
            return;
        }
    }

    // Daemon not ready yet — show live progress until it is
    println!();
    let mut last_msg = String::new();
    let mut consecutive_errors: u32 = 0;
    let mut total_secs: u32 = 0;
    const MAX_WAIT_SECS: u32 = 600; // 10 minutes — enough for large model download

    loop {
        if total_secs >= MAX_WAIT_SECS {
            // Timed out — clear line, proceed (downstream will show error)
            print!("\r\x1b[K");
            let _ = std::io::stdout().flush();
            break;
        }

        match crate::rpc::get_status().await {
            Ok(s) => {
                consecutive_errors = 0;

                if s.state == DaemonState::Ready {
                    print!("\r\x1b[K");
                    let _ = std::io::stdout().flush();
                    break;
                }

                let msg = if let Some(ref err) = s.last_error {
                    format!("Setup error (retrying): {}", err)
                } else if !s.init_status.is_empty() {
                    s.init_status.clone()
                } else {
                    "Setting up...".to_string()
                };

                if msg != last_msg {
                    print!("\r\x1b[K\x1b[2m{}\x1b[0m", msg);
                    let _ = std::io::stdout().flush();
                    last_msg = msg;
                }
            }
            Err(_) => {
                // Transient failure — daemon may be busy with pacman/ollama
                // Keep retrying: do NOT break here
                consecutive_errors += 1;
                let msg = if consecutive_errors < 5 {
                    "Waiting for Anna daemon...".to_string()
                } else {
                    format!("Waiting for Anna daemon... ({}s)", total_secs)
                };
                if msg != last_msg {
                    print!("\r\x1b[K\x1b[2m{}\x1b[0m", msg);
                    let _ = std::io::stdout().flush();
                    last_msg = msg;
                }
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        total_secs += 2;
    }
}
