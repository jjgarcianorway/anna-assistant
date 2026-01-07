use anna_rpc::{
    decode_request, encode_response, read_message, write_message, AdviceListRequest,
    AdviceListResponse, AdviceShowRequest, AdviceShowResponse, ApplyMode, ApplyRequest,
    ApplyResponse, DoctorPermsRequest, DoctorPermsResponse, ErrorResponse, IssueSeverity,
    PermissionIssue, PersonaRequest, PersonaResponse, PersonaSummaryRequest,
    PersonaSummaryResponse, PersonaTrait, QuickscanRequest, Request, Response, StatusRequest,
    StatusResponse,
};
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::notifications;
use crate::policy::{ActionDecision, Policy};

/// RPC server configuration
pub struct RpcServer {
    socket_path: PathBuf,
    config: Arc<Config>,
}

impl RpcServer {
    pub fn new(socket_path: PathBuf, config: Arc<Config>) -> Self {
        Self {
            socket_path,
            config,
        }
    }

    /// Start the RPC server
    pub fn start(self) -> Result<()> {
        let is_system_mode =
            nix::unistd::Uid::effective().is_root() || self.socket_path.starts_with("/run/anna");

        // Create socket directory if needed
        if let Some(parent) = self.socket_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create socket directory {}", parent.display()))?;

            // Set directory permissions: 0755 for system, 0700 for user
            #[cfg(unix)]
            {
                let dir_perms = if is_system_mode {
                    fs::Permissions::from_mode(0o755)
                } else {
                    fs::Permissions::from_mode(0o700)
                };
                fs::set_permissions(parent, dir_perms).with_context(|| {
                    format!("set socket directory permissions for {}", parent.display())
                })?;
            }
        }

        // Remove stale socket if it exists
        if self.socket_path.exists() {
            fs::remove_file(&self.socket_path)
                .with_context(|| format!("remove stale socket {}", self.socket_path.display()))?;
        }

        // Bind listener
        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("bind socket at {}", self.socket_path.display()))?;

        // Set permissions: 0660 for system mode (group anna), 0600 for user mode
        #[cfg(unix)]
        {
            let socket_perms = if is_system_mode {
                fs::Permissions::from_mode(0o660)
            } else {
                fs::Permissions::from_mode(0o600)
            };
            fs::set_permissions(&self.socket_path, socket_perms).with_context(|| {
                format!("set socket permissions for {}", self.socket_path.display())
            })?;

            // Try to set group ownership to anna if we're root (system mode only)
            if is_system_mode && nix::unistd::Uid::effective().is_root() {
                use nix::unistd::{chown, Gid, Uid};

                // Look up anna group
                if let Ok(Some(group)) = nix::unistd::Group::from_name("anna") {
                    let result = chown(
                        self.socket_path.as_path(),
                        Some(Uid::from_raw(0)), // root
                        Some(Gid::from_raw(group.gid.as_raw())),
                    );
                    if let Err(e) = result {
                        warn!(target: "rpc", "failed to set socket group ownership: {}", e);
                    }
                }
            }
        }

        info!(target: "rpc", "listening on {}", self.socket_path.display());

        // Accept connections
        loop {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    let config = Arc::clone(&self.config);
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, config) {
                            error!(target: "rpc", "connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!(target: "rpc", "accept error: {}", e);
                }
            }
        }
    }
}

/// Handle a single RPC connection
fn handle_connection(mut stream: UnixStream, config: Arc<Config>) -> Result<()> {
    // Read request
    let req_data = read_message(&mut stream).context("read request")?;
    let request = decode_request(&req_data).context("decode request")?;

    info!(target: "rpc", "request: {:?}", request);

    // Process request
    let response = match request {
        Request::Status(req) => handle_status(req, &config),
        Request::Quickscan(req) => handle_quickscan(req, &config),
        Request::AdviceList(req) => handle_advice_list(req, &config),
        Request::AdviceShow(req) => handle_advice_show(req, &config),
        Request::Persona(req) => handle_persona(req, &config),
        Request::PersonaSummary(req) => handle_persona_summary(req, &config),
        Request::Apply(req) => handle_apply(req, &config),
        Request::DoctorPerms(req) => handle_doctor_perms(req, &config),
    };

    // Send response
    let resp_data = encode_response(&response).context("encode response")?;
    write_message(&mut stream, &resp_data).context("write response")?;

    Ok(())
}

/// Handle status request
fn handle_status(req: StatusRequest, _config: &Config) -> Response {
    use self::paths::AnnaPaths;

    let paths = AnnaPaths::detect_for_uid(req.uid);

    // Check service state (simplified - in a full impl this would check systemd)
    let service_state = "active".to_string();

    // Get last quickscan timestamp
    let last_quickscan_ts = get_last_quickscan_ts(&paths.reports_dir);

    // Count advice entries
    let advice_count = count_advice_entries(&paths.advice_dir);

    Response::Status(StatusResponse {
        mode: paths.mode.as_str().to_string(),
        socket_path: paths.socket_path.display().to_string(),
        user_data_dir: paths.data_dir.display().to_string(),
        system_config_dir: paths.config_dir.display().to_string(),
        service_state,
        last_quickscan_ts,
        advice_count,
    })
}

/// Handle quickscan request
fn handle_quickscan(req: QuickscanRequest, config: &Config) -> Response {
    use self::paths::AnnaPaths;
    use std::time::Instant;

    let paths = AnnaPaths::detect_for_uid(req.uid);
    let started_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());
    let start = Instant::now();

    // Convert &Config to Arc<Config> for run_for_user
    let config_arc = Arc::new(config.clone());

    // Clone paths for later use
    let reports_dir = paths.reports_dir.clone();
    let advice_dir = paths.advice_dir.clone();

    // Run quickscan for this user
    let report = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            // We're in a tokio context, use spawn_blocking
            let reports_dir_clone = reports_dir.clone();
            let advice_dir_clone = advice_dir.clone();
            let config_arc_clone = Arc::clone(&config_arc);
            match handle.block_on(async move {
                tokio::task::spawn_blocking(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async move {
                        crate::quickscan::run_for_user(
                            config_arc_clone,
                            &reports_dir_clone,
                            &advice_dir_clone,
                        )
                        .await
                    })
                })
                .await
            }) {
                Ok(Ok(report)) => report,
                Ok(Err(e)) => {
                    return Response::Error(ErrorResponse {
                        code: 1,
                        message: format!("Quickscan failed: {}", e),
                    });
                }
                Err(e) => {
                    return Response::Error(ErrorResponse {
                        code: 1,
                        message: format!("Failed to spawn quickscan task: {}", e),
                    });
                }
            }
        }
        Err(_) => {
            // No tokio runtime, create one
            let rt = tokio::runtime::Runtime::new().unwrap();
            match rt.block_on(crate::quickscan::run_for_user(
                config_arc,
                &reports_dir,
                &advice_dir,
            )) {
                Ok(report) => report,
                Err(e) => {
                    return Response::Error(ErrorResponse {
                        code: 1,
                        message: format!("Quickscan failed: {}", e),
                    });
                }
            }
        }
    };

    let finished_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string());
    let elapsed_secs = start.elapsed().as_secs();

    // Find the latest report path
    let report_path = find_latest_quickscan(&reports_dir)
        .ok()
        .and_then(|opt| opt)
        .map(|(path, _)| path.display().to_string())
        .unwrap_or_else(|| format!("{}/latest/quickscan.json", reports_dir.display()));

    // Count advice seeded (count .json files in advice dir)
    let advice_count_seeded = count_advice_entries(&advice_dir);

    info!(
        target: "rpc",
        "quickscan uid={} ok={} warn={} action={} in {}s",
        req.uid, report.ok, report.warn, report.action, elapsed_secs
    );

    Response::Quickscan(anna_rpc::QuickscanResponse {
        started_at,
        finished_at,
        summary: anna_rpc::QuickscanSummary {
            ok: report.ok,
            warn: report.warn,
            action: report.action,
        },
        report_path,
        advice_count_seeded,
        mode: paths.mode.as_str().to_string(),
    })
}

/// Handle advice list request
fn handle_advice_list(req: AdviceListRequest, _config: &Config) -> Response {
    use self::paths::AnnaPaths;

    let paths = AnnaPaths::detect_for_uid(req.uid);

    match read_advice_entries(&paths.advice_dir) {
        Ok(items) => {
            let ts = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_else(|_| "unknown".to_string());

            Response::AdviceList(AdviceListResponse { items, ts })
        }
        Err(e) => Response::Error(ErrorResponse {
            code: 1,
            message: format!("Failed to read advice: {}", e),
        }),
    }
}

/// Handle advice show request
fn handle_advice_show(req: AdviceShowRequest, _config: &Config) -> Response {
    use self::paths::AnnaPaths;

    let paths = AnnaPaths::detect_for_uid(req.uid);
    let advice_path = paths.advice_dir.join(format!("{}.json", req.id));

    match fs::read_to_string(&advice_path) {
        Ok(content) => match serde_json::from_str::<anna_rpc::AdviceRecord>(&content) {
            Ok(advice) => Response::AdviceShow(AdviceShowResponse {
                advice: Some(advice),
            }),
            Err(e) => Response::Error(ErrorResponse {
                code: 1,
                message: format!("Failed to parse advice: {}", e),
            }),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Response::AdviceShow(AdviceShowResponse { advice: None })
        }
        Err(e) => Response::Error(ErrorResponse {
            code: 1,
            message: format!("Failed to read advice: {}", e),
        }),
    }
}

/// Handle persona request
fn handle_persona(req: PersonaRequest, _config: &Config) -> Response {
    // Stub implementation for now
    let content = match req.op {
        anna_rpc::PersonaOp::Show => {
            "Persona: default\nSource: config\nConfidence: 1.0".to_string()
        }
        anna_rpc::PersonaOp::Explain => "Persona explanation not yet implemented in S1".to_string(),
        anna_rpc::PersonaOp::Samples { .. } => {
            "Persona samples not yet implemented in S1".to_string()
        }
        anna_rpc::PersonaOp::Triggers => "Persona triggers not yet implemented in S1".to_string(),
    };

    Response::Persona(PersonaResponse { content })
}

/// Handle persona summary request (for debug command)
fn handle_persona_summary(req: PersonaSummaryRequest, _config: &Config) -> Response {
    use crate::persona::store::Store;
    use self::paths::AnnaPaths;

    // Get per-user persona directory
    let paths = AnnaPaths::detect_for_uid(req.uid);

    // Read current persona state from per-user storage
    let store = match Store::for_dir(&paths.persona_dir) {
        Ok(s) => s,
        Err(e) => {
            return Response::Error(ErrorResponse {
                code: 1,
                message: format!("Failed to initialize persona store: {}", e),
            });
        }
    };

    match store.read_current() {
        Ok(Some(state)) => {
            // TODO: Implement trait extraction from persona state
            // For now, return basic info with empty traits
            Response::PersonaSummary(PersonaSummaryResponse {
                persona: state.persona.as_str().to_string(),
                source: state.source.as_str().to_string(),
                confidence: state.confidence,
                traits: vec![], // Will be populated in S3.2 phase B
            })
        }
        Ok(None) => Response::PersonaSummary(PersonaSummaryResponse {
            persona: "unknown".to_string(),
            source: "default".to_string(),
            confidence: 0.0,
            traits: vec![],
        }),
        Err(e) => Response::Error(ErrorResponse {
            code: 1,
            message: format!("Failed to read persona: {}", e),
        }),
    }
}

/// Handle apply request
fn handle_apply(req: ApplyRequest, _config: &Config) -> Response {
    use self::paths::AnnaPaths;
    use std::process::Command;

    let paths = AnnaPaths::detect_for_uid(req.uid);

    // Load policy
    let policy = match Policy::load_for_uid(req.uid) {
        Ok(p) => p,
        Err(e) => {
            return Response::Error(ErrorResponse {
                code: 1,
                message: format!("Failed to load policy: {}", e),
            });
        }
    };

    // Read advice entry
    let advice_path = paths.advice_dir.join(format!("{}.json", req.id));
    if !advice_path.exists() {
        return Response::Error(ErrorResponse {
            code: 1,
            message: format!("Advice entry {} not found", req.id),
        });
    }

    let advice_data = match fs::read(&advice_path) {
        Ok(data) => data,
        Err(e) => {
            return Response::Error(ErrorResponse {
                code: 1,
                message: format!("Failed to read advice: {}", e),
            });
        }
    };

    let advice: crate::advice::types::AdviceRecord = match serde_json::from_slice(&advice_data) {
        Ok(a) => a,
        Err(e) => {
            return Response::Error(ErrorResponse {
                code: 1,
                message: format!("Failed to parse advice: {}", e),
            });
        }
    };

    // Dry run mode
    if matches!(req.mode, ApplyMode::DryRun) {
        let preview = if !advice.plan.dry_run_cmds.is_empty() {
            advice.plan.dry_run_cmds.join("\n")
        } else if !advice.plan.apply_cmds.is_empty() {
            format!("Would execute:\n{}", advice.plan.apply_cmds.join("\n"))
        } else {
            "No commands to execute".to_string()
        };

        return Response::Apply(ApplyResponse {
            status: "preview".to_string(),
            message: "Dry run completed".to_string(),
            requires_approval: false,
            output: Some(preview),
        });
    }

    // Check policy
    let decision = policy.allows_action(&advice.kind, &advice.plan.apply_cmds);

    match decision {
        ActionDecision::Allowed => {
            // Execute commands
            let mut output_lines = Vec::new();
            let mut all_success = true;

            for cmd in &advice.plan.apply_cmds {
                // Parse command (simple shell splitting)
                let parts: Vec<&str> = cmd.split_whitespace().collect();
                if parts.is_empty() {
                    continue;
                }

                let result = Command::new(parts[0]).args(&parts[1..]).output();

                match result {
                    Ok(output) => {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        if output.status.success() {
                            output_lines.push(format!("✓ {}: OK", cmd));
                            if !stdout.is_empty() {
                                output_lines.push(format!("  {}", stdout.trim()));
                            }
                        } else {
                            all_success = false;
                            output_lines.push(format!("✗ {}: FAILED", cmd));
                            if !stderr.is_empty() {
                                output_lines.push(format!("  {}", stderr.trim()));
                            }
                        }
                    }
                    Err(e) => {
                        all_success = false;
                        output_lines.push(format!("✗ {}: ERROR: {}", cmd, e));
                    }
                }
            }

            // Send notification
            let notification_msg = if all_success {
                format!("Anna applied: {} - {}", req.id, advice.kind)
            } else {
                format!("Anna attempted: {} - some commands failed", req.id)
            };
            notifications::notify(req.uid, &notification_msg, &paths.data_dir);

            // Auto-clear on success
            if all_success {
                let _ = fs::remove_file(&advice_path);
                info!(target: "rpc", "auto-cleared advice {} after successful apply", req.id);
            }

            Response::Apply(ApplyResponse {
                status: if all_success { "success" } else { "partial" }.to_string(),
                message: if all_success {
                    "All commands executed successfully".to_string()
                } else {
                    "Some commands failed".to_string()
                },
                requires_approval: false,
                output: Some(output_lines.join("\n")),
            })
        }
        ActionDecision::RequiresConfirmation { reason } => Response::Apply(ApplyResponse {
            status: "requires_confirmation".to_string(),
            message: format!("⚠ {}", reason),
            requires_approval: true,
            output: None,
        }),
        ActionDecision::RequiresElevation { reason } => Response::Apply(ApplyResponse {
            status: "requires_elevation".to_string(),
            message: format!(
                "⚠ {}. Update your policy (/etc/anna/policy.d/{}.toml) or run with sudo.",
                reason, req.uid
            ),
            requires_approval: true,
            output: None,
        }),
    }
}

/// Handle doctor perms request
fn handle_doctor_perms(req: DoctorPermsRequest, _config: &Config) -> Response {
    use self::paths::{AnnaPaths, InstallMode};

    let paths = AnnaPaths::detect_for_uid(req.uid);
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();

    match paths.mode {
        InstallMode::System => {
            // Check socket
            if !paths.socket_path.exists() {
                issues.push(PermissionIssue {
                    path: paths.socket_path.display().to_string(),
                    issue: "Socket not found".to_string(),
                    severity: IssueSeverity::Error,
                });
                suggestions.push("Start annad: sudo systemctl start annad".to_string());
            } else {
                // Check socket permissions
                if let Ok(meta) = fs::metadata(&paths.socket_path) {
                    let mode = meta.permissions().mode() & 0o777;
                    if mode != 0o660 {
                        issues.push(PermissionIssue {
                            path: paths.socket_path.display().to_string(),
                            issue: format!("Socket mode is {:o}, should be 0660", mode),
                            severity: IssueSeverity::Warning,
                        });
                        suggestions.push(format!(
                            "Fix socket permissions: sudo chmod 0660 {}",
                            paths.socket_path.display()
                        ));
                    }
                }
            }

            // Check user data directory
            if !paths.data_dir.exists() {
                issues.push(PermissionIssue {
                    path: paths.data_dir.display().to_string(),
                    issue: "User data directory not found".to_string(),
                    severity: IssueSeverity::Warning,
                });
                suggestions.push(format!(
                    "Create directory: sudo mkdir -p {}",
                    paths.data_dir.display()
                ));
            } else {
                // Check directory permissions
                if let Ok(meta) = fs::metadata(&paths.data_dir) {
                    let mode = meta.permissions().mode() & 0o777;
                    if mode != 0o770 {
                        issues.push(PermissionIssue {
                            path: paths.data_dir.display().to_string(),
                            issue: format!("Directory mode is {:o}, should be 0770", mode),
                            severity: IssueSeverity::Warning,
                        });
                        suggestions.push(format!(
                            "Fix directory permissions: sudo chmod 0770 {}",
                            paths.data_dir.display()
                        ));
                    }
                }
            }

            // Check group membership
            if let Ok(output) = std::process::Command::new("groups").output() {
                let groups = String::from_utf8_lossy(&output.stdout);
                if !groups.contains("anna") {
                    issues.push(PermissionIssue {
                        path: "group membership".to_string(),
                        issue: "User not in 'anna' group".to_string(),
                        severity: IssueSeverity::Warning,
                    });
                    suggestions.push(
                        "Add user to group: sudo usermod -aG anna $(whoami) && newgrp anna"
                            .to_string(),
                    );
                }
            }

            // Check policy file
            let policy_path = format!("/etc/anna/policy.d/{}.toml", req.uid);
            if !PathBuf::from(&policy_path).exists() {
                issues.push(PermissionIssue {
                    path: policy_path.clone(),
                    issue: "No policy file (using default: manual only)".to_string(),
                    severity: IssueSeverity::Info,
                });
                suggestions.push(format!(
                    "Create policy: sudo tee {} <<EOF\n[level]\nauto_apply = 1\n[approval]\nprompt_style = \"interactive\"\nconfirm_dangerous = true\nEOF",
                    policy_path
                ));
            }
        }
        InstallMode::User => {
            // Check user directories
            for (name, path) in [("data", &paths.data_dir), ("config", &paths.config_dir)] {
                if !path.exists() {
                    issues.push(PermissionIssue {
                        path: path.display().to_string(),
                        issue: format!("{} directory not found", name),
                        severity: IssueSeverity::Warning,
                    });
                    suggestions.push(format!("Create directory: mkdir -p {}", path.display()));
                }
            }

            // Check socket
            if !paths.socket_path.exists() {
                issues.push(PermissionIssue {
                    path: paths.socket_path.display().to_string(),
                    issue: "Socket not found".to_string(),
                    severity: IssueSeverity::Error,
                });
                suggestions.push("Start annad: systemctl --user start annad".to_string());
            }
        }
    }

    Response::DoctorPerms(DoctorPermsResponse {
        mode: paths.mode.as_str().to_string(),
        issues,
        suggestions,
    })
}

// ============================================================================
// Helper functions
// ============================================================================

/// Get last quickscan timestamp
fn get_last_quickscan_ts(reports_dir: &Path) -> Option<String> {
    let (_, report) = find_latest_quickscan(reports_dir).ok()??;
    Some(report.generated)
}

/// Count advice entries in directory
fn count_advice_entries(advice_dir: &Path) -> usize {
    fs::read_dir(advice_dir)
        .ok()
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s == "json")
                        .unwrap_or(false)
                })
                .count()
        })
        .unwrap_or(0)
}

/// Find latest quickscan report
fn find_latest_quickscan(
    reports_dir: &Path,
) -> Result<Option<(PathBuf, crate::quickscan::QuickscanReport)>> {
    if !reports_dir.exists() {
        return Ok(None);
    }

    let mut latest: Option<(std::time::SystemTime, PathBuf)> = None;

    for entry in fs::read_dir(reports_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let json = entry.path().join("quickscan.json");
            if json.exists() {
                let meta = json.metadata()?;
                let modified = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                match &mut latest {
                    Some((best_time, _)) if modified <= *best_time => {}
                    _ => latest = Some((modified, json.clone())),
                }
            }
        }
    }

    if let Some((_, path)) = latest {
        let contents = fs::read_to_string(&path)?;
        let report: crate::quickscan::QuickscanReport = serde_json::from_str(&contents)?;
        Ok(Some((path, report)))
    } else {
        Ok(None)
    }
}

/// Read all advice entries from directory
fn read_advice_entries(advice_dir: &Path) -> Result<Vec<anna_rpc::AdviceRecord>> {
    if !advice_dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::new();

    for entry in fs::read_dir(advice_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let data = fs::read(&path)?;
        let rec: crate::advice::types::AdviceRecord = serde_json::from_slice(&data)?;

        // Convert to RPC format
        let rpc_rec = anna_rpc::AdviceRecord {
            id: rec.id,
            kind: rec.kind,
            persona_hint: rec.persona_hint,
            reason: rec.reason,
            created_at: rec.created_at,
            severity: match rec.severity {
                crate::advice::types::AdviceSeverity::Info => anna_rpc::AdviceSeverity::Info,
                crate::advice::types::AdviceSeverity::Warn => anna_rpc::AdviceSeverity::Warn,
                crate::advice::types::AdviceSeverity::Action => anna_rpc::AdviceSeverity::Action,
            },
            plan: anna_rpc::AdvicePlan {
                dry_run_cmds: rec.plan.dry_run_cmds,
                apply_cmds: rec.plan.apply_cmds,
                undo_cmds: rec.plan.undo_cmds,
            },
        };
        records.push(rpc_rec);
    }

    // Sort by timestamp (newest first)
    records.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(records)
}

// We need a paths module in annad too
mod paths {
    use std::path::PathBuf;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InstallMode {
        System,
        User,
    }

    impl InstallMode {
        pub fn as_str(&self) -> &'static str {
            match self {
                InstallMode::System => "system",
                InstallMode::User => "user",
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct AnnaPaths {
        pub mode: InstallMode,
        pub data_dir: PathBuf,
        pub config_dir: PathBuf,
        pub reports_dir: PathBuf,
        pub advice_dir: PathBuf,
        #[allow(dead_code)]
        pub persona_dir: PathBuf,
        #[allow(dead_code)]
        pub signals_dir: PathBuf,
        #[allow(dead_code)]
        pub profiles_dir: PathBuf,
        pub socket_path: PathBuf,
    }

    impl AnnaPaths {
        pub fn detect_for_uid(uid: u32) -> Self {
            // Check ANNA_MODE env var first (for dev/testing)
            if let Ok(mode) = std::env::var("ANNA_MODE") {
                return match mode.as_str() {
                    "user" => Self::user_for_uid(uid),
                    "system" => Self::system_for_uid(uid),
                    _ => Self::auto_detect_for_uid(uid),
                };
            }

            Self::auto_detect_for_uid(uid)
        }

        fn auto_detect_for_uid(uid: u32) -> Self {
            // Check if system paths exist
            let system_data = PathBuf::from("/var/lib/anna");
            let system_config = PathBuf::from("/etc/anna");
            let socket_dir = PathBuf::from("/run/anna");

            if system_data.exists() && system_config.exists() {
                return Self::system_for_uid(uid);
            }

            if socket_dir.join("annad.sock").exists() {
                return Self::system_for_uid(uid);
            }

            // Default: system mode
            Self::system_for_uid(uid)
        }

        pub fn system_for_uid(uid: u32) -> Self {
            let config_dir = PathBuf::from("/etc/anna");
            let user_root = PathBuf::from(format!("/var/lib/anna/users/{}", uid));
            let socket_path = PathBuf::from("/run/anna/annad.sock");

            Self {
                mode: InstallMode::System,
                data_dir: user_root.clone(),
                config_dir,
                reports_dir: user_root.join("reports"),
                advice_dir: user_root.join("advice"),
                persona_dir: user_root.join("persona"),
                signals_dir: user_root.join("signals"),
                profiles_dir: user_root.join("profiles"),
                socket_path,
            }
        }

        pub fn user_for_uid(uid: u32) -> Self {
            let home = Self::get_home_for_uid(uid).expect("Cannot determine home directory");
            let data_dir = home.join(".anna/data");
            let config_dir = home.join(".anna/config");
            let runtime_dir = home.join(".anna/run");

            Self {
                mode: InstallMode::User,
                data_dir: data_dir.clone(),
                config_dir,
                reports_dir: data_dir.join("reports"),
                advice_dir: data_dir.join("advice"),
                persona_dir: data_dir.join("persona"),
                signals_dir: data_dir.join("signals"),
                profiles_dir: data_dir.join("profiles"),
                socket_path: runtime_dir.join("annad.sock"),
            }
        }

        fn get_home_for_uid(uid: u32) -> Result<PathBuf, std::io::Error> {
            if uid == nix::unistd::Uid::effective().as_raw() {
                return dirs::home_dir().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "home not found")
                });
            }

            use nix::unistd::{Uid, User};
            let user = User::from_uid(Uid::from_raw(uid))
                .map_err(std::io::Error::other)?
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "user not found")
                })?;

            Ok(user.dir)
        }
    }
}
