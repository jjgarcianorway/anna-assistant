//! Telegram message handlers.
//!
//! Routes messages to the same Ralph loop used by the CLI.

use std::sync::Arc;
use teloxide::prelude::*;
use tracing::{info, warn};

use super::TelegramState;
use crate::ralph;

/// Handle incoming Telegram message.
pub async fn handle_message(
    bot: Bot,
    msg: Message,
    state: Arc<TelegramState>,
) -> anyhow::Result<()> {
    let chat_id = msg.chat.id;
    let user_id = msg.from().map(|u| u.id.0).unwrap_or(0);
    let username = msg.from()
        .and_then(|u| u.username.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Security: check allowed users
    if !state.config.allowed_users.is_empty()
        && !state.config.allowed_users.contains(&user_id)
    {
        warn!("Unauthorized Telegram user: {} ({})", username, user_id);
        bot.send_message(chat_id, "Not authorized. Contact the system administrator.")
            .await?;
        return Ok(());
    }

    // Get message text
    let text = match msg.text() {
        Some(t) => t.trim(),
        None => {
            bot.send_message(chat_id, "Send me a text message with your question.")
                .await?;
            return Ok(());
        }
    };

    if text.is_empty() {
        return Ok(());
    }

    info!("Telegram message from @{}: {}", username, text);

    // Check for confirmation responses
    if text.eq_ignore_ascii_case("yes") || text.eq_ignore_ascii_case("confirm") {
        return handle_confirmation(bot, chat_id, true, state).await;
    }
    if text.eq_ignore_ascii_case("no") || text.eq_ignore_ascii_case("cancel") {
        return handle_confirmation(bot, chat_id, false, state).await;
    }

    // Quick commands (instant, no LLM)
    if let Some(response) = handle_quick_command(text).await {
        bot.send_message(chat_id, &response).await?;
        return Ok(());
    }

    // Check for reminder requests first (bypass Ralph)
    if let Some(response) = handle_reminder(text).await {
        bot.send_message(chat_id, &response).await?;
        return Ok(());
    }

    // Check for morning briefing setup (bypass Ralph)
    if let Some(response) = handle_briefing_setup(text).await {
        bot.send_message(chat_id, &response).await?;
        return Ok(());
    }

    // Check for preference updates (bypass Ralph)
    if let Some(response) = handle_preference_update(text).await {
        bot.send_message(chat_id, &response).await?;
        return Ok(());
    }

    // Send acknowledgment for long-running requests
    bot.send_message(chat_id, "Working on it...").await?;

    // Get model from Anna state
    let model = {
        let anna = state.anna_state.read().await;
        anna.model.clone().unwrap_or_else(|| "qwen2.5:14b".to_string())
    };

    // Get conversation context for follow-ups
    let context_str = {
        let contexts = state.contexts.read().await;
        contexts.get(&chat_id.0)
            .and_then(|ctx| ctx.format_for_llm())
    };

    // Build question with context
    let question_with_context = match context_str {
        Some(ctx) => format!("{}\n\nCurrent question: {}", ctx, text),
        None => text.to_string(),
    };

    // Run Ralph loop with periodic typing indicator
    let bot_clone = bot.clone();
    let typing_task = tokio::spawn(async move {
        loop {
            let _ = bot_clone.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
        }
    });

    let result = ralph::ralph_loop(&model, &question_with_context).await;
    typing_task.abort(); // Stop typing indicator

    match result {
        Ok(ask_result) => {
            // Store conversation turn for context
            {
                let mut contexts = state.contexts.write().await;
                let ctx = contexts.entry(chat_id.0).or_default();
                ctx.add_turn(text.to_string(), ask_result.answer.clone());
            }

            // Check if confirmation is needed
            if ask_result.needs_clarification {
                // Store pending confirmation
                let mut confirms = state.pending_confirms.write().await;
                confirms.insert(
                    chat_id.0,
                    (ask_result.answer.clone(), text.to_string()),
                );

                // Ask for confirmation (plain text, no markdown)
                let confirm_msg = format!(
                    "{}\n\nReply 'yes' to confirm or 'no' to cancel.",
                    &ask_result.answer
                );
                bot.send_message(chat_id, confirm_msg).await?;
            } else {
                // Send answer directly
                info!("Sending Telegram reply ({} chars)", ask_result.answer.len());
                send_long_message(&bot, chat_id, &ask_result.answer).await?;
                info!("Telegram reply sent successfully");
            }
        }
        Err(e) => {
            let error_msg = format!("Error: {}", e);
            bot.send_message(chat_id, &error_msg).await?;
        }
    }

    Ok(())
}

/// Handle confirmation response (yes/no).
async fn handle_confirmation(
    bot: Bot,
    chat_id: ChatId,
    confirmed: bool,
    state: Arc<TelegramState>,
) -> anyhow::Result<()> {
    // Clear the local pending state
    {
        let mut confirms = state.pending_confirms.write().await;
        confirms.remove(&chat_id.0);
    }

    // Check for pending plan in executor
    if confirmed {
        if let Some(plan) = crate::plan_executor::take_pending_plan("default") {
            bot.send_message(chat_id, "Executing plan...").await?;
            bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing).await?;

            // Execute the stored plan directly
            let result = crate::plan_executor::execute_plan(&plan);

            // Format result for Telegram
            let response = format_execution_result(&plan.summary, &result);
            send_long_message(&bot, chat_id, &response).await?;
        } else {
            bot.send_message(chat_id, "Plan expired or not found. Please ask again.").await?;
        }
    } else {
        // Cancel - remove the pending plan
        let _ = crate::plan_executor::take_pending_plan("default");
        bot.send_message(chat_id, "Cancelled.").await?;
    }

    Ok(())
}

/// Format plan execution result for Telegram.
fn format_execution_result(
    summary: &str,
    result: &anna_shared::action_plan::PlanExecutionResult,
) -> String {
    let mut lines = Vec::new();

    if result.success {
        lines.push(format!("Done: {}", summary));
        lines.push(String::new());
        for step_result in &result.step_results {
            let status = if step_result.success { "OK" } else { "FAILED" };
            lines.push(format!("  Step {}: {}", step_result.step_index + 1, status));
        }
        if let Some(ref verification) = result.verification_result {
            if verification.passed {
                lines.push(String::new());
                lines.push("Verified: Changes applied successfully.".to_string());
            }
        }
    } else {
        lines.push(format!("Failed: {}", summary));
        // Show failed step errors
        for step_result in &result.step_results {
            if !step_result.success {
                if let Some(ref err) = step_result.error {
                    lines.push(format!("  Step {} failed: {}", step_result.step_index + 1, err));
                }
            }
        }
        if result.rollback_performed {
            lines.push(String::new());
            lines.push("Changes have been rolled back.".to_string());
        }
    }

    lines.join("\n")
}

/// Send a long message, splitting if needed (Telegram limit: 4096 chars).
async fn send_long_message(bot: &Bot, chat_id: ChatId, text: &str) -> anyhow::Result<()> {
    const MAX_LEN: usize = 4000; // Leave margin for safety

    if text.len() <= MAX_LEN {
        bot.send_message(chat_id, text).await?;
    } else {
        // Split on newlines when possible
        let mut remaining = text;
        while !remaining.is_empty() {
            let chunk = if remaining.len() <= MAX_LEN {
                remaining
            } else {
                // Try to split at a newline
                let split_at = remaining[..MAX_LEN]
                    .rfind('\n')
                    .unwrap_or(MAX_LEN);
                &remaining[..split_at]
            };

            bot.send_message(chat_id, chunk).await?;
            remaining = remaining[chunk.len()..].trim_start();

            // Brief delay between chunks
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    Ok(())
}

/// Handle reminder requests directly (bypass Ralph).
async fn handle_reminder(text: &str) -> Option<String> {
    use anna_shared::scheduler::{parse_reminder, ScheduledTask, TaskStore};

    let (message, when) = parse_reminder(text)?;

    // Create and save the reminder
    let task = ScheduledTask::reminder(&message, when);
    let mut store = TaskStore::load();
    store.add(task);
    if let Err(e) = store.save() {
        tracing::warn!("Failed to save reminder: {}", e);
    }

    // Format response
    let duration = when - chrono::Utc::now();
    let when_str = if duration.num_hours() >= 1 {
        format!("{} hour(s)", duration.num_hours())
    } else {
        format!("{} minute(s)", duration.num_minutes())
    };

    Some(format!("Got it! I'll remind you to \"{}\" in {}.", message, when_str))
}

/// Handle morning briefing setup directly (bypass Ralph).
async fn handle_briefing_setup(text: &str) -> Option<String> {
    use anna_shared::scheduler::{parse_morning_briefing, ScheduledTask, TaskStore};
    use chrono::Timelike;

    let time = parse_morning_briefing(text)?;

    // Create and save the briefing task
    let mut store = TaskStore::load();
    store.remove_morning_briefing(); // Remove existing if any
    let task = ScheduledTask::morning_briefing(time);
    store.add(task);

    if let Err(e) = store.save() {
        tracing::warn!("Failed to save morning briefing: {}", e);
    }

    Some(format!(
        "Morning briefing set! I'll send you a daily health check at {:02}:{:02}.",
        time.hour(), time.minute()
    ))
}

/// Handle preference update requests (bypass Ralph).
async fn handle_preference_update(text: &str) -> Option<String> {
    use anna_shared::preferences::{parse_preference_update, UserPreferences};

    let update = parse_preference_update(text)?;

    let mut prefs = UserPreferences::load();
    let result = update.apply(&mut prefs);

    if let Err(e) = prefs.save() {
        tracing::warn!("Failed to save preferences: {}", e);
    }

    Some(format!("Done! {}", result))
}

/// Handle quick queries (instant responses, no LLM).
async fn handle_quick_command(text: &str) -> Option<String> {
    let q = text.trim().to_lowercase();

    // Status queries
    if q == "status" || q.contains("how's the system") || q.contains("how is the system")
        || q.contains("system status") || q == "how are things" || q == "what's up"
        || q.starts_with("how's everything") {
        return Some(get_quick_status());
    }

    // Updates queries
    if q == "updates" || q.contains("any updates") || q.contains("pending updates")
        || q.contains("are there updates") || q.contains("check for updates")
        || q == "updates?" {
        return Some(get_updates_status());
    }

    // Health queries
    if q == "health" || q.contains("health check") || q.contains("system health")
        || q.contains("how healthy") {
        return Some(crate::core_loop::get_health_summary());
    }

    // Tasks/reminders queries
    if q == "tasks" || q.contains("scheduled tasks") || q.contains("my reminders")
        || q.contains("my tasks") || q.contains("what's scheduled") {
        return Some(get_scheduled_tasks());
    }

    // Fix queries
    if q == "fix" || q == "fix it" || q == "fix issues" || q.contains("fix everything")
        || q.contains("auto fix") || q.contains("fix problems") {
        return Some(auto_fix_safe_issues().await);
    }

    // Cleanup queries
    if q == "clean" || q == "cleanup" || q.contains("clean up") || q.contains("free space")
        || q.contains("clear cache") || q.contains("clear logs") {
        return Some(run_cleanup().await);
    }

    // Optimization queries
    if q == "optimize" || q.contains("suggestions") || q.contains("what can i improve")
        || q.contains("optimization") || q.contains("any improvements")
        || q.contains("how can i optimize") {
        return Some(get_optimization_suggestions());
    }

    // Daily summary queries
    if q == "summary" || q.contains("daily summary") || q.contains("daily report")
        || q.contains("system summary") || q.contains("give me a summary") {
        return Some(get_daily_summary());
    }

    // Help queries
    if q == "help" || q.contains("what can you do") || q == "start"
        || q.contains("how do you work") {
        return Some(
            "I respond to natural questions like:\n\n\
            Status: 'how's the system?' 'what's up?'\n\
            Updates: 'any updates?' 'check for updates'\n\
            Health: 'health check' 'how healthy?'\n\
            Summary: 'daily summary' 'system summary'\n\
            Tasks: 'my reminders' 'scheduled tasks'\n\
            Fix: 'fix issues' 'fix everything'\n\
            Clean: 'clean up' 'free space'\n\
            Optimize: 'any suggestions?'\n\n\
            Smart features:\n\
            - 'remind me in 2 hours to check logs'\n\
            - 'set up morning briefing at 8am'\n\n\
            Or just ask me anything!".to_string()
        );
    }

    None
}

/// Get quick system status.
fn get_quick_status() -> String {
    use std::process::Command;

    let mut status = Vec::new();
    status.push("=== SYSTEM STATUS ===".to_string());

    // Uptime
    if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs) = uptime.split_whitespace().next() {
            if let Ok(s) = secs.parse::<f64>() {
                let hours = (s / 3600.0) as u32;
                let days = hours / 24;
                if days > 0 {
                    status.push(format!("Uptime: {}d {}h", days, hours % 24));
                } else {
                    status.push(format!("Uptime: {}h", hours));
                }
            }
        }
    }

    // Load
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = load.split_whitespace().collect();
        if parts.len() >= 3 {
            status.push(format!("Load: {} {} {}", parts[0], parts[1], parts[2]));
        }
    }

    // Memory
    if let Ok(output) = Command::new("free").args(["-h"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                status.push(format!("RAM: {}/{}", parts[2], parts[1]));
            }
        }
    }

    // Disk
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                status.push(format!("Disk: {} ({})", parts[4], parts[2]));
            }
        }
    }

    // Failed services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let failed = String::from_utf8_lossy(&output.stdout);
        let count = failed.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 {
            status.push(format!("Failed services: {}", count));
        } else {
            status.push("Services: All OK".to_string());
        }
    }

    status.join("\n")
}

/// Get update status with smart categorization.
fn get_updates_status() -> String {
    crate::update_system::get_updates_quick()
}

/// Get scheduled tasks.
fn get_scheduled_tasks() -> String {
    use anna_shared::scheduler::TaskStore;

    let store = TaskStore::load();
    if store.tasks.is_empty() {
        return "No scheduled tasks.\n\nTry: 'set up morning briefing at 8am'".to_string();
    }

    let mut result = format!("{} scheduled tasks:\n", store.tasks.len());
    for task in &store.tasks {
        let status = if task.enabled { "active" } else { "disabled" };
        result.push_str(&format!("  [{}] {}\n", status, task.description));
    }
    result
}

/// Get optimization suggestions.
fn get_optimization_suggestions() -> String {
    let suggestions = crate::anomaly::check_optimizations();

    if suggestions.is_empty() {
        return "No optimization suggestions. Your system looks well-maintained!".to_string();
    }

    let mut result = format!("=== {} OPTIMIZATION SUGGESTIONS ===\n\n", suggestions.len());
    for s in &suggestions {
        result.push_str(&format!("[{}] {}\n", s.category, s.description));
        if let Some(ref savings) = s.potential_savings {
            result.push_str(&format!("  Potential savings: {}\n", savings));
        }
        result.push_str(&format!("  Action: {}\n\n", s.action));
    }

    result.push_str("Say 'fix it' or 'clean up' to address disk issues.");
    result
}

/// Get comprehensive daily summary.
fn get_daily_summary() -> String {
    use std::process::Command;

    let mut sections = Vec::new();
    sections.push("=== DAILY SUMMARY ===".to_string());

    // Uptime
    if let Ok(uptime) = std::fs::read_to_string("/proc/uptime") {
        if let Some(secs) = uptime.split_whitespace().next() {
            if let Ok(s) = secs.parse::<f64>() {
                let days = (s / 86400.0) as u32;
                let hours = ((s % 86400.0) / 3600.0) as u32;
                sections.push(format!("\nUptime: {}d {}h", days, hours));
            }
        }
    }

    // System load
    if let Ok(load) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<&str> = load.split_whitespace().collect();
        if parts.len() >= 3 {
            sections.push(format!("Load: {} {} {}", parts[0], parts[1], parts[2]));
        }
    }

    // Memory
    if let Ok(output) = Command::new("free").args(["-h"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                sections.push(format!("Memory: {}/{}", parts[2], parts[1]));
            }
        }
    }

    // Disk
    if let Ok(output) = Command::new("df").args(["-h", "/"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(line) = out.lines().nth(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                sections.push(format!("Disk: {} used ({})", parts[4], parts[2]));
            }
        }
    }

    // Updates
    let updates = crate::update_system::check_updates();
    if updates.is_empty() {
        sections.push("\nUpdates: System up to date".to_string());
    } else {
        let security: Vec<_> = updates.iter().filter(|u| u.is_security).collect();
        let kernel: Vec<_> = updates.iter().filter(|u| u.is_kernel).collect();
        let mut update_line = format!("\nUpdates: {} available", updates.len());
        if !security.is_empty() {
            update_line.push_str(&format!(" ({} security)", security.len()));
        }
        if !kernel.is_empty() {
            update_line.push_str(" [kernel]");
        }
        sections.push(update_line);
    }

    // Failed services
    if let Ok(output) = Command::new("systemctl")
        .args(["--failed", "--no-pager", "--no-legend"])
        .output()
    {
        let out = String::from_utf8_lossy(&output.stdout);
        let count = out.lines().filter(|l| !l.trim().is_empty()).count();
        if count > 0 {
            sections.push(format!("Services: {} failed", count));
        } else {
            sections.push("Services: All OK".to_string());
        }
    }

    // Optimization suggestions count
    let suggestions = crate::anomaly::check_optimizations();
    if !suggestions.is_empty() {
        sections.push(format!("Suggestions: {} available", suggestions.len()));
    }

    // Reboot status
    if crate::update_system::needs_reboot() {
        sections.push("\n[!] Reboot recommended".to_string());
    }

    sections.join("\n")
}

/// Auto-fix safe issues.
async fn auto_fix_safe_issues() -> String {
    use std::process::Command;

    let mut fixes = Vec::new();
    let mut fixed = 0;

    // 1. Clear package cache if > 2GB
    if let Ok(output) = Command::new("du").args(["-s", "/var/cache/pacman/pkg"]).output() {
        let out = String::from_utf8_lossy(&output.stdout);
        if let Some(size) = out.split_whitespace().next() {
            if let Ok(kb) = size.parse::<u64>() {
                if kb > 2_000_000 { // > 2GB
                    if Command::new("paccache").args(["-rk2"]).status().is_ok() {
                        fixes.push("Cleaned package cache (keeping 2 versions)");
                        fixed += 1;
                    }
                }
            }
        }
    }

    // 2. Clear old journal logs (> 7 days)
    if Command::new("journalctl")
        .args(["--vacuum-time=7d"])
        .status()
        .is_ok()
    {
        fixes.push("Cleaned journal logs older than 7 days");
        fixed += 1;
    }

    // 3. Clear /tmp files older than 7 days
    if Command::new("find")
        .args(["/tmp", "-type", "f", "-mtime", "+7", "-delete"])
        .status()
        .is_ok()
    {
        fixes.push("Cleaned old /tmp files");
        fixed += 1;
    }

    // 4. Check for orphan packages (informational only)
    if let Ok(output) = Command::new("pacman").args(["-Qtdq"]).output() {
        let orphans = String::from_utf8_lossy(&output.stdout);
        let count = orphans.lines().count();
        if count > 0 {
            fixes.push("Note: orphan packages detected (use /clean to review)");
        }
    }

    if fixed > 0 {
        format!("Fixed {} issues:\n{}", fixed, fixes.join("\n"))
    } else {
        "Nothing to fix! System is clean.".to_string()
    }
}

/// Run cleanup tasks.
async fn run_cleanup() -> String {
    use std::process::Command;

    let mut cleaned = Vec::new();

    // Package cache
    if Command::new("paccache").args(["-rk2"]).status().is_ok() {
        cleaned.push("Package cache (kept 2 versions)");
    }

    // Journal
    if Command::new("journalctl")
        .args(["--vacuum-time=7d"])
        .status()
        .is_ok()
    {
        cleaned.push("Journal logs (7 days)");
    }

    // Tmp
    let _ = Command::new("find")
        .args(["/tmp", "-type", "f", "-mtime", "+7", "-delete"])
        .status();
    cleaned.push("/tmp old files");

    // Trash
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let trash = format!("{}/.local/share/Trash/files", home);
        if std::path::Path::new(&trash).exists() {
            let _ = std::fs::remove_dir_all(&trash);
            let _ = std::fs::create_dir_all(&trash);
            cleaned.push("User trash");
        }
    }

    format!("Cleanup complete:\n{}", cleaned.iter().map(|s| format!("  - {}", s)).collect::<Vec<_>>().join("\n"))
}
