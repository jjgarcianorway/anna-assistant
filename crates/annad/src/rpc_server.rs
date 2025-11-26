//! RPC Server - Unix socket server for daemon-client communication

use crate::audit::AuditLogger;
use crate::executor;
use anna_common::ipc::{ConfigData, Method, Request, Response, ResponseData, StatusData};
use anna_common::{Advice, SystemFacts};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};

const SOCKET_PATH: &str = "/run/anna/anna.sock";
const MAX_REQUEST_SIZE: usize = 64 * 1024; // 64 KB - reasonable max for JSON requests
const ANNACTL_GROUP: &str = "anna"; // Group name for authorized users

/// Check if a UID is member of a specific group
/// Returns true if user is in the group or is root (UID 0)
#[cfg(unix)]
fn is_user_in_group(uid: u32, group_name: &str) -> Result<bool> {
    use nix::unistd::{Group, Uid, User};

    // Root (UID 0) always allowed
    if uid == 0 {
        return Ok(true);
    }

    // Get user information
    let user = User::from_uid(Uid::from_raw(uid))
        .context("Failed to look up user")?
        .ok_or_else(|| anyhow::anyhow!("User with UID {} not found", uid))?;

    // Get target group
    let target_group = Group::from_name(group_name)
        .context(format!("Failed to look up group '{}'", group_name))?
        .ok_or_else(|| anyhow::anyhow!("Group '{}' not found", group_name))?;

    // Check if user's primary GID matches
    if user.gid == target_group.gid {
        return Ok(true);
    }

    // Check supplementary groups
    // Note: We need to get group members and check if username is in it
    // because nix doesn't provide direct "is user in group" check
    if target_group.mem.contains(&user.name) {
        return Ok(true);
    }

    Ok(false)
}

/// Rate limiter to prevent DoS attacks
/// Tracks requests per UID over a sliding time window
pub struct RateLimiter {
    // Map of UID -> list of request timestamps
    requests: Mutex<HashMap<u32, Vec<Instant>>>,
    max_requests_per_minute: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            max_requests_per_minute,
            window_duration: Duration::from_secs(60),
        }
    }

    /// Check if request from UID should be allowed
    /// Returns Ok(()) if allowed, Err if rate limited
    pub async fn check_rate_limit(&self, uid: u32) -> Result<()> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let window_start = now - self.window_duration;

        // Get or create entry for this UID
        let timestamps = requests.entry(uid).or_insert_with(Vec::new);

        // Remove timestamps outside the window
        timestamps.retain(|&t| t > window_start);

        // Check if limit exceeded
        if timestamps.len() >= self.max_requests_per_minute {
            return Err(anyhow::anyhow!(
                "Rate limit exceeded: {} requests in last minute (max: {})",
                timestamps.len(),
                self.max_requests_per_minute
            ));
        }

        // Add current request
        timestamps.push(now);

        Ok(())
    }

    /// Get current request count for a UID
    pub async fn get_request_count(&self, uid: u32) -> usize {
        let requests = self.requests.lock().await;
        requests.get(&uid).map(|v| v.len()).unwrap_or(0)
    }
}

/// Daemon state shared across connections
pub struct DaemonState {
    pub version: String,
    pub start_time: std::time::Instant,
    pub facts: RwLock<SystemFacts>,
    pub advice: RwLock<Vec<Advice>>,
    pub config: RwLock<ConfigData>,
    pub audit_logger: AuditLogger,
    pub action_history: crate::action_history::ActionHistory, // Beta.91: Rollback support
    pub rate_limiter: RateLimiter,
    /// Current system state (Phase 0.2c)
    /// Citation: [archwiki:system_maintenance]
    pub current_state: RwLock<crate::state::StateDetection>,
    /// Sentinel daemon (Phase 1.0)
    pub sentinel: Option<Arc<crate::sentinel::SentinelDaemon>>,
    /// Collective mind (Phase 1.3)
    pub collective: Option<Arc<crate::collective::CollectiveMind>>,
    /// Mirror protocol (Phase 1.4)
    pub mirror: Option<Arc<crate::mirror::MirrorProtocol>>,
    /// Chronos loop (Phase 1.5)
    pub chronos: Option<Arc<crate::chronos::ChronosLoop>>,
    /// Mirror audit (Phase 1.6)
    pub mirror_audit: Option<Arc<tokio::sync::RwLock<crate::mirror_audit::MirrorAudit>>>,
    /// Historian - long-term memory and trend analysis (Phase 5.7)
    pub historian: Option<Arc<tokio::sync::Mutex<anna_common::historian::Historian>>>,
    /// System Knowledge Manager - persistent memory (6.12.0)
    pub knowledge: Arc<tokio::sync::RwLock<crate::system_knowledge::SystemKnowledgeManager>>,
    /// Daemon Health - crash loop detection and Safe Mode (6.20.0)
    pub health: Arc<tokio::sync::RwLock<crate::daemon_health::DaemonHealth>>,
    /// Anna Mode - normal vs safe mode (6.22.0)
    pub anna_mode: Arc<tokio::sync::RwLock<crate::daemon_health::AnnaMode>>,
    /// Update Manager - background auto-update system (6.22.0)
    pub update_manager: Arc<tokio::sync::RwLock<crate::update_manager::UpdateManager>>,
    /// Session Context - in-memory context for follow-ups (v6.26.0)
    pub session_context: Arc<tokio::sync::RwLock<anna_common::session_context::SessionContext>>,
    /// Toolchain Health - v6.58.0 tool self-test results
    pub toolchain_health: Arc<tokio::sync::RwLock<anna_common::command_exec::ToolchainHealth>>,
}

impl DaemonState {
    pub async fn new(
        version: String,
        facts: SystemFacts,
        advice: Vec<Advice>,
        health: crate::daemon_health::DaemonHealth,
    ) -> Result<Self> {
        let audit_logger = AuditLogger::new().await?;
        let action_history = crate::action_history::ActionHistory::new().await?;

        // Phase 0.2c: Detect initial system state
        let current_state = crate::state::detect_state()?;
        info!(
            "Initial state detected: {} - {}",
            current_state.state, current_state.citation
        );

        // 6.12.0: Initialize system knowledge manager
        let knowledge_path = crate::system_knowledge::default_knowledge_path();
        let mut knowledge_mgr = crate::system_knowledge::SystemKnowledgeManager::load_or_init(&knowledge_path)?;
        // Take initial snapshot
        let _ = knowledge_mgr.snapshot_now();

        // 6.22.0: Initialize Anna Mode - enter Safe Mode if health indicates it
        let anna_mode = if health.health_state == crate::daemon_health::DaemonHealthState::SafeMode {
            crate::daemon_health::AnnaMode::Safe {
                reason: health.last_exit_reason.clone().unwrap_or_else(|| "Unknown reason".to_string()),
                since: std::time::Instant::now(),
            }
        } else {
            crate::daemon_health::AnnaMode::Normal
        };

        // 6.22.0: Initialize Update Manager with config
        let config = ConfigData::default();
        let update_manager = crate::update_manager::UpdateManager::new(
            version.clone(),
            config.auto_update_check,
        )?;

        // v6.26.0: Initialize Session Context for follow-up queries
        let session_context = anna_common::session_context::SessionContext::new();

        // v6.58.0: Run tool self-test at startup - Toolchain Reality Lock
        let command_exec = anna_common::command_exec::CommandExec::new();
        let toolchain_health = command_exec.self_test();
        match toolchain_health.status {
            anna_common::command_exec::ToolchainStatus::Healthy => {
                info!("🔧  Toolchain self-test: HEALTHY - all essential tools available");
            }
            anna_common::command_exec::ToolchainStatus::Degraded => {
                warn!("⚠️  Toolchain self-test: DEGRADED - some optional tools missing");
                for tool in &toolchain_health.tools {
                    if !tool.available {
                        warn!("   └─ {}: {}", tool.name, tool.status_message);
                    }
                }
            }
            anna_common::command_exec::ToolchainStatus::Critical => {
                error!("🚨  Toolchain self-test: CRITICAL - essential tools missing!");
                for tool in &toolchain_health.tools {
                    if !tool.available {
                        error!("   └─ {}: {}", tool.name, tool.status_message);
                    }
                }
            }
        }

        Ok(Self {
            version,
            start_time: std::time::Instant::now(),
            facts: RwLock::new(facts),
            advice: RwLock::new(advice),
            config: RwLock::new(config),
            audit_logger,
            action_history,
            rate_limiter: RateLimiter::new(120), // 120 requests per minute (2 per second)
            current_state: RwLock::new(current_state),
            sentinel: None,     // Will be set later in main.rs
            collective: None,   // Will be set later in main.rs
            mirror: None,       // Will be set later in main.rs
            chronos: None,      // Will be set later in main.rs
            mirror_audit: None, // Will be set later in main.rs
            historian: None,    // Will be set later in main.rs
            knowledge: Arc::new(tokio::sync::RwLock::new(knowledge_mgr)),
            health: Arc::new(tokio::sync::RwLock::new(health)),
            anna_mode: Arc::new(tokio::sync::RwLock::new(anna_mode)),
            update_manager: Arc::new(tokio::sync::RwLock::new(update_manager)),
            session_context: Arc::new(tokio::sync::RwLock::new(session_context)),
            toolchain_health: Arc::new(tokio::sync::RwLock::new(toolchain_health)),
        })
    }
}

/// Filter advice to remove items satisfied by previously applied advice
async fn filter_satisfied_advice(
    advice: Vec<Advice>,
    audit_logger: &AuditLogger,
    all_advice: &[Advice],
) -> Vec<Advice> {
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
        info!(
            "Filtered out {} satisfied advice items",
            original_count - filtered.len()
        );
    }

    filtered
}

/// Detect if advice recommends resource-intensive software
fn is_resource_intensive_advice(advice: &Advice) -> bool {
    let text = format!("{} {}", advice.title, advice.reason).to_lowercase();

    // Resource-intensive keywords
    let heavy_keywords = vec![
        "electron",
        "vscode",
        "code",
        "chrome",
        "chromium",
        "intellij",
        "pycharm",
        "clion",
        "webstorm",
        "rider",
        "docker",
        "kubernetes",
        "k8s",
        "virtualbox",
        "vmware",
        "blender",
        "gimp",
        "inkscape",
        "kdenlive",
        "davinci",
        "steam",
        "lutris",
        "proton",
        "gaming",
        "gnome",
        "kde",
        "plasma",
        "cinnamon", // Heavy DEs
        "compiz",
        "picom",
        "compositor", // GPU-intensive
        "btop++",
        "btop", // More resource-intensive than htop
        "conky",
        "eww",
        "polybar", // Resource monitoring/bars
    ];

    heavy_keywords.iter().any(|keyword| text.contains(keyword))
}

/// AGGRESSIVE relevance filter (Beta.106+) - Only show what's ACTUALLY relevant
///
/// Problem: Showing 272 random recommendations is overwhelming and useless
/// Solution: Only show advice for software the user ACTUALLY has/uses
///
/// RC.9.8: Now uses system score from telemetry for resource-aware filtering:
/// - Potato computers (score 0-30): Only essential, lightweight recommendations
/// - Mid-range (score 31-65): Moderate recommendations
/// - High-end (score 66-100): All recommendations including resource-intensive
fn filter_by_relevance(advice: Vec<Advice>, facts: &SystemFacts) -> Vec<Advice> {
    use anna_common::Priority;

    // Use performance score from telemetry (calculated once during system scan)
    let score = facts.performance_score;

    advice
        .into_iter()
        .filter(|item| {
            // ALWAYS show security & privacy issues (critical)
            if item.category.contains("Security") || item.category.contains("Privacy") {
                return true;
            }

            // ALWAYS show Mandatory items (critical fixes)
            if matches!(item.priority, Priority::Mandatory) {
                return true;
            }

            // Resource-aware filtering for Recommended+ items
            // Check if advice is resource-intensive based on title/reason
            let is_resource_intensive = is_resource_intensive_advice(item);

            if is_resource_intensive {
                // Only show resource-intensive advice for high-score systems
                if score < 50 {
                    // Potato/low-end system: skip resource-intensive recommendations
                    return false;
                }
            }

            // Show Recommended items (unless filtered out above)
            if matches!(item.priority, Priority::Recommended) {
                return true;
            }

            // For bundle items - ONLY if the bundle WM/DE is installed
            // This is handled in the bundle builder now, but double-check
            if item.bundle.is_some() {
                // Bundles should already be filtered by installed WM
                return true;
            }

            // For other items: only show if related to installed software
            // Check if the advice is about an installed package by extracting package names
            let advice_lower = item.title.to_lowercase();

            // Common package names to check (extracted from title/reason)
            let potential_packages: Vec<&str> = vec![
                "hyprland", "waybar", "rofi", "kitty", "nautilus", "mako", "i3", "bspwm",
                "awesome", "sway", "firefox", "chromium", "docker", "git", "vim", "nvim", "vscode",
                "python", "rust", "gnome", "kde", "plasma", "xfce", "lxqt", "cinnamon", "mate",
            ];

            // Check if any common package mentioned in advice is actually installed
            let mentions_installed = potential_packages.iter().any(|&pkg| {
                if advice_lower.contains(pkg) {
                    // Check if this package is installed
                    use std::process::Command;
                    Command::new("pacman")
                        .args(&["-Q", pkg])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                } else {
                    false
                }
            });

            if mentions_installed {
                return true;
            }

            // Show system-wide optimizations (not tied to specific software)
            if item.category.contains("performance")
                || item.category.contains("cleanup")
                || item.category.contains("System")
            {
                return true;
            }

            // For documentation items - only if about installed software
            if item.category == "documentation" {
                return mentions_installed;
            }

            // Default: HIDE IT (don't spam user with irrelevant stuff)
            false
        })
        .collect()
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
    let listener = UnixListener::bind(SOCKET_PATH).context("Failed to bind Unix socket")?;

    info!("RPC server listening on {}", SOCKET_PATH);

    // Phase 0.4: Set socket permissions to 0660 (owner and group only)
    // SECURITY: Only root and users in the socket's group can connect
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o660))
            .context("Failed to set socket permissions to 0660")?;

        // Phase 0.4: Change socket group to 'anna' so users in that group can connect
        // Check current ownership first - systemd may have already set it correctly
        use std::os::unix::fs::MetadataExt;
        use std::process::Command;

        let metadata = std::fs::metadata(SOCKET_PATH).context("Failed to read socket metadata")?;
        let current_gid = metadata.gid();

        // Get 'anna' group GID if it exists
        let anna_gid_result = Command::new("getent").args(&["group", "anna"]).output();

        let needs_chown = if let Ok(output) = anna_gid_result {
            if output.status.success() {
                let getent_output = String::from_utf8_lossy(&output.stdout);
                // Format: anna:x:964:users
                if let Some(gid_str) = getent_output.split(':').nth(2) {
                    if let Ok(anna_gid) = gid_str.trim().parse::<u32>() {
                        current_gid != anna_gid
                    } else {
                        true // Can't parse, try chown anyway
                    }
                } else {
                    true // Can't parse, try chown anyway
                }
            } else {
                false // Group doesn't exist, skip chown
            }
        } else {
            false // getent not available, skip chown
        };

        if needs_chown {
            let chown_result = Command::new("chown")
                .args(&["root:anna", SOCKET_PATH])
                .output()
                .context("Failed to execute chown command")?;

            if !chown_result.status.success() {
                // Check if error is just "Operation not permitted" (systemd might manage it)
                let stderr = String::from_utf8_lossy(&chown_result.stderr);
                if stderr.contains("Operation not permitted") {
                    tracing::debug!(
                        "Socket chown skipped (systemd manages permissions): {}",
                        stderr
                    );
                    info!("Socket permissions managed by systemd (this is fine)");
                } else {
                    warn!("Failed to set socket group to 'anna': {}", stderr);
                    warn!("Socket will be owned by current user/group");
                    warn!("Run: sudo groupadd --system anna");
                }
            } else {
                info!("Socket permissions set to root:anna 0660");
            }
        } else {
            info!("Socket already has correct group ownership");
        }
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
    use crate::executor::ExecutionChunk;
    use anna_common::ipc::{Response, ResponseData, StreamChunkType};

    info!("Handling streaming apply for advice: {}", advice_id);

    // Find the advice
    let advice_list = state.advice.read().await;
    let advice = match advice_list.iter().find(|a| a.id == advice_id).cloned() {
        Some(adv) => adv,
        None => {
            let response = Response {
                id: request_id,
                result: Err(format!("Advice not found: {}", advice_id)),
                version: "1.0.0".to_string(),
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
            version: "1.0.0".to_string(),
        };
        let json = serde_json::to_string(&response)? + "\n";
        writer.write_all(json.as_bytes()).await?;

        let end_response = Response {
            id: request_id,
            result: Ok(ResponseData::StreamEnd {
                success: true,
                message: "Dry run completed".to_string(),
            }),
            version: "1.0.0".to_string(),
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
                version: "1.0.0".to_string(),
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
            ExecutionChunk::Stdout(line)
            | ExecutionChunk::Stderr(line)
            | ExecutionChunk::Status(line) => line,
        };

        let response = Response {
            id: request_id,
            result: Ok(ResponseData::StreamChunk { chunk_type, data }),
            version: "1.0.0".to_string(),
        };

        // Send the chunk immediately
        let json = serde_json::to_string(&response)? + "\n";
        writer.write_all(json.as_bytes()).await?;
    }

    // Wait for execution to complete and get result
    let success = exec_handle.await.context("Execution task panicked")??;

    // Generate rollback command (Beta.89)
    let (rollback_command, can_rollback, rollback_unavailable_reason) =
        anna_common::rollback::generate_rollback_command(&command);

    // Create audit entry
    let action = anna_common::Action {
        id: format!("action_{}", uuid::Uuid::new_v4()),
        advice_id: advice.id.clone(),
        command,
        executed_at: chrono::Utc::now(),
        success,
        output: full_output.clone(),
        error: if success {
            None
        } else {
            Some("Command failed".to_string())
        },
        rollback_command,
        can_rollback,
        rollback_unavailable_reason,
    };

    let audit_entry = executor::create_audit_entry(&action, "annactl");
    if let Err(e) = state.audit_logger.log(&audit_entry).await {
        warn!("Failed to log audit entry: {}", e);
    }

    // Save to action history for rollback support (Beta.91)
    if !dry_run && success {
        if let Err(e) = state.action_history.save(&action).await {
            warn!("Failed to save action to history: {}", e);
        }
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
        version: "1.0.0".to_string(),
    };
    let json = serde_json::to_string(&end_response)? + "\n";
    writer.write_all(json.as_bytes()).await?;

    Ok(())
}

/// Handle a single client connection
async fn handle_connection(stream: UnixStream, state: Arc<DaemonState>) -> Result<()> {
    // SECURITY: Verify peer credentials before processing any requests
    #[cfg(unix)]
    let client_uid = {
        use nix::sys::socket::{getsockopt, sockopt::PeerCredentials};

        let cred =
            getsockopt(&stream, PeerCredentials).context("Failed to get peer credentials")?;

        let uid = cred.uid();
        let gid = cred.gid();
        let pid = cred.pid();

        // v6.40.0: Log routine connections at DEBUG level to reduce noise
        // Only security failures (access denied) remain at WARN level
        debug!("Connection from UID {} GID {} PID {}", uid, gid, pid);

        // SECURITY: Enforce group-based access control
        // Check if user is in 'annactl' group (or is root)
        match is_user_in_group(uid, ANNACTL_GROUP) {
            Ok(true) => {
                debug!("Access granted: UID {} is authorized", uid);
            }
            Ok(false) => {
                warn!(
                    "SECURITY: Access denied for UID {} - not in '{}' group",
                    uid, ANNACTL_GROUP
                );
                return Err(anyhow::anyhow!(
                    "Access denied: User not in authorized group"
                ));
            }
            Err(e) => {
                // If group doesn't exist, log warning but allow (graceful degradation)
                // This prevents lockout if group isn't set up yet
                warn!(
                    "Group check failed (allowing): {} - Is '{}' group created?",
                    e, ANNACTL_GROUP
                );
            }
        }

        uid
    };

    #[cfg(not(unix))]
    let client_uid = 0; // Default UID for non-Unix systems

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

        // SECURITY: Check message size to prevent DoS attacks
        if line.len() > MAX_REQUEST_SIZE {
            warn!(
                "Request too large from UID {}: {} bytes (max: {})",
                client_uid,
                line.len(),
                MAX_REQUEST_SIZE
            );
            let error_response = Response {
                id: 0, // We don't have the request ID yet
                result: Err(format!(
                    "Request too large: {} bytes (max: {} bytes)",
                    line.len(),
                    MAX_REQUEST_SIZE
                )),
                version: "1.0.0".to_string(),
            };
            let response_json = serde_json::to_string(&error_response)? + "\n";
            let _ = writer.write_all(response_json.as_bytes()).await;
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid request JSON: {}", e);
                continue;
            }
        };

        // SECURITY: Check rate limit for this client
        if let Err(e) = state.rate_limiter.check_rate_limit(client_uid).await {
            warn!("Rate limit exceeded for UID {}: {}", client_uid, e);
            let error_response = Response {
                id: request.id,
                result: Err(format!("Rate limit exceeded. Please try again later.")),
                version: "1.0.0".to_string(),
            };
            let response_json = serde_json::to_string(&error_response)? + "\n";
            let _ = writer.write_all(response_json.as_bytes()).await;
            continue;
        }

        // Check if this is a streaming ApplyAction request
        if let Method::ApplyAction {
            advice_id,
            dry_run,
            stream: true,
        } = &request.method
        {
            // Handle streaming separately
            if let Err(e) =
                handle_streaming_apply(request.id, advice_id, *dry_run, &state, &mut writer).await
            {
                error!("Streaming apply failed: {}", e);
                let error_response = Response {
                    id: request.id,
                    result: Err(format!("Streaming failed: {}", e)),
                    version: "1.0.0".to_string(),
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

            // 6.20.0: Read daemon health state
            let health = state.health.read().await;
            let health_state_str = health.health_state.as_str().to_string();
            let health_reason = if health.health_state != crate::daemon_health::DaemonHealthState::Healthy {
                health.last_exit_reason.clone()
            } else {
                None
            };

            // 6.22.0: Read Anna mode and update state
            let anna_mode = state.anna_mode.read().await;
            let anna_mode_str = anna_mode.as_str().to_string();
            let anna_mode_reason = anna_mode.reason().map(|s| s.to_string());

            let update_mgr = state.update_manager.read().await;
            let update_status_str = update_mgr.get_state().status_string();

            let status = StatusData {
                version: state.version.clone(),
                uptime_seconds: state.start_time.elapsed().as_secs(),
                last_telemetry_check: "Just now".to_string(), // TODO: track actual last check
                pending_recommendations: advice.len(),
                health_state: Some(health_state_str),
                health_reason,
                anna_mode: Some(anna_mode_str),
                anna_mode_reason,
                update_status: Some(update_status_str),
            };
            Ok(ResponseData::Status(status))
        }

        Method::GetFacts => {
            let facts = state.facts.read().await.clone();
            Ok(ResponseData::Facts(facts))
        }

        Method::GetAdvice => {
            let all_advice = state.advice.read().await.clone();

            // Get system facts for requirement checking (RC.6)
            let facts = state.facts.read().await.clone();

            // Filter by requirements first (RC.6 - Smart filtering)
            let requirement_filtered: Vec<_> = all_advice
                .iter()
                .filter(|advice| advice.requirements_met(&facts))
                .cloned()
                .collect();

            // Then filter satisfied advice
            let filtered_advice = filter_satisfied_advice(
                requirement_filtered.clone(),
                &state.audit_logger,
                &requirement_filtered,
            )
            .await;
            Ok(ResponseData::Advice(filtered_advice))
        }

        Method::GetAdviceWithContext {
            username,
            desktop_env,
            shell,
            display_server,
        } => {
            // Filter advice based on user context AND system requirements (RC.6)
            let all_advice = state.advice.read().await.clone();
            let total_count = all_advice.len();

            // Get system facts for requirement checking (RC.6)
            let facts = state.facts.read().await.clone();

            // First filter by requirements (RC.6 - Smart filtering)
            let requirement_filtered: Vec<_> = all_advice
                .iter()
                .filter(|advice| advice.requirements_met(&facts))
                .cloned()
                .collect();

            // Then apply context filtering
            let context_filtered: Vec<_> = requirement_filtered
                .into_iter()
                .filter(|advice| {
                    // System-wide advice (security, updates, etc.) - show to everyone
                    if matches!(
                        advice.category.as_str(),
                        "security" | "updates" | "performance" | "cleanup"
                    ) {
                        return true;
                    }

                    // Desktop-specific advice - filter by user's DE
                    if advice.category == "desktop" {
                        if let Some(ref de) = desktop_env {
                            let de_lower = de.to_lowercase();
                            // Check if advice ID matches user's DE
                            return advice.id.contains(&de_lower)
                                || advice.title.to_lowercase().contains(&de_lower);
                        }
                        return false;
                    }

                    // Gaming advice - show to all (they chose to install Steam)
                    if advice.category == "gaming" {
                        return true;
                    }

                    // Shell-specific advice
                    if advice.id.contains("shell")
                        || advice.id.contains("zsh")
                        || advice.id.contains("bash")
                    {
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

            // Finally filter out satisfied advice
            let filtered_advice =
                filter_satisfied_advice(context_filtered, &state.audit_logger, &all_advice).await;

            info!(
                "Filtered advice for user {} (req->ctx->satisfied): {} -> {} items",
                username,
                total_count,
                filtered_advice.len()
            );

            Ok(ResponseData::Advice(filtered_advice))
        }

        Method::ApplyAction {
            advice_id,
            dry_run,
            stream,
        } => {
            if stream {
                info!(
                    "Streaming requested for action {} (not yet implemented)",
                    advice_id
                );
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

                            // Save to action history for rollback support (Beta.91)
                            if !dry_run && action.success {
                                if let Err(e) = state.action_history.save(&action).await {
                                    warn!("Failed to save action to history: {}", e);
                                }
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

                                let mut history =
                                    anna_common::ApplicationHistory::load().unwrap_or_default();
                                history.record(history_entry);
                                info!("History entries before save: {}", history.entries.len());
                                match history.save() {
                                    Ok(()) => info!(
                                        "Application history saved successfully to: {:?}",
                                        anna_common::ApplicationHistory::history_path()
                                    ),
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

                    // AGGRESSIVE RELEVANCE FILTERING (Beta.106)
                    // Only show advice that's ACTUALLY relevant to this specific user/system
                    let total_before = advice.len();
                    advice = filter_by_relevance(advice, &facts);
                    let total_after = advice.len();

                    info!(
                        "Relevance filter: {} → {} recommendations ({} removed)",
                        total_before,
                        total_after,
                        total_before - total_after
                    );

                    // Update state
                    *state.facts.write().await = facts;
                    *state.advice.write().await = advice.clone();

                    info!(
                        "Advice manually refreshed: {} recommendations",
                        advice.len()
                    );
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

        Method::CheckUpdate => {
            info!("Update check requested (delegated to daemon)");
            match anna_common::updater::check_for_updates().await {
                Ok(update_info) => {
                    info!(
                        "Update check complete: current={}, latest={}, available={}",
                        update_info.current_version,
                        update_info.latest_version,
                        update_info.is_update_available
                    );

                    Ok(ResponseData::UpdateCheck {
                        current_version: update_info.current_version,
                        latest_version: update_info.latest_version,
                        is_update_available: update_info.is_update_available,
                        download_url: Some(update_info.download_url_annad),
                        release_notes: Some(update_info.release_notes_url),
                    })
                }
                Err(e) => {
                    error!("Update check failed: {}", e);
                    Err(format!("Update check failed: {}", e))
                }
            }
        }

        Method::PerformUpdate { restart_only } => {
            if restart_only {
                info!("Restart-only update requested");
                // Schedule restart to happen AFTER response is sent
                tokio::spawn(async {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let _ = tokio::process::Command::new("systemctl")
                        .args(&["restart", "annad"])
                        .status()
                        .await;
                });

                info!("Daemon restart scheduled");
                Ok(ResponseData::UpdateResult {
                    success: true,
                    message: "Daemon restart scheduled".to_string(),
                    old_version: state.version.clone(),
                    new_version: state.version.clone(),
                })
            } else {
                // RC.9.3: Proper daemon-based update without race condition
                // Solution: Download & install, THEN schedule restart after response sent
                info!("Full update requested - performing daemon-based update");

                // Check for updates first
                match anna_common::updater::check_for_updates().await {
                    Ok(update_info) => {
                        if !update_info.is_update_available {
                            info!("Already on latest version: {}", update_info.current_version);
                            Ok(ResponseData::UpdateResult {
                                success: false,
                                message: format!(
                                    "Already on latest version: {}",
                                    update_info.current_version
                                ),
                                old_version: update_info.current_version.clone(),
                                new_version: update_info.current_version,
                            })
                        } else {
                            info!(
                                "Update available: {} → {}",
                                update_info.current_version, update_info.latest_version
                            );

                            // Download and install binaries (daemon runs as root, no sudo needed!)
                            // Use a separate async block to handle all the update logic
                            let update_result = async {
                                use anna_common::updater::download_binary;
                                use std::path::Path;

                                // Create temp directory
                                let temp_dir = std::env::temp_dir().join("anna-update");
                                tokio::fs::create_dir_all(&temp_dir).await.map_err(|e| {
                                    format!("Failed to create temp directory: {}", e)
                                })?;

                                let temp_annad = temp_dir.join("annad");
                                let temp_annactl = temp_dir.join("annactl");

                                // Download binaries
                                info!("Downloading binaries...");
                                download_binary(&update_info.download_url_annad, &temp_annad)
                                    .await
                                    .map_err(|e| format!("Failed to download annad: {}", e))?;
                                download_binary(&update_info.download_url_annactl, &temp_annactl)
                                    .await
                                    .map_err(|e| format!("Failed to download annactl: {}", e))?;

                                // Install binaries (daemon is root, can write directly)
                                info!("Installing binaries...");
                                let annad_dest = Path::new("/usr/local/bin/annad");
                                let annactl_dest = Path::new("/usr/local/bin/annactl");

                                tokio::fs::copy(&temp_annad, annad_dest)
                                    .await
                                    .map_err(|e| format!("Failed to install annad: {}", e))?;
                                tokio::fs::copy(&temp_annactl, annactl_dest)
                                    .await
                                    .map_err(|e| format!("Failed to install annactl: {}", e))?;

                                // Set executable permissions
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    let perms = std::fs::Permissions::from_mode(0o755);
                                    let _ = std::fs::set_permissions(annad_dest, perms.clone());
                                    let _ = std::fs::set_permissions(annactl_dest, perms);
                                }

                                // Clean up temp files
                                let _ = tokio::fs::remove_dir_all(&temp_dir).await;

                                Ok::<(), String>(())
                            }
                            .await;

                            match update_result {
                                Ok(()) => {
                                    let old_ver = update_info.current_version.clone();
                                    let new_ver = update_info.latest_version.clone();

                                    // Schedule restart to happen AFTER response is sent (critical!)
                                    tokio::spawn(async {
                                        tokio::time::sleep(std::time::Duration::from_millis(500))
                                            .await;
                                        info!("Restarting daemon after update...");
                                        let _ = tokio::process::Command::new("systemctl")
                                            .args(&["restart", "annad"])
                                            .status()
                                            .await;
                                    });

                                    info!("Update complete, restart scheduled");
                                    Ok(ResponseData::UpdateResult {
                                        success: true,
                                        message:
                                            "Update successful! Daemon will restart momentarily."
                                                .to_string(),
                                        old_version: old_ver,
                                        new_version: new_ver,
                                    })
                                }
                                Err(e) => {
                                    error!("Update failed: {}", e);
                                    Ok(ResponseData::UpdateResult {
                                        success: false,
                                        message: e,
                                        old_version: update_info.current_version.clone(),
                                        new_version: update_info.latest_version.clone(),
                                    })
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Update check failed: {}", e);
                        Err(format!("Update check failed: {}", e))
                    }
                }
            }
        }

        Method::ListRollbackable => {
            info!("Listing rollbackable actions");
            match state.action_history.get_rollbackable_actions().await {
                Ok(actions) => {
                    use anna_common::ipc::RollbackableAction;

                    // Get advice titles by loading current advice
                    let advice_list = state.advice.read().await;

                    let rollbackable: Vec<RollbackableAction> = actions
                        .iter()
                        .map(|action| {
                            // Find matching advice for title
                            let title = advice_list
                                .iter()
                                .find(|a| a.id == action.advice_id)
                                .map(|a| a.title.clone())
                                .unwrap_or_else(|| action.advice_id.clone());

                            RollbackableAction {
                                advice_id: action.advice_id.clone(),
                                title,
                                executed_at: action.executed_at.to_rfc3339(),
                                command: action.command.clone(),
                                rollback_command: action.rollback_command.clone(),
                                can_rollback: action.can_rollback,
                                rollback_unavailable_reason: action
                                    .rollback_unavailable_reason
                                    .clone(),
                            }
                        })
                        .collect();

                    info!("Found {} rollbackable actions", rollbackable.len());
                    Ok(ResponseData::RollbackableActions(rollbackable))
                }
                Err(e) => {
                    error!("Failed to get rollbackable actions: {}", e);
                    Err(format!("Failed to get rollbackable actions: {}", e))
                }
            }
        }

        Method::RollbackAction { advice_id, dry_run } => {
            info!(
                "Rollback requested for advice: {} (dry_run={})",
                advice_id, dry_run
            );

            match state.action_history.get_by_advice_id(&advice_id).await {
                Ok(Some(action)) => {
                    if !action.can_rollback {
                        let reason = action
                            .rollback_unavailable_reason
                            .unwrap_or_else(|| "Rollback not available".to_string());
                        return Response {
                            id,
                            result: Err(format!("Cannot rollback: {}", reason)),
                            version: "1.0.0".to_string(),
                        };
                    }

                    let rollback_cmd = match &action.rollback_command {
                        Some(cmd) => cmd.clone(),
                        None => {
                            return Response {
                                id,
                                result: Err("No rollback command available".to_string()),
                                version: "1.0.0".to_string(),
                            };
                        }
                    };

                    if dry_run {
                        info!("Dry-run rollback for {}: {}", advice_id, rollback_cmd);
                        Ok(ResponseData::RollbackResult {
                            success: true,
                            message: format!("[DRY RUN] Would execute rollback: {}", rollback_cmd),
                            actions_reversed: vec![advice_id],
                        })
                    } else {
                        // Execute the rollback command
                        info!("Executing rollback: {}", rollback_cmd);
                        match executor::execute_command(&rollback_cmd).await {
                            Ok(output) => {
                                info!("Rollback successful for {}", advice_id);
                                Ok(ResponseData::RollbackResult {
                                    success: true,
                                    message: format!("Rollback successful:\n{}", output),
                                    actions_reversed: vec![advice_id],
                                })
                            }
                            Err(e) => {
                                error!("Rollback failed for {}: {}", advice_id, e);
                                Err(format!("Rollback failed: {}", e))
                            }
                        }
                    }
                }
                Ok(None) => Err(format!("No action found for advice: {}", advice_id)),
                Err(e) => {
                    error!("Failed to retrieve action: {}", e);
                    Err(format!("Failed to retrieve action: {}", e))
                }
            }
        }

        Method::RollbackLast { count, dry_run } => {
            info!("Rollback last {} actions (dry_run={})", count, dry_run);

            match state.action_history.get_last_n_actions(count).await {
                Ok(actions) => {
                    if actions.is_empty() {
                        return Response {
                            id,
                            result: Err("No actions to rollback".to_string()),
                            version: "1.0.0".to_string(),
                        };
                    }

                    // Filter rollbackable actions
                    let rollbackable: Vec<_> = actions
                        .into_iter()
                        .filter(|a| a.can_rollback && a.rollback_command.is_some())
                        .collect();

                    if rollbackable.is_empty() {
                        return Response {
                            id,
                            result: Err("No rollbackable actions in last {} actions".to_string()),
                            version: "1.0.0".to_string(),
                        };
                    }

                    if dry_run {
                        let commands: Vec<String> = rollbackable
                            .iter()
                            .map(|a| {
                                format!("{}: {}", a.advice_id, a.rollback_command.as_ref().unwrap())
                            })
                            .collect();

                        Ok(ResponseData::RollbackResult {
                            success: true,
                            message: format!(
                                "[DRY RUN] Would rollback {} actions:\n{}",
                                rollbackable.len(),
                                commands.join("\n")
                            ),
                            actions_reversed: rollbackable
                                .iter()
                                .map(|a| a.advice_id.clone())
                                .collect(),
                        })
                    } else {
                        // Execute rollbacks in reverse order (most recent first)
                        let mut reversed_ids = Vec::new();
                        let mut all_outputs = Vec::new();

                        for action in rollbackable {
                            let cmd = action.rollback_command.as_ref().unwrap();
                            info!("Executing rollback for {}: {}", action.advice_id, cmd);

                            match executor::execute_command(cmd).await {
                                Ok(output) => {
                                    info!("Rollback successful for {}", action.advice_id);
                                    reversed_ids.push(action.advice_id.clone());
                                    all_outputs.push(format!(
                                        "✓ {}: {}",
                                        action.advice_id,
                                        output.lines().next().unwrap_or("")
                                    ));
                                }
                                Err(e) => {
                                    error!("Rollback failed for {}: {}", action.advice_id, e);
                                    all_outputs.push(format!("✗ {}: {}", action.advice_id, e));
                                }
                            }
                        }

                        let success = !reversed_ids.is_empty();
                        Ok(ResponseData::RollbackResult {
                            success,
                            message: format!(
                                "Rolled back {} of {} actions:\n{}",
                                reversed_ids.len(),
                                count,
                                all_outputs.join("\n")
                            ),
                            actions_reversed: reversed_ids,
                        })
                    }
                }
                Err(e) => {
                    error!("Failed to get last actions: {}", e);
                    Err(format!("Failed to get last actions: {}", e))
                }
            }
        }

        Method::GetState => {
            info!("GetState method called");

            // Phase 0.2c: Use cached state instead of detecting every time
            let detection = state.current_state.read().await.clone();

            // Convert to IPC response type
            let data = anna_common::ipc::StateDetectionData {
                state: detection.state.to_string(),
                detected_at: detection.detected_at,
                details: anna_common::ipc::StateDetailsData {
                    uefi: detection.details.uefi,
                    disks: detection.details.disks,
                    network: anna_common::ipc::NetworkStatusData {
                        has_interface: detection.details.network.has_interface,
                        has_route: detection.details.network.has_route,
                        can_resolve: detection.details.network.can_resolve,
                    },
                    state_file_present: detection.details.state_file_present,
                    health_ok: detection.details.health_ok,
                },
                citation: detection.citation,
            };
            Ok(ResponseData::StateDetection(data))
        }

        Method::GetCapabilities => {
            info!("GetCapabilities method called");

            // Phase 0.2c: Use cached state
            let detection = state.current_state.read().await.clone();
            let capabilities = crate::state::get_capabilities(detection.state);

            // Convert to IPC response type
            let commands: Vec<anna_common::ipc::CommandCapabilityData> = capabilities
                .into_iter()
                .map(|cap| anna_common::ipc::CommandCapabilityData {
                    name: cap.name,
                    description: cap.description,
                    since: cap.since,
                    citation: cap.citation,
                    requires_root: cap.requires_root,
                })
                .collect();

            // Phase 3.0: Add system profile for adaptive intelligence
            let mut profiler = crate::profile::SystemProfiler::new();
            let (monitoring_mode, monitoring_rationale, is_constrained) = match profiler
                .collect_profile()
            {
                Ok(profile) => {
                    let mode = match profile.recommended_monitoring_mode {
                        crate::profile::MonitoringMode::Full => "full",
                        crate::profile::MonitoringMode::Light => "light",
                        crate::profile::MonitoringMode::Minimal => "minimal",
                    }
                    .to_string();
                    let rationale = profile.monitoring_rationale();
                    let constrained = profile.is_constrained();
                    (mode, rationale, constrained)
                }
                Err(e) => {
                    warn!("Failed to collect system profile: {}", e);
                    (
                        "light".to_string(),
                        "Unable to detect system profile, using light mode as default".to_string(),
                        false,
                    )
                }
            };

            let data = anna_common::ipc::CapabilitiesData {
                commands,
                monitoring_mode,
                monitoring_rationale,
                is_constrained,
            };

            Ok(ResponseData::Capabilities(data))
        }

        Method::GetProfile => {
            info!("GetProfile method called");

            // Phase 3.0: Collect system profile for adaptive intelligence
            let mut profiler = crate::profile::SystemProfiler::new();
            match profiler.collect_profile() {
                Ok(profile) => {
                    // Convert enums to strings for IPC
                    let virtualization = match &profile.virtualization {
                        crate::profile::VirtualizationInfo::None => "none".to_string(),
                        crate::profile::VirtualizationInfo::VM(vtype) => format!("vm:{}", vtype),
                        crate::profile::VirtualizationInfo::Container(ctype) => {
                            format!("container:{}", ctype)
                        }
                        crate::profile::VirtualizationInfo::Unknown => "unknown".to_string(),
                    };

                    let session_type = match &profile.session_type {
                        crate::profile::SessionType::Desktop(s) => format!("desktop:{}", s),
                        crate::profile::SessionType::Headless => "headless".to_string(),
                        crate::profile::SessionType::SSH {
                            client_ip,
                            display_forwarding,
                        } => {
                            format!(
                                "ssh:{}:forwarding={}",
                                client_ip.as_deref().unwrap_or("unknown"),
                                display_forwarding
                            )
                        }
                        crate::profile::SessionType::Console => "console".to_string(),
                        crate::profile::SessionType::Unknown => "unknown".to_string(),
                    };

                    let monitoring_mode = match profile.recommended_monitoring_mode {
                        crate::profile::MonitoringMode::Full => "full",
                        crate::profile::MonitoringMode::Light => "light",
                        crate::profile::MonitoringMode::Minimal => "minimal",
                    }
                    .to_string();

                    // Call methods that borrow profile before moving fields
                    let monitoring_rationale = profile.monitoring_rationale();
                    let is_constrained = profile.is_constrained();
                    let timestamp = profile.timestamp.to_rfc3339();

                    let data = anna_common::ipc::ProfileData {
                        total_memory_mb: profile.total_memory_mb,
                        available_memory_mb: profile.available_memory_mb,
                        cpu_cores: profile.cpu_cores,
                        total_disk_gb: profile.total_disk_gb,
                        available_disk_gb: profile.available_disk_gb,
                        uptime_seconds: profile.uptime_seconds,
                        virtualization,
                        session_type,
                        gpu_present: profile.gpu_info.present,
                        gpu_vendor: profile.gpu_info.vendor,
                        gpu_model: profile.gpu_info.model,
                        recommended_monitoring_mode: monitoring_mode,
                        monitoring_rationale,
                        is_constrained,
                        timestamp,
                    };

                    Ok(ResponseData::Profile(data))
                }
                Err(e) => {
                    error!("Failed to collect system profile: {}", e);
                    Err(format!("Failed to collect system profile: {}", e))
                }
            }
        }

        Method::GetHistorianSummary => {
            info!("GetHistorianSummary method called");

            // Beta.53: Return 30-day Historian summary
            if let Some(historian_mutex) = &state.historian {
                match historian_mutex.try_lock() {
                    Ok(historian) => match historian.get_system_summary() {
                        Ok(summary) => {
                            info!(
                                "Generated Historian summary: {} days analyzed",
                                summary.health_summary.days_analyzed
                            );
                            Ok(ResponseData::HistorianSummary(summary))
                        }
                        Err(e) => {
                            error!("Failed to generate Historian summary: {}", e);
                            Err(format!("Failed to generate Historian summary: {}", e))
                        }
                    },
                    Err(_) => {
                        warn!("Historian is currently locked, cannot generate summary");
                        Err("Historian is busy, try again later".to_string())
                    }
                }
            } else {
                warn!("Historian not initialized");
                Err("Historian not available".to_string())
            }
        }

        Method::GetTelemetrySnapshot => {
            info!("GetTelemetrySnapshot method called");
            // Beta.213: Return minimal telemetry snapshot for welcome reports
            match crate::telemetry::collect_facts().await {
                Ok(facts) => {
                    // Extract the 6 required fields from SystemFacts
                    let snapshot = anna_common::telemetry::TelemetrySnapshot {
                        cpu_count: facts.cpu_cores,
                        total_ram_mb: (facts.total_memory_gb * 1024.0) as u64,
                        hostname: facts.hostname,
                        kernel_version: facts.kernel,
                        package_count: facts.installed_packages,
                        available_disk_gb: {
                            // Sum available space across all storage devices (size - used)
                            facts.storage_devices.iter()
                                .map(|d| (d.size_gb - d.used_gb).max(0.0))
                                .sum::<f64>() as u64
                        },
                    };
                    info!("Generated telemetry snapshot: hostname={}, packages={}, disk={}GB",
                          snapshot.hostname, snapshot.package_count, snapshot.available_disk_gb);
                    Ok(ResponseData::TelemetrySnapshot(snapshot))
                }
                Err(e) => {
                    error!("Failed to collect telemetry: {}", e);
                    Err(format!("Failed to collect telemetry: {}", e))
                }
            }
        }

        Method::HealthProbe => {
            info!("HealthProbe method called");
            Ok(ResponseData::HealthProbe {
                ok: true,
                version: state.version.clone(),
            })
        }

        Method::HealthRun { timeout_ms, probes } => {
            info!(
                "HealthRun method called with timeout={}ms, probes={:?}",
                timeout_ms, probes
            );

            // Run health checks (Phase 0.5b)
            let summary = match crate::health::run_all_probes().await {
                Ok(s) => s,
                Err(e) => {
                    error!("Health run failed: {}", e);
                    return Response {
                        id,
                        result: Err(format!("Health run failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // Get current state
            let detection = state.current_state.read().await.clone();

            // Convert to IPC response type
            let mut ok_count = 0;
            let mut warn_count = 0;
            let mut fail_count = 0;

            let results: Vec<anna_common::ipc::HealthProbeResult> = summary
                .probes
                .into_iter()
                .map(|probe| {
                    match probe.status {
                        crate::health::ProbeStatus::Ok => ok_count += 1,
                        crate::health::ProbeStatus::Warn => warn_count += 1,
                        crate::health::ProbeStatus::Fail => fail_count += 1,
                    }

                    anna_common::ipc::HealthProbeResult {
                        probe: probe.probe,
                        status: format!("{:?}", probe.status).to_lowercase(),
                        details: probe.details,
                        citation: probe.citation,
                        duration_ms: probe.duration_ms,
                        ts: chrono::Utc::now().to_rfc3339(),
                    }
                })
                .collect();

            let data = anna_common::ipc::HealthRunData {
                state: detection.state.to_string(),
                summary: anna_common::ipc::HealthSummaryCount {
                    ok: ok_count,
                    warn: warn_count,
                    fail: fail_count,
                },
                results,
                citation: "[archwiki:General_recommendations]".to_string(),
            };

            Ok(ResponseData::HealthRun(data))
        }

        Method::HealthSummary => {
            info!("HealthSummary method called");

            // Get health summary from last run (Phase 0.5b)
            let summary_opt = match crate::health::get_health_summary().await {
                Ok(s) => s,
                Err(e) => {
                    warn!("Health summary failed: {}", e);
                    None // Return empty summary if read fails
                }
            };

            let (summary_count, last_run_ts, alerts) = if let Some(summary) = summary_opt {
                let mut ok_count = 0;
                let mut warn_count = 0;
                let mut fail_count = 0;
                let mut alert_probes = Vec::new();

                for probe in &summary.probes {
                    match probe.status {
                        crate::health::ProbeStatus::Ok => ok_count += 1,
                        crate::health::ProbeStatus::Warn => warn_count += 1,
                        crate::health::ProbeStatus::Fail => {
                            fail_count += 1;
                            alert_probes.push(probe.probe.clone());
                        }
                    }
                }

                (
                    anna_common::ipc::HealthSummaryCount {
                        ok: ok_count,
                        warn: warn_count,
                        fail: fail_count,
                    },
                    summary.timestamp,
                    alert_probes,
                )
            } else {
                // No previous run
                (
                    anna_common::ipc::HealthSummaryCount {
                        ok: 0,
                        warn: 0,
                        fail: 0,
                    },
                    chrono::Utc::now().to_rfc3339(),
                    vec![],
                )
            };

            let detection = state.current_state.read().await.clone();

            let data = anna_common::ipc::HealthSummaryData {
                state: detection.state.to_string(),
                summary: summary_count,
                last_run_ts,
                alerts,
                citation: "[archwiki:General_recommendations]".to_string(),
            };

            Ok(ResponseData::HealthSummary(data))
        }

        Method::RecoveryPlans => {
            info!("RecoveryPlans method called");

            // Return list of available recovery plans (Phase 0.5b)
            let plans = vec![
                anna_common::ipc::RecoveryPlanItem {
                    id: "bootloader".to_string(),
                    desc: "Inspect and repair bootloader entries".to_string(),
                    citation: "[archwiki:GRUB]".to_string(),
                },
                anna_common::ipc::RecoveryPlanItem {
                    id: "initramfs".to_string(),
                    desc: "Rebuild initramfs via mkinitcpio".to_string(),
                    citation: "[archwiki:mkinitcpio]".to_string(),
                },
                anna_common::ipc::RecoveryPlanItem {
                    id: "pacman-db".to_string(),
                    desc: "Rebuild pacman DB and keys".to_string(),
                    citation: "[archwiki:pacman#Database_is_corrupted]".to_string(),
                },
                anna_common::ipc::RecoveryPlanItem {
                    id: "fstab".to_string(),
                    desc: "Validate and repair fstab".to_string(),
                    citation: "[archwiki:Fstab]".to_string(),
                },
                anna_common::ipc::RecoveryPlanItem {
                    id: "systemd".to_string(),
                    desc: "Analyze failed units and default target".to_string(),
                    citation: "[archwiki:systemd]".to_string(),
                },
            ];

            let data = anna_common::ipc::RecoveryPlansData {
                plans,
                citation: "[archwiki:General_troubleshooting]".to_string(),
            };

            Ok(ResponseData::RecoveryPlans(data))
        }

        Method::RepairProbe { probe, dry_run } => {
            info!(
                "RepairProbe method called: probe={}, dry_run={}",
                probe, dry_run
            );

            // Get current health results if needed (for "all" mode)
            let health_results = if probe == "all" {
                match crate::health::run_all_probes().await {
                    Ok(summary) => Some(summary.probes),
                    Err(e) => {
                        error!("Failed to run health probes: {}", e);
                        return Response {
                            id,
                            result: Err(format!("Failed to run health probes: {}", e)),
                            version: state.version.clone(),
                        };
                    }
                }
            } else {
                None
            };

            // Execute repair
            let repairs =
                match crate::repair::repair_probe(&probe, dry_run, health_results.as_deref()).await
                {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Repair failed: {}", e);
                        return Response {
                            id,
                            result: Err(format!("Repair failed: {}", e)),
                            version: state.version.clone(),
                        };
                    }
                };

            // Log to audit trail
            for repair in &repairs {
                use anna_common::AuditEntry;
                use chrono::Utc;

                let entry = AuditEntry {
                    timestamp: Utc::now(),
                    actor: "annad".to_string(),
                    action_type: format!("repair_{}", repair.probe),
                    details: format!(
                        "{} - {} (dry_run={}): {}",
                        repair.probe, repair.action, dry_run, repair.details
                    ),
                    success: repair.success,
                };

                if let Err(e) = state.audit_logger.log(&entry).await {
                    warn!("Failed to write audit log: {}", e);
                }
            }

            // Determine overall success and current state
            let all_success = repairs.iter().all(|r| r.success);
            let detection = state.current_state.read().await.clone();

            let data = anna_common::ipc::RepairResultData {
                dry_run,
                state: detection.state.to_string(),
                repairs,
                success: all_success,
                message: if dry_run {
                    "Repair simulation completed".to_string()
                } else if all_success {
                    "All repairs completed successfully".to_string()
                } else {
                    "Some repairs failed".to_string()
                },
                citation: "[archwiki:System_maintenance]".to_string(),
            };

            Ok(ResponseData::RepairResult(data))
        }

        Method::PerformInstall { config, dry_run } => {
            info!(
                "PerformInstall method called: hostname={}, dry_run={}",
                config.hostname, dry_run
            );

            // Convert IPC config to internal config
            use crate::install::{BootloaderType, DiskSetupMode, InstallConfig};

            let bootloader = match config.bootloader.as_str() {
                "systemd-boot" => BootloaderType::SystemdBoot,
                "grub" => BootloaderType::Grub,
                _ => BootloaderType::SystemdBoot,
            };

            let disk_setup = match &config.disk_setup {
                anna_common::ipc::DiskSetupData::Manual {
                    root_partition,
                    boot_partition,
                    swap_partition,
                } => DiskSetupMode::Manual {
                    root_partition: root_partition.clone(),
                    boot_partition: boot_partition.clone(),
                    swap_partition: swap_partition.clone(),
                },
                anna_common::ipc::DiskSetupData::AutoBtrfs {
                    target_disk,
                    create_swap,
                    swap_size_gb,
                } => DiskSetupMode::AutoBtrfs {
                    target_disk: target_disk.clone(),
                    create_swap: *create_swap,
                    swap_size_gb: *swap_size_gb,
                },
            };

            let install_config = InstallConfig {
                disk_setup,
                bootloader,
                hostname: config.hostname.clone(),
                username: config.username.clone(),
                timezone: config.timezone.clone(),
                locale: config.locale.clone(),
                extra_packages: config.extra_packages.clone(),
                enable_multilib: false,
            };

            // Perform installation
            let result = match crate::install::perform_installation(&install_config, dry_run).await
            {
                Ok(r) => r,
                Err(e) => {
                    error!("Installation failed: {}", e);
                    return Response {
                        id,
                        result: Err(format!("Installation failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // Convert to IPC response
            let steps: Vec<anna_common::ipc::InstallStepData> = result
                .steps
                .into_iter()
                .map(|step| anna_common::ipc::InstallStepData {
                    name: step.name,
                    description: step.description,
                    success: step.success,
                    details: step.details,
                    citation: step.citation,
                })
                .collect();

            let data = anna_common::ipc::InstallResultData {
                dry_run,
                success: result.success,
                steps,
                message: result.message,
                citation: result.citation,
            };

            Ok(ResponseData::InstallResult(data))
        }

        Method::SystemHealth => {
            info!("SystemHealth method called");

            // Perform health check
            let report = match crate::steward::check_system_health().await {
                Ok(r) => r,
                Err(e) => {
                    error!("Health check failed: {}", e);
                    return Response {
                        id,
                        result: Err(format!("Health check failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // Log to steward log
            let log_entry = crate::steward::logging::StewardLogEntry::new(
                "health".to_string(),
                report.overall_status != crate::steward::HealthStatus::Critical,
                report.message.clone(),
                report
                    .services
                    .iter()
                    .filter(|s| s.state == "failed")
                    .map(|s| s.name.clone())
                    .collect(),
                report.citation.clone(),
            );
            let _ = log_entry.write().await;

            // Convert to IPC response
            let services: Vec<anna_common::ipc::ServiceStatusData> = report
                .services
                .into_iter()
                .map(|s| anna_common::ipc::ServiceStatusData {
                    name: s.name.clone(),
                    state: s.state.clone(),
                    active: s.state == "active",
                    enabled: s.load != "not-found",
                })
                .collect();

            let packages: Vec<anna_common::ipc::PackageStatusData> = report
                .packages
                .into_iter()
                .map(|p| anna_common::ipc::PackageStatusData {
                    name: p.name,
                    status: if p.update_available {
                        "update-available".to_string()
                    } else {
                        "up-to-date".to_string()
                    },
                    version: p.version,
                    update_available: p.update_available,
                })
                .collect();

            let log_issues: Vec<anna_common::ipc::LogIssueData> = report
                .log_issues
                .into_iter()
                .map(|l| anna_common::ipc::LogIssueData {
                    timestamp: l.first_seen.to_rfc3339(),
                    severity: l.severity,
                    message: l.message,
                    unit: l.source,
                })
                .collect();

            let data = anna_common::ipc::HealthReportData {
                timestamp: report.timestamp.to_rfc3339(),
                overall_status: format!("{:?}", report.overall_status),
                services,
                packages,
                log_issues,
                recommendations: report.recommendations,
                message: report.message,
                citation: report.citation,
            };

            Ok(ResponseData::HealthReport(data))
        }

        Method::BrainAnalysis => {
            info!("BrainAnalysis method called");

            // Perform health check first to get system data
            let health_report = match crate::steward::check_system_health().await {
                Ok(r) => r,
                Err(e) => {
                    error!("Health check failed for brain analysis: {}", e);
                    return Response {
                        id,
                        result: Err(format!("Health check failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // 6.12.0: Refresh system knowledge if needed
            {
                let mut knowledge = state.knowledge.write().await;
                if knowledge.needs_refresh() {
                    info!("Refreshing system knowledge snapshot");
                    if let Err(e) = knowledge.snapshot_now() {
                        warn!("Failed to refresh knowledge: {}", e);
                    }
                }
            }

            // Run sysadmin brain analysis (v6.57.0: stub returns empty)
            let insights = crate::intel::analyze_system_health(&health_report);
            let formatted_output = crate::intel::format_insights(&insights);

            // Count by severity
            let critical_count = insights
                .iter()
                .filter(|i| matches!(i.severity, crate::intel::DiagnosticSeverity::Critical))
                .count();
            let warning_count = insights
                .iter()
                .filter(|i| matches!(i.severity, crate::intel::DiagnosticSeverity::Warning))
                .count();

            // Beta.271: Compute proactive assessment
            let proactive_input = crate::intel::ProactiveInput {
                current_health: &health_report,
                brain_insights: &insights,
                network_monitoring: health_report.network_monitoring.as_ref(),
                previous_assessment: None, // TODO: Load from state in later iteration
                historian_context: None,    // TODO: Populate from historian in later iteration
            };
            let proactive_assessment = crate::intel::compute_proactive_assessment(&proactive_input);
            let proactive_summaries = crate::intel::assessment_to_summaries(&proactive_assessment);

            // Convert to IPC format
            let insights_data: Vec<anna_common::ipc::DiagnosticInsightData> = insights
                .into_iter()
                .map(|i| anna_common::ipc::DiagnosticInsightData {
                    rule_id: i.rule_id,
                    severity: format!("{:?}", i.severity).to_lowercase(),
                    summary: i.summary,
                    details: i.details,
                    commands: i.commands,
                    citations: i.citations,
                    evidence: i.evidence,
                })
                .collect();

            // Convert proactive summaries to IPC format
            let proactive_issues_data: Vec<anna_common::ipc::ProactiveIssueSummaryData> =
                proactive_summaries
                    .into_iter()
                    .map(|s| anna_common::ipc::ProactiveIssueSummaryData {
                        root_cause: s.root_cause,
                        severity: s.severity,
                        summary: s.summary,
                        rule_id: s.rule_id,
                        confidence: s.confidence,
                        first_seen: s.first_seen,
                        last_seen: s.last_seen,
                        suggested_fix: None, // 6.8.1: Will be populated by planner later
                    })
                    .collect();

            let data = anna_common::ipc::BrainAnalysisData {
                timestamp: chrono::Utc::now().to_rfc3339(),
                insights: insights_data,
                formatted_output,
                critical_count,
                warning_count,
                proactive_issues: proactive_issues_data,
                proactive_health_score: proactive_assessment.health_score,
            };

            Ok(ResponseData::BrainAnalysis(data))
        }

        Method::SystemUpdate { dry_run } => {
            info!("SystemUpdate method called: dry_run={}", dry_run);

            // Perform update
            let report = match crate::steward::perform_system_update(dry_run).await {
                Ok(r) => r,
                Err(e) => {
                    error!("System update failed: {}", e);
                    return Response {
                        id,
                        result: Err(format!("System update failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // Log to steward log
            let log_entry = crate::steward::logging::StewardLogEntry::new(
                "update".to_string(),
                report.success,
                report.message.clone(),
                report
                    .packages_updated
                    .iter()
                    .map(|p| p.name.clone())
                    .collect(),
                report.citation.clone(),
            );
            let _ = log_entry.write().await;

            // Convert to IPC response
            let packages_updated: Vec<anna_common::ipc::PackageUpdateData> = report
                .packages_updated
                .into_iter()
                .map(|p| anna_common::ipc::PackageUpdateData {
                    name: p.name,
                    old_version: p.old_version,
                    new_version: p.new_version,
                    size_change: p.size_change,
                })
                .collect();

            let data = anna_common::ipc::UpdateReportData {
                timestamp: report.timestamp.to_rfc3339(),
                dry_run: report.dry_run,
                success: report.success,
                packages_updated,
                services_restarted: report.services_restarted,
                snapshot_path: report.snapshot_path,
                message: report.message,
                citation: report.citation,
            };

            Ok(ResponseData::UpdateReport(data))
        }

        Method::SystemAudit => {
            info!("SystemAudit method called");

            // Perform audit
            let report = match crate::steward::perform_system_audit().await {
                Ok(r) => r,
                Err(e) => {
                    error!("System audit failed: {}", e);
                    return Response {
                        id,
                        result: Err(format!("System audit failed: {}", e)),
                        version: state.version.clone(),
                    };
                }
            };

            // Log to steward log
            let log_entry = crate::steward::logging::StewardLogEntry::new(
                "audit".to_string(),
                report.compliant,
                report.message.clone(),
                report
                    .integrity
                    .iter()
                    .filter(|i| !i.passed)
                    .map(|i| i.component.clone())
                    .collect(),
                report.citation.clone(),
            );
            let _ = log_entry.write().await;

            // Convert to IPC response
            let integrity: Vec<anna_common::ipc::IntegrityStatusData> = report
                .integrity
                .into_iter()
                .map(|i| anna_common::ipc::IntegrityStatusData {
                    component: i.component,
                    check_type: i.check_type,
                    passed: i.passed,
                    details: i.details,
                })
                .collect();

            let security_findings: Vec<anna_common::ipc::SecurityFindingData> = report
                .security_findings
                .into_iter()
                .map(|f| anna_common::ipc::SecurityFindingData {
                    severity: f.severity,
                    description: f.description,
                    recommendation: f.recommendation,
                    reference: f.reference,
                })
                .collect();

            let config_issues: Vec<anna_common::ipc::ConfigIssueData> = report
                .config_issues
                .into_iter()
                .map(|c| anna_common::ipc::ConfigIssueData {
                    file: c.file,
                    issue: c.issue,
                    expected: c.expected,
                    actual: c.actual,
                })
                .collect();

            let data = anna_common::ipc::AuditReportData {
                timestamp: report.timestamp.to_rfc3339(),
                compliant: report.compliant,
                integrity,
                security_findings,
                config_issues,
                message: report.message,
                citation: report.citation,
            };

            Ok(ResponseData::AuditReport(data))
        }

        Method::SentinelStatus => {
            info!("SentinelStatus method called");

            // Get sentinel state
            let enabled = crate::sentinel::is_enabled().await;
            let state_snapshot = crate::sentinel::load_state().await.unwrap_or_default();
            let config = crate::sentinel::load_config().await.unwrap_or_default();

            let data = anna_common::ipc::SentinelStatusData {
                enabled,
                autonomous_mode: config.autonomous_mode,
                uptime_seconds: state_snapshot.uptime_seconds,
                system_state: state_snapshot.system_state,
                last_health_status: state_snapshot.last_health.status,
                last_health_check: Some(state_snapshot.last_health.timestamp.to_rfc3339()),
                last_update_scan: state_snapshot.last_update_check.map(|t| t.to_rfc3339()),
                last_audit: state_snapshot.last_audit.map(|t| t.to_rfc3339()),
                error_rate: state_snapshot.error_rate,
                drift_index: state_snapshot.drift_index,
            };

            Ok(ResponseData::SentinelStatus(data))
        }

        Method::SentinelMetrics => {
            info!("SentinelMetrics method called");

            // Get sentinel state for metrics
            let state_snapshot = crate::sentinel::load_state().await.unwrap_or_default();

            let total_events: u64 = state_snapshot.event_counters.values().sum();
            let manual_commands = state_snapshot
                .event_counters
                .get("ManualCommand")
                .copied()
                .unwrap_or(0);

            let data = anna_common::ipc::SentinelMetricsData {
                uptime_seconds: state_snapshot.uptime_seconds,
                total_events,
                automated_actions: state_snapshot
                    .event_counters
                    .get("AutoRepair")
                    .copied()
                    .unwrap_or(0)
                    + state_snapshot
                        .event_counters
                        .get("AutoUpdate")
                        .copied()
                        .unwrap_or(0),
                manual_commands,
                health_checks: state_snapshot
                    .event_counters
                    .get("HealthCheck")
                    .copied()
                    .unwrap_or(0),
                update_scans: state_snapshot
                    .event_counters
                    .get("UpdateScan")
                    .copied()
                    .unwrap_or(0),
                audits: state_snapshot
                    .event_counters
                    .get("Audit")
                    .copied()
                    .unwrap_or(0),
                current_health: state_snapshot.last_health.status,
                error_rate: state_snapshot.error_rate,
                drift_index: state_snapshot.drift_index,
            };

            Ok(ResponseData::SentinelMetrics(data))
        }

        Method::SentinelGetConfig => {
            info!("SentinelGetConfig method called");

            // Load current configuration
            let config = crate::sentinel::load_config().await.unwrap_or_default();

            let data = anna_common::ipc::SentinelConfigData {
                autonomous_mode: config.autonomous_mode,
                health_check_interval: config.health_check_interval,
                update_scan_interval: config.update_scan_interval,
                audit_interval: config.audit_interval,
                auto_repair_services: config.auto_repair_services,
                auto_update: config.auto_update,
                auto_update_threshold: config.auto_update_threshold,
                adaptive_scheduling: config.adaptive_scheduling,
            };

            Ok(ResponseData::SentinelConfig(data))
        }

        Method::SentinelSetConfig { config } => {
            info!("SentinelSetConfig method called");

            // Convert IPC config to internal config
            let new_config = crate::sentinel::types::SentinelConfig {
                autonomous_mode: config.autonomous_mode,
                health_check_interval: config.health_check_interval,
                update_scan_interval: config.update_scan_interval,
                audit_interval: config.audit_interval,
                auto_repair_services: config.auto_repair_services,
                auto_update: config.auto_update,
                auto_update_threshold: config.auto_update_threshold,
                adaptive_scheduling: config.adaptive_scheduling,
            };

            // Save configuration
            if let Err(e) = crate::sentinel::save_config(&new_config).await {
                error!("Failed to save sentinel configuration: {}", e);
                return Response {
                    id,
                    result: Err(format!("Failed to save configuration: {}", e)),
                    version: state.version.clone(),
                };
            }

            // Return updated config
            let data = anna_common::ipc::SentinelConfigData {
                autonomous_mode: new_config.autonomous_mode,
                health_check_interval: new_config.health_check_interval,
                update_scan_interval: new_config.update_scan_interval,
                audit_interval: new_config.audit_interval,
                auto_repair_services: new_config.auto_repair_services,
                auto_update: new_config.auto_update,
                auto_update_threshold: new_config.auto_update_threshold,
                adaptive_scheduling: new_config.adaptive_scheduling,
            };

            Ok(ResponseData::SentinelConfig(data))
        }

        // Phase 1.1: Conscience commands
        Method::ConscienceReview => {
            info!("ConscienceReview method called");

            // Get conscience from sentinel
            let conscience = match &state.sentinel {
                Some(s) => s.get_conscience(),
                None => None,
            };

            match conscience {
                Some(c) => {
                    let state = c.get_state().await;
                    let pending_actions: Vec<anna_common::ipc::PendingActionData> = state
                        .pending_actions
                        .iter()
                        .map(|a| anna_common::ipc::PendingActionData {
                            id: a.id.clone(),
                            timestamp: a.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            action: format!("{:?}", a.action),
                            flag_reason: a.flag_reason.clone(),
                            uncertainty: a.uncertainty,
                            ethical_score: a.ethical_score.overall(),
                            weakest_dimension: a.ethical_score.weakest_dimension().to_string(),
                        })
                        .collect();

                    let data = anna_common::ipc::ConsciencePendingData { pending_actions };
                    Ok(ResponseData::ConsciencePending(data))
                }
                None => Err("Conscience layer not initialized".to_string()),
            }
        }

        Method::ConscienceExplain { decision_id } => {
            info!("ConscienceExplain method called: {}", decision_id);

            let conscience = match &state.sentinel {
                Some(s) => s.get_conscience(),
                None => None,
            };

            match conscience {
                Some(c) => match c.get_decision(&decision_id).await {
                    Some(decision) => {
                        let data = anna_common::ipc::ConscienceDecisionData {
                            id: decision.id.clone(),
                            timestamp: decision
                                .timestamp
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            action: format!("{:?}", decision.action),
                            outcome: format!("{:?}", decision.outcome),
                            ethical_score: decision.ethical_score.overall(),
                            safety: decision.ethical_score.safety,
                            privacy: decision.ethical_score.privacy,
                            integrity: decision.ethical_score.integrity,
                            autonomy: decision.ethical_score.autonomy,
                            confidence: decision.confidence,
                            reasoning: crate::conscience::format_reasoning_tree(
                                &decision.reasoning,
                                0,
                            ),
                            has_rollback_plan: decision.rollback_plan.is_some(),
                        };
                        Ok(ResponseData::ConscienceDecision(data))
                    }
                    None => Err(format!("Decision not found: {}", decision_id)),
                },
                None => Err("Conscience layer not initialized".to_string()),
            }
        }

        Method::ConscienceApprove { decision_id } => {
            info!("ConscienceApprove method called: {}", decision_id);

            let conscience = match &state.sentinel {
                Some(s) => s.get_conscience(),
                None => None,
            };

            match conscience {
                Some(c) => match c.approve_action(&decision_id).await {
                    Ok(_) => Ok(ResponseData::ConscienceActionResult(format!(
                        "Action {} approved",
                        decision_id
                    ))),
                    Err(e) => Err(format!("Failed to approve action: {}", e)),
                },
                None => Err("Conscience layer not initialized".to_string()),
            }
        }

        Method::ConscienceReject { decision_id } => {
            info!("ConscienceReject method called: {}", decision_id);

            let conscience = match &state.sentinel {
                Some(s) => s.get_conscience(),
                None => None,
            };

            match conscience {
                Some(c) => match c.reject_action(&decision_id).await {
                    Ok(_) => Ok(ResponseData::ConscienceActionResult(format!(
                        "Action {} rejected",
                        decision_id
                    ))),
                    Err(e) => Err(format!("Failed to reject action: {}", e)),
                },
                None => Err("Conscience layer not initialized".to_string()),
            }
        }

        Method::ConscienceIntrospect => {
            info!("ConscienceIntrospect method called");

            let conscience = match &state.sentinel {
                Some(s) => s.get_conscience(),
                None => None,
            };

            match conscience {
                Some(c) => match c.introspect().await {
                    Ok(report) => {
                        let data = anna_common::ipc::ConscienceIntrospectionData {
                            timestamp: report.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            period: format!(
                                "{} to {}",
                                report.period_start.format("%H:%M:%S"),
                                report.period_end.format("%H:%M:%S")
                            ),
                            decisions_reviewed: report.decisions_reviewed,
                            approved_count: report.approved_count,
                            rejected_count: report.rejected_count,
                            flagged_count: report.flagged_count,
                            avg_ethical_score: report.avg_ethical_score,
                            avg_confidence: report.avg_confidence,
                            violations_count: report.violations.len() as u64,
                            recommendations: report.recommendations,
                        };
                        Ok(ResponseData::ConscienceIntrospection(data))
                    }
                    Err(e) => Err(format!("Introspection failed: {}", e)),
                },
                None => Err("Conscience layer not initialized".to_string()),
            }
        }

        // Phase 1.2: Empathy commands
        Method::EmpathyPulse => {
            info!("EmpathyPulse method called");

            // Get empathy kernel from sentinel
            let empathy = match &state.sentinel {
                Some(s) => s.get_empathy(),
                None => None,
            };

            match empathy {
                Some(e) => {
                    let pulse = e.get_pulse().await;

                    let data = anna_common::ipc::EmpathyPulseData {
                        timestamp: pulse.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                        empathy_index: pulse.empathy_index,
                        strain_index: pulse.strain_index,
                        resonance_map: anna_common::ipc::ResonanceMapData {
                            user_resonance: pulse.resonance_map.user_resonance,
                            system_resonance: pulse.resonance_map.system_resonance,
                            environment_resonance: pulse.resonance_map.environment_resonance,
                            recent_adjustments: pulse
                                .resonance_map
                                .recent_adjustments
                                .iter()
                                .map(|adj| anna_common::ipc::ResonanceAdjustmentData {
                                    timestamp: adj
                                        .timestamp
                                        .format("%Y-%m-%d %H:%M:%S UTC")
                                        .to_string(),
                                    stakeholder: adj.stakeholder.clone(),
                                    delta: adj.delta,
                                    reason: adj.reason.clone(),
                                })
                                .collect(),
                        },
                        context_summary: pulse.context_summary,
                        recent_perceptions: pulse
                            .recent_perceptions
                            .iter()
                            .map(|p| anna_common::ipc::PerceptionRecordData {
                                timestamp: p.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                                action: format!("{:?}", p.action),
                                stakeholder_impacts: anna_common::ipc::StakeholderImpactsData {
                                    user: anna_common::ipc::StakeholderImpactData {
                                        score: p.stakeholder_impacts.user.score,
                                        impact_type: p.stakeholder_impacts.user.impact_type.clone(),
                                        reasoning: p.stakeholder_impacts.user.reasoning.clone(),
                                    },
                                    system: anna_common::ipc::StakeholderImpactData {
                                        score: p.stakeholder_impacts.system.score,
                                        impact_type: p
                                            .stakeholder_impacts
                                            .system
                                            .impact_type
                                            .clone(),
                                        reasoning: p.stakeholder_impacts.system.reasoning.clone(),
                                    },
                                    environment: anna_common::ipc::StakeholderImpactData {
                                        score: p.stakeholder_impacts.environment.score,
                                        impact_type: p
                                            .stakeholder_impacts
                                            .environment
                                            .impact_type
                                            .clone(),
                                        reasoning: p
                                            .stakeholder_impacts
                                            .environment
                                            .reasoning
                                            .clone(),
                                    },
                                },
                                context_factors: p.context_factors.clone(),
                                adaptation: p.adaptation.clone(),
                            })
                            .collect(),
                    };

                    Ok(ResponseData::EmpathyPulse(data))
                }
                None => Err("Empathy kernel not initialized".to_string()),
            }
        }

        Method::EmpathySimulate { action } => {
            info!("EmpathySimulate method called for action: {}", action);

            // Get empathy kernel from sentinel
            let empathy = match &state.sentinel {
                Some(s) => s.get_empathy(),
                None => None,
            };

            match empathy {
                Some(e) => {
                    // Parse action string to SentinelAction
                    let parsed_action = match action.as_str() {
                        "SystemUpdate" => {
                            crate::sentinel::SentinelAction::SystemUpdate { dry_run: false }
                        }
                        "RestartService" => crate::sentinel::SentinelAction::RestartService {
                            service: "example-service".to_string(),
                        },
                        "SyncDatabases" => crate::sentinel::SentinelAction::SyncDatabases,
                        _ => crate::sentinel::SentinelAction::None,
                    };

                    match e.simulate(&parsed_action).await {
                        Ok(simulation) => {
                            let data = anna_common::ipc::EmpathySimulationData {
                                action: simulation.action,
                                evaluation: anna_common::ipc::EmpathyEvaluationData {
                                    should_defer: simulation.evaluation.should_defer,
                                    deferral_reason: simulation.evaluation.deferral_reason,
                                    stakeholder_impacts: anna_common::ipc::StakeholderImpactsData {
                                        user: anna_common::ipc::StakeholderImpactData {
                                            score: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .user
                                                .score,
                                            impact_type: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .user
                                                .impact_type,
                                            reasoning: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .user
                                                .reasoning,
                                        },
                                        system: anna_common::ipc::StakeholderImpactData {
                                            score: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .system
                                                .score,
                                            impact_type: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .system
                                                .impact_type,
                                            reasoning: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .system
                                                .reasoning,
                                        },
                                        environment: anna_common::ipc::StakeholderImpactData {
                                            score: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .environment
                                                .score,
                                            impact_type: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .environment
                                                .impact_type,
                                            reasoning: simulation
                                                .evaluation
                                                .stakeholder_impacts
                                                .environment
                                                .reasoning,
                                        },
                                    },
                                    context_factors: simulation.evaluation.context_factors,
                                    recommended_delay: simulation.evaluation.recommended_delay,
                                    tone_adaptation: simulation.evaluation.tone_adaptation,
                                },
                                reasoning: simulation.reasoning,
                                would_proceed: simulation.would_proceed,
                            };
                            Ok(ResponseData::EmpathySimulation(data))
                        }
                        Err(e) => Err(format!("Simulation failed: {}", e)),
                    }
                }
                None => Err("Empathy kernel not initialized".to_string()),
            }
        }

        // Phase 1.3: Collective mind commands
        Method::CollectiveStatus => {
            info!("CollectiveStatus method called");

            match &state.collective {
                Some(collective) => {
                    let status = collective.get_status().await;

                    let data = anna_common::ipc::CollectiveStatusData {
                        enabled: status.enabled,
                        node_id: status.node_id,
                        connected_peers: status.connected_peers,
                        total_peers: status.total_peers,
                        avg_network_empathy: status.avg_network_empathy,
                        avg_network_strain: status.avg_network_strain,
                        recent_decisions: status.recent_decisions,
                        network_health: status.network_health,
                    };

                    Ok(ResponseData::CollectiveStatus(data))
                }
                None => {
                    // Return disabled state if collective not initialized
                    let data = anna_common::ipc::CollectiveStatusData {
                        enabled: false,
                        node_id: "not_initialized".to_string(),
                        connected_peers: 0,
                        total_peers: 0,
                        avg_network_empathy: 0.0,
                        avg_network_strain: 0.0,
                        recent_decisions: 0,
                        network_health: 0.0,
                    };

                    Ok(ResponseData::CollectiveStatus(data))
                }
            }
        }

        Method::CollectiveTrust { peer_id } => {
            info!("CollectiveTrust method called for peer: {}", peer_id);

            match &state.collective {
                Some(collective) => match collective.get_peer_trust(&peer_id).await {
                    Some(trust_details) => {
                        let data = anna_common::ipc::CollectiveTrustData {
                            peer_id: trust_details.peer_info.id.clone(),
                            peer_name: trust_details.peer_info.name.clone(),
                            peer_address: trust_details.peer_info.address.to_string(),
                            overall_trust: trust_details.trust_score.overall,
                            honesty: trust_details.trust_score.honesty,
                            reliability: trust_details.trust_score.reliability,
                            ethical_alignment: trust_details.trust_score.ethical_alignment,
                            messages_received: trust_details.recent_messages,
                            messages_validated: trust_details.trust_score.messages_validated,
                            last_interaction: trust_details
                                .last_interaction
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            connected: trust_details.peer_info.connected,
                        };

                        Ok(ResponseData::CollectiveTrust(data))
                    }
                    None => Err(format!("Peer {} not found in network", peer_id)),
                },
                None => Err("Collective mind not initialized".to_string()),
            }
        }

        Method::CollectiveExplain { consensus_id } => {
            info!(
                "CollectiveExplain method called for consensus: {}",
                consensus_id
            );

            match &state.collective {
                Some(collective) => {
                    match collective.get_consensus_explanation(&consensus_id).await {
                        Some(explanation) => {
                            let votes_data: Vec<anna_common::ipc::ConsensusVoteData> = explanation
                                .record
                                .votes
                                .values()
                                .map(|vote| {
                                    anna_common::ipc::ConsensusVoteData {
                                        peer_id: vote.peer_id.clone(),
                                        vote: format!("{:?}", vote.vote),
                                        weight: vote.weight,
                                        ethical_score: vote.ethical_score,
                                        reasoning: vote.reasoning.clone(),
                                        trust_score: 0.0, // Will be filled with actual trust score
                                    }
                                })
                                .collect();

                            let data = anna_common::ipc::CollectiveExplanationData {
                                consensus_id: explanation.record.id.clone(),
                                action: explanation.record.action.clone(),
                                decision: format!("{:?}", explanation.record.decision),
                                timestamp: explanation
                                    .record
                                    .timestamp
                                    .format("%Y-%m-%d %H:%M:%S UTC")
                                    .to_string(),
                                votes: votes_data,
                                total_participants: explanation.record.votes.len(),
                                approval_percentage: explanation.approval_percentage,
                                weighted_approval: explanation.weighted_approval,
                                reasoning: explanation.reasoning_trail,
                            };

                            Ok(ResponseData::CollectiveExplanation(data))
                        }
                        None => Err(format!("Consensus decision {} not found", consensus_id)),
                    }
                }
                None => Err("Collective mind not initialized".to_string()),
            }
        }

        // Phase 1.4: Mirror protocol commands
        Method::MirrorReflect => {
            info!("MirrorReflect method called");

            match &state.mirror {
                Some(mirror) => match mirror.generate_reflection().await {
                    Ok(reflection) => {
                        let data = anna_common::ipc::MirrorReflectionData {
                            reflection_id: reflection.id,
                            timestamp: reflection
                                .timestamp
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            period_start: reflection
                                .period_start
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            period_end: reflection
                                .period_end
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            self_coherence: reflection.self_coherence,
                            ethical_decisions_count: reflection.ethical_decisions.len(),
                            conscience_actions_count: reflection.conscience_actions.len(),
                            avg_empathy_index: reflection.empathy_summary.avg_empathy_index,
                            avg_strain_index: reflection.empathy_summary.avg_strain_index,
                            empathy_trend: reflection.empathy_summary.empathy_trend,
                            adaptations_count: reflection.empathy_summary.adaptations_count,
                            self_identified_biases: reflection.self_identified_biases,
                        };

                        Ok(ResponseData::MirrorReflection(data))
                    }
                    Err(e) => Err(format!("Reflection generation failed: {}", e)),
                },
                None => Err("Mirror protocol not initialized".to_string()),
            }
        }

        Method::MirrorAudit => {
            info!("MirrorAudit method called");

            match &state.mirror {
                Some(mirror) => match mirror.get_audit_summary().await {
                    Ok(audit) => {
                        let data = anna_common::ipc::MirrorAuditData {
                            enabled: audit.enabled,
                            current_coherence: audit.current_coherence,
                            last_reflection: audit
                                .last_reflection
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                            last_consensus: audit
                                .last_consensus
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                            recent_reflections_count: audit.recent_reflections_count,
                            received_critiques_count: audit.received_critiques_count,
                            active_remediations_count: audit.active_remediations_count,
                            network_coherence: audit.network_coherence,
                            recent_critiques: audit
                                .recent_critiques
                                .into_iter()
                                .map(|c| anna_common::ipc::CritiqueSummary {
                                    critic_id: c.critic_id,
                                    coherence_assessment: c.coherence_assessment,
                                    inconsistencies_count: c.inconsistencies_count,
                                    biases_count: c.biases_count,
                                    recommendations: c.recommendations,
                                })
                                .collect(),
                        };

                        Ok(ResponseData::MirrorAudit(data))
                    }
                    Err(e) => Err(format!("Audit generation failed: {}", e)),
                },
                None => Err("Mirror protocol not initialized".to_string()),
            }
        }

        Method::MirrorRepair => {
            info!("MirrorRepair method called");

            match &state.mirror {
                Some(mirror) => match mirror.apply_pending_remediations().await {
                    Ok(report) => {
                        let data = anna_common::ipc::MirrorRepairData {
                            timestamp: report.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            total_remediations: report.total_remediations,
                            successful_remediations: report.successful_remediations,
                            failed_remediations: report.failed_remediations,
                            summary: report.summary.clone(),
                            applied_remediations: report
                                .details
                                .into_iter()
                                .filter_map(|r| {
                                    if r.applied {
                                        Some(anna_common::ipc::RemediationSummary {
                                            description: format!("Action {} applied", r.action_id),
                                            remediation_type: "ParameterAdjustment".to_string(),
                                            expected_impact: r.reason.clone(),
                                            parameter_adjustments: r.adjustments_made,
                                        })
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        };

                        Ok(ResponseData::MirrorRepair(data))
                    }
                    Err(e) => Err(format!("Remediation failed: {}", e)),
                },
                None => Err("Mirror protocol not initialized".to_string()),
            }
        }

        // Phase 1.5: Chronos loop commands
        Method::ChronosForecast { window_hours } => {
            info!("ChronosForecast method called for {} hours", window_hours);

            match &state.chronos {
                Some(chronos) => {
                    match chronos.generate_forecast(window_hours).await {
                        Ok(result) => {
                            let forecast = &result.forecast;
                            let projection = &result.projection;

                            // Get final projected state
                            let final_state = forecast
                                .consensus_scenario
                                .as_ref()
                                .and_then(|s| s.snapshots.last());

                            let (final_health, final_empathy, final_strain, final_coherence) =
                                if let Some(state) = final_state {
                                    (
                                        state.metrics.health_score,
                                        state.metrics.empathy_index,
                                        state.metrics.strain_index,
                                        state.metrics.network_coherence,
                                    )
                                } else {
                                    (0.0, 0.0, 0.0, 0.0)
                                };

                            // Map stakeholder impacts
                            let stakeholder_impacts: std::collections::HashMap<String, f64> =
                                projection
                                    .stakeholder_impacts
                                    .iter()
                                    .map(|(k, v)| (k.clone(), v.impact_score))
                                    .collect();

                            let trajectory_str = format!("{:?}", projection.ethical_trajectory);

                            let data = anna_common::ipc::ChronosForecastData {
                                forecast_id: forecast.forecast_id.clone(),
                                generated_at: forecast
                                    .generated_at
                                    .format("%Y-%m-%d %H:%M:%S UTC")
                                    .to_string(),
                                horizon_hours: forecast.horizon_hours,
                                confidence: forecast.confidence,
                                final_health,
                                final_empathy,
                                final_strain,
                                final_coherence,
                                temporal_empathy_index: projection.temporal_empathy_index,
                                moral_cost: projection.moral_cost,
                                ethical_trajectory: trajectory_str,
                                stakeholder_impacts,
                                divergence_warnings: forecast.divergence_warnings.clone(),
                                recommendations: projection.intervention_recommendations.clone(),
                                archive_hash: result.archive_hash,
                            };

                            Ok(ResponseData::ChronosForecast(data))
                        }
                        Err(e) => Err(format!("Forecast generation failed: {}", e)),
                    }
                }
                None => Err("Chronos loop not initialized".to_string()),
            }
        }

        Method::ChronosAudit => {
            info!("ChronosAudit method called");

            match &state.chronos {
                Some(chronos) => {
                    let summary = chronos.get_audit_summary().await;

                    let recent_forecasts: Vec<anna_common::ipc::ForecastSummary> = summary
                        .recent_forecasts
                        .iter()
                        .map(|f| anna_common::ipc::ForecastSummary {
                            forecast_id: f.forecast_id.clone(),
                            generated_at: f
                                .generated_at
                                .format("%Y-%m-%d %H:%M:%S UTC")
                                .to_string(),
                            horizon_hours: f.horizon_hours,
                            confidence: f.confidence,
                            warnings_count: f.warnings_count,
                            moral_cost: f.moral_cost,
                        })
                        .collect();

                    let data = anna_common::ipc::ChronosAuditData {
                        total_archived: summary.total_archived,
                        recent_forecasts,
                    };

                    Ok(ResponseData::ChronosAudit(data))
                }
                None => Err("Chronos loop not initialized".to_string()),
            }
        }

        Method::ChronosAlign => {
            info!("ChronosAlign method called");

            match &state.chronos {
                Some(chronos) => match chronos.align_parameters().await {
                    Ok(()) => {
                        let data = anna_common::ipc::ChronosAlignData {
                            status: "Parameters aligned successfully".to_string(),
                            parameters_aligned: 0,
                            parameter_changes: std::collections::HashMap::new(),
                        };

                        Ok(ResponseData::ChronosAlign(data))
                    }
                    Err(e) => Err(format!("Parameter alignment failed: {}", e)),
                },
                None => Err("Chronos loop not initialized".to_string()),
            }
        }

        Method::MirrorAuditForecast { window_hours } => {
            info!("MirrorAuditForecast method called");

            let window = window_hours.unwrap_or(24);

            match &state.mirror_audit {
                Some(audit_lock) => {
                    let audit = audit_lock.read().await;
                    let summary = audit.get_summary();

                    // Convert types to IPC data structures
                    let active_biases: Vec<anna_common::ipc::BiasFindingData> = summary
                        .active_biases
                        .iter()
                        .map(|b| anna_common::ipc::BiasFindingData {
                            kind: format!("{:?}", b.kind),
                            confidence: b.confidence,
                            evidence: b.evidence.clone(),
                            magnitude: b.magnitude,
                            sample_size: b.sample_size,
                        })
                        .collect();

                    let pending_adjustments: Vec<anna_common::ipc::AdjustmentPlanData> = summary
                        .pending_adjustments
                        .iter()
                        .map(|p| anna_common::ipc::AdjustmentPlanData {
                            plan_id: p.plan_id.clone(),
                            created_at: p.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            target: format!("{:?}", p.target),
                            adjustments: p
                                .adjustments
                                .iter()
                                .map(|a| anna_common::ipc::ParameterAdjustmentData {
                                    parameter: a.parameter.clone(),
                                    current_value: a.current_value,
                                    recommended_value: a.recommended_value,
                                    reason: a.reason.clone(),
                                })
                                .collect(),
                            expected_improvement: p.expected_improvement,
                            rationale: p.rationale.clone(),
                        })
                        .collect();

                    let data = anna_common::ipc::MirrorAuditTemporalData {
                        total_audits: summary.total_audits,
                        last_audit_at: summary
                            .last_audit_at
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()),
                        average_temporal_integrity: summary.average_temporal_integrity,
                        active_biases,
                        pending_adjustments,
                    };

                    Ok(ResponseData::MirrorAuditForecast(data))
                }
                None => Err("Mirror audit not initialized".to_string()),
            }
        }

        Method::MirrorReflectTemporal { window_hours } => {
            info!("MirrorReflectTemporal method called");

            let window = window_hours.unwrap_or(24);

            match &state.mirror_audit {
                Some(audit_lock) => {
                    let audit = audit_lock.read().await;
                    let summary = audit.get_summary();

                    // Generate reflection summary
                    let biases_count = summary.active_biases.len();
                    let avg_score = summary.average_temporal_integrity.unwrap_or(0.5);

                    let reflection_summary = if biases_count > 0 {
                        format!(
                            "Temporal self-reflection reveals {} systematic bias pattern(s) with average integrity {:.1}%. \
                             Recommended adjustments have been generated for review.",
                            biases_count,
                            avg_score * 100.0
                        )
                    } else {
                        format!(
                            "Temporal self-reflection shows healthy forecast accuracy with {:.1}% integrity. \
                             No systematic biases detected.",
                            avg_score * 100.0
                        )
                    };

                    // Convert biases
                    let biases_detected: Vec<anna_common::ipc::BiasFindingData> = summary
                        .active_biases
                        .iter()
                        .map(|b| anna_common::ipc::BiasFindingData {
                            kind: format!("{:?}", b.kind),
                            confidence: b.confidence,
                            evidence: b.evidence.clone(),
                            magnitude: b.magnitude,
                            sample_size: b.sample_size,
                        })
                        .collect();

                    // Get first adjustment plan if available
                    let recommended_adjustments = summary.pending_adjustments.first().map(|p| {
                        anna_common::ipc::AdjustmentPlanData {
                            plan_id: p.plan_id.clone(),
                            created_at: p.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                            target: format!("{:?}", p.target),
                            adjustments: p
                                .adjustments
                                .iter()
                                .map(|a| anna_common::ipc::ParameterAdjustmentData {
                                    parameter: a.parameter.clone(),
                                    current_value: a.current_value,
                                    recommended_value: a.recommended_value,
                                    reason: a.reason.clone(),
                                })
                                .collect(),
                            expected_improvement: p.expected_improvement,
                            rationale: p.rationale.clone(),
                        }
                    });

                    let data = anna_common::ipc::MirrorReflectTemporalData {
                        reflection_id: uuid::Uuid::new_v4().to_string(),
                        generated_at: chrono::Utc::now()
                            .format("%Y-%m-%d %H:%M:%S UTC")
                            .to_string(),
                        window_hours: window,
                        temporal_integrity_score: avg_score,
                        biases_detected,
                        recommended_adjustments,
                        summary: reflection_summary,
                    };

                    Ok(ResponseData::MirrorReflectTemporal(data))
                }
                None => Err("Mirror audit not initialized".to_string()),
            }
        }

        // 6.7.0: Reflection endpoint (not yet implemented in daemon)
        Method::GetReflection => {
            // For now, return empty reflection - client builds it locally
            Ok(ResponseData::Ok)
        }

        Method::GetSystemKnowledge => {
            // 6.12.0: Return system knowledge snapshot
            let knowledge = state.knowledge.read().await;
            let kb = knowledge.get_cached();
            let data = kb.to_rpc_data();
            Ok(ResponseData::SystemKnowledge(data))
        }

        Method::GetToolchainHealth => {
            // v6.58.0: Return toolchain health from startup self-test
            let health = state.toolchain_health.read().await;
            Ok(ResponseData::ToolchainHealth(health.clone()))
        }
    };

    Response {
        id,
        result,
        version: "1.0.0".to_string(),
    }
}
