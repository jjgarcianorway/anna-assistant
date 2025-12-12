//! Generic pattern matching for specialist learning (v0.0.401).
//! Extracts reusable patterns from specialist lessons.

use crate::specialist_learning::{
    detect_pattern_category, extract_target, GenericPattern, PatternVariable,
    SpecialistLearningStore,
};

/// Extract a generic pattern from a query and answer
/// Returns a pattern with {target} placeholder if applicable
pub fn extract_generic_pattern(
    query: &str,
    answer: &str,
    probes: &[String],
) -> Option<GenericPattern> {
    let category = detect_pattern_category(query)?;
    let target = extract_target(query)?;

    // Create probe templates by replacing target with placeholder
    let probe_templates: Vec<String> = probes
        .iter()
        .map(|p| p.replace(&target, "{target}"))
        .filter(|p| p.contains("{target}"))
        .collect();

    // Create answer template
    let answer_template = answer.replace(&target, "{target}");
    if !answer_template.contains("{target}") {
        return None;
    }

    Some(GenericPattern {
        category,
        variables: vec![PatternVariable {
            name: "target".to_string(),
            detection_hint: format!("app/service name like {}", target),
            example_values: vec![target],
        }],
        probe_templates,
        answer_template,
    })
}

/// Match a query against learned generic patterns
/// Returns (pattern, extracted_target) if matched
pub fn match_generic_pattern(
    store: &SpecialistLearningStore,
    query: &str,
) -> Option<(GenericPattern, String)> {
    let category = detect_pattern_category(query)?;
    let target = extract_target(query)?;

    // Find lessons with same category that have generic patterns
    if let Some(lesson_ids) = store.category_index.get(&category) {
        for id in lesson_ids {
            if let Some(lesson) = store.lessons.get(id) {
                if let Some(ref pattern) = lesson.generic_pattern {
                    // Check if this is a different target (reuse pattern)
                    if !pattern
                        .variables
                        .iter()
                        .any(|v| v.example_values.contains(&target))
                    {
                        return Some((pattern.clone(), target));
                    }
                }
            }
        }
    }
    None
}

/// Apply a generic pattern to generate probes/answer for a new target
pub fn apply_pattern(pattern: &GenericPattern, target: &str) -> (Vec<String>, String) {
    let probes = pattern
        .probe_templates
        .iter()
        .map(|t| t.replace("{target}", target))
        .collect();
    let answer = pattern.answer_template.replace("{target}", target);
    (probes, answer)
}

/// Get a learning hint that includes pattern matching info
pub fn get_pattern_hint(store: &SpecialistLearningStore, query: &str) -> Option<String> {
    if let Some((pattern, target)) = match_generic_pattern(store, query) {
        let example = pattern
            .variables
            .first()
            .and_then(|v| v.example_values.first())
            .map(|e| e.as_str())
            .unwrap_or("similar apps");
        return Some(format!("I know how to handle {} like {}", target, example));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specialist_learning::PatternCategory;

    #[test]
    fn test_extract_generic_pattern() {
        let query = "check hyprland config";
        let answer = "Your hyprland configuration is at ~/.config/hyprland/hyprland.conf";
        let probes = vec!["cat ~/.config/hyprland/hyprland.conf".to_string()];

        let pattern = extract_generic_pattern(query, answer, &probes);
        assert!(pattern.is_some());

        let p = pattern.unwrap();
        assert_eq!(p.category, PatternCategory::ConfigCheck);
        assert!(p.answer_template.contains("{target}"));
    }

    #[test]
    fn test_apply_pattern() {
        let pattern = GenericPattern {
            category: PatternCategory::ConfigCheck,
            variables: vec![PatternVariable {
                name: "target".to_string(),
                detection_hint: "app name".to_string(),
                example_values: vec!["hyprland".to_string()],
            }],
            probe_templates: vec!["cat ~/.config/{target}/*".to_string()],
            answer_template: "Your {target} config is here".to_string(),
        };

        let (probes, answer) = apply_pattern(&pattern, "vim");
        assert_eq!(probes[0], "cat ~/.config/vim/*");
        assert_eq!(answer, "Your vim config is here");
    }
}
