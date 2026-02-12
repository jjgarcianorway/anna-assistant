//! Cleanup Detection - Anna finds cleanable space and proposes safe cleanup.
//!
//! Philosophy: Proactively find waste, categorize by safety, propose cleanup.
//! NO HARDCODING: LLM decides how to clean based on what we find.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// A cleanable item Anna found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanableItem {
    /// What it is
    pub description: String,
    /// Where it is
    pub path: String,
    /// Size in MB
    pub size_mb: f64,
    /// Safety level
    pub safety: SafetyLevel,
    /// How to clean it (command suggestion)
    pub cleanup_method: String,
    /// Why it's cleanable
    pub reason: String,
}

/// Safety level for cleanup.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SafetyLevel {
    Safe,        // 100% safe (cache, tmp)
    MostlySafe,  // 95% safe (old logs, unused packages)
    Careful,     // 80% safe (old Docker images)
    Risky,       // <80% (user data, configs)
}

/// Cleanup analysis result.
#[derive(Debug, Clone)]
pub struct CleanupAnalysis {
    pub total_cleanable_mb: f64,
    pub items: Vec<CleanableItem>,
    pub recommendations: Vec<String>,
}

/// Scan system for cleanable space.
pub async fn scan_for_cleanable_space() -> Result<CleanupAnalysis> {
    info!("Scanning for cleanable space...");

    let mut items = Vec::new();

    // Check package cache
    if let Ok(size) = get_directory_size("/var/cache/pacman/pkg").await {
        if size > 100.0 {
            items.push(CleanableItem {
                description: "Package cache".to_string(),
                path: "/var/cache/pacman/pkg".to_string(),
                size_mb: size,
                safety: SafetyLevel::Safe,
                cleanup_method: "paccache -r or paccache -rk1".to_string(),
                reason: "Old package versions no longer needed".to_string(),
            });
        }
    }

    // Check journal logs
    if let Ok(size) = get_directory_size("/var/log/journal").await {
        if size > 500.0 {
            items.push(CleanableItem {
                description: "System journal logs".to_string(),
                path: "/var/log/journal".to_string(),
                size_mb: size,
                safety: SafetyLevel::MostlySafe,
                cleanup_method: "journalctl --vacuum-size=100M or journalctl --vacuum-time=30d".to_string(),
                reason: format!("Logs using {:.1}GB (can reduce to 100MB safely)", size / 1024.0),
            });
        }
    }

    // Check /tmp
    if let Ok(size) = get_directory_size("/tmp").await {
        if size > 100.0 {
            items.push(CleanableItem {
                description: "Temporary files".to_string(),
                path: "/tmp".to_string(),
                size_mb: size,
                safety: SafetyLevel::Safe,
                cleanup_method: "find /tmp -type f -atime +7 -delete".to_string(),
                reason: "Temporary files older than 7 days".to_string(),
            });
        }
    }

    // Check Docker if installed
    if let Ok(output) = crate::core_loop::execute_command("docker system df 2>/dev/null") {
        if !output.is_empty() {
            // Parse docker system df output
            let lines: Vec<&str> = output.lines().collect();
            
            for line in lines {
                if line.contains("Images") {
                    if let Some(size) = extract_size_from_docker_line(line) {
                        if size > 500.0 {
                            items.push(CleanableItem {
                                description: "Docker images".to_string(),
                                path: "Docker storage".to_string(),
                                size_mb: size,
                                safety: SafetyLevel::Careful,
                                cleanup_method: "docker image prune -a --filter 'until=30d'".to_string(),
                                reason: "Unused Docker images (older than 30 days)".to_string(),
                            });
                        }
                    }
                }
                
                if line.contains("Build Cache") {
                    if let Some(size) = extract_size_from_docker_line(line) {
                        if size > 100.0 {
                            items.push(CleanableItem {
                                description: "Docker build cache".to_string(),
                                path: "Docker storage".to_string(),
                                size_mb: size,
                                safety: SafetyLevel::Safe,
                                cleanup_method: "docker builder prune -a".to_string(),
                                reason: "Build cache can be safely removed".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for core dumps
    if let Ok(size) = get_directory_size("/var/lib/systemd/coredump").await {
        if size > 100.0 {
            items.push(CleanableItem {
                description: "Core dumps".to_string(),
                path: "/var/lib/systemd/coredump".to_string(),
                size_mb: size,
                safety: SafetyLevel::MostlySafe,
                cleanup_method: "rm /var/lib/systemd/coredump/*".to_string(),
                reason: "Old crash dumps no longer needed for debugging".to_string(),
            });
        }
    }

    // Check for old kernels (if multiple installed)
    if let Ok(output) = crate::core_loop::execute_command("ls /boot/vmlinuz-* 2>/dev/null | wc -l") {
        if let Ok(count) = output.trim().parse::<u32>() {
            if count > 2 {
                items.push(CleanableItem {
                    description: format!("Old kernels ({} installed)", count),
                    path: "/boot".to_string(),
                    size_mb: (count as f64 - 2.0) * 50.0, // Estimate 50MB per kernel
                    safety: SafetyLevel::MostlySafe,
                    cleanup_method: "Remove old kernels via package manager".to_string(),
                    reason: format!("Keep current + 1 backup (remove {} old)", count - 2),
                });
            }
        }
    }

    // Check for orphaned packages (Arch)
    if let Ok(output) = crate::core_loop::execute_command("pacman -Qdtq 2>/dev/null | wc -l") {
        if let Ok(count) = output.trim().parse::<u32>() {
            if count > 0 {
                items.push(CleanableItem {
                    description: format!("Orphaned packages ({})", count),
                    path: "System packages".to_string(),
                    size_mb: count as f64 * 10.0, // Estimate 10MB per package
                    safety: SafetyLevel::MostlySafe,
                    cleanup_method: "pacman -Rns $(pacman -Qdtq)".to_string(),
                    reason: "Packages no longer needed by any installed software".to_string(),
                });
            }
        }
    }

    // Sort by size (largest first)
    items.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());

    let total = items.iter().map(|i| i.size_mb).sum();

    let recommendations = generate_recommendations(&items);

    Ok(CleanupAnalysis {
        total_cleanable_mb: total,
        items,
        recommendations,
    })
}

/// Get directory size in MB.
async fn get_directory_size(path: &str) -> Result<f64> {
    let cmd = format!("du -sm {} 2>/dev/null | cut -f1", path);
    let output = crate::core_loop::execute_command(&cmd)?;
    
    let size_mb: f64 = output.trim().parse()?;
    Ok(size_mb)
}

/// Extract size from docker system df line.
fn extract_size_from_docker_line(line: &str) -> Option<f64> {
    // Line format: "Images  10  5  1.2GB"
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    for (i, part) in parts.iter().enumerate() {
        if part.ends_with("GB") || part.ends_with("MB") {
            let size_str = part.trim_end_matches("GB").trim_end_matches("MB");
            if let Ok(size) = size_str.parse::<f64>() {
                return Some(if part.ends_with("GB") {
                    size * 1024.0
                } else {
                    size
                });
            }
        }
    }
    
    None
}

/// Generate cleanup recommendations.
fn generate_recommendations(items: &[CleanableItem]) -> Vec<String> {
    let mut recs = Vec::new();

    // Count by safety level
    let safe_count = items.iter().filter(|i| i.safety == SafetyLevel::Safe).count();
    let safe_size: f64 = items.iter()
        .filter(|i| i.safety == SafetyLevel::Safe)
        .map(|i| i.size_mb)
        .sum();

    if safe_count > 0 {
        recs.push(format!(
            "Clean {} safe items first ({:.1}GB)",
            safe_count,
            safe_size / 1024.0
        ));
    }

    // Specific recommendations
    if items.iter().any(|i| i.description.contains("Package cache")) {
        recs.push("Run paccache -r to keep only last 3 package versions".to_string());
    }

    if items.iter().any(|i| i.description.contains("journal logs")) {
        recs.push("Limit journal size to 100MB with vacuum command".to_string());
    }

    if items.iter().any(|i| i.description.contains("Docker")) {
        recs.push("Clean Docker with 'docker system prune -a' (removes unused images)".to_string());
    }

    recs
}

/// Format cleanup analysis for display.
pub fn format_cleanup_analysis(analysis: &CleanupAnalysis) -> String {
    let mut response = format!(
        "Cleanable Space Found: {:.2}GB\n\n",
        analysis.total_cleanable_mb / 1024.0
    );

    // Group by safety level
    let safe: Vec<_> = analysis.items.iter()
        .filter(|i| i.safety == SafetyLevel::Safe)
        .collect();
    
    let mostly_safe: Vec<_> = analysis.items.iter()
        .filter(|i| i.safety == SafetyLevel::MostlySafe)
        .collect();
    
    let careful: Vec<_> = analysis.items.iter()
        .filter(|i| i.safety == SafetyLevel::Careful)
        .collect();

    if !safe.is_empty() {
        response.push_str("SAFE to Clean:\n");
        for item in safe {
            response.push_str(&format!(
                "  - {} ({:.1}GB): {}\n",
                item.description,
                item.size_mb / 1024.0,
                item.reason
            ));
        }
        response.push('\n');
    }

    if !mostly_safe.is_empty() {
        response.push_str("Mostly Safe (Review First):\n");
        for item in mostly_safe {
            response.push_str(&format!(
                "  - {} ({:.1}GB): {}\n",
                item.description,
                item.size_mb / 1024.0,
                item.reason
            ));
        }
        response.push('\n');
    }

    if !careful.is_empty() {
        response.push_str("Careful (Check Before Cleaning):\n");
        for item in careful {
            response.push_str(&format!(
                "  - {} ({:.1}GB): {}\n",
                item.description,
                item.size_mb / 1024.0,
                item.reason
            ));
        }
        response.push('\n');
    }

    if !analysis.recommendations.is_empty() {
        response.push_str("Recommendations:\n");
        for (i, rec) in analysis.recommendations.iter().enumerate() {
            response.push_str(&format!("{}. {}\n", i + 1, rec));
        }
    }

    response.push_str("\nWould you like me to:\n");
    response.push_str("1. Clean all safe items automatically\n");
    response.push_str("2. Show commands for you to review first\n");
    response.push_str("3. Clean specific items only\n");
    response.push_str("4. Schedule cleanup for later\n");

    response
}

/// Check if cleanup should be proposed (disk >75% or on request).
pub async fn should_propose_cleanup(disk_usage_pct: f32) -> bool {
    disk_usage_pct > 75.0
}
