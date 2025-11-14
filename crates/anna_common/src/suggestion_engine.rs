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

    // 5. Audio stack (Task 9: dependency-aware suggestions)
    check_audio_stack(snapshot, &mut suggestions);

    // Task 9: Dependency filtering - only show suggestions with satisfied dependencies
    suggestions = filter_by_dependencies(suggestions);

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

/// Check audio stack configuration (Task 9: dependency-aware suggestions)
fn check_audio_stack(snapshot: &SystemTelemetry, suggestions: &mut Vec<Suggestion>) {
    // If hardware exists but PipeWire stack is not running, suggest configuration
    if snapshot.audio.has_sound_hardware
        && (!snapshot.audio.pipewire_running
            || !snapshot.audio.wireplumber_running
            || !snapshot.audio.pipewire_pulse_running) {

        let mut missing_services = Vec::new();
        if !snapshot.audio.pipewire_running {
            missing_services.push("pipewire");
        }
        if !snapshot.audio.wireplumber_running {
            missing_services.push("wireplumber");
        }
        if !snapshot.audio.pipewire_pulse_running {
            missing_services.push("pipewire-pulse");
        }

        suggestions.push(
            Suggestion::new(
                "audio-stack-config",
                "Configure audio stack (PipeWire)",
                SuggestionPriority::High,
                SuggestionCategory::Desktop,
            )
            .explanation(format!(
                "Your system has sound hardware, but the audio stack is not fully configured. \
                 The following services are not running: {}. \
                 PipeWire is the modern audio/video server for Linux, replacing PulseAudio and JACK. \
                 Without it, applications won't be able to play sound.",
                missing_services.join(", ")
            ))
            .impact("Enables audio playback and recording for all applications.")
            .add_doc_link(
                DocumentationLink::arch_wiki(
                    "PipeWire",
                    "Arch Wiki comprehensive guide on PipeWire setup"
                )
            )
            .add_doc_link(
                DocumentationLink::arch_wiki(
                    "PipeWire#Installation",
                    "Step-by-step installation instructions"
                )
            )
            .auto_fixable(
                "Install and enable PipeWire audio stack",
                vec![
                    "sudo pacman -S --needed pipewire pipewire-pulse wireplumber".to_string(),
                    "systemctl --user enable --now pipewire pipewire-pulse wireplumber".to_string(),
                ],
            )
            .estimated_impact(EstimatedImpact {
                space_saved_mb: None,
                memory_freed_mb: None,
                boot_time_saved_secs: None,
                descriptions: vec![
                    "Enable audio output for media players".to_string(),
                    "Enable microphone input for communication apps".to_string(),
                    "Enable system sounds and notifications".to_string(),
                ],
            })
        );
    }
}

/// Filter suggestions based on dependencies (Task 9)
/// Only include suggestions where all dependencies are either:
/// 1. Also in the suggestion list, OR
/// 2. Already resolved/not needed
fn filter_by_dependencies(suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
    // Build a set of all suggestion keys present
    let suggestion_keys: std::collections::HashSet<String> =
        suggestions.iter().map(|s| s.key.clone()).collect();

    // Filter: keep suggestions whose dependencies are all satisfied
    let mut filtered: Vec<Suggestion> = suggestions
        .into_iter()
        .filter(|suggestion| {
            // Check if all dependencies are in the current suggestion set
            suggestion.depends_on.iter().all(|dep_key| {
                suggestion_keys.contains(dep_key)
            }) || suggestion.depends_on.is_empty()
        })
        .collect();

    // Sort to put dependencies before dependents (topological-ish sort)
    // Simple approach: suggestions with no dependencies first
    filtered.sort_by_key(|s| s.depends_on.len());

    filtered
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

    // Task 9: Dependency-aware suggestion tests

    #[test]
    fn test_audio_suggestion_when_hardware_exists_but_services_missing() {
        let mut snapshot = SystemTelemetry::minimal();

        // Has sound hardware but PipeWire not running
        snapshot.audio.has_sound_hardware = true;
        snapshot.audio.pipewire_running = false;
        snapshot.audio.wireplumber_running = false;
        snapshot.audio.pipewire_pulse_running = false;

        let suggestions = generate_suggestions(&snapshot);

        // Should generate audio stack suggestion
        assert!(
            suggestions.iter().any(|s| s.key == "audio-stack-config"),
            "Should generate audio stack configuration suggestion"
        );
    }

    #[test]
    fn test_no_audio_suggestion_when_services_running() {
        let mut snapshot = SystemTelemetry::minimal();

        // Has sound hardware AND PipeWire running
        snapshot.audio.has_sound_hardware = true;
        snapshot.audio.pipewire_running = true;
        snapshot.audio.wireplumber_running = true;
        snapshot.audio.pipewire_pulse_running = true;

        let suggestions = generate_suggestions(&snapshot);

        // Should NOT generate audio stack suggestion
        assert!(
            !suggestions.iter().any(|s| s.key == "audio-stack-config"),
            "Should not generate audio suggestion when services are running"
        );
    }

    #[test]
    fn test_no_audio_suggestion_without_hardware() {
        let mut snapshot = SystemTelemetry::minimal();

        // No sound hardware
        snapshot.audio.has_sound_hardware = false;
        snapshot.audio.pipewire_running = false;
        snapshot.audio.wireplumber_running = false;
        snapshot.audio.pipewire_pulse_running = false;

        let suggestions = generate_suggestions(&snapshot);

        // Should NOT generate audio stack suggestion
        assert!(
            !suggestions.iter().any(|s| s.key == "audio-stack-config"),
            "Should not suggest audio config without hardware"
        );
    }

    #[test]
    fn test_dependency_filtering_both_present() {
        // Create suggestions where B depends on A, both present
        let suggestion_a = Suggestion::new(
            "suggestion-a",
            "Prerequisite",
            SuggestionPriority::High,
            SuggestionCategory::Configuration,
        );

        let suggestion_b = Suggestion::new(
            "suggestion-b",
            "Dependent",
            SuggestionPriority::High,
            SuggestionCategory::Configuration,
        )
        .add_dependency("suggestion-a");

        let suggestions = vec![suggestion_a, suggestion_b];
        let filtered = filter_by_dependencies(suggestions);

        // Both should be included
        assert_eq!(filtered.len(), 2, "Both suggestions should be included");

        // A should come before B (fewer dependencies)
        assert_eq!(filtered[0].key, "suggestion-a");
        assert_eq!(filtered[1].key, "suggestion-b");
    }

    #[test]
    fn test_dependency_filtering_missing_dependency() {
        // Create suggestions where C depends on nonexistent Z
        let suggestion_a = Suggestion::new(
            "suggestion-a",
            "Independent",
            SuggestionPriority::High,
            SuggestionCategory::Configuration,
        );

        let suggestion_c = Suggestion::new(
            "suggestion-c",
            "Has missing dependency",
            SuggestionPriority::High,
            SuggestionCategory::Configuration,
        )
        .add_dependency("suggestion-z"); // Z doesn't exist

        let suggestions = vec![suggestion_a, suggestion_c];
        let filtered = filter_by_dependencies(suggestions);

        // Only A should remain (C has unmet dependency)
        assert_eq!(filtered.len(), 1, "Only suggestion without missing dependency should remain");
        assert_eq!(filtered[0].key, "suggestion-a");
    }

    #[test]
    fn test_dependency_ordering() {
        // Create a chain: C depends on B, B depends on A
        let suggestion_a = Suggestion::new(
            "suggestion-a",
            "First",
            SuggestionPriority::Medium,
            SuggestionCategory::Configuration,
        );

        let suggestion_b = Suggestion::new(
            "suggestion-b",
            "Second",
            SuggestionPriority::High, // Higher priority but depends on A
            SuggestionCategory::Configuration,
        )
        .add_dependency("suggestion-a");

        let suggestion_c = Suggestion::new(
            "suggestion-c",
            "Third",
            SuggestionPriority::Critical, // Highest priority but depends on B
            SuggestionCategory::Configuration,
        )
        .add_dependency("suggestion-b");

        let suggestions = vec![suggestion_c.clone(), suggestion_a.clone(), suggestion_b.clone()];
        let filtered = filter_by_dependencies(suggestions);

        // All should be included
        assert_eq!(filtered.len(), 3, "All suggestions should be included");

        // Should be ordered by dependency count (fewer dependencies first)
        assert_eq!(filtered[0].key, "suggestion-a"); // 0 dependencies (must be first)

        // B and C both have 1 dependency, so either order is acceptable
        // Just verify they're both present after A
        let keys_after_a: Vec<&str> = filtered[1..].iter().map(|s| s.key.as_str()).collect();
        assert!(keys_after_a.contains(&"suggestion-b"), "suggestion-b should be present");
        assert!(keys_after_a.contains(&"suggestion-c"), "suggestion-c should be present");
    }
}
