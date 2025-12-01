//! Knowledge Detail Command v5.2.4 - Full Object Profile
//!
//! Shows a complete sysadmin-grade profile of a single object.
//!
//! Sections:
//! - [BASIC INFO] Name, type, package, description
//! - [INSTALLATION] Installed status, paths, service state
//! - [USAGE] Runs, CPU time, memory (with proper formatting)
//! - [ERRORS] Recent errors with timestamps
//! - [SECURITY] Auth failures if applicable
//! - [NOTES] Version info

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeStore, KnowledgeObject, ErrorIndex, IntrusionIndex,
    ServiceIndex, ObjectType, KnowledgeCategory,
    format_duration_ms, format_time_ago, format_bytes, truncate_str,
    format_timestamp,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge detail command for a specific object
pub async fn run(name: &str) -> Result<()> {
    println!();
    println!("{}", format!("  Anna Knowledge: {}", name).bold());
    println!("{}", THIN_SEP);
    println!();

    let store = KnowledgeStore::load();
    let error_index = ErrorIndex::load();
    let intrusion_index = IntrusionIndex::load();
    let service_index = ServiceIndex::load();

    // Try to find the object
    let obj = find_object(&store, name);

    match obj {
        Some(obj) => {
            print_basic_info_section(obj);
            print_installation_section(obj, &service_index);
            print_usage_section(obj);
            print_errors_section(obj, &error_index);
            print_security_section(obj, &intrusion_index);
            print_notes_section();
        }
        None => {
            // Object not found - check for errors anyway
            print_not_found_section(name, &error_index);
        }
    }

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn find_object<'a>(store: &'a KnowledgeStore, name: &str) -> Option<&'a KnowledgeObject> {
    // Exact match first
    if let Some(obj) = store.objects.get(name) {
        return Some(obj);
    }

    // Case-insensitive match
    let lower = name.to_lowercase();
    for (key, obj) in &store.objects {
        if key.to_lowercase() == lower {
            return Some(obj);
        }
    }

    // Package name match
    for obj in store.objects.values() {
        if let Some(pkg) = &obj.package_name {
            if pkg.to_lowercase() == lower {
                return Some(obj);
            }
        }
    }

    // Service unit match
    for obj in store.objects.values() {
        if let Some(unit) = &obj.service_unit {
            if unit.to_lowercase() == lower || unit.to_lowercase().starts_with(&lower) {
                return Some(obj);
            }
        }
    }

    None
}

fn print_basic_info_section(obj: &KnowledgeObject) {
    println!("{}", "[BASIC INFO]".cyan());
    println!("  Name:        {}", obj.name);
    println!("  Type:        {}", obj.category.as_str());

    if let Some(pkg) = &obj.package_name {
        println!("  Package:     {}", pkg);
    }

    // Object types
    let types: Vec<_> = obj.object_types.iter().map(|t| t.as_str()).collect();
    if !types.is_empty() {
        println!("  Types:       {}", types.join(", "));
    }

    // Description (generated from type and name)
    let desc = generate_description(obj);
    println!("  Description: {}", desc);

    // Inventory sources
    if !obj.inventory_source.is_empty() {
        println!("  Sources:     {}", obj.inventory_source.join(", "));
    }

    println!();
}

fn print_installation_section(obj: &KnowledgeObject, service_index: &ServiceIndex) {
    println!("{}", "[INSTALLATION]".cyan());

    let installed_str = if obj.installed {
        "yes".green().to_string()
    } else {
        "no".red().to_string()
    };
    println!("  Installed:    {}", installed_str);

    if let Some(pkg) = &obj.package_name {
        println!("  Package:      {}", pkg);
    }

    if let Some(ver) = &obj.package_version {
        println!("  Version:      {}", ver);
    }

    if let Some(path) = &obj.binary_path {
        println!("  Binary path:  {}", path);
    }

    // Config paths
    if !obj.config_paths.is_empty() {
        println!("  Config paths:");
        for path in obj.config_paths.iter().take(5) {
            println!("    - {}", path);
        }
        if obj.config_paths.len() > 5 {
            println!("    (... {} more)", obj.config_paths.len() - 5);
        }
    }

    // Service state if this is a service or has a related service
    if let Some(unit) = &obj.service_unit {
        if let Some(state) = service_index.services.get(unit) {
            let active_str = match state.active_state {
                anna_common::ActiveState::Active => "running".green().to_string(),
                anna_common::ActiveState::Failed => "failed".red().to_string(),
                _ => state.active_state.as_str().to_string(),
            };
            let enabled_str = match state.enabled_state {
                anna_common::EnabledState::Enabled => "enabled".to_string(),
                anna_common::EnabledState::Disabled => "disabled".to_string(),
                anna_common::EnabledState::Masked => "masked".yellow().to_string(),
                _ => state.enabled_state.as_str().to_string(),
            };
            println!("  Service:      {} ({}/{})", unit, active_str, enabled_str);
        }
    }

    println!();
}

fn print_usage_section(obj: &KnowledgeObject) {
    println!("{}", "[USAGE]".cyan());

    // Handle long-running processes vs regular commands
    let is_daemon = obj.object_types.contains(&ObjectType::Service)
        || obj.total_cpu_time_ms > 0
        || obj.total_mem_bytes_peak > 0;

    if obj.usage_count == 0 && is_daemon {
        println!("  Runs observed: daemon (long-running)");
    } else {
        println!("  Runs observed: {}", obj.usage_count);
    }

    // First/last seen
    if obj.first_seen_at > 0 {
        println!("  First seen:    {}", format_time_ago(obj.first_seen_at));
    }

    if obj.last_seen_at > 0 {
        println!("  Last seen:     {}", format_time_ago(obj.last_seen_at));
    }

    // CPU time with proper formatting
    if obj.total_cpu_time_ms > 0 {
        println!("  Total CPU time: {}", format_duration_ms(obj.total_cpu_time_ms));
    }

    // Memory with proper formatting
    if obj.total_mem_bytes_peak > 0 {
        println!("  Max RSS:       {}", format_bytes(obj.total_mem_bytes_peak));
    }

    println!();
}

fn print_errors_section(obj: &KnowledgeObject, error_index: &ErrorIndex) {
    println!("{}", "[ERRORS]".cyan());

    let obj_errors = error_index
        .get_object_errors(&obj.name)
        .or_else(|| error_index.get_object_errors(&obj.name.to_lowercase()));

    if let Some(errors) = obj_errors {
        let errors_24h = errors.errors_24h();
        let warnings_24h = errors.warnings_24h();

        if errors_24h.is_empty() && warnings_24h.is_empty() {
            println!("  No errors or warnings found in the last 24h");
        } else {
            println!("  Errors (24h):   {}", errors_24h.len());
            println!("  Warnings (24h): {}", warnings_24h.len());

            // Recent log entries
            if !errors_24h.is_empty() {
                println!();
                println!("  Recent errors:");
                for entry in errors_24h.iter().rev().take(5) {
                    let ts = format_timestamp(entry.timestamp);
                    let unit = entry.unit.as_deref().unwrap_or("-");
                    println!("    {} [{}] {}", ts, unit, truncate_str(&entry.message, 50));
                }
                if errors_24h.len() > 5 {
                    println!("    (... {} more, use journalctl for full view)", errors_24h.len() - 5);
                }
            }
        }
    } else {
        println!("  No errors or warnings found for this object in the last 24h");
    }

    println!();
}

fn print_security_section(obj: &KnowledgeObject, intrusion_index: &IntrusionIndex) {
    // Only show if there are intrusion events
    let obj_intrusions = intrusion_index.get_object_intrusions(&obj.name);

    if let Some(intrusions) = obj_intrusions {
        if intrusions.total_events() > 0 {
            println!("{}", "[SECURITY]".red());
            println!("  Auth failures:  {}", intrusions.total_events());

            // Recent events
            let recent: Vec<_> = intrusions.events.iter().rev().take(3).collect();
            if !recent.is_empty() {
                println!("  Recent events:");
                for event in recent {
                    let ts = format_timestamp(event.timestamp);
                    let ip = event.source_ip.as_deref().unwrap_or("-");
                    println!("    {} [{}] from {}", ts, event.intrusion_type.as_str(), ip);
                }
            }
            println!();
        }
    }
}

fn print_notes_section() {
    println!("{}", "[NOTES]".cyan());
    println!("  Data collected by anna daemon (v{} Knowledge Core).", VERSION);
    println!();
}

fn print_not_found_section(name: &str, error_index: &ErrorIndex) {
    // Check if we have errors for this object anyway
    let obj_errors = error_index
        .get_object_errors(name)
        .or_else(|| error_index.get_object_errors(&name.to_lowercase()));

    if let Some(errors) = obj_errors {
        if !errors.logs.is_empty() {
            println!("{}", "[ERRORS]".red());
            println!("  No object indexed, but logs found:");
            for entry in errors.errors_only().iter().rev().take(5) {
                let ts = format_timestamp(entry.timestamp);
                println!("    [{}] {}", ts, truncate_str(&entry.message, 50));
            }
            println!();
        }
    }

    println!();
    println!("  Anna has no knowledge about '{}' yet.", name);
    println!();
    println!("  It might not be installed, or it has not been observed in use.");
    println!();

    // Priority indexing display
    println!("{}", "[PRIORITY INDEX]".yellow());
    println!("  Indexing priority increased for: {}", name.cyan());
    println!("  Next scan will prioritize discovering this object.");
    println!("  Run 'annactl status' again in ~5 minutes to check if indexed.");
    println!();
}

fn generate_description(obj: &KnowledgeObject) -> String {
    // Generate a short description based on category and type
    let category_desc = match obj.category {
        KnowledgeCategory::Editor => "Text editor",
        KnowledgeCategory::Terminal => "Terminal emulator",
        KnowledgeCategory::Shell => "Command shell",
        KnowledgeCategory::Compositor => "Wayland compositor",
        KnowledgeCategory::Wm => "Window manager",
        KnowledgeCategory::Browser => "Web browser",
        KnowledgeCategory::Service => "System service",
        KnowledgeCategory::Tool => "System tool",
        KnowledgeCategory::Unknown => "Unknown application",
    };

    // Add more context based on object types
    if obj.object_types.contains(&ObjectType::Service) {
        format!("{} (systemd service)", category_desc)
    } else if obj.object_types.contains(&ObjectType::Command) {
        format!("{} (command-line)", category_desc)
    } else {
        category_desc.to_string()
    }
}
