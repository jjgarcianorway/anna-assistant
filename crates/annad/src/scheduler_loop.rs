//! Scheduler loop for running scheduled tasks.

use anna_shared::scheduler::{TaskAction, TaskStore};
use std::process::Command;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::telegram::notifier::push_notification;

/// Pick a random greeting based on current time.
fn random_greeting() -> &'static str {
    let greetings = [
        "Good morning! Here's what's happening with your system.",
        "Morning! Quick update on how things are running.",
        "Hey, your daily system check is ready.",
        "Rise and shine! Here's your system status.",
        "Good morning! Everything you need to know today.",
    ];
    let idx = (chrono::Utc::now().timestamp() as usize) % greetings.len();
    greetings[idx]
}

/// Pick a random closing based on overall health.
fn random_closing(all_good: bool) -> &'static str {
    if all_good {
        let closings = [
            "Everything looks healthy. Have a great day!",
            "All systems running smoothly. Nothing to worry about.",
            "Your system is in good shape. Enjoy your day!",
            "Looking good! No action needed from you.",
            "All clear on my end. Let me know if you need anything.",
        ];
        let idx = (chrono::Utc::now().timestamp() as usize / 60) % closings.len();
        closings[idx]
    } else {
        let closings = [
            "A few things might need attention when you have time.",
            "Some items to review, but nothing urgent.",
            "Flagged a few things for you to look at.",
            "Let me know if you want me to handle any of these.",
        ];
        let idx = (chrono::Utc::now().timestamp() as usize / 60) % closings.len();
        closings[idx]
    }
}

/// Generate comprehensive morning briefing in natural language.
fn generate_morning_briefing() -> String {
    let mut parts = Vec::new();
    let mut issues_found = false;

    parts.push(random_greeting().to_string());

    // 1. Updates
    if let Ok(output) = Command::new("checkupdates").output() {
        let updates = String::from_utf8_lossy(&output.stdout);
        let count = updates.lines().count();
        if count > 0 {
            let security_count = updates.lines()
                .filter(|l| l.contains("openssl") || l.contains("sudo") || l.contains("systemd") || l.contains("polkit"))
                .count();

            if security_count > 0 {
                parts.push(format!(
                    "\nThere are {} updates available, {} of which are security-related. You might want to update soon.",
                    count, security_count
                ));
                issues_found = true;
            } else if count > 20 {
                parts.push(format!(
                    "\n{} updates are waiting. Might be a good time for a system update.",
                    count
                ));
            } else {
                parts.push(format!(
                    "\n{} updates available when you're ready.",
                    count
                ));
            }
        } else {
            parts.push("\nYour system is fully up to date.".to_string());
        }
    }

    // 2. Security events
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "warning", "-u", "sshd", "-u", "sudo", "--no-pager", "-q"])
        .output()
    {
        let logs = String::from_utf8_lossy(&output.stdout);
        let failed: Vec<&str> = logs.lines()
            .filter(|l| l.contains("Failed") || l.contains("authentication failure") || l.contains("FAILED"))
            .collect();
        if !failed.is_empty() {
            parts.push(format!(
                "\nI noticed {} failed authentication attempts in the last 24 hours. Worth keeping an eye on.",
                failed.len()
            ));
            issues_found = true;
        }
    }

    // 3. System errors
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "err", "--no-pager", "-q", "-n", "50"])
        .output()
    {
        let errors = String::from_utf8_lossy(&output.stdout);
        let count = errors.lines().count();
        if count > 10 {
            parts.push(format!(
                "\nThere were {} errors logged in the past day. Most are probably harmless, but you might want to check the journal.",
                count
            ));
            issues_found = true;
        } else if count > 0 {
            parts.push(format!(
                "\nJust {} minor errors in the logs. Nothing unusual.",
                count
            ));
        }
    }

    // 4. Disk and memory status
    let mut disk_pct: Option<u32> = None;
    let mut disk_free: Option<String> = None;
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        let df = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = df.lines().nth(1) {
            let parts_df: Vec<&str> = line.split_whitespace().collect();
            if parts_df.len() >= 5 {
                disk_pct = parts_df[4].trim_end_matches('%').parse().ok();
                disk_free = Some(parts_df[3].to_string());
            }
        }
    }

    let mut mem_used: Option<String> = None;
    let mut mem_total: Option<String> = None;
    if let Ok(output) = Command::new("free").args(["-h"]).output() {
        let free = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = free.lines().nth(1) {
            let parts_mem: Vec<&str> = line.split_whitespace().collect();
            if parts_mem.len() >= 3 {
                mem_used = Some(parts_mem[2].to_string());
                mem_total = Some(parts_mem[1].to_string());
            }
        }
    }

    if let (Some(pct), Some(free)) = (disk_pct, &disk_free) {
        if pct > 85 {
            parts.push(format!(
                "\nDisk is getting full at {}% used, only {} free. Consider cleaning up.",
                pct, free
            ));
            issues_found = true;
        } else {
            parts.push(format!(
                "\nDisk usage is at {}% with {} free.",
                pct, free
            ));
        }
    }

    if let (Some(used), Some(total)) = (mem_used, mem_total) {
        parts.push(format!("Memory: {} used out of {}.", used, total));
    }

    // 5. Services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let failed = String::from_utf8_lossy(&output.stdout);
        let failed_services: Vec<&str> = failed.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| l.split_whitespace().next())
            .collect();

        if !failed_services.is_empty() {
            if failed_services.len() == 1 {
                parts.push(format!(
                    "\nOne service is having trouble: {}. I can try to restart it if you'd like.",
                    failed_services[0]
                ));
            } else {
                parts.push(format!(
                    "\n{} services are in a failed state: {}. Let me know if you want me to look into it.",
                    failed_services.len(),
                    failed_services.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                ));
            }
            issues_found = true;
        }
    }

    // 6. Load average context
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        if let Some(load1) = load.split_whitespace().next() {
            if let Ok(load_val) = load1.parse::<f32>() {
                if load_val > 4.0 {
                    parts.push(format!(
                        "\nSystem load is elevated at {}. Something might be working hard.",
                        load1
                    ));
                    issues_found = true;
                }
            }
        }
    }

    // 7. Include anomaly data if available
    let store = crate::anomaly::AnomalyStore::load();
    let anomalies: Vec<_> = store.metrics.values()
        .filter_map(|h| {
            if let Some(ref baseline) = h.baseline {
                if let Some(sample) = h.samples.last() {
                    if baseline.is_anomaly(sample.value) {
                        return Some(format!("{} ({:.1}{})", h.name, sample.value, h.unit));
                    }
                }
            }
            None
        })
        .collect();

    if !anomalies.is_empty() {
        parts.push(format!(
            "\nSome metrics are outside normal ranges: {}. Probably fine, but worth noting.",
            anomalies.join(", ")
        ));
        issues_found = true;
    }

    // Closing
    parts.push(format!("\n{}", random_closing(!issues_found)));

    parts.join("")
}

/// Background loop that checks for and executes scheduled tasks.
pub async fn scheduler_loop() {
    info!("Scheduler loop starting (30s delay)...");

    // Wait for system to stabilize before starting
    tokio::time::sleep(Duration::from_secs(30)).await;

    info!("Scheduler loop active - checking every 60s");

    let mut interval = interval(Duration::from_secs(60)); // Check every minute
    let mut health_check_counter = 0u32;

    // Run proactive checks every 6 hours (360 minutes)
    const HEALTH_CHECK_INTERVAL: u32 = 360;

    loop {
        interval.tick().await;
        health_check_counter += 1;

        // Proactive health check (every 6 hours)
        if health_check_counter >= HEALTH_CHECK_INTERVAL {
            health_check_counter = 0;
            run_proactive_health_check();
        }

        let mut store = TaskStore::load();
        let task_count = store.tasks.len();
        if task_count > 0 {
            debug!("Checking {} scheduled tasks...", task_count);
        }

        let due_tasks: Vec<_> = store.get_due().iter().map(|t| (*t).clone()).collect();

        if due_tasks.is_empty() {
            continue;
        }

        info!("Found {} due tasks", due_tasks.len());

        for task in due_tasks {
            info!("Running scheduled task: {}", task.description);

            match &task.action {
                TaskAction::Reminder { message } => {
                    push_notification(&format!("Reminder: {}", message));
                }
                TaskAction::HealthCheck => {
                    // Run comprehensive morning briefing
                    let briefing = generate_morning_briefing();
                    push_notification(&briefing);
                }
                TaskAction::Question { question } => {
                    // Scheduled questions are logged but not pushed
                    // They can be expanded later to actually run queries
                    info!("Scheduled task due: {}", question);
                }
            }

            store.mark_run(&task.id);
        }

        // Cleanup and save
        store.cleanup();
        if let Err(e) = store.save() {
            debug!("Failed to save task store: {}", e);
        }
    }
}

/// Run proactive health checks - completely silent.
/// Self-healing runs automatically, all info goes to morning briefing.
fn run_proactive_health_check() {
    debug!("Running proactive health check...");

    // 1. Run self-healing silently
    let healing_results = crate::self_healing::run_self_healing();
    for r in &healing_results {
        if r.success {
            info!("Self-healed: {}", r.action);
        }
    }

    // 2. Run anomaly detection (stores data for morning briefing)
    crate::anomaly::run_anomaly_check();

    // No push notifications - everything goes to morning briefing
    // The user explicitly asked for critical-only, and even those should be
    // in the briefing unless it's a true emergency like disk at 0%
}
