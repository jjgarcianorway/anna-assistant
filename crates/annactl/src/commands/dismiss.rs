//! Dismiss command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn dismiss(id: Option<String>, num: Option<usize>) -> Result<()> {
    use anna_common::beautiful::{header};

    println!("{}", header("Dismiss Recommendation"));
    println!();

    // Connect to daemon
    let mut client = match RpcClient::connect().await {
        Ok(c) => c,
        Err(_) => {
            println!("{}", beautiful::status(Level::Error, "Daemon not running"));
            println!();
            println!("{}", beautiful::status(Level::Info, "Start with: sudo systemctl start annad"));
            return Ok(());
        }
    };

    // Get the advice ID
    let advice_id = if let Some(id) = id {
        id
    } else if let Some(num) = num {
        // Get all advice and find by number
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

        let advice_data = client.call(Method::GetAdviceWithContext {
            username,
            desktop_env,
            shell,
            display_server,
        }).await?;

        if let ResponseData::Advice(advice_list) = advice_data {
            if num < 1 || num > advice_list.len() {
                println!("{}", beautiful::status(Level::Error,
                    &format!("Number {} out of range (1-{})", num, advice_list.len())));
                return Ok(());
            }
            advice_list[num - 1].id.clone()
        } else {
            println!("{}", beautiful::status(Level::Error, "Failed to get advice list"));
            return Ok(());
        }
    } else {
        println!("{}", beautiful::status(Level::Error, "Please specify either --id or --num"));
        println!();
        println!("  Examples:");
        println!("    annactl dismiss --id orphan-packages");
        println!("    annactl dismiss --num 5");
        return Ok(());
    };

    // Record dismissal in feedback log
    let username = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());

    // Get advice details to know the category
    let advice_data = client.call(Method::GetAdvice).await?;
    if let ResponseData::Advice(advice_list) = advice_data {
        if let Some(advice) = advice_list.iter().find(|a| a.id == advice_id) {
            let mut log = anna_common::UserFeedbackLog::load().unwrap_or_default();
            log.record(anna_common::FeedbackEvent {
                advice_id: advice_id.clone(),
                advice_category: advice.category.clone(),
                event_type: anna_common::FeedbackType::Dismissed,
                timestamp: chrono::Utc::now(),
                username,
            });

            if let Err(e) = log.save() {
                println!("{}", beautiful::status(Level::Warning,
                    &format!("Failed to save feedback: {}", e)));
            } else {
                println!("{}", beautiful::status(Level::Success,
                    &format!("Dismissed: {}", advice.title)));
                println!();
                println!("  This recommendation won't be shown again.");
                println!("  Anna will learn from your preferences over time.");
            }
        } else {
            println!("{}", beautiful::status(Level::Error,
                &format!("Advice '{}' not found", advice_id)));
        }
    }

    println!();
    Ok(())
}

/// Show dismissed recommendations and optionally un-dismiss
