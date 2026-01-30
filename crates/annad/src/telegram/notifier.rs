//! Telegram push notifications for proactive alerts.

use std::path::Path;
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// Message type for notifications
pub enum NotifyMessage {
    Text(String),
    Document { path: String, caption: String },
}

/// Channel for sending notifications to Telegram.
static NOTIFY_TX: OnceLock<mpsc::Sender<NotifyMessage>> = OnceLock::new();

/// Chat ID to send notifications to (loaded from env).
static NOTIFY_CHAT_ID: OnceLock<Option<i64>> = OnceLock::new();

/// Bot token for document uploads
static BOT_TOKEN: OnceLock<String> = OnceLock::new();

/// Initialize the notification channel.
/// Called during Telegram bot startup.
pub fn init_notifier(tx: mpsc::Sender<NotifyMessage>, chat_id: Option<i64>, token: Option<String>) {
    let _ = NOTIFY_TX.set(tx);
    let _ = NOTIFY_CHAT_ID.set(chat_id);
    if let Some(t) = token {
        let _ = BOT_TOKEN.set(t);
    }
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

    if let Err(e) = tx.try_send(NotifyMessage::Text(message.to_string())) {
        warn!("Failed to queue Telegram notification: {}", e);
    }
}

/// Send a PDF report to Telegram.
pub fn send_pdf_report(path: &Path) {
    let Some(tx) = NOTIFY_TX.get() else {
        return;
    };

    let msg = NotifyMessage::Document {
        path: path.to_string_lossy().to_string(),
        caption: "Your daily system report is ready.".to_string(),
    };

    if let Err(e) = tx.try_send(msg) {
        warn!("Failed to queue PDF report: {}", e);
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
pub async fn notification_sender(mut rx: mpsc::Receiver<NotifyMessage>, bot_token: String, chat_id: i64) {
    use teloxide::prelude::*;
    use teloxide::types::InputFile;

    let bot = Bot::new(&bot_token);

    while let Some(msg) = rx.recv().await {
        match msg {
            NotifyMessage::Text(text) => {
                match bot.send_message(ChatId(chat_id), &text).await {
                    Ok(_) => info!("Sent Telegram notification"),
                    Err(e) => error!("Failed to send Telegram notification: {}", e),
                }
            }
            NotifyMessage::Document { path, caption } => {
                let file_path = std::path::Path::new(&path);
                if file_path.exists() {
                    let input_file = InputFile::file(file_path);
                    match bot.send_document(ChatId(chat_id), input_file)
                        .caption(&caption)
                        .await
                    {
                        Ok(_) => {
                            info!("Sent PDF report via Telegram");
                            // Clean up the temp file
                            let _ = std::fs::remove_file(file_path);
                        }
                        Err(e) => error!("Failed to send PDF report: {}", e),
                    }
                } else {
                    error!("PDF file not found: {}", path);
                }
            }
        }
    }
}
