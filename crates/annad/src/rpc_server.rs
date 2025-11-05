//! RPC Server - Unix socket server for daemon-client communication

use crate::audit::AuditLogger;
use crate::executor;
use anna_common::ipc::{ConfigData, Method, Request, Response, ResponseData, StatusData};
use anna_common::{Advice, SystemFacts};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

const SOCKET_PATH: &str = "/run/anna/anna.sock";

/// Daemon state shared across connections
pub struct DaemonState {
    pub version: String,
    pub start_time: std::time::Instant,
    pub facts: RwLock<SystemFacts>,
    pub advice: RwLock<Vec<Advice>>,
    pub config: RwLock<ConfigData>,
    pub audit_logger: AuditLogger,
}

impl DaemonState {
    pub async fn new(version: String, facts: SystemFacts, advice: Vec<Advice>) -> Result<Self> {
        let audit_logger = AuditLogger::new().await?;

        Ok(Self {
            version,
            start_time: std::time::Instant::now(),
            facts: RwLock::new(facts),
            advice: RwLock::new(advice),
            config: RwLock::new(ConfigData::default()),
            audit_logger,
        })
    }
}

/// Start the RPC server
pub async fn start_server(state: Arc<DaemonState>) -> Result<()> {
    // Ensure socket directory exists
    let socket_dir = Path::new(SOCKET_PATH).parent().unwrap();
    tokio::fs::create_dir_all(socket_dir)
        .await
        .context("Failed to create socket directory")?;

    // Remove old socket if it exists
    let _ = tokio::fs::remove_file(SOCKET_PATH).await;

    // Bind to Unix socket
    let listener = UnixListener::bind(SOCKET_PATH)
        .context("Failed to bind Unix socket")?;

    info!("RPC server listening on {}", SOCKET_PATH);

    // Set socket permissions (readable/writable by all users for now)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o666))?;
    }

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, state).await {
                        error!("Connection handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read from socket")?;

        if bytes_read == 0 {
            // Connection closed
            break;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid request JSON: {}", e);
                continue;
            }
        };

        let response = handle_request(request.id, request.method, &state).await;

        let response_json = serde_json::to_string(&response)? + "\n";
        writer
            .write_all(response_json.as_bytes())
            .await
            .context("Failed to write response")?;
    }

    Ok(())
}

/// Handle a single request
async fn handle_request(id: u64, method: Method, state: &DaemonState) -> Response {
    let result = match method {
        Method::Ping => Ok(ResponseData::Ok),

        Method::Status => {
            let advice = state.advice.read().await;
            let status = StatusData {
                version: state.version.clone(),
                uptime_seconds: state.start_time.elapsed().as_secs(),
                last_telemetry_check: "Just now".to_string(), // TODO: track actual last check
                pending_recommendations: advice.len(),
            };
            Ok(ResponseData::Status(status))
        }

        Method::GetFacts => {
            let facts = state.facts.read().await.clone();
            Ok(ResponseData::Facts(facts))
        }

        Method::GetAdvice => {
            let advice = state.advice.read().await.clone();
            Ok(ResponseData::Advice(advice))
        }

        Method::GetAdviceWithContext { username, desktop_env, shell, display_server } => {
            // Filter advice based on user context
            let all_advice = state.advice.read().await.clone();
            let total_count = all_advice.len();

            let filtered_advice: Vec<_> = all_advice.into_iter()
                .filter(|advice| {
                    // System-wide advice (security, updates, etc.) - show to everyone
                    if matches!(advice.category.as_str(), "security" | "updates" | "performance" | "cleanup") {
                        return true;
                    }

                    // Desktop-specific advice - filter by user's DE
                    if advice.category == "desktop" {
                        if let Some(ref de) = desktop_env {
                            let de_lower = de.to_lowercase();
                            // Check if advice ID matches user's DE
                            return advice.id.contains(&de_lower) ||
                                   advice.title.to_lowercase().contains(&de_lower);
                        }
                        return false;
                    }

                    // Gaming advice - show to all (they chose to install Steam)
                    if advice.category == "gaming" {
                        return true;
                    }

                    // Shell-specific advice
                    if advice.id.contains("shell") || advice.id.contains("zsh") || advice.id.contains("bash") {
                        return advice.title.to_lowercase().contains(&shell.to_lowercase());
                    }

                    // Display server specific (Wayland/X11)
                    if advice.id.contains("wayland") || advice.id.contains("xwayland") {
                        return display_server.is_some();
                    }

                    // Everything else - show to all
                    true
                })
                .collect();

            info!("Filtered advice for user {}: {} -> {} items",
                  username, total_count, filtered_advice.len());

            Ok(ResponseData::Advice(filtered_advice))
        }

        Method::ApplyAction { advice_id, dry_run } => {
            // Find the advice
            let advice_list = state.advice.read().await;
            let advice = advice_list.iter().find(|a| a.id == advice_id).cloned();

            match advice {
                Some(adv) => {
                    // Execute the action
                    match executor::execute_action(&adv, dry_run).await {
                        Ok(action) => {
                            // Log to audit
                            let audit_entry = executor::create_audit_entry(&action, "annactl");
                            if let Err(e) = state.audit_logger.log(&audit_entry).await {
                                warn!("Failed to log audit entry: {}", e);
                            }

                            // Record to application history (only for actual execution, not dry-run)
                            if !dry_run {
                                info!("Recording application history for: {}", adv.id);
                                let history_entry = anna_common::HistoryEntry {
                                    advice_id: adv.id.clone(),
                                    advice_title: adv.title.clone(),
                                    category: adv.category.clone(),
                                    applied_at: chrono::Utc::now(),
                                    applied_by: "annactl".to_string(),
                                    command_run: adv.command.clone(),
                                    success: action.success,
                                    output: action.output.clone(),
                                    health_score_before: None, // TODO: Could capture from current facts
                                    health_score_after: None,
                                };

                                let mut history = anna_common::ApplicationHistory::load().unwrap_or_default();
                                history.record(history_entry);
                                info!("History entries before save: {}", history.entries.len());
                                match history.save() {
                                    Ok(()) => info!("Application history saved successfully to: {:?}",
                                        anna_common::ApplicationHistory::history_path()),
                                    Err(e) => warn!("Failed to save application history: {}", e),
                                }
                            }

                            let message = if action.success {
                                if dry_run {
                                    action.output
                                } else {
                                    format!("Action completed successfully:\n{}", action.output)
                                }
                            } else {
                                format!("Action failed: {}", action.error.unwrap_or_default())
                            };

                            Ok(ResponseData::ActionResult {
                                success: action.success,
                                message,
                            })
                        }
                        Err(e) => Err(format!("Failed to execute action: {}", e)),
                    }
                }
                None => Err(format!("Advice not found: {}", advice_id)),
            }
        }

        Method::Refresh => {
            info!("Manual refresh requested");
            // Re-collect facts and regenerate advice
            match crate::telemetry::collect_facts().await {
                Ok(facts) => {
                    let mut advice = crate::recommender::generate_advice(&facts);
                    advice.extend(crate::intelligent_recommender::generate_intelligent_advice(&facts));

                    // Update state
                    *state.facts.write().await = facts;
                    *state.advice.write().await = advice.clone();

                    info!("Advice manually refreshed: {} recommendations", advice.len());
                    Ok(ResponseData::ActionResult {
                        success: true,
                        message: format!("System scanned! Found {} recommendations", advice.len()),
                    })
                }
                Err(e) => {
                    tracing::error!("Failed to refresh advice: {}", e);
                    Err(format!("Failed to refresh: {}", e))
                }
            }
        }

        Method::UpdateWikiCache => {
            info!("Wiki cache update requested");
            match crate::wiki_cache::update_common_pages().await {
                Ok(()) => {
                    info!("Wiki cache updated successfully");
                    Ok(ResponseData::ActionResult {
                        success: true,
                        message: "Wiki cache updated successfully! Common Arch Wiki pages are now available offline.".to_string(),
                    })
                }
                Err(e) => {
                    error!("Failed to update wiki cache: {}", e);
                    Err(format!("Failed to update wiki cache: {}", e))
                }
            }
        }

        Method::GetConfig => {
            let config = state.config.read().await.clone();
            Ok(ResponseData::Config(config))
        }

        Method::SetConfig { key, value } => {
            let mut config = state.config.write().await;
            match key.as_str() {
                "autonomy_tier" => {
                    if let Ok(tier) = value.parse::<u8>() {
                        if tier <= 3 {
                            config.autonomy_tier = tier;
                            Ok(ResponseData::Ok)
                        } else {
                            Err("Invalid autonomy tier (must be 0-3)".to_string())
                        }
                    } else {
                        Err("Invalid value for autonomy_tier".to_string())
                    }
                }
                "auto_update_check" => {
                    if let Ok(enabled) = value.parse::<bool>() {
                        config.auto_update_check = enabled;
                        Ok(ResponseData::Ok)
                    } else {
                        Err("Invalid value for auto_update_check".to_string())
                    }
                }
                "wiki_cache_path" => {
                    config.wiki_cache_path = value;
                    Ok(ResponseData::Ok)
                }
                _ => Err(format!("Unknown config key: {}", key)),
            }
        }
    };

    Response { id, result }
}
