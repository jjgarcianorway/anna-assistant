//! Knowledge Stats Command v5.2.4 - Detailed Coverage Statistics
//!
//! Shows detailed knowledge-related statistics and coverage.
//!
//! Sections:
//! - [COMMAND COVERAGE] Commands, packages, services tracked
//! - [KNOWLEDGE COVERAGE] Objects by category with names
//! - [DISCOVERY LATENCY] First/last object discovery times
//! - [USAGE HOTSPOTS] Most-used, heaviest processes
//! - [ERRORS BY OBJECT] Top error objects

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeCategory, KnowledgeStore, ErrorIndex,
    count_path_binaries, count_systemd_services,
    format_duration_secs, format_time_ago, format_percent, format_bytes,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Run the knowledge stats command
pub async fn run() -> Result<()> {
    println!();
    println!("{}", "  Anna Knowledge Statistics".bold());
    println!("{}", THIN_SEP);
    println!();

    let store = KnowledgeStore::load();
    let error_index = ErrorIndex::load();

    // [COMMAND COVERAGE]
    print_command_coverage_section(&store);

    // [KNOWLEDGE COVERAGE]
    print_knowledge_coverage_section(&store);

    // [DISCOVERY LATENCY]
    print_discovery_latency_section(&store);

    // [USAGE HOTSPOTS]
    print_usage_hotspots_section(&store);

    // [ERRORS BY OBJECT]
    print_errors_by_object_section(&error_index);

    println!("{}", THIN_SEP);
    println!();

    Ok(())
}

fn print_command_coverage_section(store: &KnowledgeStore) {
    println!("{}", "[COMMAND COVERAGE]".cyan());

    let total_path_cmds = count_path_binaries();
    let total_services = count_systemd_services();
    let (commands, packages, services) = store.count_by_type();

    // Commands with runs
    let commands_with_runs = store
        .objects
        .values()
        .filter(|o| o.usage_count > 0)
        .count();

    println!(
        "  Commands in PATH:   {}/{} ({})",
        commands,
        total_path_cmds,
        format_percent((commands as f64 / total_path_cmds.max(1) as f64) * 100.0)
    );
    println!("  Packages tracked:   {}", packages);
    println!(
        "  Services tracked:   {}/{} ({})",
        services,
        total_services,
        format_percent((services as f64 / total_services.max(1) as f64) * 100.0)
    );
    println!(
        "  Commands with runs: {}/{} ({})",
        commands_with_runs,
        commands,
        format_percent((commands_with_runs as f64 / commands.max(1) as f64) * 100.0)
    );

    println!();
}

fn print_knowledge_coverage_section(store: &KnowledgeStore) {
    println!("{}", "[KNOWLEDGE COVERAGE]".cyan());

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

    for (cat, label) in &categories {
        let objects: Vec<_> = store
            .objects
            .values()
            .filter(|o| o.category == *cat)
            .collect();

        if objects.is_empty() {
            println!("  {:<14} 0", format!("{}:", label));
        } else {
            // Get unique names (normalized)
            let mut names: Vec<String> = objects
                .iter()
                .map(|o| o.name.to_lowercase())
                .collect();
            names.sort();
            names.dedup();

            let names_str = if names.len() <= 3 {
                names.join(", ")
            } else {
                format!("{}, ... (+{})", names[..3].join(", "), names.len() - 3)
            };

            println!("  {:<14} {} ({})", format!("{}:", label), names.len(), names_str);
        }
    }

    println!();
}

fn print_discovery_latency_section(store: &KnowledgeStore) {
    println!("{}", "[DISCOVERY LATENCY]".cyan());

    // First object discovered (time since daemon start)
    let first_discovery = store
        .objects
        .values()
        .filter(|o| o.first_seen_at > 0)
        .map(|o| o.first_seen_at)
        .min();

    if let Some(first) = first_discovery {
        if first > store.created_at {
            let latency = first.saturating_sub(store.created_at);
            println!("  First object discovered: {} after daemon start", format_duration_secs(latency));
        } else {
            println!("  First object discovered: at startup");
        }
    } else {
        println!("  First object discovered: n/a");
    }

    // Last new object
    let last_new = store
        .objects
        .values()
        .map(|o| o.first_seen_at)
        .max()
        .unwrap_or(0);

    if last_new > 0 {
        println!("  Last new object:         {}", format_time_ago(last_new));
    } else {
        println!("  Last new object:         n/a");
    }

    println!();
}

fn print_usage_hotspots_section(store: &KnowledgeStore) {
    println!("{}", "[USAGE HOTSPOTS]".cyan());

    // Most-used command (by runs)
    let most_used = store
        .objects
        .values()
        .filter(|o| o.usage_count > 0)
        .max_by_key(|o| o.usage_count);

    if let Some(obj) = most_used {
        println!("  Most-used command:   {} ({} runs)", obj.name, obj.usage_count);
    } else {
        println!("  Most-used command:   n/a");
    }

    // Heaviest CPU process
    let heaviest_cpu = store
        .objects
        .values()
        .filter(|o| o.total_cpu_time_ms > 0)
        .max_by_key(|o| o.total_cpu_time_ms);

    if let Some(obj) = heaviest_cpu {
        println!(
            "  Heaviest CPU process: {} ({})",
            obj.name,
            format_duration_secs(obj.total_cpu_time_ms / 1000)
        );
    } else {
        println!("  Heaviest CPU process: n/a");
    }

    // Heaviest RAM process
    let heaviest_ram = store
        .objects
        .values()
        .filter(|o| o.total_mem_bytes_peak > 0)
        .max_by_key(|o| o.total_mem_bytes_peak);

    if let Some(obj) = heaviest_ram {
        println!("  Heaviest RAM process: {} ({})", obj.name, format_bytes(obj.total_mem_bytes_peak));
    } else {
        println!("  Heaviest RAM process: n/a");
    }

    println!();
}

fn print_errors_by_object_section(error_index: &ErrorIndex) {
    println!("{}", "[ERRORS BY OBJECT]".cyan());

    let top_errors = error_index.top_by_errors(5);

    if top_errors.is_empty() {
        println!("  No errors indexed");
    } else {
        println!("  Top error objects:");
        for (name, count) in &top_errors {
            println!("    {}: {} errors", name, count);
        }
    }

    println!();
}
