//! Telegram message handlers.
//!
//! Routes messages to the same Ralph loop used by the CLI.

use std::sync::Arc;
use teloxide::prelude::*;
use tracing::{info, warn};

use super::TelegramState;
use crate::ralph;

/// Handle incoming Telegram message.
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    state: Arc<TelegramState>,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;
    let user_id = msg.from().map(|u| u.id.0).unwrap_or(0);
    let username = msg.from()
        .and_then(|u| u.username.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Security: check allowed users
    if !state.config.allowed_users.is_empty()
        && !state.config.allowed_users.contains(&user_id)
    {
        warn!("Unauthorized Telegram user: {} ({})", username, user_id);
        bot.send_message(chat_id, "Not authorized. Contact the system administrator.")
            .await?;
        return Ok(());
    }

    // Get message text
    let text = match msg.text() {
        Some(t) => t.trim(),
        None => {
            bot.send_message(chat_id, "Send me a text message with your question.")
                .await?;
            return Ok(());
        }
    };

    if text.is_empty() {
        return Ok(());
    }

    info!("Telegram message from @{}: {}", username, text);

    // Check for confirmation responses
    if text.eq_ignore_ascii_case("yes") || text.eq_ignore_ascii_case("confirm") {
        return handle_confirmation(bot, chat_id, true, state).await;
    }
    if text.eq_ignore_ascii_case("no") || text.eq_ignore_ascii_case("cancel") {
        return handle_confirmation(bot, chat_id, false, state).await;
    }

    // Send typing indicator
    bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await?;

    // Get model from Anna state
    let model = {
        let anna = state.anna_state.read().await;
        anna.model.clone().unwrap_or_else(|| "qwen2.5:14b".to_string())
    };

    // Route to Ralph loop (same as CLI)
    let result = ralph::ralph_loop(&model, text).await;

    match result {
        Ok(ask_result) => {
            // Check if confirmation is needed
            if ask_result.needs_clarification {
                // Store pending confirmation
                let mut confirms = state.pending_confirms.write().await;
                confirms.insert(
                    chat_id.0,
                    (ask_result.answer.clone(), text.to_string()),
                );

                // Ask for confirmation (plain text, no markdown)
                let confirm_msg = format!(
                    "{}\n\nReply 'yes' to confirm or 'no' to cancel.",
                    &ask_result.answer
                );
                bot.send_message(chat_id, confirm_msg).await?;
            } else {
                // Send answer directly
                send_long_message(&bot, chat_id, &ask_result.answer).await?;
            }
        }
        Err(e) => {
            let error_msg = format!("Error: {}", e);
            bot.send_message(chat_id, &error_msg).await?;
        }
    }

    Ok(())
}

/// Handle confirmation response (yes/no).
async fn handle_confirmation(
    bot: Bot,
    chat_id: ChatId,
    confirmed: bool,
    state: Arc<TelegramState>,
) -> anyhow::Result<()> {
    let pending = {
        let mut confirms = state.pending_confirms.write().await;
        confirms.remove(&chat_id.0)
    };

    match pending {
        Some((plan_description, original_question)) => {
            if confirmed {
                bot.send_message(chat_id, "Executing...").await?;
                bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await?;

                // Execute the confirmed action
                // For now, re-run with confirmation flag
                // TODO: Store and execute actual ActionPlan
                let model = {
                    let anna = state.anna_state.read().await;
                    anna.model.clone().unwrap_or_else(|| "qwen2.5:14b".to_string())
                };

                // Re-run with "yes, do it" context
                let confirmed_question = format!("{} (confirmed, execute it)", original_question);
                let result = ralph::ralph_loop(&model, &confirmed_question).await;

                match result {
                    Ok(ask_result) => {
                        send_long_message(&bot, chat_id, &ask_result.answer).await?;
                    }
                    Err(e) => {
                        bot.send_message(chat_id, format!("Execution failed: {}", e)).await?;
                    }
                }
            } else {
                bot.send_message(chat_id, "Cancelled.").await?;
            }
        }
        None => {
            bot.send_message(chat_id, "Nothing pending to confirm.").await?;
        }
    }

    Ok(())
}

/// Send a long message, splitting if needed (Telegram limit: 4096 chars).
async fn send_long_message(bot: &Bot, chat_id: ChatId, text: &str) -> anyhow::Result<()> {
    const MAX_LEN: usize = 4000; // Leave margin for safety

    if text.len() <= MAX_LEN {
        bot.send_message(chat_id, text).await?;
    } else {
        // Split on newlines when possible
        let mut remaining = text;
        while !remaining.is_empty() {
            let chunk = if remaining.len() <= MAX_LEN {
                remaining
            } else {
                // Try to split at a newline
                let split_at = remaining[..MAX_LEN]
                    .rfind('\n')
                    .unwrap_or(MAX_LEN);
                &remaining[..split_at]
            };

            bot.send_message(chat_id, chunk).await?;
            remaining = remaining[chunk.len()..].trim_start();

            // Brief delay between chunks
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    Ok(())
}
