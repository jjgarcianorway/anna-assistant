//! Knowledge Detail Command v5.4.1 - Full Object Profile
//!
//! Shows a complete sysadmin-grade profile of a single object.
//! Exception: Can show info about non-installed objects when explicitly asked.
//!
//! v5.4.1: Truthful metrics - usage_count is actual invocations, not process samples.
//!
//! Sections shown only when they have meaningful data:
//! - [IDENTITY] Name, category, description, ecosystem
//! - [INSTALLATION] Package, version, paths, service state
//! - [RELATIONSHIPS] Related objects (e.g., aquamarine -> hyprland)
//! - [USAGE] Only if observed running (since daemon start)
//! - [ERRORS] Only if errors exist (24h window)
//! - [SECURITY] Only if intrusions detected

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeStore, KnowledgeObject, ErrorIndex, IntrusionIndex,
    ServiceIndex, ObjectType, KnowledgeCategory,
    format_duration_ms, format_time_ago, format_bytes, truncate_str,
    format_timestamp, get_description, get_relationship, get_ecosystem,
};

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
            print_identity_section(obj);
            print_installation_section(obj, &service_index);
            print_relationships_section(obj);
            print_usage_section(obj);
            print_errors_section(obj, &error_index);
            print_security_section(obj, &intrusion_index);
        }
        None => {
            // Object not found - show what we know from metadata
            print_unknown_object_section(name, &error_index);
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

fn print_identity_section(obj: &KnowledgeObject) {
    println!("{}", "[IDENTITY]".cyan());
    println!("  Name:        {}", obj.name.bold());

    // Description from metadata or generated
    let desc = get_description(&obj.name)
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_description(obj));
    println!("  Description: {}", desc);

    println!("  Category:    {}", obj.category.as_str());

    // Object types (only if meaningful)
    let types: Vec<_> = obj.object_types.iter().map(|t| t.as_str()).collect();
    if !types.is_empty() && types.len() > 1 {
        println!("  Types:       {}", types.join(", "));
    }

    // Ecosystem
    if let Some(ecosystem) = get_ecosystem(&obj.name) {
        println!("  Ecosystem:   {}", ecosystem.cyan());
    }

    println!();
}

fn print_installation_section(obj: &KnowledgeObject, service_index: &ServiceIndex) {
    println!("{}", "[INSTALLATION]".cyan());

    // Installed status
    let installed_str = if obj.installed {
        "yes".green().to_string()
    } else {
        "no".red().to_string()
    };
    println!("  Installed:  {}", installed_str);

    // Package info (only if available)
    if let Some(pkg) = &obj.package_name {
        if let Some(ver) = &obj.package_version {
            println!("  Package:    {} ({})", pkg, ver);
        } else {
            println!("  Package:    {}", pkg);
        }
    }

    // Binary path (simplified - just show first one)
    if !obj.paths.is_empty() {
        println!("  Binary:     {}", obj.paths[0]);
        if obj.paths.len() > 1 {
            println!("              ({} more paths)", obj.paths.len() - 1);
        }
    } else if let Some(path) = &obj.binary_path {
        println!("  Binary:     {}", path);
    }

    // Config paths (only if available)
    if !obj.config_paths.is_empty() {
        if obj.config_paths.len() == 1 {
            println!("  Config:     {}", obj.config_paths[0]);
        } else {
            println!("  Config:     {} paths", obj.config_paths.len());
        }
    }

    // Service state (only for services)
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
            println!("  Service:    {} ({}/{})", unit, active_str, enabled_str);
        }
    }

    println!();
}

fn print_relationships_section(obj: &KnowledgeObject) {
    let relationship = get_relationship(&obj.name);
    let ecosystem = get_ecosystem(&obj.name);

    // Skip if no relationships
    if relationship.is_none() && ecosystem.is_none() {
        return;
    }

    println!("{}", "[RELATIONSHIPS]".cyan());

    if let Some(rel) = relationship {
        println!(
            "  {} {} {}",
            obj.name,
            rel.relationship_type.as_str(),
            rel.related_to.cyan()
        );
    }

    if let Some(eco) = ecosystem {
        println!("  Part of {} ecosystem", eco);
    }

    println!();
}

fn print_usage_section(obj: &KnowledgeObject) {
    // Skip entirely if no usage data observed
    let has_usage = obj.usage_count > 0
        || obj.total_cpu_time_ms > 0
        || obj.total_mem_bytes_peak > 0
        || obj.first_seen_at > 0;

    if !has_usage {
        // For commands with no observed usage, show a simple message
        if obj.installed && !obj.object_types.contains(&ObjectType::Service) {
            println!("{}", "[USAGE]".cyan());
            println!("  {}", "No runtime usage observed yet".dimmed());
            println!();
        }
        return;
    }

    println!("{}", "[USAGE]".cyan());
    println!("  {}", "(since daemon start)".dimmed());

    // Handle long-running processes vs regular commands
    let is_daemon = obj.object_types.contains(&ObjectType::Service)
        || (obj.total_cpu_time_ms > 0 && obj.usage_count == 0);

    if is_daemon {
        println!("  Type:       daemon (long-running process)");
    } else if obj.usage_count > 0 {
        println!("  Runs:       {} observed", obj.usage_count);
    }

    // First/last seen
    if obj.first_seen_at > 0 {
        println!("  First seen: {}", format_time_ago(obj.first_seen_at));
    }

    if obj.last_seen_at > 0 && obj.last_seen_at != obj.first_seen_at {
        println!("  Last seen:  {}", format_time_ago(obj.last_seen_at));
    }

    // Resource usage (only if non-zero)
    if obj.total_cpu_time_ms > 0 {
        println!("  CPU time:   {} (total)", format_duration_ms(obj.total_cpu_time_ms));
    }

    if obj.total_mem_bytes_peak > 0 {
        println!("  Peak memory: {}", format_bytes(obj.total_mem_bytes_peak));
    }

    println!();
}

fn print_errors_section(obj: &KnowledgeObject, error_index: &ErrorIndex) {
    let obj_errors = error_index
        .get_object_errors(&obj.name)
        .or_else(|| error_index.get_object_errors(&obj.name.to_lowercase()));

    let has_errors = obj_errors
        .as_ref()
        .map(|e| !e.logs.is_empty())
        .unwrap_or(false);

    // Skip section entirely if no errors
    if !has_errors {
        return;
    }

    println!("{}", "[ERRORS]".cyan());
    println!("  {}", "(last 24 hours)".dimmed());

    if let Some(errors) = obj_errors {
        let errors_24h = errors.errors_24h();
        let warnings_24h = errors.warnings_24h();

        if !errors_24h.is_empty() {
            println!("  Errors:   {}", errors_24h.len());
        }
        if !warnings_24h.is_empty() {
            println!("  Warnings: {}", warnings_24h.len());
        }

        // Show recent error messages
        if !errors_24h.is_empty() {
            println!("  Recent:");
            for entry in errors_24h.iter().rev().take(3) {
                let ts = format_timestamp(entry.timestamp);
                let msg = truncate_str(&entry.message, 45);
                println!("    [{}] {}", ts, msg);
            }
            if errors_24h.len() > 3 {
                println!("    ({} more)", errors_24h.len() - 3);
            }
        }
    }

    println!();
}

fn print_security_section(obj: &KnowledgeObject, intrusion_index: &IntrusionIndex) {
    let obj_intrusions = intrusion_index.get_object_intrusions(&obj.name);

    let has_intrusions = obj_intrusions
        .as_ref()
        .map(|i| i.total_events() > 0)
        .unwrap_or(false);

    // Skip section entirely if no security events
    if !has_intrusions {
        return;
    }

    println!("{}", "[SECURITY]".red());
    println!("  {}", "(authentication failures detected)".dimmed());

    if let Some(intrusions) = obj_intrusions {
        println!("  Failures: {}", intrusions.total_events());

        // Recent events
        let recent: Vec<_> = intrusions.events.iter().rev().take(3).collect();
        if !recent.is_empty() {
            println!("  Recent:");
            for event in recent {
                let ts = format_timestamp(event.timestamp);
                let ip = event.source_ip.as_deref().unwrap_or("-");
                println!("    [{}] {} from {}", ts, event.intrusion_type.as_str(), ip);
            }
        }
    }

    println!();
}

fn print_unknown_object_section(name: &str, error_index: &ErrorIndex) {
    // Check if we have a description from metadata
    if let Some(desc) = get_description(name) {
        println!("{}", "[IDENTITY]".cyan());
        println!("  Name:        {}", name.bold());
        println!("  Description: {}", desc);

        if let Some(ecosystem) = get_ecosystem(name) {
            println!("  Ecosystem:   {}", ecosystem.cyan());
        }

        if let Some(rel) = get_relationship(name) {
            println!("  Related to:  {} ({})", rel.related_to, rel.relationship_type.as_str());
        }

        println!();
        println!("{}", "[INSTALLATION]".cyan());
        println!("  Installed:  {}", "no".red());
        println!("  Status:     not found on this system");
        println!();
    }

    // Check if we have errors for this object anyway
    let obj_errors = error_index
        .get_object_errors(name)
        .or_else(|| error_index.get_object_errors(&name.to_lowercase()));

    if let Some(errors) = obj_errors {
        if !errors.logs.is_empty() {
            println!("{}", "[ERRORS]".cyan());
            println!("  {}", "(logs found but object not indexed)".dimmed());
            for entry in errors.errors_only().iter().rev().take(3) {
                let ts = format_timestamp(entry.timestamp);
                println!("    [{}] {}", ts, truncate_str(&entry.message, 50));
            }
            println!();
        }
    }

    // If we have no info at all
    if get_description(name).is_none() {
        println!("  Anna has no knowledge about '{}'.", name);
        println!();
        println!("  This could mean:");
        println!("    - Not installed on this system");
        println!("    - Not yet discovered by the daemon");
        println!("    - Unknown object type");
        println!();
    }
}

fn generate_description(obj: &KnowledgeObject) -> String {
    let category_desc = match obj.category {
        KnowledgeCategory::Editor => "Text editor",
        KnowledgeCategory::Terminal => "Terminal emulator",
        KnowledgeCategory::Shell => "Command shell",
        KnowledgeCategory::Compositor => "Wayland compositor",
        KnowledgeCategory::Wm => "Window manager",
        KnowledgeCategory::Browser => "Web browser",
        KnowledgeCategory::Service => "System service",
        KnowledgeCategory::Tool => "System tool",
        KnowledgeCategory::Unknown => "Application",
    };

    if obj.object_types.contains(&ObjectType::Service) {
        format!("{} (systemd service)", category_desc)
    } else if obj.object_types.contains(&ObjectType::Command) {
        format!("{} (command-line)", category_desc)
    } else {
        category_desc.to_string()
    }
}
