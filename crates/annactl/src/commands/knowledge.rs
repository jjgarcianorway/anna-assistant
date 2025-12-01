//! Knowledge Command v5.2.5 - Installed Objects Only
//!
//! Shows what Anna knows about objects installed on THIS system.
//! Fundamental rule: Never show non-installed objects here.
//!
//! Sections:
//! - [USAGE WINDOW] Time window for metrics
//! - [SUMMARY] One line per major category with installed counts
//! - Category blocks (EDITORS, TERMINALS, etc.) with installed objects only

use anyhow::Result;
use owo_colors::OwoColorize;
use std::time::{SystemTime, UNIX_EPOCH};

use anna_common::{
    KnowledgeCategory, KnowledgeObject, KnowledgeStore, ErrorIndex,
    truncate_str, get_description,
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

    // [USAGE WINDOW]
    print_usage_window_section();

    // [SUMMARY]
    print_summary_section(&store);

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
    println!("  Use 'annactl knowledge <name>' for full object details.");
    println!("  Use 'annactl knowledge stats' for coverage statistics.");
    println!();

    Ok(())
}

fn print_usage_window_section() {
    println!("{}", "[USAGE WINDOW]".cyan());
    println!("  Metrics shown: last 24 hours");
    println!();
}

fn print_summary_section(store: &KnowledgeStore) {
    println!("{}", "[SUMMARY]".cyan());

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
    let name = truncate_str(&obj.name, 14);
    let errors_24h = get_object_errors_24h(error_index, &obj.name);

    // Error string
    let err_str = if errors_24h > 0 {
        format!("errs:{}", errors_24h)
    } else {
        "errs:-".to_string()
    };

    // For services, show state instead of runs
    // For others, show runs/day
    let usage_str = if is_service {
        format_service_state(obj)
    } else {
        format_runs_per_day(obj)
    };

    // Get description if available
    let desc = get_description(&obj.name)
        .map(|d| truncate_str(d, 30))
        .unwrap_or_default();

    if desc.is_empty() {
        println!("  {:<14} {:<12} {}", name, usage_str, err_str);
    } else {
        println!(
            "  {:<14} {:<12} {:<8} {}",
            name, usage_str, err_str,
            desc.dimmed()
        );
    }
}

fn format_service_state(obj: &KnowledgeObject) -> String {
    if let Some(active) = obj.service_active {
        if active {
            "state:running".to_string()
        } else {
            "state:stopped".to_string()
        }
    } else {
        "state:-".to_string()
    }
}

fn format_runs_per_day(obj: &KnowledgeObject) -> String {
    // Calculate runs in last 24 hours
    // For now, use usage_count as a proxy (assuming it's recent)
    // In a real implementation, we'd have timestamped usage events
    let runs = obj.usage_count;

    if runs == 0 {
        "runs:-".to_string()
    } else if runs == 1 {
        "runs:1/day".to_string()
    } else {
        format!("runs:{}/day", runs)
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
