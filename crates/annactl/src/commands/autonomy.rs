//! Autonomy command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn autonomy(limit: usize) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Autonomous Actions Log"));
    println!();

    // Load autonomy log
    let log = match anna_common::AutonomyLog::load() {
        Ok(l) => l,
        Err(_) => {
            println!("{}", beautiful::status(Level::Info, "No autonomous actions have been performed yet"));
            println!();
            println!("  Autonomous maintenance runs based on your autonomy tier setting.");
            println!("  Use \x1b[38;5;159mannactl config --set autonomy_tier=1\x1b[0m to enable safe auto-apply.");
            println!();
            return Ok(());
        }
    };

    let recent = log.recent(limit);

    if recent.is_empty() {
        println!("{}", beautiful::status(Level::Info, "No autonomous actions recorded"));
        println!();
        return Ok(());
    }

    println!("{}", section(&format!("🤖 Recent {} Actions", recent.len())));
    println!();

    for (i, action) in recent.iter().enumerate() {
        // Success/failure indicator
        let status_icon = if action.success {
            "\x1b[92m✓\x1b[0m"
        } else {
            "\x1b[91m✗\x1b[0m"
        };

        // Action type badge
        let type_badge = match action.action_type.as_str() {
            "clean_orphans" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_cache" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_journal" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "clean_tmp" => "\x1b[46m\x1b[97m\x1b[1m CLEANUP \x1b[0m",
            "remove_old_kernels" => "\x1b[43m\x1b[30m\x1b[1m MAINT \x1b[0m",
            "update_mirrorlist" => "\x1b[45m\x1b[97m\x1b[1m UPDATE \x1b[0m",
            _ => "\x1b[100m\x1b[97m\x1b[1m OTHER \x1b[0m",
        };

        println!("  \x1b[90m[{}]\x1b[0m  {} {}", i + 1, status_icon, type_badge);
        println!();
        println!("      \x1b[1m{}\x1b[0m", action.description);
        println!("      \x1b[90mExecuted: {}\x1b[0m", action.executed_at.format("%Y-%m-%d %H:%M:%S"));
        println!();
        println!("      \x1b[96mCommand:\x1b[0m");
        println!("      \x1b[90m❯\x1b[0m \x1b[97m{}\x1b[0m", action.command_run);

        if !action.output.is_empty() {
            let trimmed_output = action.output.trim();
            if !trimmed_output.is_empty() {
                println!();
                println!("      \x1b[96mOutput:\x1b[0m");
                // Show first 3 lines of output
                for (idx, line) in trimmed_output.lines().take(3).enumerate() {
                    if idx < 3 {
                        println!("      \x1b[90m{}\x1b[0m", line);
                    }
                }
                if trimmed_output.lines().count() > 3 {
                    println!("      \x1b[90m... ({} more lines)\x1b[0m", trimmed_output.lines().count() - 3);
                }
            }
        }

        if action.can_undo {
            if let Some(ref undo) = action.undo_command {
                println!();
                println!("      \x1b[93m⟲ Can undo:\x1b[0m \x1b[90m{}\x1b[0m", undo);
            }
        }

        if i < recent.len() - 1 {
            println!();
            println!("  \x1b[90m{}\x1b[0m", "─".repeat(78));
            println!();
        }
    }

    println!();
    println!("{}", section("ℹ️  Information"));
    println!();
    println!("  Total actions logged: {}", log.actions.len());
    println!("  Showing most recent: {}", recent.len());
    println!();
    println!("  Use \x1b[38;5;159mannactl autonomy --limit=<n>\x1b[0m to change the number of actions shown.");
    println!();

    Ok(())
}

/// Update Arch Wiki cache
/// NOTE: Removed for v1.0 - wiki cache now maintained automatically by daemon
#[allow(dead_code)]
