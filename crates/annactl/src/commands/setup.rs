//! Setup command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn setup(desktop: Option<&str>, preview: bool) -> Result<()> {
    use anna_common::beautiful::{header, section};

    // If no desktop specified, show available options
    if desktop.is_none() {
        println!("{}", header("Desktop Environment Setup"));
        println!();
        println!("  Anna can install and configure complete desktop environments for you.");
        println!("  Everything ready to use: window manager, terminal, launcher, bar, theme, etc.");
        println!();
        println!("{}", section("Available Setups"));
        println!();
        println!("  \x1b[1mhyprland\x1b[0m - Modern Wayland compositor with animations");
        println!("    • Automatic hardware detection (NVIDIA/AMD/Intel)");
        println!("    • Optimized for your system's capabilities");
        println!("    • Complete working environment out of the box");
        println!();
        println!("{}", section("Usage"));
        println!();
        println!("  \x1b[38;5;159mannactl setup hyprland\x1b[0m           # Install Hyprland environment");
        println!("  \x1b[38;5;159mannactl setup hyprland --preview\x1b[0m # Show what would be installed");
        println!();
        return Ok(());
    }

    let desktop_name = desktop.unwrap();

    // For now, only Hyprland is supported
    if desktop_name != "hyprland" {
        println!("{}", header("Desktop Setup"));
        println!();
        println!("  \x1b[91m✗\x1b[0m Desktop '{}' not available", desktop_name);
        println!();
        println!("  Currently supported:");
        println!("    • hyprland");
        println!();
        println!("  Run \x1b[38;5;159mannactl setup\x1b[0m to see all options");
        println!();
        return Ok(());
    }

    println!("{}", header(&format!("Setup: {}", desktop_name)));
    println!();

    // Show installed bundles first if any exist
    if let Ok(history) = anna_common::BundleHistory::load() {
        if !history.entries.is_empty() {
            println!("{}", section("📦 Installed Bundles"));
            println!();

            let installed: Vec<_> = history.entries.iter()
                .filter(|e| e.status == anna_common::BundleStatus::Completed && e.rollback_available)
                .collect();

            if !installed.is_empty() {
                for (i, entry) in installed.iter().enumerate() {
                    let rollback_id = i + 1;
                    println!("  \x1b[1;96m[#{}]\x1b[0m \x1b[1m{}\x1b[0m", rollback_id, entry.bundle_name);
                    println!("      Installed: {} by {}",
                        entry.installed_at.format("%Y-%m-%d %H:%M:%S"),
                        entry.installed_by
                    );
                    println!("      Items: {} package(s)", entry.installed_items.len());
                    println!();
                }
                println!("  \x1b[90mTo rollback:\x1b[0m annactl rollback <bundle-name> \x1b[90mor\x1b[0m annactl rollback #<number>");
                println!();
            }
        }
    }


    // Connect to daemon to get advice
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Daemon not running"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    // Get all advice
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let desktop_env = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .ok();
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| "bash".to_string())
        .split('/')
        .last()
        .unwrap_or("bash")
        .to_string();

    let display_server = if std::env::var("WAYLAND_DISPLAY").is_ok() {
        Some("wayland".to_string())
    } else if std::env::var("DISPLAY").is_ok() {
        Some("x11".to_string())
    } else {
        None
    };

    let result = client
        .call(Method::GetAdviceWithContext {
            username,
            desktop_env,
            shell,
            display_server,
        })
        .await?;

    if let ResponseData::Advice(advice_list) = result {
        // Group by bundle
        let mut bundles: std::collections::HashMap<String, Vec<anna_common::Advice>> = std::collections::HashMap::new();

        for advice in advice_list {
            if let Some(ref bundle_name) = advice.bundle {
                bundles.entry(bundle_name.clone()).or_insert_with(Vec::new).push(advice);
            }
        }

        if bundles.is_empty() {
            println!("  \x1b[93m⚠\x1b[0m  No workflow bundles available for your system right now.");
            println!("     Install some base tools (Docker, Python, Rust) to see bundle suggestions!");
            println!();
            return Ok(());
        }

        println!("{}", section("📦 Available Bundles"));
        println!();

        for (bundle_name, items) in bundles.iter() {
            println!("  \x1b[1m{}\x1b[0m", bundle_name);
            println!("  \x1b[90m{} recommendation(s)\x1b[0m", items.len());
            println!();

            // Show what's included
            for (i, item) in items.iter().enumerate() {
                let marker = if !item.depends_on.is_empty() {
                    "  ├─"
                } else {
                    "  •"
                };
                println!("    {} \x1b[38;5;159m{}\x1b[0m", marker, item.title);

                if i == items.len() - 1 {
                    println!();
                }
            }

            // Show install command
            println!("    \x1b[96m❯\x1b[0m  annactl apply --bundle \"{}\"", bundle_name);
            println!();
        }

        println!("{}", section("💡 Tips"));
        println!();
        println!("  • Bundles install tools in dependency order");
        println!("  • Use \x1b[38;5;159m--dry-run\x1b[0m to see what will be installed");
        println!("  • Dependencies are automatically installed first");
        println!();

    } else {
        println!("{}", beautiful::status(Level::Error, "Unexpected response from daemon"));
    }

    Ok(())
}
