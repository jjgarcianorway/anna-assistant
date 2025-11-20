//! Dismissed command

use anna_common::ipc::{Method, ResponseData};
use anna_common::{beautiful, header, section, kv, Level};
use anyhow::Result;

use crate::rpc_client::RpcClient;

pub async fn dismissed(undismiss_num: Option<usize>) -> Result<()> {
    use anna_common::beautiful::{header, section};

    println!("{}", header("Dismissed Recommendations"));
    println!();

    // Load feedback log
    let mut log = match anna_common::UserFeedbackLog::load() {
        Ok(l) => l,
        Err(_) => {
            println!("{}", beautiful::status(Level::Info, "No dismissed recommendations"));
            println!();
            return Ok(());
        }
    };

    // Get dismissed events
    let dismissed: Vec<_> = log.events.iter()
        .filter(|e| matches!(e.event_type, anna_common::FeedbackType::Dismissed))
        .collect();

    if dismissed.is_empty() {
        println!("{}", beautiful::status(Level::Info, "No dismissed recommendations"));
        println!();
        println!("Use 'annactl dismiss <number>' to dismiss recommendations from the advise list");
        return Ok(());
    }

    // If undismiss requested
    if let Some(num) = undismiss_num {
        if num < 1 || num > dismissed.len() {
            println!("{}", beautiful::status(Level::Error,
                &format!("Number {} out of range (1-{})", num, dismissed.len())));
            return Ok(());
        }

        let event = dismissed[num - 1];
        let advice_id_to_remove = event.advice_id.clone();

        // Remove from log
        log.events.retain(|e| e.advice_id != advice_id_to_remove);
        log.save()?;

        println!("{}", beautiful::status(Level::Success,
            &format!("Un-dismissed: {}", advice_id_to_remove)));
        println!();
        println!("Run 'annactl advise' to see this recommendation again");
        println!();
        return Ok(());
    }

    // Show all dismissed items
    println!("{}", beautiful::status(Level::Info,
        &format!("{} dismissed recommendation{}", dismissed.len(), if dismissed.len() == 1 { "" } else { "s" })));
    println!();

    // Group by category
    let mut by_category: std::collections::HashMap<String, Vec<&anna_common::FeedbackEvent>> =
        std::collections::HashMap::new();

    for event in &dismissed {
        by_category.entry(event.advice_category.clone())
            .or_insert_with(Vec::new)
            .push(event);
    }

    // Display by category
    for (category, events) in by_category.iter() {
        let category_emoji = anna_common::get_category_emoji(category);
        println!("{}", section(&format!("{} {}", category_emoji, category)));

        for event in events {
            let time_ago = {
                let duration = chrono::Utc::now().signed_duration_since(event.timestamp);
                if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} minutes ago", duration.num_minutes())
                } else {
                    "just now".to_string()
                }
            };

            println!("  • {} \x1b[90m({})\x1b[0m",
                event.advice_id.replace('-', " "),
                time_ago);
        }
        println!();
    }

    // Show commands
    println!("{}", section("Commands"));
    println!();
    println!("  annactl dismissed --undismiss <number>  # Restore a dismissed item");
    println!("  annactl advise                          # View current recommendations");
    println!();

    Ok(())
}

/// View application history and analytics
