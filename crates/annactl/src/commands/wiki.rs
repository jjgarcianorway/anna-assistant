//! Wiki command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn wiki_cache(force: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Arch Wiki Cache"));
    println!();

    // Check if we need to refresh
    let needs_refresh = anna_common::WikiCache::load()
        .map(|cache| cache.needs_refresh())
        .unwrap_or(true);

    if !force && !needs_refresh {
        println!("{}", beautiful::status(Level::Info, "Wiki cache is up to date"));
        println!();
        println!("  Use \x1b[38;5;159m--force\x1b[0m to refresh anyway.");
        println!();
        return Ok(());
    }

    println!("{}", section("📥 Updating Cache"));
    println!();

    if force {
        println!("{}", beautiful::status(Level::Info, "Forcing cache refresh..."));
    } else {
        println!("{}", beautiful::status(Level::Info, "Cache is stale, refreshing..."));
    }
    println!();

    // Connect to daemon to request wiki cache update
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Daemon not running"));
            println!("  The wiki cache update requires the daemon to be running.");
            println!("  Please start the daemon: \x1b[38;5;159msudo systemctl start annad\x1b[0m");
            println!();
            return Ok(());
        }
    };

    // Request cache update via RPC
    println!("{}", beautiful::status(Level::Info, "Updating Arch Wiki cache..."));
    println!("  This will download \x1b[1m88+ essential Arch Wiki pages\x1b[0m for offline access.");
    println!("  Progress details are logged by the daemon.");
    println!();
    println!("  \x1b[2mTip: Watch progress in another terminal:\x1b[0m");
    println!("       \x1b[38;5;159mjournalctl -u annad -f\x1b[0m");
    println!();

    use std::io::{self, Write};
    print!("  \x1b[38;5;226m⏳\x1b[0m Downloading wiki pages");
    io::stdout().flush()?;

    // Spawn progress animation
    let animation_handle = tokio::spawn(async {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let mut i = 0;
        loop {
            print!("\r  \x1b[38;5;226m{}\x1b[0m Downloading wiki pages... ", frames[i % frames.len()]);
            io::stdout().flush().ok();
            tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
            i += 1;
        }
    });

    let response = client
        .call(Method::UpdateWikiCache)
        .await
        .context("Failed to update wiki cache")?;

    // Stop animation
    animation_handle.abort();
    print!("\r\x1b[K"); // Clear line
    io::stdout().flush()?;

    match response {
        ResponseData::ActionResult { success, message } => {
            if success {
                println!("{}", beautiful::status(Level::Success, "Wiki cache updated successfully!"));
                println!("  {}", message);
                println!();
                println!("  \x1b[2m88 essential pages cached for offline use\x1b[0m");
            } else {
                println!("{}", beautiful::status(Level::Error, "Failed to update cache"));
                println!("  {}", message);
            }
        }
        _ => {
            println!("{}", beautiful::status(Level::Warning, "Unexpected response from daemon"));
        }
    }
    println!();

    Ok(())
}

/// Display system health score
/// NOTE: Removed for v1.0 - merged into status command
#[allow(dead_code)]
