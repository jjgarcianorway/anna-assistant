use crate::paths::AnnaPaths;
use crate::rpc::RpcClient;
use crate::ui::{self, Style, UiCfg};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::time::SystemTime;

pub struct QuickscanArgs {
    pub raw: bool,
    pub auto: bool,
}

pub fn run(args: QuickscanArgs, ui_cfg: &UiCfg, style: &Style) -> Result<()> {
    let paths = AnnaPaths::detect();

    if args.auto {
        // Check last run timestamp
        if let Some(last_run) = get_last_run_time(&paths) {
            let now = SystemTime::now();
            let elapsed = now
                .duration_since(last_run)
                .unwrap_or(std::time::Duration::from_secs(0));

            // Skip if <24 hours
            if elapsed < std::time::Duration::from_secs(24 * 3600) {
                let hours_left = (24 * 3600 - elapsed.as_secs()) / 3600;
                if args.raw {
                    println!(
                        "{{\"status\": \"skipped\", \"reason\": \"recent\", \"hours_until_next\": {}}}",
                        hours_left
                    );
                } else {
                    ui::banner(style, "Quick Health Check (auto)");
                    println!(
                        "{}",
                        ui::info(
                            style,
                            &format!(
                                "Already fresh (last run {} hours ago, next in {} hours)",
                                elapsed.as_secs() / 3600,
                                hours_left
                            )
                        )
                    );
                }
                return Ok(());
            }
        }
    }

    // Get current UID
    let uid = nix::unistd::Uid::current().as_raw();

    // Connect to RPC and run quickscan
    let client = RpcClient::new(&paths.socket_path);
    let request = anna_rpc::Request::Quickscan(anna_rpc::QuickscanRequest { uid });

    let response = client.call(request).context("RPC call failed")?;

    let quickscan_response = match response {
        anna_rpc::Response::Quickscan(r) => r,
        anna_rpc::Response::Error(e) => {
            return Err(anyhow!("Quickscan failed: {}", e.message));
        }
        _ => {
            return Err(anyhow!("Unexpected response type"));
        }
    };

    // Update last run timestamp
    if args.auto {
        update_last_run_time(&paths)?;
    }

    if args.raw {
        println!("{}", serde_json::to_string_pretty(&quickscan_response)?);
        return Ok(());
    }

    if args.auto {
        ui::banner(style, "Quick Health Check (auto)");
    }

    render_summary(&quickscan_response, ui_cfg, style);
    Ok(())
}

fn render_summary(response: &anna_rpc::QuickscanResponse, ui_cfg: &UiCfg, style: &Style) {
    println!(
        "{}",
        ui::head(style, &format!("⚙ Quickscan (mode: {})", response.mode))
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Started",
            &ui::fmt_local(&response.started_at, ui_cfg)
        )
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Finished",
            &ui::fmt_local(&response.finished_at, ui_cfg)
        )
    );
    println!(
        "{}",
        ui::kv(
            style,
            "Summary",
            &format!(
                "ok {}  warn {}  action {}",
                response.summary.ok, response.summary.warn, response.summary.action
            )
        )
    );
    println!("{}", ui::kv(style, "Report", &response.report_path));
    println!(
        "{}",
        ui::kv(
            style,
            "Seeded advice",
            &response.advice_count_seeded.to_string()
        )
    );
}

fn get_last_run_time(paths: &AnnaPaths) -> Option<SystemTime> {
    let timestamp_file = paths.reports_dir.join(".last_quickscan");
    if let Ok(metadata) = fs::metadata(&timestamp_file) {
        metadata.modified().ok()
    } else {
        None
    }
}

fn update_last_run_time(paths: &AnnaPaths) -> Result<()> {
    let timestamp_file = paths.reports_dir.join(".last_quickscan");
    if let Some(parent) = timestamp_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create directory {}", parent.display()))?;
    }
    fs::write(&timestamp_file, "")
        .with_context(|| format!("write timestamp {}", timestamp_file.display()))?;
    Ok(())
}
