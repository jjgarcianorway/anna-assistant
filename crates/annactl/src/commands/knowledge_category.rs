//! Knowledge Category Command v5.5.0
//!
//! Lists objects in a specific category (editors, terminals, shells, etc).
//!
//! Usage:
//!   annactl knowledge editors
//!   annactl knowledge shells
//!   annactl knowledge terminals
//!   annactl knowledge browsers
//!   annactl knowledge compositors
//!   annactl knowledge services
//!   annactl knowledge tools

use anyhow::Result;
use owo_colors::OwoColorize;

use anna_common::{
    KnowledgeStore, KnowledgeObject, ObjectType, ServiceIndex,
    KnowledgeCategory, get_description,
};

const THIN_SEP: &str = "------------------------------------------------------------";

/// Map user-facing category names to internal categories
fn map_category(name: &str) -> Option<KnowledgeCategory> {
    match name.to_lowercase().as_str() {
        "editors" | "editor" => Some(KnowledgeCategory::Editor),
        "terminals" | "terminal" => Some(KnowledgeCategory::Terminal),
        "shells" | "shell" => Some(KnowledgeCategory::Shell),
        "browsers" | "browser" => Some(KnowledgeCategory::Browser),
        "compositors" | "compositor" => Some(KnowledgeCategory::Compositor),
        "wms" | "wm" => Some(KnowledgeCategory::Wm),
        "services" | "service" => Some(KnowledgeCategory::Service),
        "tools" | "tool" => Some(KnowledgeCategory::Tool),
        _ => None,
    }
}

/// Run the knowledge category command
pub async fn run(category_name: &str) -> Result<()> {
    let category = match map_category(category_name) {
        Some(c) => c,
        None => {
            println!();
            println!("  Unknown category: '{}'", category_name);
            println!();
            println!("  Available categories:");
            println!("    editors, terminals, shells, browsers,");
            println!("    compositors, wms, services, tools");
            println!();
            return Ok(());
        }
    };

    let store = KnowledgeStore::load();
    let service_index = ServiceIndex::load();

    // Collect objects in this category that are installed
    let mut objects: Vec<&KnowledgeObject> = store
        .objects
        .values()
        .filter(|obj| {
            matches_category(obj, &category) && is_installed_or_active(obj, &category)
        })
        .collect();

    // Sort by name
    objects.sort_by(|a, b| a.name.cmp(&b.name));

    println!();
    println!("{}", format!("  Anna Knowledge: {}", category.display_name()).bold());
    println!("{}", THIN_SEP);
    println!();

    if objects.is_empty() {
        println!("  No installed {} found.", category_name.to_lowercase());
        println!();
    } else {
        println!("  {} {} installed:", objects.len(), category_name.to_lowercase());
        println!();

        for obj in &objects {
            print_object_summary(obj, &service_index);
        }
    }

    println!("{}", THIN_SEP);
    println!();
    println!("  'annactl knowledge <name>' for full profile.");
    println!();

    Ok(())
}

/// Check if an object matches the requested category
fn matches_category(obj: &KnowledgeObject, category: &KnowledgeCategory) -> bool {
    // For services, check ObjectType::Service since category might be Unknown
    if *category == KnowledgeCategory::Service {
        return obj.object_types.contains(&ObjectType::Service);
    }

    // For other categories, check the actual category field
    obj.category == *category
}

/// Check if an object should be shown (installed or active service)
fn is_installed_or_active(obj: &KnowledgeObject, category: &KnowledgeCategory) -> bool {
    // For services, check if the service exists (not installed flag)
    if *category == KnowledgeCategory::Service ||
       obj.object_types.contains(&ObjectType::Service) {
        return obj.service_unit.is_some();
    }

    // For commands/packages, check installed flag
    obj.installed
}

/// Print a summary line for an object
fn print_object_summary(obj: &KnowledgeObject, service_index: &ServiceIndex) {
    let name = &obj.name;

    // Get description
    let desc = get_description(name)
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            if obj.object_types.contains(&ObjectType::Service) {
                "System service".to_string()
            } else if obj.object_types.contains(&ObjectType::Command) {
                "Command-line tool".to_string()
            } else {
                "Application".to_string()
            }
        });

    // Truncate description if needed
    let desc_display = if desc.len() > 40 {
        format!("{}...", &desc[..37])
    } else {
        desc
    };

    // For services, show state
    if obj.object_types.contains(&ObjectType::Service) {
        if let Some(unit) = &obj.service_unit {
            if let Some(state) = service_index.services.get(unit) {
                let state_str = match state.active_state {
                    anna_common::ActiveState::Active => "running".green().to_string(),
                    anna_common::ActiveState::Failed => "failed".red().to_string(),
                    anna_common::ActiveState::Inactive => "inactive".dimmed().to_string(),
                    _ => state.active_state.as_str().to_string(),
                };
                println!(
                    "  {} {}",
                    format!("{:<20}", name).cyan(),
                    format!("[{}] {}", state_str, desc_display)
                );
                return;
            }
        }
    }

    // For packages/commands, show version if available
    let version_str = obj.package_version
        .as_ref()
        .map(|v| format!(" ({})", v))
        .unwrap_or_default();

    println!(
        "  {} {}{}",
        format!("{:<20}", name).cyan(),
        desc_display,
        version_str.dimmed()
    );
}
