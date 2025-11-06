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

/// Filter advice to remove items satisfied by previously applied advice
async fn filter_satisfied_advice(advice: Vec<Advice>, audit_logger: &AuditLogger, all_advice: &[Advice]) -> Vec<Advice> {
    // Get all applied advice IDs from audit log
    let applied_ids = match audit_logger.get_applied_advice_ids().await {
        Ok(ids) => ids,
        Err(e) => {
            warn!("Failed to get applied advice IDs: {}", e);
            return advice; // Return unfiltered if we can't check
        }
    };

    // Build a set of satisfied advice IDs by checking what each applied advice satisfies
    let mut satisfied_ids = std::collections::HashSet::new();
    for applied_id in &applied_ids {
        // Find the applied advice in the full advice list
        if let Some(applied_advice) = all_advice.iter().find(|a| &a.id == applied_id) {
            // Add all advice IDs that this applied advice satisfies
            for satisfied_id in &applied_advice.satisfies {
                satisfied_ids.insert(satisfied_id.clone());
            }
        }
    }

    // Filter out satisfied advice
    let original_count = advice.len();
    let filtered: Vec<Advice> = advice
        .into_iter()
        .filter(|adv| !satisfied_ids.contains(&adv.id))
        .collect();

    if filtered.len() < original_count {
        info!("Filtered out {} satisfied advice items", original_count - filtered.len());
    }

    filtered
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

    // Set socket permissions to 0660 (owner and group only)
    // SECURITY: Only root and users in the socket's group can connect
    // TODO: Set socket group ownership to 'annactl' group in installation
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))?;
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

/// Handle a streaming ApplyAction request
async fn handle_streaming_apply(
    request_id: u64,
    advice_id: &str,
    dry_run: bool,
    state: &DaemonState,
    writer: &mut tokio::net::unix::OwnedWriteHalf,
) -> Result<()> {
    use anna_common::ipc::{Response, ResponseData, StreamChunkType};
    use crate::executor::ExecutionChunk;

    info!("Handling streaming apply for advice: {}", advice_id);

    // Find the advice
    let advice_list = state.advice.read().await;
    let advice = match advice_list.iter().find(|a| a.id == advice_id).cloned() {
        Some(adv) => adv,
        None => {
            let response = Response {
                id: request_id,
                result: Err(format!("Advice not found: {}", advice_id)),
            };
            let json = serde_json::to_string(&response)? + "\n";
            writer.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    };
    drop(advice_list);

    if dry_run {
        // For dry run, just send a status message
        let response = Response {
            id: request_id,
            result: Ok(ResponseData::StreamChunk {
                chunk_type: StreamChunkType::Status,
                data: format!("[DRY RUN] Would execute: {:?}", advice.command),
            }),
        };
        let json = serde_json::to_string(&response)? + "\n";
        writer.write_all(json.as_bytes()).await?;

        let end_response = Response {
            id: request_id,
            result: Ok(ResponseData::StreamEnd {
                success: true,
                message: "Dry run completed".to_string(),
            }),
        };
        let json = serde_json::to_string(&end_response)? + "\n";
        writer.write_all(json.as_bytes()).await?;
        return Ok(());
    }

    let command = match &advice.command {
        Some(cmd) => cmd.clone(),
        None => {
            let response = Response {
                id: request_id,
                result: Err("No command specified".to_string()),
            };
            let json = serde_json::to_string(&response)? + "\n";
            writer.write_all(json.as_bytes()).await?;
            return Ok(());
        }
    };

    // Execute with streaming via channel
    let (mut rx, exec_handle) = executor::execute_command_streaming_channel(&command).await?;

    let mut full_output = String::new();

    // Receive and send chunks in real-time
    while let Some(chunk) = rx.recv().await {
        let chunk_type = match &chunk {
            ExecutionChunk::Stdout(line) => {
                full_output.push_str(line);
                full_output.push('\n');
                StreamChunkType::Stdout
            }
            ExecutionChunk::Stderr(line) => {
                full_output.push_str(line);
                full_output.push('\n');
                StreamChunkType::Stderr
            }
            ExecutionChunk::Status(_) => StreamChunkType::Status,
        };

        let data = match chunk {
            ExecutionChunk::Stdout(line) | ExecutionChunk::Stderr(line) | ExecutionChunk::Status(line) => line,
        };

        let response = Response {
            id: request_id,
            result: Ok(ResponseData::StreamChunk {
                chunk_type,
                data,
            }),
        };

        // Send the chunk immediately
        let json = serde_json::to_string(&response)? + "\n";
        writer.write_all(json.as_bytes()).await?;
    }

    // Wait for execution to complete and get result
    let success = exec_handle.await.context("Execution task panicked")??;

    // Create audit entry
    let action = anna_common::Action {
        id: format!("action_{}", uuid::Uuid::new_v4()),
        advice_id: advice.id.clone(),
        command,
        executed_at: chrono::Utc::now(),
        success,
        output: full_output.clone(),
        error: if success { None } else { Some("Command failed".to_string()) },
    };

    let audit_entry = executor::create_audit_entry(&action, "annactl");
    if let Err(e) = state.audit_logger.log(&audit_entry).await {
        warn!("Failed to log audit entry: {}", e);
    }

    // Record to application history
    let history_entry = anna_common::HistoryEntry {
        advice_id: advice.id.clone(),
        advice_title: advice.title.clone(),
        category: advice.category.clone(),
        applied_at: chrono::Utc::now(),
        applied_by: "annactl".to_string(),
        command_run: advice.command.clone(),
        success,
        output: full_output.clone(),
        health_score_before: None,
        health_score_after: None,
    };

    let mut history = anna_common::ApplicationHistory::load().unwrap_or_default();
    history.record(history_entry);
    if let Err(e) = history.save() {
        warn!("Failed to save application history: {}", e);
    }

    // Send final response
    let end_response = Response {
        id: request_id,
        result: Ok(ResponseData::StreamEnd {
            success,
            message: if success {
                "Action completed successfully".to_string()
            } else {
                "Action failed".to_string()
            },
        }),
    };
    let json = serde_json::to_string(&end_response)? + "\n";
    writer.write_all(json.as_bytes()).await?;

    Ok(())
}

/// Handle a single client connection
async fn handle_connection(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    // SECURITY: Verify peer credentials before processing any requests
    #[cfg(unix)]
    {
        use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};

        let cred = getsockopt(&stream, PeerCredentials)
            .context("Failed to get peer credentials")?;

        // Log the connection attempt for audit purposes
        info!("Connection from UID {} GID {} PID {}", cred.uid(), cred.gid(), cred.pid());

        // TODO: Implement group-based access control
        // For now, we log credentials but allow all connections
        // Future: Check if user is in 'annactl' group and reject if not
    }

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

        // Check if this is a streaming ApplyAction request
        if let Method::ApplyAction { advice_id, dry_run, stream: true } = &request.method {
            // Handle streaming separately
            if let Err(e) = handle_streaming_apply(request.id, advice_id, *dry_run, &state, &mut writer).await {
                error!("Streaming apply failed: {}", e);
                let error_response = Response {
                    id: request.id,
                    result: Err(format!("Streaming failed: {}", e)),
                };
                let response_json = serde_json::to_string(&error_response)? + "\n";
                writer.write_all(response_json.as_bytes()).await?;
            }
        } else {
            // Normal non-streaming request
            let response = handle_request(request.id, request.method, &state).await;

            let response_json = serde_json::to_string(&response)? + "\n";
            writer
                .write_all(response_json.as_bytes())
                .await
                .context("Failed to write response")?;
        }
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
            let all_advice = state.advice.read().await.clone();
            let filtered_advice = filter_satisfied_advice(all_advice.clone(), &state.audit_logger, &all_advice).await;
            Ok(ResponseData::Advice(filtered_advice))
        }

        Method::GetAdviceWithContext { username, desktop_env, shell, display_server } => {
            // Filter advice based on user context
            let all_advice = state.advice.read().await.clone();
            let total_count = all_advice.len();

            let context_filtered: Vec<_> = all_advice.clone().into_iter()
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

            // Also filter out satisfied advice
            let filtered_advice = filter_satisfied_advice(context_filtered, &state.audit_logger, &all_advice).await;

            info!("Filtered advice for user {}: {} -> {} items",
                  username, total_count, filtered_advice.len());

            Ok(ResponseData::Advice(filtered_advice))
        }

        Method::ApplyAction { advice_id, dry_run, stream } => {
            if stream {
                info!("Streaming requested for action {} (not yet implemented)", advice_id);
                // TODO: Implement streaming using execute_command_streaming
            }

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
