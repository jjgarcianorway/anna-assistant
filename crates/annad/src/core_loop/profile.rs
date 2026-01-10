//! System profile management and monitoring loops.

use anna_shared::profile::{self, SystemProfile};
use anna_shared::monitor::{self, IssueStore, MonitorThresholds, Severity};
use anna_shared::safe_ops;
use std::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

/// Cached system profile (refreshable)
pub static SYSTEM_PROFILE: RwLock<Option<SystemProfile>> = RwLock::new(None);

/// System context commands - always run first to understand the environment
pub const SYSTEM_CONTEXT_COMMANDS: &[&str] = &[
    "loginctl show-session $(loginctl list-sessions --no-legend | head -1 | awk '{print $1}') -p Type --value 2>/dev/null",
    "loginctl show-session $(loginctl list-sessions --no-legend | head -1 | awk '{print $1}') -p Desktop --value 2>/dev/null",
    "cat /etc/os-release 2>/dev/null | grep -E '^(NAME|VERSION)=' | head -2",
    "systemctl is-active gdm sddm lightdm 2>/dev/null | grep -v inactive | head -1",
    "grep -i wayland /etc/gdm/custom.conf 2>/dev/null | head -1",
];

/// Initialize system profile on daemon startup - always scans fresh
pub fn init_system_profile() {
    info!("Initializing system profile (fresh scan)...");
    let profile = match profile::scan::scan_system() {
        Ok(p) => {
            if let Err(e) = p.save() {
                warn!("Failed to save system profile: {}", e);
            }
            info!(
                "Profile initialized: bootloader={:?}, editor={:?}, shell={:?}, fs={:?}",
                p.system.bootloader, p.system.editor, p.system.shell, p.system.root_filesystem
            );
            p
        }
        Err(e) => {
            warn!("Failed to scan system: {}", e);
            SystemProfile::default()
        }
    };

    if let Ok(mut guard) = SYSTEM_PROFILE.write() {
        *guard = Some(profile);
    }
}

/// Refresh system profile if needed (called periodically)
pub fn refresh_profile_if_needed() {
    let needs_refresh = {
        let guard = SYSTEM_PROFILE.read().ok();
        guard
            .as_ref()
            .and_then(|g| g.as_ref())
            .map(|p| p.needs_refresh())
            .unwrap_or(true)
    };

    if needs_refresh {
        info!("Profile needs refresh, rescanning...");
        init_system_profile();
    }
}

/// Background loop that periodically refreshes the system profile
pub async fn profile_refresh_loop() {
    let mut interval = interval(Duration::from_secs(30 * 60));

    loop {
        interval.tick().await;
        debug!("Periodic profile refresh check...");
        refresh_profile_if_needed();

        if let Err(e) = safe_ops::cleanup_old_backups() {
            warn!("Failed to cleanup old backups: {}", e);
        }
    }
}

/// Background loop for proactive system monitoring
pub async fn monitoring_loop() {
    let mut interval = interval(Duration::from_secs(5 * 60));
    let thresholds = MonitorThresholds::default();

    tokio::time::sleep(Duration::from_secs(60)).await;

    loop {
        interval.tick().await;
        debug!("Running proactive monitoring checks...");

        let results = monitor::run_checks(&thresholds);
        let mut store = IssueStore::load().unwrap_or_default();
        store.update(results.clone());

        for issue in store.get_critical() {
            warn!("CRITICAL: {}", issue.summary);
        }

        let unnotified = store.get_unnotified();
        if !unnotified.is_empty() {
            info!("Detected {} new issues:", unnotified.len());
            for issue in &unnotified {
                match issue.severity {
                    Severity::Critical => warn!("  [CRIT] {}", issue.summary),
                    Severity::Warning => info!("  [WARN] {}", issue.summary),
                    Severity::Info => debug!("  [INFO] {}", issue.summary),
                }
            }
            store.mark_notified();
        }

        if let Err(e) = store.save() {
            warn!("Failed to save issue store: {}", e);
        }
    }
}

/// Get system profile (returns clone to avoid lock issues)
pub fn get_system_profile() -> SystemProfile {
    if let Ok(guard) = SYSTEM_PROFILE.read() {
        if let Some(ref profile) = *guard {
            return profile.clone();
        }
    }
    init_system_profile();
    if let Ok(guard) = SYSTEM_PROFILE.read() {
        if let Some(ref profile) = *guard {
            return profile.clone();
        }
    }
    SystemProfile::default()
}

/// Get proactive insights relevant to the question
pub fn get_proactive_insights(question: &str, topic: Option<&str>) -> Option<String> {
    let store = IssueStore::load().ok()?;
    if store.active_issues.is_empty() {
        return None;
    }

    let q_lower = question.to_lowercase();
    let topic_lower = topic.map(|t| t.to_lowercase());

    let relevant_issues: Vec<_> = store
        .active_issues
        .iter()
        .filter(|issue| {
            if let Some(ref topic) = topic_lower {
                let issue_type = format!("{:?}", issue.issue_type).to_lowercase();
                if issue_type.contains(topic) || topic.contains(&issue_type) {
                    return true;
                }
            }
            let issue_summary = issue.summary.to_lowercase();
            let keywords = ["disk", "memory", "ram", "cpu", "service", "network", "storage", "boot", "fail"];
            keywords.iter().any(|kw| q_lower.contains(kw) && issue_summary.contains(kw))
        })
        .take(2)
        .collect();

    if relevant_issues.is_empty() {
        return None;
    }

    let mut insights = String::from("Related system status:\n");
    for issue in relevant_issues {
        insights.push_str(&format!(
            "  - {}: {}\n",
            format!("{:?}", issue.severity).to_uppercase(),
            issue.summary
        ));
    }
    Some(insights)
}

/// Gather basic system context (parallelized for speed)
pub fn gather_system_context() -> String {
    use super::command::execute_command;

    let mut context = String::new();

    let profile = get_system_profile();
    let profile_summary = profile.summary_for_llm();
    if !profile_summary.is_empty() {
        context.push_str(&profile_summary);
        context.push('\n');
    }

    let results: Vec<_> = std::thread::scope(|s| {
        let handles: Vec<_> = SYSTEM_CONTEXT_COMMANDS
            .iter()
            .map(|cmd| {
                let cmd = *cmd;
                s.spawn(move || execute_command(cmd).ok().map(|output| (cmd, output)))
            })
            .collect();
        handles.into_iter().map(|h| h.join().ok().flatten()).collect()
    });

    for result in results.into_iter().flatten() {
        let (cmd, output) = result;
        let output = output.trim();
        if !output.is_empty() && !output.contains("command not found") {
            context.push_str(&format!("$ {}\n{}\n", cmd, output));
        }
    }
    context
}

/// Get relevant configs for a question
pub fn get_relevant_configs_for_question(question: &str) -> String {
    let profile = get_system_profile();
    let relevant = profile.get_relevant_configs(question);

    if relevant.is_empty() {
        return String::new();
    }

    let mut context = String::from("\nExisting system configurations:\n");
    for cfg in relevant {
        context.push_str(&format!("--- {} ---\n{}\n", cfg.path, cfg.content));
    }
    context
}
