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

        // Dependencies information (Task 9: dependency-aware UX)
        if !suggestion.depends_on.is_empty() {
            println!("   ⚠️  Prerequisites:");
            println!("      This suggestion assumes you first address:");
            for dep_key in &suggestion.depends_on {
                // Try to find the dependency suggestion to show its title
                if let Some(dep) = suggestions.iter().find(|s| &s.key == dep_key) {
                    println!("      • {}", dep.title);
                } else {
                    println!("      • {}", dep_key);
                }
            }
            println!();
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

/// Generate suggestions from real system telemetry (Task 8: Deep Caretaker v0.1)
/// Uses local telemetry collection and rule-based suggestion engine
pub fn generate_suggestions_from_telemetry() -> Vec<Suggestion> {
    use anna_common::telemetry::SystemTelemetry;
    use anna_common::suggestion_engine;

    // Collect telemetry snapshot (Task 8: fast, local, read-only)
    let snapshot = SystemTelemetry::collect();

    // Generate suggestions using rule-based engine
    suggestion_engine::generate_suggestions(&snapshot)
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
