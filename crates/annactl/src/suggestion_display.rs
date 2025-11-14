//! Suggestion Display - Format and show suggestions to the user
//!
//! Phase 5.1: Conversational UX
//! Display suggestions with proper formatting and Arch Wiki links

use anna_common::suggestions::*;
use anna_common::display::UI;

/// Display suggestions in a user-friendly format
pub fn display_suggestions(suggestions: &[&Suggestion]) {
    let ui = UI::auto();

    if suggestions.is_empty() {
        println!();
        ui.success("✓ Great! Your system is in good shape.");
        ui.info("I don't have any immediate suggestions.");
        println!();
        return;
    }

    println!();
    ui.section_header("💡", "Top Suggestions for Your System");
    ui.info(&format!("I've identified {} priority items for your attention:", suggestions.len()));
    println!();

    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("{}. {} {}", i + 1, priority_emoji(suggestion.priority), suggestion.title);
        println!();

        // Explanation
        println!("   {}", suggestion.explanation);
        println!();

        // Impact
        if !suggestion.impact.is_empty() {
            println!("   💪 Impact: {}", suggestion.impact);
            println!();
        }

        // Estimated metrics if available
        if let Some(ref impact) = suggestion.estimated_impact {
            if let Some(space_saved) = impact.space_saved_mb {
                println!("   📊 Est. space saved: {:.1} GB", space_saved / 1024.0);
            }
            if let Some(memory_freed) = impact.memory_freed_mb {
                println!("   📊 Est. memory freed: {:.0} MB", memory_freed);
            }
            if let Some(boot_saved) = impact.boot_time_saved_secs {
                println!("   📊 Est. boot time saved: {:.1}s", boot_saved);
            }
            if !impact.descriptions.is_empty() {
                for desc in &impact.descriptions {
                    println!("   📊 {}", desc);
                }
            }
            println!();
        }

        // Documentation links
        if !suggestion.docs.is_empty() {
            println!("   📚 Learn more:");
            for doc in &suggestion.docs {
                let source_icon = match doc.source {
                    DocSource::ArchWiki => "🏛️",
                    DocSource::OfficialDocs => "📖",
                    DocSource::ManPage => "📄",
                };
                println!("      {} {}", source_icon, doc.description);
                println!("         {}", doc.url);
            }
            println!();
        }

        // Fix information
        if suggestion.auto_fixable {
            if let Some(ref fix_desc) = suggestion.fix_description {
                println!("   🔧 Fix: {}", fix_desc);
                if !suggestion.fix_commands.is_empty() {
                    println!("      Commands:");
                    for cmd in &suggestion.fix_commands {
                        println!("         {}", cmd);
                    }
                }
                println!();
            }
        }

        // Separator between suggestions
        if i < suggestions.len() - 1 {
            println!("   ───────────────────────────────────────");
            println!();
        }
    }

    println!("To apply a suggestion, ask me:");
    println!("  \"Can you help me fix [issue]?\"");
    println!();
    println!("To hide a suggestion you don't want:");
    println!("  \"Discard the [issue] suggestion\"");
    println!();
}

/// Get emoji for priority level
fn priority_emoji(priority: SuggestionPriority) -> &'static str {
    match priority {
        SuggestionPriority::Critical => "🔴",
        SuggestionPriority::High => "🟠",
        SuggestionPriority::Medium => "🟡",
        SuggestionPriority::Low => "🟢",
        SuggestionPriority::Info => "ℹ️",
    }
}

/// Generate suggestions from real system telemetry
pub fn generate_suggestions_from_telemetry() -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    // Query system telemetry
    match crate::system_query::query_system_telemetry() {
        Ok(telemetry) => {
            // Check pacman cache
            if telemetry.packages.cache_size_mb > 1000.0 {
                suggestions.push(common_suggestions::pacman_cache_cleanup(
                    telemetry.packages.cache_size_mb
                ));
            }

            // Check orphaned packages
            if telemetry.packages.orphaned > 0 {
                suggestions.push(common_suggestions::orphaned_packages(
                    telemetry.packages.orphaned as usize
                ));
            }

            // Check failed services
            if !telemetry.services.failed_units.is_empty() {
                let service_names: Vec<String> = telemetry.services.failed_units
                    .iter()
                    .map(|u| u.name.clone())
                    .collect();
                suggestions.push(common_suggestions::failed_services(service_names));
            }

            // Check disk space
            for disk in &telemetry.disks {
                if disk.usage_percent > 90.0 {
                    suggestions.push(Suggestion::new(
                        format!("disk-space-{}", disk.mount_point.replace('/', "-")),
                        format!("Critical disk space on {}", disk.mount_point),
                        SuggestionPriority::Critical,
                        SuggestionCategory::Disk,
                    )
                    .explanation(format!(
                        "The filesystem mounted at {} is {}% full ({:.1} GB used of {:.1} GB total). \
                         Systems become unstable when disk space is critically low.",
                        disk.mount_point,
                        disk.usage_percent,
                        disk.used_mb as f64 / 1024.0,
                        disk.total_mb as f64 / 1024.0
                    ))
                    .impact("Prevent system instability, application crashes, and data loss.")
                    .add_doc_link(
                        DocumentationLink::arch_wiki(
                            "System_maintenance#Clean_the_filesystem",
                            "Arch Wiki guide on cleaning filesystems"
                        )
                    ));
                } else if disk.usage_percent > 80.0 {
                    suggestions.push(Suggestion::new(
                        format!("disk-space-{}", disk.mount_point.replace('/', "-")),
                        format!("High disk usage on {}", disk.mount_point),
                        SuggestionPriority::Medium,
                        SuggestionCategory::Disk,
                    )
                    .explanation(format!(
                        "The filesystem at {} is {}% full. It's good practice to keep \
                         at least 15-20% free space for system operations.",
                        disk.mount_point, disk.usage_percent
                    ))
                    .impact("Maintain system performance and stability.")
                    .add_doc_link(
                        DocumentationLink::arch_wiki(
                            "System_maintenance#Clean_the_filesystem",
                            "Arch Wiki guide on filesystem maintenance"
                        )
                    ));
                }
            }

            // Check firewall
            if !telemetry.network.firewall_active && telemetry.hardware.machine_type == anna_common::telemetry::MachineType::Laptop {
                suggestions.push(Suggestion::new(
                    "firewall-inactive",
                    "Firewall is not active",
                    SuggestionPriority::High,
                    SuggestionCategory::Security,
                )
                .explanation(
                    "No firewall is currently active. Laptops that connect to various networks \
                     should have a firewall enabled for basic protection against network-based threats."
                )
                .impact("Improve security posture, especially on untrusted networks.")
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "Uncomplicated_Firewall",
                        "Arch Wiki guide on UFW (simple firewall)"
                    )
                )
                .auto_fixable(
                    "Install and enable UFW (Uncomplicated Firewall) with sensible defaults",
                    vec![
                        "sudo pacman -S ufw".to_string(),
                        "sudo ufw default deny incoming".to_string(),
                        "sudo ufw default allow outgoing".to_string(),
                        "sudo ufw enable".to_string(),
                        "sudo systemctl enable ufw".to_string(),
                    ],
                ));
            }

            // Check memory pressure
            if telemetry.memory.usage_percent > 90.0 {
                suggestions.push(Suggestion::new(
                    "high-memory-usage",
                    "Very high memory usage",
                    SuggestionPriority::High,
                    SuggestionCategory::Memory,
                )
                .explanation(format!(
                    "Memory usage is at {}%. The system has only {:.0} MB available out of {:.0} MB total. \
                     This can cause slowdowns and trigger the OOM killer.",
                    telemetry.memory.usage_percent,
                    telemetry.memory.available_mb,
                    telemetry.memory.total_mb
                ))
                .impact("Prevent application crashes and system freezes.")
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "Improving_performance#Reduce_memory_usage",
                        "Arch Wiki guide on reducing memory usage"
                    )
                ));
            }

            // Check if updates available
            if telemetry.packages.updates_available > 50 {
                suggestions.push(Suggestion::new(
                    "many-updates-available",
                    format!("{} package updates available", telemetry.packages.updates_available),
                    SuggestionPriority::Medium,
                    SuggestionCategory::Packages,
                )
                .explanation(format!(
                    "There are {} package updates available. Regular updates are important for \
                     security, stability, and new features. Arch is a rolling release distribution \
                     that works best with frequent updates.",
                    telemetry.packages.updates_available
                ))
                .impact("Improve security, fix bugs, and get new features.")
                .add_doc_link(
                    DocumentationLink::arch_wiki(
                        "System_maintenance#Upgrading_the_system",
                        "Arch Wiki guide on system upgrades"
                    )
                )
                .auto_fixable(
                    "Perform a full system upgrade",
                    vec!["sudo pacman -Syu".to_string()],
                ));
            }
        }
        Err(e) => {
            eprintln!("Warning: Could not query system telemetry: {}", e);
            // Fall back to basic suggestions
            suggestions.push(Suggestion::new(
                "telemetry-unavailable",
                "Unable to query system state",
                SuggestionPriority::Info,
                SuggestionCategory::Configuration,
            )
            .explanation(
                "I couldn't query your system state. This might be a temporary issue. \
                 Make sure the system is properly configured and I have necessary permissions."
            )
            .impact("Limited ability to provide specific suggestions."));
        }
    }

    suggestions
}

/// Display suggestions for conversational interface
pub fn show_suggestions_conversational() {
    let suggestions = generate_suggestions_from_telemetry();
    let mut engine = SuggestionEngine::new();

    for suggestion in suggestions {
        engine.add_suggestion(suggestion);
    }

    let top = engine.get_top_suggestions(5);
    display_suggestions(&top);
}
