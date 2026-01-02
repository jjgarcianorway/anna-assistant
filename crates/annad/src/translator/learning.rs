//! Probe learning and recommendation functions.

use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use tracing::info;

/// v0.0.322: Get probe recommendations from learning store
/// v0.0.325: Also uses keyword-based suggestions
/// v0.0.327: Uses load_with_decay() for automatic decay
/// v0.0.333: Only returns recommendations if learning confidence is sufficient
pub(crate) fn get_probe_recommendations(query: &str) -> String {
    let store = ProbeLearningStore::load_with_decay();

    // v0.0.333: Check if we should trust the learning data
    if !store.should_use_learning() {
        info!(
            "Learning confidence too low ({:.0}%), skipping recommendations",
            store.confidence_factor() * 100.0
        );
        return String::new();
    }

    let category = QueryCategory::from_query(query);

    // Get category-based recommendations
    let category_recs = store.get_recommended_probes(&category);

    // v0.0.325: Get keyword-based suggestions
    let keyword_suggestions = store.suggest_probes_for_query(query);

    // Combine both sources, prioritizing keyword matches
    let mut combined: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

    // Add category recommendations (threshold based on confidence)
    let score_threshold = 0.5 + (store.confidence_factor() * 0.2); // 0.5-0.7 based on confidence
    for (probe_id, score) in &category_recs {
        if *score > score_threshold {
            combined.insert(probe_id.clone(), *score);
        }
    }

    // Boost probes that also match keywords
    for (probe_id, keyword_count) in &keyword_suggestions {
        let boost = (*keyword_count as f32 * 0.1).min(0.3); // Max 30% boost
        let entry = combined.entry(probe_id.clone()).or_insert(0.5);
        *entry = (*entry + boost).min(1.0);
    }

    // Sort by score
    let mut sorted: Vec<_> = combined.into_iter().collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let good_probes: Vec<String> = sorted
        .into_iter()
        .take(5) // Top 5
        .map(|(probe_id, score)| format!("{} ({:.0}%)", probe_id, score * 100.0))
        .collect();

    if good_probes.is_empty() {
        String::new()
    } else {
        info!(
            "Using learned probes (confidence {:.0}%): {}",
            store.confidence_factor() * 100.0,
            good_probes.join(", ")
        );
        good_probes.join(", ")
    }
}

/// v0.0.374: Filter probes that are known to fail for similar queries
pub(crate) fn filter_bad_combos(query: &str, probes: Vec<String>) -> Vec<String> {
    let store = ProbeLearningStore::load();
    if let Some(reason) = store.is_known_bad_combo(query, &probes) {
        // Log why we're filtering
        info!("Learning: avoiding probes due to past failure: {}", reason);
        // Remove probes that match the bad pattern
        let filtered: Vec<String> = probes
            .into_iter()
            .filter(|p| store.is_known_bad_combo(query, &[p.clone()]).is_none())
            .collect();
        if filtered.is_empty() {
            // Don't return empty - keep at least one probe
            vec!["memory_info".to_string()] // Safe default
        } else {
            filtered
        }
    } else {
        probes
    }
}
