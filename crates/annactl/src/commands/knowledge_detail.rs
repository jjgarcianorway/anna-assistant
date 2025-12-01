//! Knowledge Detail Command v5.2.5 - Full Object Profile
//!
//! Shows a complete sysadmin-grade profile of a single object.
//! Exception: Can show info about non-installed objects when explicitly asked.
//!
//! Sections:
//! - [IDENTITY] Name, category, description, ecosystem
//! - [INSTALLATION] Package, version, paths, service state
//! - [RELATIONSHIPS] Related objects (e.g., aquamarine -> hyprland)
//! - [USAGE] Runs, CPU time, memory
//! - [ERRORS] Recent errors with timestamps
//! - [SECURITY] Auth failures if applicable
//! - [NOTES] Additional context

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeStore, KnowledgeObject, ErrorIndex, IntrusionIndex,
    ServiceIndex, ObjectType, KnowledgeCategory,
    format_duration_ms, format_time_ago, format_bytes, truncate_str,
    format_timestamp, get_description, get_relationship, get_ecosystem,
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
            print_identity_section(obj);
            print_installation_section(obj, &service_index);
            print_relationships_section(obj);
            print_usage_section(obj);
            print_errors_section(obj, &error_index);
            print_security_section(obj, &intrusion_index);
            print_notes_section(obj);
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
    println!("  Category:    {}", obj.category.as_str());

    // Object types
    let types: Vec<_> = obj.object_types.iter().map(|t| t.as_str()).collect();
    if !types.is_empty() {
        println!("  Types:       {}", types.join(", "));
    }

    // Description from metadata or generated
    let desc = get_description(&obj.name)
        .map(|s| s.to_string())
        .unwrap_or_else(|| generate_description(obj));
    println!("  Description: {}", desc);

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
    println!("  Installed:    {}", installed_str);

    // Package info
    if let Some(pkg) = &obj.package_name {
        println!("  Package:      {}", pkg);
    }

    if let Some(ver) = &obj.package_version {
        println!("  Version:      {}", ver);
    }

    // Install time
    if let Some(installed_at) = obj.installed_at {
        println!("  Installed at: {}", format_time_ago(installed_at));
    }

    // Binary paths
    if !obj.paths.is_empty() {
        if obj.paths.len() == 1 {
            println!("  Binary:       {}", obj.paths[0]);
        } else {
            println!("  Binaries:     {}", obj.paths.len());
            for path in obj.paths.iter().take(3) {
                println!("    - {}", path);
            }
            if obj.paths.len() > 3 {
                println!("    (... {} more)", obj.paths.len() - 3);
            }
        }
    } else if let Some(path) = &obj.binary_path {
        println!("  Binary:       {}", path);
    }

    // Config paths (if any)
    if !obj.config_paths.is_empty() {
        println!("  Config:");
        for path in obj.config_paths.iter().take(3) {
            println!("    - {}", path);
        }
        if obj.config_paths.len() > 3 {
            println!("    (... {} more)", obj.config_paths.len() - 3);
        }
    }

    // Service state
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
        } else {
            println!("  Service:      {}", unit);
        }
    }

    // Inventory source
    if !obj.inventory_source.is_empty() {
        println!("  Discovered:   {}", obj.inventory_source.join(", "));
    }

    println!();
}

fn print_relationships_section(obj: &KnowledgeObject) {
    // Get relationship info
    let relationship = get_relationship(&obj.name);
    let ecosystem = get_ecosystem(&obj.name);

    if relationship.is_none() && ecosystem.is_none() {
        return; // Skip section if no relationships
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
        println!("  Part of:      {} ecosystem", eco);
    }

    println!();
}

fn print_usage_section(obj: &KnowledgeObject) {
    // Skip if no usage data
    if obj.usage_count == 0 && obj.total_cpu_time_ms == 0 && obj.total_mem_bytes_peak == 0 {
        return;
    }

    println!("{}", "[USAGE]".cyan());

    // Handle long-running processes vs regular commands
    let is_daemon = obj.object_types.contains(&ObjectType::Service)
        || (obj.total_cpu_time_ms > 0 && obj.usage_count == 0);

    if obj.usage_count == 0 && is_daemon {
        println!("  Runs:         daemon (long-running)");
    } else if obj.usage_count > 0 {
        println!("  Runs:         {}/day", obj.usage_count);
    }

    // First/last seen
    if obj.first_seen_at > 0 {
        println!("  First seen:   {}", format_time_ago(obj.first_seen_at));
    }

    if obj.last_seen_at > 0 && obj.last_seen_at != obj.first_seen_at {
        println!("  Last seen:    {}", format_time_ago(obj.last_seen_at));
    }

    // CPU time
    if obj.total_cpu_time_ms > 0 {
        println!("  CPU time:     {}", format_duration_ms(obj.total_cpu_time_ms));
    }

    // Memory
    if obj.total_mem_bytes_peak > 0 {
        println!("  Peak memory:  {}", format_bytes(obj.total_mem_bytes_peak));
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

    if !has_errors {
        return; // Skip section if no errors
    }

    println!("{}", "[ERRORS]".cyan());

    if let Some(errors) = obj_errors {
        let errors_24h = errors.errors_24h();
        let warnings_24h = errors.warnings_24h();

        if !errors_24h.is_empty() {
            println!("  Errors (24h):   {}", errors_24h.len());
        }
        if !warnings_24h.is_empty() {
            println!("  Warnings (24h): {}", warnings_24h.len());
        }

        // Recent log entries
        if !errors_24h.is_empty() {
            println!("  Recent:");
            for entry in errors_24h.iter().rev().take(5) {
                let ts = format_timestamp(entry.timestamp);
                let msg = truncate_str(&entry.message, 45);
                println!("    [{}] {}", ts, msg);
            }
            if errors_24h.len() > 5 {
                println!("    (... {} more)", errors_24h.len() - 5);
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

    if !has_intrusions {
        return; // Skip section if no security events
    }

    println!("{}", "[SECURITY]".red());

    if let Some(intrusions) = obj_intrusions {
        println!("  Auth failures: {}", intrusions.total_events());

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

fn print_notes_section(obj: &KnowledgeObject) {
    println!("{}", "[NOTES]".cyan());

    // Wiki reference
    if let Some(wiki) = &obj.wiki_ref {
        println!("  Wiki:   {}", wiki);
    }

    println!("  Source: anna daemon v{}", VERSION);
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
        println!("  Installed:    {}", "no".red());
        println!("  Status:       not found on this system");
        println!();
    }

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

    // If we have no info at all
    if get_description(name).is_none() {
        println!("  Anna has no knowledge about '{}'.", name);
        println!();
        println!("  Possible reasons:");
        println!("    - Not installed on this system");
        println!("    - Not yet discovered by the daemon");
        println!("    - Unknown/unrecognized object");
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
