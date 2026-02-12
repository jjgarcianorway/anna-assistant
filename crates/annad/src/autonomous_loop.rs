//! Autonomous Learning Loop - Anna's "idle time" behavior.
//!
//! When Anna isn't actively responding to queries, she:
//! - Monitors system logs for anomalies
//! - Researches common errors she encounters
//! - Learns from Arch Wiki about packages installed
//! - Builds knowledge about the user's system
//! - Improves pattern detection
//!
//! Philosophy: Anna should be like a vigilant sysadmin, always learning and monitoring.

use anyhow::Result;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, warn};

use crate::personality::PersonalityState;

/// How often Anna checks for learning opportunities (5 minutes)
const LEARNING_INTERVAL_SECS: u64 = 300;

/// Autonomous learning activities Anna performs during idle time
#[derive(Debug, Clone)]
pub enum LearningActivity {
    /// Scan system logs for new error patterns
    LogAnalysis,
    /// Research installed packages
    PackageResearch,
    /// Check for system optimizations
    OptimizationScan,
    /// Scan for proactive suggestions
    ProactiveSuggestions,
    /// Automatically heal common issues
    AutoHealing,
    /// Update knowledge base from Arch Wiki
    WikiSync,
    /// Analyze command history patterns
    CommandPatternAnalysis,
}

impl LearningActivity {
    /// Get all learning activities
    fn all() -> Vec<Self> {
        vec![
            LearningActivity::LogAnalysis,
            LearningActivity::PackageResearch,
            LearningActivity::OptimizationScan,
            LearningActivity::ProactiveSuggestions,
            LearningActivity::AutoHealing,
            LearningActivity::WikiSync,
            LearningActivity::CommandPatternAnalysis,
        ]
    }

    /// Description of this activity
    fn description(&self) -> &'static str {
        match self {
            LearningActivity::LogAnalysis => "Analyzing system logs for patterns",
            LearningActivity::PackageResearch => "Researching installed packages",
            LearningActivity::OptimizationScan => "Scanning for optimization opportunities",
            LearningActivity::ProactiveSuggestions => "Generating proactive suggestions",
            LearningActivity::AutoHealing => "Auto-healing common issues",
            LearningActivity::WikiSync => "Syncing knowledge from Arch Wiki",
            LearningActivity::CommandPatternAnalysis => "Learning command usage patterns",
        }
    }

    /// Execute this learning activity
    async fn execute(&self, personality: &mut PersonalityState) -> Result<Option<String>> {
        match self {
            LearningActivity::LogAnalysis => analyze_logs(personality).await,
            LearningActivity::PackageResearch => research_packages(personality).await,
            LearningActivity::OptimizationScan => scan_optimizations(personality).await,
            LearningActivity::ProactiveSuggestions => generate_suggestions(personality).await,
            LearningActivity::AutoHealing => run_auto_healing(personality).await,
            LearningActivity::WikiSync => sync_wiki(personality).await,
            LearningActivity::CommandPatternAnalysis => analyze_commands(personality).await,
        }
    }
}

/// Main autonomous learning loop
pub async fn autonomous_learning_loop() {
    info!("Autonomous learning loop started (Anna's idle-time behavior)");

    // Wait for system to stabilize
    tokio::time::sleep(Duration::from_secs(60)).await;

    let mut interval = interval(Duration::from_secs(LEARNING_INTERVAL_SECS));
    let mut activity_index = 0;

    loop {
        interval.tick().await;

        // Load personality state
        let mut personality = PersonalityState::load();

        // Select next activity (round-robin)
        let activities = LearningActivity::all();
        let activity = &activities[activity_index % activities.len()];
        activity_index += 1;

        debug!("Anna performing: {}", activity.description());

        // Execute activity
        match activity.execute(&mut personality).await {
            Ok(Some(lesson)) => {
                info!("Anna learned: {}", lesson);
                personality.learn_lesson(lesson);
                let _ = personality.save();
            }
            Ok(None) => {
                debug!("No new learnings from {}", activity.description());
            }
            Err(e) => {
                warn!("Error during {}: {}", activity.description(), e);
            }
        }
    }
}

/// Analyze system logs for new error patterns
async fn analyze_logs(personality: &mut PersonalityState) -> Result<Option<String>> {
    // Check journalctl for recent errors
    let output = std::process::Command::new("journalctl")
        .args(["-p", "err", "-n", "50", "--no-pager"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let logs = String::from_utf8_lossy(&output.stdout);

    // Extract unique error patterns
    let mut error_patterns: HashSet<String> = HashSet::new();
    for line in logs.lines() {
        if let Some(msg_part) = line.split("]: ").nth(1) {
            // Take first 50 chars as pattern signature
            let pattern = msg_part.chars().take(50).collect::<String>();
            error_patterns.insert(pattern);
        }
    }

    if !error_patterns.is_empty() {
        let lesson = format!(
            "Monitored {} unique error patterns in recent logs",
            error_patterns.len()
        );
        Ok(Some(lesson))
    } else {
        Ok(None)
    }
}

/// Research installed packages to learn about system configuration
async fn research_packages(_personality: &mut PersonalityState) -> Result<Option<String>> {
    // Get list of explicitly installed packages
    let output = std::process::Command::new("pacman")
        .args(["-Qe"])
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let packages = String::from_utf8_lossy(&output.stdout);
    let package_count = packages.lines().count();

    // Learn about system type from packages
    let has_nvidia = packages.contains("nvidia");
    let has_amd = packages.contains("amd");
    let has_docker = packages.contains("docker");
    let has_cuda = packages.contains("cuda");

    let mut insights = Vec::new();
    if has_nvidia {
        insights.push("NVIDIA GPU detected");
    }
    if has_amd {
        insights.push("AMD hardware detected");
    }
    if has_docker {
        insights.push("Docker environment");
    }
    if has_cuda {
        insights.push("CUDA capable system");
    }

    if !insights.is_empty() {
        let lesson = format!(
            "System profile: {} packages installed. Config: {}",
            package_count,
            insights.join(", ")
        );
        Ok(Some(lesson))
    } else {
        Ok(None)
    }
}

/// Scan for optimization opportunities
async fn scan_optimizations(_personality: &mut PersonalityState) -> Result<Option<String>> {
    // Check for orphaned packages
    let output = std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()?;

    if output.status.success() {
        let orphans = String::from_utf8_lossy(&output.stdout);
        let orphan_count = orphans.lines().count();

        if orphan_count > 0 {
            let lesson = format!("Found {} orphaned packages that could be removed", orphan_count);
            return Ok(Some(lesson));
        }
    }

    // Check pacman cache size
    if let Ok(metadata) = std::fs::metadata("/var/cache/pacman/pkg") {
        if let Ok(entries) = std::fs::read_dir("/var/cache/pacman/pkg") {
            let count = entries.count();
            if count > 1000 {
                let lesson = format!(
                    "Pacman cache has {} packages - consider cleaning with paccache",
                    count
                );
                return Ok(Some(lesson));
            }
        }
    }

    Ok(None)
}

/// Sync knowledge from Arch Wiki
async fn sync_wiki(_personality: &mut PersonalityState) -> Result<Option<String>> {
    debug!("Syncing knowledge from Arch Wiki");

    // Fetch relevant wiki pages based on recent errors
    match crate::wiki_sync::sync_wiki_pages().await {
        Ok(synced) if !synced.is_empty() => {
            let lesson = format!("Cached {} Arch Wiki pages: {}", synced.len(), synced.join(", "));
            info!("{}", lesson);
            Ok(Some(lesson))
        }
        Ok(_) => {
            debug!("No new wiki pages to sync");
            Ok(None)
        }
        Err(e) => {
            warn!("Wiki sync failed: {}", e);
            Ok(None) // Don't fail the learning loop
        }
    }
}

/// Generate proactive suggestions
async fn generate_suggestions(personality: &mut PersonalityState) -> Result<Option<String>> {
    debug!("Scanning for proactive suggestions");

    // Scan system for suggestions
    let suggestions = crate::suggestions::scan_for_suggestions().await?;

    if suggestions.is_empty() {
        debug!("No new suggestions found");
        return Ok(None);
    }

    // Load existing suggestions state
    let mut state = crate::suggestions::SuggestionsState::load();
    state.last_scan = chrono::Utc::now().to_rfc3339();

    // Add new suggestions
    let mut added_count = 0;
    for suggestion in suggestions {
        state.add(suggestion);
        added_count += 1;
    }

    // Save state
    state.save()?;

    if added_count > 0 {
        let lesson = format!("Generated {} proactive suggestions for user", added_count);
        Ok(Some(lesson))
    } else {
        Ok(None)
    }
}

/// Analyze command patterns from bash history
async fn analyze_commands(_personality: &mut PersonalityState) -> Result<Option<String>> {
    debug!("Analyzing command patterns from bash history");

    // Find bash history files
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    let history_path = format!("{}/.bash_history", home);

    if !std::path::Path::new(&history_path).exists() {
        debug!("No bash history found at {}", history_path);
        return Ok(None);
    }

    // Read recent commands (last 100)
    let content = match std::fs::read_to_string(&history_path) {
        Ok(c) => c,
        Err(_) => return Ok(None),
    };

    let commands: Vec<&str> = content.lines().rev().take(100).collect();

    // Detect common patterns
    let mut patterns = Vec::new();

    // Pattern: docker ps followed by docker logs
    let docker_pattern = detect_sequence(&commands, &["docker ps", "docker logs"], 5);
    if docker_pattern > 2 {
        patterns.push("docker ps → docker logs (container debugging)".to_string());
    }

    // Pattern: git status followed by git add/commit
    let git_pattern = detect_sequence(&commands, &["git status", "git add"], 10);
    if git_pattern > 3 {
        patterns.push("git status → git add (version control workflow)".to_string());
    }

    // Pattern: systemctl status followed by journalctl
    let systemd_pattern = detect_sequence(&commands, &["systemctl", "journalctl"], 5);
    if systemd_pattern > 2 {
        patterns.push("systemctl → journalctl (service debugging)".to_string());
    }

    // Pattern: frequent pacman -Syu
    let pacman_updates = commands.iter().filter(|c| c.contains("pacman") && c.contains("Syu")).count();
    if pacman_updates > 5 {
        patterns.push(format!("Frequent system updates ({} times)", pacman_updates));
    }

    // Pattern: frequent restarts of specific service
    let restart_commands: Vec<&&str> = commands.iter()
        .filter(|c| c.contains("systemctl restart"))
        .collect();

    if let Some(most_restarted) = find_most_common_service(&restart_commands) {
        if restart_commands.iter().filter(|c| c.contains(most_restarted)).count() > 3 {
            patterns.push(format!("Frequent restarts of {} service", most_restarted));
        }
    }

    if !patterns.is_empty() {
        let lesson = format!("Learned {} command patterns: {}", patterns.len(), patterns.join("; "));
        info!("{}", lesson);
        Ok(Some(lesson))
    } else {
        debug!("No significant command patterns detected");
        Ok(None)
    }
}

/// Detect command sequences in history
fn detect_sequence(commands: &[&str], pattern: &[&str], window: usize) -> usize {
    let mut count = 0;

    for i in 0..commands.len().saturating_sub(1) {
        let window_end = (i + window).min(commands.len());
        let window_commands = &commands[i..window_end];

        // Check if pattern appears in this window
        let mut pattern_idx = 0;
        for cmd in window_commands {
            if cmd.contains(pattern[pattern_idx]) {
                pattern_idx += 1;
                if pattern_idx >= pattern.len() {
                    count += 1;
                    break;
                }
            }
        }
    }

    count
}

/// Find most commonly restarted service
fn find_most_common_service<'a>(restart_commands: &'a [&&str]) -> Option<&'a str> {
    use std::collections::HashMap;

    let mut service_counts: HashMap<&str, usize> = HashMap::new();

    for cmd in restart_commands {
        // Extract service name from "systemctl restart SERVICE"
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if let Some(service) = parts.get(2) {
            *service_counts.entry(service).or_insert(0) += 1;
        }
    }

    service_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(service, _)| service)
}

/// Run automatic healing for common issues
async fn run_auto_healing(_personality: &mut PersonalityState) -> Result<Option<String>> {
    debug!("Running auto-healing checks");

    match crate::autohealing::run_safe_healing().await {
        Ok(healed) if !healed.is_empty() => {
            let lesson = format!("Auto-healed {} issues: {}", healed.len(), healed.join("; "));
            info!("{}", lesson);
            Ok(Some(lesson))
        }
        Ok(_) => {
            debug!("No issues requiring auto-healing");
            Ok(None)
        }
        Err(e) => {
            warn!("Auto-healing failed: {}", e);
            Ok(None) // Don't fail the learning loop
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_activities() {
        let activities = LearningActivity::all();
        assert_eq!(activities.len(), 7); // Updated to 7 (added AutoHealing)

        for activity in activities {
            assert!(!activity.description().is_empty());
        }
    }
}
