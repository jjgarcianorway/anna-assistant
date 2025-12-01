//! Knowledge Command v5.4.0 - Installed Objects Only
//!
//! Shows what Anna knows about objects installed on THIS system.
//! Fundamental rule: Never show non-installed objects here.
//!
//! v5.4.0: Full descriptions (no truncation).
//! - Usage: "runs" is total observed since daemon start (not per day)
//! - Errors: 24h window, explicitly stated
//!
//! Sections:
//! - [INSTALLED] Count of installed objects by category
//! - Category blocks with installed objects only

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    KnowledgeCategory, KnowledgeObject, KnowledgeStore, ErrorIndex,
    get_description,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge overview command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge".bold());
    println!("{}", THIN_SEP);
    println!();

    let store = KnowledgeStore::load();
    let error_index = ErrorIndex::load();

    // [INSTALLED]
    print_installed_section(&store);

    // Category blocks - installed objects only
    print_category_block(&store, &error_index, KnowledgeCategory::Editor, "EDITORS");
    print_category_block(&store, &error_index, KnowledgeCategory::Terminal, "TERMINALS");
    print_category_block(&store, &error_index, KnowledgeCategory::Shell, "SHELLS");
    print_category_block(&store, &error_index, KnowledgeCategory::Compositor, "COMPOSITORS");
    print_category_block(&store, &error_index, KnowledgeCategory::Wm, "WINDOW MANAGERS");
    print_category_block(&store, &error_index, KnowledgeCategory::Browser, "BROWSERS");
    print_category_block(&store, &error_index, KnowledgeCategory::Service, "SERVICES");
    print_category_block(&store, &error_index, KnowledgeCategory::Tool, "TOOLS");

    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl knowledge <name>' for details on a specific object.");
    println!("  'annactl knowledge stats' for coverage metrics.");
    println!();

    Ok(())
}

fn print_installed_section(store: &KnowledgeStore) {
    println!("{}", "[INSTALLED]".cyan());

    let categories = [
        (KnowledgeCategory::Editor, "Editors"),
        (KnowledgeCategory::Terminal, "Terminals"),
        (KnowledgeCategory::Shell, "Shells"),
        (KnowledgeCategory::Compositor, "Compositors"),
        (KnowledgeCategory::Wm, "Window Mgrs"),
        (KnowledgeCategory::Browser, "Browsers"),
        (KnowledgeCategory::Service, "Services"),
        (KnowledgeCategory::Tool, "Tools"),
    ];

    // Count only INSTALLED objects per category
    for (cat, label) in &categories {
        let count = store
            .objects
            .values()
            .filter(|o| o.category == *cat && o.installed)
            .count();

        // Only show categories with installed objects
        if count > 0 {
            println!("  {:<14} {}", format!("{}:", label), count);
        }
    }

    // Total installed
    let total_installed = store.objects.values().filter(|o| o.installed).count();
    println!("  {:<14} {}", "Total:", total_installed);

    println!();
}

fn print_category_block(
    store: &KnowledgeStore,
    error_index: &ErrorIndex,
    category: KnowledgeCategory,
    label: &str,
) {
    // Filter to INSTALLED objects only
    let objects: Vec<&KnowledgeObject> = store
        .objects
        .values()
        .filter(|o| o.category == category && o.installed)
        .collect();

    // Skip empty categories entirely
    if objects.is_empty() {
        return;
    }

    println!("{}", format!("[{}]", label).cyan());

    // Normalize and deduplicate objects by lowercase name
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut unique_objects: Vec<&KnowledgeObject> = Vec::new();

    for obj in &objects {
        let normalized = obj.name.to_lowercase();
        if !seen_names.contains(&normalized) {
            seen_names.insert(normalized);
            unique_objects.push(obj);
        }
    }

    // Sort by name
    unique_objects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    for obj in &unique_objects {
        print_object_line(obj, error_index, category == KnowledgeCategory::Service);
    }

    println!();
}

fn print_object_line(obj: &KnowledgeObject, error_index: &ErrorIndex, is_service: bool) {
    // v5.4.0: No name truncation - show full names
    let name = &obj.name;
    let errors_24h = get_object_errors_24h(error_index, &obj.name);

    // v5.4.0: Full descriptions (no truncation)
    let desc = get_description(&obj.name).unwrap_or_default();

    // For services, show state
    // For commands, show if we have usage data
    let usage_info = if is_service {
        format_service_state(obj)
    } else {
        format_usage_info(obj)
    };

    // Error indicator (only show if errors exist)
    let err_indicator = if errors_24h > 0 {
        format!(" [{}]", format!("{} errs", errors_24h).red())
    } else {
        String::new()
    };

    if desc.is_empty() {
        println!("  {:<18} {}{}", name, usage_info, err_indicator);
    } else {
        println!(
            "  {:<18} {:<12} {}{}",
            name, usage_info,
            desc.dimmed(),
            err_indicator
        );
    }
}

fn format_service_state(obj: &KnowledgeObject) -> String {
    if let Some(active) = obj.service_active {
        if active {
            "running".green().to_string()
        } else {
            "stopped".to_string()
        }
    } else {
        "installed".to_string()
    }
}

fn format_usage_info(obj: &KnowledgeObject) -> String {
    // Show if we have observed usage, but don't pretend it's "per day"
    // since usage_count is lifetime total
    if obj.usage_count > 0 {
        format!("{} runs", obj.usage_count)
    } else {
        "installed".to_string()
    }
}

fn get_object_errors_24h(error_index: &ErrorIndex, name: &str) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let cutoff = now.saturating_sub(86400);

    let obj_errors = error_index
        .get_object_errors(name)
        .or_else(|| error_index.get_object_errors(&name.to_lowercase()));

    if let Some(obj) = obj_errors {
        obj.logs
            .iter()
            .filter(|log| log.timestamp >= cutoff && log.severity.is_error())
            .count() as u64
    } else {
        0
    }
}
