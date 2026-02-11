//! Scheduler loop for running scheduled tasks.

use anna_shared::monitor::{DailySnapshot, LongTermHistory};
use anna_shared::scheduler::{ScheduledTask, TaskAction, TaskStore};
use chrono::{Local, NaiveTime};
use std::process::Command;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

use crate::anomaly::AnomalyStore;
use crate::briefing::generate_morning_briefing;
use crate::telegram::notifier::{push_notification, send_pdf_report};

/// Ensure morning briefing task exists (creates one at 8am if missing).
fn ensure_morning_briefing() {
    let mut store = TaskStore::load();

    if !store.has_morning_briefing() {
        // Create default morning briefing at 8:00 AM local time (no username for default)
        let time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let task = ScheduledTask::morning_briefing(time, None);
        store.add(task);

        if let Err(e) = store.save() {
            warn!("Failed to save morning briefing task: {}", e);
        } else {
            info!("Created default morning briefing at 8:00 AM");
        }
    } else {
        info!("Morning briefing already configured");
    }
}

/// Background loop that checks for and executes scheduled tasks.
pub async fn scheduler_loop() {
    info!("Scheduler loop starting (30s delay)...");

    // Wait for system to stabilize before starting
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Ensure morning briefing exists (creates default at 8am if missing)
    ensure_morning_briefing();

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
                TaskAction::HealthCheck { username } => {
                    // Collect daily snapshot before generating report
                    collect_daily_snapshot();

                    // v0.3.156: Generate LLM-based briefing with username
                    let briefing = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            crate::briefing::generate_morning_briefing_llm(username.as_deref())
                                .await
                                .unwrap_or_else(|_| "Good morning! System status check failed.".to_string())
                        })
                    });

                    // Generate PDF report and send via Telegram
                    match crate::report::generate_pdf_report() {
                        Ok(pdf_path) => {
                            info!("Generated morning report: {}", pdf_path.display());
                            send_pdf_report(&pdf_path);
                            // Send LLM-generated briefing
                            push_notification(&briefing);
                        }
                        Err(e) => {
                            warn!("Failed to generate PDF report: {}", e);
                            // Fall back to text briefing
                            push_notification(&briefing);
                        }
                    }
                }
                TaskAction::Question { question } => {
                    // Scheduled questions are logged but not pushed
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
}

/// Collect and save daily snapshot for long-term trend analysis.
fn collect_daily_snapshot() {
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Get metrics from anomaly store
    let anomaly_store = AnomalyStore::load();

    let avg_memory = anomaly_store.metrics.get("RAM")
        .and_then(|h| h.baseline.as_ref())
        .map(|b| b.mean as f32)
        .unwrap_or(0.0);

    let avg_load = anomaly_store.metrics.get("Load1")
        .and_then(|h| h.baseline.as_ref())
        .map(|b| b.mean as f32)
        .unwrap_or(0.0);

    // Get boot time
    let boot_time = Command::new("systemd-analyze")
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.split('=').nth(1)
                .and_then(|s| s.trim().trim_end_matches('s').parse::<f32>().ok())
        })
        .unwrap_or(0.0);

    // Get disk usage
    let disk_used_gb = Command::new("df")
        .args(["--output=used", "-BG", "/"])
        .output()
        .ok()
        .and_then(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.lines().nth(1)
                .and_then(|l| l.trim().trim_end_matches('G').parse::<f32>().ok())
        })
        .unwrap_or(0.0);

    // Count installed packages
    let packages = Command::new("pacman")
        .args(["-Q"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count() as u32)
        .unwrap_or(0);

    // Count questions (from sessions)
    let questions = std::fs::read_to_string("/var/lib/anna/sessions.json")
        .ok()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        .and_then(|v| v.as_array().map(|a| a.len() as u32))
        .unwrap_or(0);

    let snapshot = DailySnapshot {
        date: today.clone(),
        avg_boot_time: boot_time,
        avg_memory_pct: avg_memory,
        avg_load,
        disk_used_gb,
        packages_installed: packages,
        questions_asked: questions,
    };

    // Save to history
    let mut history = LongTermHistory::load();
    history.add_snapshot(snapshot);
    if let Err(e) = history.save() {
        warn!("Failed to save daily snapshot: {}", e);
    } else {
        info!("Saved daily snapshot for {}", today);
    }
}
