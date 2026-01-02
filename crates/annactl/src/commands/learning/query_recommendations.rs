//! Query recommendation analysis.
//!
//! v0.0.328: Show what probes Anna would recommend for a query.
//! v0.0.334: Shows whether learning is active for this query.

use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::ui::{
    colors, kv, print_footer, print_hint, print_label, print_section_header, print_step,
    print_title,
};
use anyhow::Result;

/// v0.0.328: Show what probes Anna would recommend for a query
/// v0.0.334: Shows whether learning is active for this query
pub fn show_query_recommendations(query: &str) -> Result<()> {
    let store = ProbeLearningStore::load_with_decay();
    let category = QueryCategory::from_query(query);

    println!();
    print_title("Query Analysis");
    println!();

    print_section_header("input");
    kv("query", &format!("\"{}\"", query));
    kv("category", &format!("{:?}", category));

    // v0.0.334: Show if learning would be used for this query
    let confidence = store.confidence_factor();
    if store.should_use_learning() {
        kv(
            "learning",
            &format!(
                "{}Active{} ({:.0}% confidence)",
                colors::OK,
                colors::RESET,
                confidence * 100.0
            ),
        );
    } else {
        kv(
            "learning",
            &format!(
                "{}Inactive{} ({:.0}% - need 30%)",
                colors::WARN,
                colors::RESET,
                confidence * 100.0
            ),
        );
    }
    println!();

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);
    print_section_header("category recommendations");
    if !category_recs.is_empty() {
        for (probe_id, score) in category_recs.iter().take(5) {
            let score_color = if *score >= 0.7 {
                colors::OK
            } else if *score >= 0.5 {
                colors::WARN
            } else {
                colors::DIM
            };
            print_step(&format!(
                "{} {}{:.0}%{}",
                probe_id,
                score_color,
                score * 100.0,
                colors::RESET
            ));
        }
    } else {
        print_hint("No category-based recommendations yet");
    }
    println!();

    // Get keyword-based suggestions
    let keyword_suggestions = store.suggest_probes_for_query(query);
    print_section_header("keyword suggestions");
    if !keyword_suggestions.is_empty() {
        for (probe_id, count) in keyword_suggestions.iter().take(5) {
            print_step(&format!("{} (matches: {})", probe_id, count));
        }
    } else {
        print_hint("No keyword-based suggestions yet");
    }
    println!();

    // Check for known bad combinations
    let probes: Vec<String> = category_recs.iter().map(|(p, _)| p.clone()).collect();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        print_label(
            "warn",
            &format!("Similar query had issues: {}", reason),
            colors::WARN,
        );
        println!();
    }

    print_footer();
    Ok(())
}
