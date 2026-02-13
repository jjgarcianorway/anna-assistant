//! Wait for daemon initialization to complete, showing live progress.

use anna_shared::status::DaemonState;
use std::io::Write;

/// Poll daemon status until ready, showing live init_status messages.
/// Returns immediately if already ready or daemon unreachable.
pub async fn wait_for_ready() {
    let status = match crate::rpc::get_status().await {
        Ok(s) => s,
        Err(_) => return,
    };

    if status.state == DaemonState::Ready {
        return;
    }

    println!();
    let mut last_msg = String::new();

    loop {
        match crate::rpc::get_status().await {
            Ok(s) => {
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
            Err(_) => break,
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
