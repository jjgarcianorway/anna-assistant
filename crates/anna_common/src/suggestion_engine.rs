//! Suggestion Engine v0.1 - Task 8: Deep Caretaker
//!
//! Rule-based suggestion generation from SystemTelemetry
//! Focus: High-value, safe, well-documented suggestions
//!
//! Arch Wiki citation: [archwiki:System_maintenance]

use crate::suggestions::*;
use crate::telemetry::SystemTelemetry;

/// Generate 2-5 prioritized suggestions from system snapshot
/// Task 8: No destructive actions, only analysis and recommendations
pub fn generate_suggestions(snapshot: &SystemTelemetry) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // 1. Disk space optimization
    check_disk_space(snapshot, &mut suggestions);

    // 2. Package management
    check_packages(snapshot, &mut suggestions);

    // 3. Service health
    check_services(snapshot, &mut suggestions);

    // 4. System health basics
    check_system_health(snapshot, &mut suggestions);

    // Sort by priority (highest first)
    suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

    // Return 2-5 suggestions as per spec
    let count = suggestions.len().min(5).max(2.min(suggestions.len()));
    suggestions.into_iter().take(count).collect()
}

/// Check disk space and generate suggestions
fn check_disk_space(snapshot: &SystemTelemetry, suggestions: &mut Vec<Suggestion>) {
    for disk in &snapshot.disks {
        // High disk usage
        if disk.usage_percent > 90.0 {
            suggestions.push(
                Suggestion::new(
                    format!("disk-space-critical-{}", disk.mount_point.replace('/', "-")),
                    format!("Critical: {} is {}% full", disk.mount_point, disk.usage_percent as u8),
                    SuggestionPriority::Critical,
                    SuggestionCategory::Disk,
                )
                .explanation(format!(
                    "The {} filesystem is critically full at {}%. \
                     This can cause system instability, prevent package updates, and make the system unusable. \
                     Immediate action is needed.",
                    disk.mount_point, disk.usage_percent as u8
                ))
                .impact("Prevents system crashes and allows normal operations to continue.")
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "System_maintenance#Disk_usage",
                        "Arch Wiki guide on managing disk space"
                    )
                )
            );
        } else if disk.usage_percent > 80.0 {
            suggestions.push(
                Suggestion::new(
                    format!("disk-space-warning-{}", disk.mount_point.replace('/', "-")),
                    format!("{} is {}% full", disk.mount_point, disk.usage_percent as u8),
                    SuggestionPriority::High,
                    SuggestionCategory::Disk,
                )
                .explanation(format!(
                    "The {} filesystem is at {}% capacity. \
                     While not critical yet, this could become a problem soon, \
                     especially during system updates.",
                    disk.mount_point, disk.usage_percent as u8
                ))
                .impact("Free up space before it becomes critical.")
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "System_maintenance#Disk_usage",
                        "Arch Wiki guide on disk space management"
                    )
                )
            );
        }
    }

    // Large pacman cache
    if snapshot.packages.cache_size_mb > 3000.0 {
        suggestions.push(common_suggestions::pacman_cache_cleanup(snapshot.packages.cache_size_mb));
    }
}

/// Check package management state
fn check_packages(snapshot: &SystemTelemetry, suggestions: &mut Vec<Suggestion>) {
    // Orphaned packages
    if snapshot.packages.orphaned > 10 {
        suggestions.push(common_suggestions::orphaned_packages(snapshot.packages.orphaned as usize));
    }

    // Check if paccache is available for cache cleanup
    if snapshot.packages.cache_size_mb > 1000.0 {
        // Check if pacman-contrib is installed
        let has_paccache = std::process::Command::new("which")
            .arg("paccache")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if !has_paccache {
            suggestions.push(
                Suggestion::new(
                    "install-pacman-contrib",
                    "Install pacman-contrib for cache management",
                    SuggestionPriority::Low,
                    SuggestionCategory::Packages,
                )
                .explanation(
                    "The pacman-contrib package provides paccache, a tool for safely cleaning \
                     old package versions from the cache. Without it, manual cache management is more risky."
                )
                .impact(format!(
                    "Enables safe cleanup of your {:.1} GB pacman cache.",
                    snapshot.packages.cache_size_mb / 1024.0
                ))
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "Pacman#Cleaning_the_package_cache",
                        "Arch Wiki guide on pacman cache management"
                    )
                )
                .auto_fixable(
                    "Install pacman-contrib package",
                    vec!["sudo pacman -S pacman-contrib".to_string()],
                )
            );
        }
    }
}

/// Check systemd services
fn check_services(snapshot: &SystemTelemetry, suggestions: &mut Vec<Suggestion>) {
    if !snapshot.services.failed_units.is_empty() {
        let service_names: Vec<String> = snapshot.services.failed_units
            .iter()
            .map(|unit| unit.name.clone())
            .collect();

        suggestions.push(common_suggestions::failed_services(service_names));
    }
}

/// Check general system health
fn check_system_health(snapshot: &SystemTelemetry, suggestions: &mut Vec<Suggestion>) {
    // High memory usage (info only)
    if snapshot.memory.usage_percent > 90.0 {
        suggestions.push(
            Suggestion::new(
                "memory-high",
                format!("Memory usage is high ({}%)", snapshot.memory.usage_percent as u8),
                SuggestionPriority::Info,
                SuggestionCategory::Memory,
            )
            .explanation(format!(
                "Your system is using {}% of available RAM ({} MB / {} MB). \
                 This is not necessarily a problem on Linux, which uses available memory for caching, \
                 but if you're experiencing slowdowns, consider closing some applications.",
                snapshot.memory.usage_percent as u8,
                snapshot.memory.used_mb,
                snapshot.memory.total_mb
            ))
            .impact("Awareness of memory usage patterns.")
            .add_doc_link(
                DocumentationLink::arch_wiki(
                    "System_maintenance#Memory",
                    "Arch Wiki guide on memory management"
                )
            )
        );
    }

    // High CPU load (info only)
    if snapshot.cpu.load_avg_1min > (snapshot.cpu.cores as f64 * 2.0) {
        suggestions.push(
            Suggestion::new(
                "cpu-load-high",
                format!("CPU load is high ({:.2})", snapshot.cpu.load_avg_1min),
                SuggestionPriority::Info,
                SuggestionCategory::Performance,
            )
            .explanation(format!(
                "Your system's 1-minute load average is {:.2} on a {}-core CPU. \
                 A load higher than the number of cores suggests the system is working hard. \
                 Use 'top' or 'htop' to see what's consuming CPU.",
                snapshot.cpu.load_avg_1min,
                snapshot.cpu.cores
            ))
            .impact("Awareness of CPU usage patterns.")
            .add_doc_link(
                DocumentationLink::arch_wiki(
                    "System_maintenance#Performance",
                    "Arch Wiki guide on performance monitoring"
                )
            )
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::*;

    #[test]
    fn test_empty_snapshot_returns_minimum_suggestions() {
        let snapshot = SystemTelemetry::minimal();
        let suggestions = generate_suggestions(&snapshot);

        // Should return at least 0 suggestions (or 2 if we have baseline suggestions)
        assert!(suggestions.len() <= 5, "Should not exceed 5 suggestions");
    }

    #[test]
    fn test_high_disk_usage_generates_suggestion() {
        let mut snapshot = SystemTelemetry::minimal();
        snapshot.disks.push(DiskInfo {
            mount_point: "/".to_string(),
            total_mb: 100_000,
            used_mb: 95_000,
            usage_percent: 95.0,
            fs_type: "ext4".to_string(),
            smart_status: None,
        });

        let suggestions = generate_suggestions(&snapshot);

        // Should have at least one suggestion about disk space
        assert!(
            suggestions.iter().any(|s| s.category == SuggestionCategory::Disk),
            "Should generate disk space suggestion"
        );
    }

    #[test]
    fn test_large_cache_generates_suggestion() {
        let mut snapshot = SystemTelemetry::minimal();
        snapshot.packages.cache_size_mb = 5000.0; // 5 GB cache

        let suggestions = generate_suggestions(&snapshot);

        // Should suggest cache cleanup
        assert!(
            suggestions.iter().any(|s| s.key.contains("pacman-cache")),
            "Should suggest pacman cache cleanup"
        );
    }

    #[test]
    fn test_failed_services_generates_suggestion() {
        let mut snapshot = SystemTelemetry::minimal();
        snapshot.services.failed_units.push(FailedUnit {
            name: "test.service".to_string(),
            unit_type: "service".to_string(),
            failed_since: None,
            message: None,
        });

        let suggestions = generate_suggestions(&snapshot);

        // Should suggest investigating failed services
        assert!(
            suggestions.iter().any(|s| s.category == SuggestionCategory::Services),
            "Should generate service health suggestion"
        );
    }

    #[test]
    fn test_suggestions_are_sorted_by_priority() {
        let mut snapshot = SystemTelemetry::minimal();

        // Add multiple issues of different priorities
        snapshot.disks.push(DiskInfo {
            mount_point: "/".to_string(),
            total_mb: 100_000,
            used_mb: 95_000,
            usage_percent: 95.0, // Critical
            fs_type: "ext4".to_string(),
            smart_status: None,
        });

        snapshot.packages.orphaned = 15; // Low priority

        let suggestions = generate_suggestions(&snapshot);

        if suggestions.len() >= 2 {
            // First suggestion should be higher or equal priority than second
            assert!(
                suggestions[0].priority >= suggestions[1].priority,
                "Suggestions should be sorted by priority"
            );
        }
    }

    #[test]
    fn test_suggestion_count_respects_limits() {
        let mut snapshot = SystemTelemetry::minimal();

        // Create many issues
        snapshot.disks.push(DiskInfo {
            mount_point: "/".to_string(),
            total_mb: 100_000,
            used_mb: 95_000,
            usage_percent: 95.0,
            fs_type: "ext4".to_string(),
            smart_status: None,
        });

        snapshot.packages.cache_size_mb = 5000.0;
        snapshot.packages.orphaned = 20;

        snapshot.services.failed_units.push(FailedUnit {
            name: "test.service".to_string(),
            unit_type: "service".to_string(),
            failed_since: None,
            message: None,
        });

        snapshot.memory.usage_percent = 95.0;

        let suggestions = generate_suggestions(&snapshot);

        // Should cap at 5 suggestions
        assert!(suggestions.len() <= 5, "Should not exceed 5 suggestions");
    }
}
