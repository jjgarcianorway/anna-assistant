use anyhow::Result;

use crate::paths::AnnaPaths;
use crate::rpc::RpcClient;
use crate::ui::{self, Style, UiCfg};

pub fn run(style: &Style, _cfg: &UiCfg) -> Result<()> {
    println!("{}", ui::head(style, "Anna status"));

    let paths = AnnaPaths::detect();
    let uid = nix::unistd::Uid::effective().as_raw();

    // Check if socket exists
    if !paths.socket_path.exists() {
        println!(
            "{}",
            ui::err(style, "Anna daemon socket not found. Is annad running?")
        );
        println!("Socket path: {}", paths.socket_path.display());
        println!("\nTry:");
        println!(
            "{}",
            ui::bullet(style, "Check service: systemctl status annad")
        );
        println!(
            "{}",
            ui::bullet(style, "Start service: sudo systemctl start annad")
        );
        println!(
            "{}",
            ui::bullet(style, "Check permissions: annactl doctor perms")
        );
        return Ok(());
    }

    // Try RPC
    let client = RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::Status(anna_rpc::StatusRequest { uid });

    match client.call(request) {
        Ok(anna_rpc::Response::Status(status)) => {
            println!("{}", ui::kv(style, "Install mode", &status.mode));

            // Show ANNA_MODE if set
            if let Ok(anna_mode) = std::env::var("ANNA_MODE") {
                println!(
                    "{}",
                    ui::kv(style, "ANNA_MODE", &format!("{} (env override)", anna_mode))
                );
            }

            println!("{}", ui::kv(style, "Socket path", &status.socket_path));
            println!("{}", ui::kv(style, "User data dir", &status.user_data_dir));
            println!(
                "{}",
                ui::kv(style, "System config", &status.system_config_dir)
            );
            println!("{}", ui::kv(style, "Service state", &status.service_state));

            if let Some(ts) = status.last_quickscan_ts {
                println!("{}", ui::kv(style, "Last quickscan", &ts));
            } else {
                println!("{}", ui::kv(style, "Last quickscan", "never"));
            }

            println!(
                "{}",
                ui::kv(style, "Advice count", &format!("{}", status.advice_count))
            );
        }
        Ok(anna_rpc::Response::Error(err)) => {
            println!("{}", ui::err(style, &format!("Error: {}", err.message)));
        }
        Ok(_) => {
            println!("{}", ui::err(style, "Unexpected response type"));
        }
        Err(e) => {
            println!(
                "{}",
                ui::err(style, &format!("Failed to get status: {}", e))
            );
            println!(
                "{}",
                ui::note(style, "Verify annad is running and socket is accessible")
            );
        }
    }

    Ok(())
}
