//! Specialist learning capture (v0.0.401).
//!
//! Captures lessons from specialist interactions (escalations, LLM self-healing)
//! to improve Anna's responses over time.

use anna_shared::facts::{FactKey, FactsStore};
use anna_shared::facts_types::FactSource;
use anna_shared::probe_learning::{ProbeLearningStore, QueryCategory};
use anna_shared::rpc::{ProbeResult, SpecialistDomain};
use anna_shared::specialist_learning::{
    detect_pattern_category, PatternCategory, SpecialistLesson, SpecialistLearningStore,
    SolutionType,
};
use anna_shared::specialist_patterns::{extract_generic_pattern, get_pattern_hint};
use anna_shared::specialist_recipes::try_learn_from_specialist;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, warn};

/// Parse domain string to SpecialistDomain enum
/// v0.0.406: Updated to handle all 10 domains
pub fn parse_domain(s: &str) -> SpecialistDomain {
    match s.to_lowercase().as_str() {
        "system" => SpecialistDomain::System,
        "boot" => SpecialistDomain::Boot,
        "services" | "service" => SpecialistDomain::Services,
        "network" => SpecialistDomain::Network,
        "storage" => SpecialistDomain::Storage,
        "packages" => SpecialistDomain::Packages,
        "audio" | "sound" => SpecialistDomain::Audio,
        "display" | "graphics" | "gpu" => SpecialistDomain::Display,
        "desktop" | "de" | "wm" => SpecialistDomain::Desktop,
        "security" => SpecialistDomain::Security,
        _ => SpecialistDomain::System,
    }
}

/// Map specialist pattern category to probe learning query category
fn map_pattern_to_query_category(pattern: Option<PatternCategory>) -> QueryCategory {
    match pattern {
        Some(PatternCategory::ConfigCheck) => QueryCategory::General,
        Some(PatternCategory::ConfigEdit) => QueryCategory::General,
        Some(PatternCategory::ServiceAction) => QueryCategory::Services,
        Some(PatternCategory::PackageQuery) => QueryCategory::Packages,
        Some(PatternCategory::DiskAnalysis) => QueryCategory::Storage,
        Some(PatternCategory::ProcessQuery) => QueryCategory::SystemHealth,
        Some(PatternCategory::Other) => QueryCategory::General,
        None => QueryCategory::General,
    }
}

/// Capture a lesson from specialist interaction
///
/// Called after:
/// - LLM self-healing succeeds
/// - Senior escalation succeeds
pub fn capture_lesson(
    user_request: &str,
    domain: &str,
    answer: &str,
    probe_results: &[ProbeResult],
    solution_type: SolutionType,
    confidence: u8,
) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Extract effective probes (those that succeeded)
    let effective_probes: Vec<String> = probe_results
        .iter()
        .filter(|p| p.exit_code == 0)
        .map(|p| p.command.clone())
        .collect();

    // Detect pattern category for generic learning
    let pattern_category = detect_pattern_category(user_request);
    let category = map_pattern_to_query_category(pattern_category);

    // v0.0.401: Extract generic pattern if applicable
    let generic_pattern = extract_generic_pattern(user_request, answer, &effective_probes);

    let lesson = SpecialistLesson {
        id: format!("lesson-{}", now),
        query_pattern: user_request.to_string(),
        domain: parse_domain(domain),
        category,
        issues_fixed: vec![],
        solution_type,
        effective_probes,
        answer_template: answer.to_string(),
        confidence,
        success_count: 1,
        learned_at: now,
        last_success_at: now,
        generic_pattern,
    };

    // Load, record, and save the lesson
    let mut store = SpecialistLearningStore::load();
    let lesson_category = lesson.category.clone();
    let lesson_domain = lesson.domain.clone();
    let probes_to_boost = lesson.effective_probes.clone();

    if store.record_lesson(lesson) {
        if let Err(e) = store.save() {
            warn!("Failed to save specialist lesson: {}", e);
        } else {
            info!(
                "Captured specialist lesson (total: {}, pending: {})",
                store.lesson_count(),
                store.pending_count()
            );

            // v0.0.401: Boost probes in probe_learning store
            if !probes_to_boost.is_empty() {
                let mut probe_store = ProbeLearningStore::load();
                probe_store.boost_specialist_probes(lesson_category, &probes_to_boost, 3);
                let _ = probe_store.save();
            }

            // v0.0.401: Extract facts from the answer
            store_extracted_facts(answer, &lesson_domain);

            // v0.0.401: Try to create a recipe from this lesson
            for (_, lesson) in &store.lessons {
                if lesson.confidence >= 70 {
                    let result = try_learn_from_specialist(lesson);
                    if result.learned {
                        info!("Created recipe from specialist lesson: {:?}", result.recipe_id);
                        break; // Only create one recipe per capture
                    }
                }
            }
        }
    }
}

/// Get a subtle learning hint for user-facing messages
/// Checks both keyword matches and generic pattern matches
pub fn get_learning_hint(query: &str) -> Option<String> {
    let store = SpecialistLearningStore::load();
    // First try generic pattern matching (more specific)
    if let Some(hint) = get_pattern_hint(&store, query) {
        return Some(hint);
    }
    // Fall back to keyword-based hints
    store.get_learning_hint(query)
}

/// v0.0.401: Extract facts from specialist answers using pattern matching
/// Returns list of (key, value) pairs extracted
pub fn extract_facts_from_answer(answer: &str, domain: &SpecialistDomain) -> Vec<(FactKey, String)> {
    let mut facts = vec![];
    let answer_lower = answer.to_lowercase();

    // Package installed patterns: "X is installed", "found X installed"
    if let Ok(re) = Regex::new(r"(?i)(\w+)\s+is\s+installed") {
        for cap in re.captures_iter(answer) {
            if let Some(pkg) = cap.get(1) {
                let pkg_name = pkg.as_str().to_lowercase();
                if pkg_name.len() > 2 && !["the", "it", "this"].contains(&pkg_name.as_str()) {
                    facts.push((FactKey::InstalledPackage(pkg_name.clone()), "true".to_string()));
                }
            }
        }
    }

    // Desktop environment: "running Hyprland", "using KDE", "on GNOME"
    if let Ok(re) = Regex::new(r"(?i)(?:running|using|on)\s+(hyprland|kde|gnome|xfce|sway|i3|bspwm)") {
        if let Some(cap) = re.captures(answer) {
            if let Some(de) = cap.get(1) {
                facts.push((FactKey::Desktop, de.as_str().to_lowercase()));
            }
        }
    }

    // Package manager patterns
    if answer_lower.contains("pacman") || answer_lower.contains("arch linux") {
        facts.push((FactKey::PackageManager, "pacman".to_string()));
    } else if answer_lower.contains("apt") || answer_lower.contains("debian") || answer_lower.contains("ubuntu") {
        facts.push((FactKey::PackageManager, "apt".to_string()));
    }

    // Init system
    if answer_lower.contains("systemd") || answer_lower.contains("systemctl") {
        facts.push((FactKey::InitSystem, "systemd".to_string()));
    }

    // GPU presence
    if answer_lower.contains("nvidia") || answer_lower.contains("amd gpu") || answer_lower.contains("radeon") {
        facts.push((FactKey::GpuPresent, "true".to_string()));
    }

    // Service existence patterns
    if *domain == SpecialistDomain::System {
        if let Ok(re) = Regex::new(r"(?i)(\w+\.service)\s+(?:is\s+)?(?:active|running|enabled)") {
            for cap in re.captures_iter(answer) {
                if let Some(svc) = cap.get(1) {
                    facts.push((FactKey::UnitExists(svc.as_str().to_lowercase()), "true".to_string()));
                }
            }
        }
    }

    facts
}

/// v0.0.401: Store extracted facts from specialist answers
fn store_extracted_facts(answer: &str, domain: &SpecialistDomain) {
    let facts = extract_facts_from_answer(answer, domain);
    if facts.is_empty() { return; }

    let mut store = FactsStore::load();
    let mut stored = 0;
    for (key, value) in facts {
        store.upsert_verified(key, value.into(), FactSource::SpecialistAnswer, 80);
        stored += 1;
    }
    if stored > 0 {
        let _ = store.save();
        info!("Extracted {} facts from specialist answer", stored);
    }
}

/// v0.0.401: Record user feedback for a query
/// This boosts confidence in matching lessons if helpful, or decreases if not
pub fn record_user_feedback(query: &str, helpful: bool) -> Option<String> {
    let mut store = SpecialistLearningStore::load();
    let lessons = store.find_lessons(query);

    if lessons.is_empty() {
        return None;
    }

    // Get the best matching lesson ID
    let lesson_id = lessons[0].id.clone();

    // Update the lesson in the store
    if let Some(lesson) = store.lessons.get_mut(&lesson_id) {
        if helpful {
            // Boost confidence and success count
            lesson.success_count += 1;
            lesson.confidence = (lesson.confidence + 5).min(100);
            lesson.last_success_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            info!("User feedback: +helpful for lesson {}", lesson_id);
        } else {
            // Decrease confidence
            lesson.confidence = lesson.confidence.saturating_sub(10);
            info!("User feedback: -helpful for lesson {} (confidence now {})", lesson_id, lesson.confidence);
        }

        if let Err(e) = store.save() {
            warn!("Failed to save feedback: {}", e);
        }

        let msg = if helpful {
            "Thanks! I'll remember this pattern.".to_string()
        } else {
            "I'll try a different approach next time.".to_string()
        };
        return Some(msg);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_domain() {
        assert_eq!(parse_domain("system"), SpecialistDomain::System);
        assert_eq!(parse_domain("Network"), SpecialistDomain::Network);
        assert_eq!(parse_domain("boot"), SpecialistDomain::Boot);
        assert_eq!(parse_domain("services"), SpecialistDomain::Services);
        assert_eq!(parse_domain("audio"), SpecialistDomain::Audio);
        assert_eq!(parse_domain("sound"), SpecialistDomain::Audio);
        assert_eq!(parse_domain("display"), SpecialistDomain::Display);
        assert_eq!(parse_domain("desktop"), SpecialistDomain::Desktop);
        assert_eq!(parse_domain("unknown"), SpecialistDomain::System);
    }

    #[test]
    fn test_map_pattern_category() {
        assert_eq!(
            map_pattern_to_query_category(Some(PatternCategory::ServiceAction)),
            QueryCategory::Services
        );
        assert_eq!(
            map_pattern_to_query_category(Some(PatternCategory::DiskAnalysis)),
            QueryCategory::Storage
        );
        assert_eq!(
            map_pattern_to_query_category(None),
            QueryCategory::General
        );
    }
}
