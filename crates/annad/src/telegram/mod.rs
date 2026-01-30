//! Telegram bot channel for Anna.
//!
//! Provides mobile access to Anna via Telegram.
//! Uses the same LLM-first architecture as the CLI.

mod handlers;
pub mod notifier;

use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::state::SharedState;

/// Configuration for the Telegram bot.
#[derive(Clone)]
pub struct TelegramConfig {
    /// Bot token from @BotFather
    pub token: String,
    /// Allowed user IDs (empty = allow all, dangerous!)
    pub allowed_users: Vec<u64>,
}

impl TelegramConfig {
    /// Load config from environment.
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("ANNA_TELEGRAM_TOKEN").ok()?;

        // Parse allowed users from comma-separated list
        let allowed_users = std::env::var("ANNA_TELEGRAM_USERS")
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|id| id.trim().parse::<u64>().ok())
                    .collect()
            })
            .unwrap_or_default();

        Some(Self { token, allowed_users })
    }
}

/// A conversation turn (question + answer).
#[derive(Clone)]
pub struct ConversationTurn {
    pub question: String,
    pub answer: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Conversation context for a chat.
#[derive(Clone, Default)]
pub struct ConversationContext {
    /// Recent turns (max 5).
    pub turns: Vec<ConversationTurn>,
}

impl ConversationContext {
    /// Add a new turn, keeping only the last 5.
    pub fn add_turn(&mut self, question: String, answer: String) {
        self.turns.push(ConversationTurn {
            question,
            answer,
            timestamp: chrono::Utc::now(),
        });
        // Keep only last 5 turns
        if self.turns.len() > 5 {
            self.turns.remove(0);
        }
        // Remove turns older than 30 minutes
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(30);
        self.turns.retain(|t| t.timestamp > cutoff);
    }

    /// Format context for LLM prompt.
    pub fn format_for_llm(&self) -> Option<String> {
        if self.turns.is_empty() {
            return None;
        }
        let mut lines = vec!["Recent conversation:".to_string()];
        for turn in &self.turns {
            lines.push(format!("User: {}", turn.question));
            // Truncate long answers
            let answer = if turn.answer.len() > 200 {
                format!("{}...", &turn.answer[..200])
            } else {
                turn.answer.clone()
            };
            lines.push(format!("Anna: {}", answer));
        }
        Some(lines.join("\n"))
    }
}

/// Shared state for the Telegram bot.
pub struct TelegramState {
    pub anna_state: SharedState,
    pub config: TelegramConfig,
    /// Pending confirmations: chat_id -> (plan_id, question)
    pub pending_confirms: Arc<RwLock<std::collections::HashMap<i64, (String, String)>>>,
    /// Conversation context per chat.
    pub contexts: Arc<RwLock<std::collections::HashMap<i64, ConversationContext>>>,
}

/// Start the Telegram bot if configured.
pub async fn start_telegram_bot(anna_state: SharedState) -> Result<()> {
    let config = match TelegramConfig::from_env() {
        Some(c) => c,
        None => {
            info!("Telegram bot not configured (set ANNA_TELEGRAM_TOKEN)");
            return Ok(());
        }
    };

    if config.allowed_users.is_empty() {
        warn!("ANNA_TELEGRAM_USERS not set - bot will respond to ANYONE!");
        warn!("Set ANNA_TELEGRAM_USERS=your_telegram_id for security");
    } else {
        info!("Telegram bot restricted to {} users", config.allowed_users.len());
    }

    let tg_state = Arc::new(TelegramState {
        anna_state,
        config: config.clone(),
        pending_confirms: Arc::new(RwLock::new(std::collections::HashMap::new())),
        contexts: Arc::new(RwLock::new(std::collections::HashMap::new())),
    });

    info!("Starting Telegram bot...");

    let bot = Bot::new(&config.token);

    // Verify bot connection
    match bot.get_me().await {
        Ok(me) => {
            info!("Telegram bot connected: @{}", me.username.as_deref().unwrap_or("unknown"));
        }
        Err(e) => {
            error!("Failed to connect Telegram bot: {}", e);
            return Err(anyhow::anyhow!("Telegram bot connection failed: {}", e));
        }
    }

    // Initialize push notification channel
    let notify_chat_id = std::env::var("ANNA_TELEGRAM_NOTIFY_CHAT")
        .ok()
        .and_then(|s| s.parse::<i64>().ok())
        .or_else(|| config.allowed_users.first().map(|&id| id as i64));

    if let Some(chat_id) = notify_chat_id {
        let (tx, rx) = tokio::sync::mpsc::channel::<notifier::NotifyMessage>(100);
        notifier::init_notifier(tx, Some(chat_id), Some(config.token.clone()));

        let token_clone = config.token.clone();
        tokio::spawn(async move {
            notifier::notification_sender(rx, token_clone, chat_id).await;
        });
        info!("Push notifications enabled for chat {}", chat_id);
    }

    // Start message handler
    let handler = Update::filter_message()
        .endpoint(move |bot: Bot, msg: Message| {
            let state = tg_state.clone();
            async move {
                if let Err(e) = handlers::handle_message(bot, msg, state).await {
                    error!("Telegram handler error: {}", e);
                }
                Ok::<(), std::convert::Infallible>(())
            }
        });

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
