//! Scheduler loop for running scheduled tasks.

use anna_shared::scheduler::{TaskAction, TaskStore};
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::telegram::notifier::push_notification;

/// Background loop that checks for and executes scheduled tasks.
pub async fn scheduler_loop() {
    // Wait for system to stabilize before starting
    tokio::time::sleep(Duration::from_secs(30)).await;

    let mut interval = interval(Duration::from_secs(60)); // Check every minute

    loop {
        interval.tick().await;
        debug!("Checking scheduled tasks...");

        let mut store = TaskStore::load();
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
                    // Run health check and send summary
                    let summary = crate::core_loop::get_health_summary();
                    push_notification(&format!("Health Check:\n{}", summary));
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
