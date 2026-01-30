//! Scheduler loop for running scheduled tasks.

use anna_shared::scheduler::{TaskAction, TaskStore};
use std::process::Command;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::telegram::notifier::push_notification;

/// Generate comprehensive morning briefing.
fn generate_morning_briefing() -> String {
    let mut sections = Vec::new();

    sections.push("=== MORNING BRIEFING ===".to_string());

    // 1. Pending updates
    if let Ok(output) = Command::new("checkupdates").output() {
        let updates = String::from_utf8_lossy(&output.stdout);
        let count = updates.lines().count();
        if count > 0 {
            sections.push(format!("\n[UPDATES] {} pending", count));
            // Show first 5
            for line in updates.lines().take(5) {
                sections.push(format!("  - {}", line));
            }
            if count > 5 {
                sections.push(format!("  ... and {} more", count - 5));
            }
        } else {
            sections.push("\n[UPDATES] System up to date".to_string());
        }
    }

    // 2. Security - failed logins (last 24h)
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "warning", "-u", "sshd", "-u", "sudo", "--no-pager", "-q"])
        .output()
    {
        let logs = String::from_utf8_lossy(&output.stdout);
        let failed: Vec<&str> = logs.lines()
            .filter(|l| l.contains("Failed") || l.contains("authentication failure") || l.contains("FAILED"))
            .collect();
        if !failed.is_empty() {
            sections.push(format!("\n[SECURITY] {} auth failures (24h)", failed.len()));
            for line in failed.iter().take(3) {
                sections.push(format!("  - {}", line.chars().take(80).collect::<String>()));
            }
        } else {
            sections.push("\n[SECURITY] No auth failures".to_string());
        }
    }

    // 3. Recent errors from journal
    if let Ok(output) = Command::new("journalctl")
        .args(["--since", "24 hours ago", "-p", "err", "--no-pager", "-q", "-n", "10"])
        .output()
    {
        let errors = String::from_utf8_lossy(&output.stdout);
        let count = errors.lines().count();
        if count > 0 {
            sections.push(format!("\n[ERRORS] {} in last 24h", count));
            for line in errors.lines().take(3) {
                sections.push(format!("  - {}", line.chars().take(80).collect::<String>()));
            }
        } else {
            sections.push("\n[ERRORS] None".to_string());
        }
    }

    // 4. Disk usage
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        let df = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = df.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                sections.push(format!("\n[DISK] Root: {} used ({})", parts[4], parts[2]));
            }
        }
    }

    // 5. Memory
    if let Ok(output) = Command::new("free").args(["-h"]).output() {
        let free = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = free.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                sections.push(format!("[MEMORY] {} used / {}", parts[2], parts[1]));
            }
        }
    }

    // 6. Failed services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let failed = String::from_utf8_lossy(&output.stdout);
        let count = failed.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 {
            sections.push(format!("\n[SERVICES] {} failed", count));
            for line in failed.lines().take(3) {
                if let Some(name) = line.split_whitespace().next() {
                    sections.push(format!("  - {}", name));
                }
            }
        } else {
            sections.push("\n[SERVICES] All OK".to_string());
        }
    }

    // 7. Load average
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = load.split_whitespace().collect();
        if parts.len() >= 3 {
            sections.push(format!("[LOAD] {} {} {}", parts[0], parts[1], parts[2]));
        }
    }

    sections.join("\n")
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
                    // Execute through Anna and send result
                    // For now, just notify that the task would run
                    push_notification(&format!("Scheduled task: {}", question));
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

/// Run proactive health checks - anomaly detection and optimization suggestions.
fn run_proactive_health_check() {
    info!("Running proactive health check...");

    // Run anomaly detection
    crate::anomaly::run_anomaly_check();

    // Check for optimization opportunities (only notify if significant)
    let suggestions = crate::anomaly::check_optimizations();
    let significant: Vec<_> = suggestions.iter()
        .filter(|s| {
            // Only notify for disk issues or failed services
            s.category == "Disk" || s.category == "Services"
        })
        .collect();

    if !significant.is_empty() {
        let mut msg = format!("Proactive check: {} items need attention\n", significant.len());
        for s in &significant {
            msg.push_str(&format!("- {}: {}\n", s.category, s.description));
        }
        msg.push_str("\nSay 'suggestions' for details.");
        push_notification(&msg);
    }
}
