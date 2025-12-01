//! Knowledge Command v5.2.4 - Overview of What Anna Knows
//!
//! Shows a high-level overview of Anna's knowledge by category.
//! No stats, no log scanner internals, no inventory progress.
//!
//! Sections:
//! - [SUMMARY] One line per major category with counts
//! - Category blocks (EDITORS, TERMINALS, SHELLS, etc.)

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeCategory, KnowledgeObject, KnowledgeStore, ErrorIndex,
    truncate_str,
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

    // [SUMMARY]
    print_summary_section(&store);

    // Category blocks
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
    println!("  Use 'annactl knowledge stats' for detailed coverage statistics.");
    println!();

    Ok(())
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

    let counts = store.count_by_category();
    for (cat, label) in &categories {
        let count = counts.get(cat).unwrap_or(&0);
        println!("  {:<14} {}", format!("{}:", label), count);
    }

    println!();
}

fn print_category_block(
    store: &KnowledgeStore,
    error_index: &ErrorIndex,
    category: KnowledgeCategory,
    label: &str,
) {
    let objects: Vec<&KnowledgeObject> = store
        .objects
        .values()
        .filter(|o| o.category == category)
        .collect();

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
        let installed = if obj.installed { "yes" } else { "no" };
        let runs = obj.usage_count;
        let errors = get_object_error_count(error_index, &obj.name);

        let err_str = if errors > 0 {
            errors.to_string()
        } else {
            "-".to_string()
        };

        // Compact format: name inst:yes runs:N errs:N
        println!(
            "  {:<14} inst:{:<3} runs:{:<5} errs:{}",
            truncate_str(&obj.name, 14),
            installed,
            runs,
            err_str
        );
    }

    println!();
}

fn get_object_error_count(error_index: &ErrorIndex, name: &str) -> u64 {
    error_index
        .get_object_errors(name)
        .or_else(|| error_index.get_object_errors(&name.to_lowercase()))
        .map(|e| e.total_errors())
        .unwrap_or(0)
}
