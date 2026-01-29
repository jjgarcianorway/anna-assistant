//! Telegram bot channel for Anna.
//!
//! Provides mobile access to Anna via Telegram.
//! Uses the same LLM-first architecture as the CLI.

mod handlers;

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

/// Shared state for the Telegram bot.
pub struct TelegramState {
    pub anna_state: SharedState,
    pub config: TelegramConfig,
    /// Pending confirmations: chat_id -> (plan_id, question)
    pub pending_confirms: Arc<RwLock<std::collections::HashMap<i64, (String, String)>>>,
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
