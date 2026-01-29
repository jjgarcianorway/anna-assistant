//! Telegram push notifications for proactive alerts.

use std::sync::OnceLock;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Channel for sending notifications to Telegram.
static NOTIFY_TX: OnceLock<mpsc::Sender<String>> = OnceLock::new();

/// Chat ID to send notifications to (loaded from env).
static NOTIFY_CHAT_ID: OnceLock<Option<i64>> = OnceLock::new();

/// Initialize the notification channel.
/// Called during Telegram bot startup.
pub fn init_notifier(tx: mpsc::Sender<String>, chat_id: Option<i64>) {
    let _ = NOTIFY_TX.set(tx);
    let _ = NOTIFY_CHAT_ID.set(chat_id);
    if chat_id.is_some() {
        info!("Telegram notifier initialized for chat {}", chat_id.unwrap());
    }
}

/// Send a push notification to Telegram.
/// Non-blocking: queues message for async delivery.
pub fn push_notification(message: &str) {
    let Some(tx) = NOTIFY_TX.get() else {
        return; // Notifier not initialized (Telegram not configured)
    };

    if let Err(e) = tx.try_send(message.to_string()) {
        warn!("Failed to queue Telegram notification: {}", e);
    }
}

/// Send an alert to Telegram (for critical/warning issues).
pub fn push_alert(severity: &str, summary: &str, suggested_fix: Option<&str>) {
    let message = if let Some(fix) = suggested_fix {
        format!("{}: {}\nSuggested fix: {}", severity, summary, fix)
    } else {
        format!("{}: {}", severity, summary)
    };
    push_notification(&message);
}

/// Background task that sends queued notifications.
pub async fn notification_sender(mut rx: mpsc::Receiver<String>, bot_token: String, chat_id: i64) {
    use teloxide::prelude::*;

    let bot = Bot::new(&bot_token);

    while let Some(message) = rx.recv().await {
        match bot.send_message(ChatId(chat_id), &message).await {
            Ok(_) => info!("Sent Telegram notification"),
            Err(e) => error!("Failed to send Telegram notification: {}", e),
        }
    }
}
