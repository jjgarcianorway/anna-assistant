//! Config command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn config(set: Option<String>) -> Result<()> {
    use anna_common::Config;

    println!("{}", header("Anna Configuration"));
    println!();

    // Load config
    let mut config = Config::load()?;

    // If setting a value
    if let Some(set_expr) = set {
        let parts: Vec<&str> = set_expr.splitn(2, '=').collect();
        if parts.len() != 2 {
            println!("{}", beautiful::status(Level::Error, "Invalid format. Use: key=value"));
            return Ok(());
        }

        let key = parts[0].trim();
        let value = parts[1].trim();

        // Handle different configuration keys
        match key {
            "autonomy_tier" => {
                let tier: u8 = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid autonomy tier. Use 0-3"))?;
                config.autonomy.tier = match tier {
                    0 => anna_common::AutonomyTier::AdviseOnly,
                    1 => anna_common::AutonomyTier::SafeAutoApply,
                    2 => anna_common::AutonomyTier::SemiAutonomous,
                    3 => anna_common::AutonomyTier::FullyAutonomous,
                    _ => anyhow::bail!("Autonomy tier must be 0-3"),
                };
                println!("{}", beautiful::status(Level::Success,
                    &format!("Set autonomy tier to {}", tier)));
            }
            "snapshots_enabled" => {
                config.snapshots.enabled = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Snapshots {}", if config.snapshots.enabled { "enabled" } else { "disabled" })));
            }
            "snapshot_method" => {
                if !["btrfs", "timeshift", "rsync", "none"].contains(&value) {
                    anyhow::bail!("Snapshot method must be: btrfs, timeshift, rsync, or none");
                }
                config.snapshots.method = value.to_string();
                println!("{}", beautiful::status(Level::Success,
                    &format!("Snapshot method set to {}", value)));
            }
            "learning_enabled" => {
                config.learning.enabled = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Learning {}", if config.learning.enabled { "enabled" } else { "disabled" })));
            }
            "desktop_notifications" => {
                config.notifications.desktop_notifications = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid boolean. Use true or false"))?;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Desktop notifications {}",
                        if config.notifications.desktop_notifications { "enabled" } else { "disabled" })));
            }
            "refresh_interval" => {
                let seconds: u64 = value.parse()
                    .map_err(|_| anyhow::anyhow!("Invalid number"))?;
                if seconds < 60 {
                    anyhow::bail!("Refresh interval must be at least 60 seconds");
                }
                config.general.refresh_interval_seconds = seconds;
                println!("{}", beautiful::status(Level::Success,
                    &format!("Refresh interval set to {} seconds", seconds)));
            }
            _ => {
                println!("{}", beautiful::status(Level::Error,
                    &format!("Unknown configuration key: {}", key)));
                println!();
                println!("{}", beautiful::status(Level::Info, "Available keys:"));
                println!("  autonomy_tier (0-3)");
                println!("  snapshots_enabled (true/false)");
                println!("  snapshot_method (btrfs/timeshift/rsync/none)");
                println!("  learning_enabled (true/false)");
                println!("  desktop_notifications (true/false)");
                println!("  refresh_interval (seconds)");
                return Ok(());
            }
        }

        // Save updated config
        config.save()?;
        println!();
        println!("{}", beautiful::status(Level::Info,
            &format!("Configuration saved to {}",
                Config::default_path()?.display())));

        return Ok(());
    }

    // Display current config
    println!("{}", section("🎛️  General"));
    println!("  {}", kv("Refresh Interval",
        &format!("{} seconds", config.general.refresh_interval_seconds)));
    println!("  {}", kv("Verbosity", &config.general.verbosity.to_string()));
    println!("  {}", kv("Emoji", if config.general.enable_emoji { "enabled" } else { "disabled" }));
    println!();

    println!("{}", section("🤖 Autonomy"));
    let tier_name = match config.autonomy.tier {
        anna_common::AutonomyTier::AdviseOnly => "0 (Advise Only)",
        anna_common::AutonomyTier::SafeAutoApply => "1 (Safe Auto-Apply)",
        anna_common::AutonomyTier::SemiAutonomous => "2 (Semi-Autonomous)",
        anna_common::AutonomyTier::FullyAutonomous => "3 (Fully Autonomous)",
    };
    println!("  {}", kv("Tier", tier_name));
    println!("  {}", kv("Confirm High Risk",
        if config.autonomy.confirm_high_risk { "yes" } else { "no" }));
    println!("  {}", kv("Snapshot Before Apply",
        if config.autonomy.snapshot_before_apply { "yes" } else { "no" }));
    println!();

    println!("{}", section("📸 Snapshots"));
    println!("  {}", kv("Enabled", if config.snapshots.enabled { "yes" } else { "no" }));
    println!("  {}", kv("Method", &config.snapshots.method));
    println!("  {}", kv("Max Snapshots", &config.snapshots.max_snapshots.to_string()));
    println!();

    println!("{}", section("🧠 Learning"));
    println!("  {}", kv("Enabled", if config.learning.enabled { "yes" } else { "no" }));
    println!("  {}", kv("Track Dismissed", if config.learning.track_dismissed { "yes" } else { "no" }));
    println!("  {}", kv("History Days", &config.learning.command_history_days.to_string()));
    println!();

    println!("{}", section("🔔 Notifications"));
    println!("  {}", kv("Desktop", if config.notifications.desktop_notifications { "enabled" } else { "disabled" }));
    println!("  {}", kv("On Critical", if config.notifications.notify_on_critical { "yes" } else { "no" }));
    println!("  {}", kv("On Auto-Apply", if config.notifications.notify_on_auto_apply { "yes" } else { "no" }));
    println!();

    println!("{}", beautiful::status(Level::Info, "Use --set key=value to change settings"));
    println!();
    println!("{}", beautiful::status(Level::Info, "Examples:"));
    println!("  annactl config --set autonomy_tier=1");
    println!("  annactl config --set snapshots_enabled=true");
    println!("  annactl config --set learning_enabled=true");

    Ok(())
}

/// New config interface supporting get/set/TUI
/// Simplified config command - no action parameter needed
/// Examples:
///   annactl config                  -> show all
///   annactl config autonomy_tier    -> get value
///   annactl config autonomy_tier 1  -> set value
/// NOTE: Removed for v1.0 - not implemented yet
#[allow(dead_code)]
pub async fn config_simple(
    key: Option<String>,
    value: Option<String>,
) -> Result<()> {
    match (key, value) {
        // No key, no value -> show all config
        (None, None) => {
            config(None).await
        }
        // Key but no value -> get that key
        (Some(k), None) => {
            // Connect to daemon to get current config
            let mut client = match RpcClient::connect().await {
                Ok(c) => c,
                Err(_) => {
                    println!("{}", beautiful::status(Level::Error, "Daemon not running"));
                    println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
                    return Ok(());
                }
            };

            let config_data = client.call(Method::GetConfig).await?;
            if let ResponseData::Config(config) = config_data {
                // Match the key and return the value
                let val = match k.as_str() {
                    "autonomy_tier" => config.autonomy_tier.to_string(),
                    "auto_update_check" => config.auto_update_check.to_string(),
                    "wiki_cache_path" => config.wiki_cache_path.clone(),
                    _ => {
                        println!("{}", beautiful::status(Level::Error, &format!("Unknown config key: {}", k)));
                        println!();
                        println!("Available keys:");
                        println!("  autonomy_tier       - Current autonomy tier (0-3)");
                        println!("  auto_update_check   - Enable automatic update checking");
                        println!("  wiki_cache_path     - Path to Arch Wiki cache directory");
                        println!();
                        println!("To see all settings: \x1b[96mannactl config\x1b[0m");
                        return Ok(());
                    }
                };
                println!("{} = {}", k, val);
            } else {
                println!("{}", beautiful::status(Level::Error, "Failed to get configuration"));
            }
            Ok(())
        }
        // Key and value -> set that key
        (Some(k), Some(v)) => {
            config(Some(format!("{}={}", k, v))).await
        }
        // Value without key makes no sense
        (None, Some(_)) => {
            println!("{}", beautiful::status(Level::Error, "Cannot set value without key"));
            println!();
            println!("Usage: annactl config <key> <value>");
            Ok(())
        }
    }
}

