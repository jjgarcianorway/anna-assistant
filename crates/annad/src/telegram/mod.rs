//! Telegram bot channel for Anna.
//!
//! Provides mobile access to Anna via Telegram.
//! Uses the same LLM-first architecture as the CLI.

mod handlers;
pub mod notifier;
mod chart_sender;

use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use anna_shared::config::{AnnaConfig, TelegramUserRole};
use crate::state::SharedState;

/// Configuration for the Telegram bot.
#[derive(Clone)]
pub struct TelegramConfig {
    /// Bot token from @BotFather (from ANNA_TELEGRAM_TOKEN env var)
    pub token: String,
    /// Role map: user_id → role. Loaded from [telegram.users] in config.toml.
    /// Users not in this map are silently ghosted.
    pub user_roles: std::collections::HashMap<u64, TelegramUserRole>,
}

impl TelegramConfig {
    /// Load config from environment + config file.
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("ANNA_TELEGRAM_TOKEN").ok()?;
        let user_roles = AnnaConfig::load()
            .map(|c| c.telegram.users)
            .unwrap_or_default();
        Some(Self { token, user_roles })
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

    if config.user_roles.is_empty() {
        warn!("No users configured in [telegram.users] - bot will ghost all messages");
        warn!("Add user IDs to /etc/anna/config.toml [telegram.users] to grant access");
    } else {
        info!("Telegram bot configured with {} user(s)", config.user_roles.len());
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
        .or_else(|| {
            // Default to first admin user if ANNA_TELEGRAM_NOTIFY_CHAT not set
            config.user_roles.iter()
                .find(|(_, role)| **role == TelegramUserRole::Admin)
                .map(|(&id, _)| id as i64)
        });

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
