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

/// Sync knowledge from Arch Wiki (stub - can be expanded)
async fn sync_wiki(_personality: &mut PersonalityState) -> Result<Option<String>> {
    // In future: fetch and cache Arch Wiki pages for common topics
    // For now: just acknowledge we're staying current
    debug!("Wiki sync scheduled for future implementation");
    Ok(None)
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

/// Analyze command patterns (stub - can be expanded)
async fn analyze_commands(_personality: &mut PersonalityState) -> Result<Option<String>> {
    // In future: analyze bash history to learn user's workflow
    // For now: basic acknowledgment
    debug!("Command pattern analysis scheduled for future implementation");
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_activities() {
        let activities = LearningActivity::all();
        assert_eq!(activities.len(), 6); // Updated from 5 to 6

        for activity in activities {
            assert!(!activity.description().is_empty());
        }
    }
}
