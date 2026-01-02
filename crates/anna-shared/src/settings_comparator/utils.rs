// v0.0.601: Settings Comparator Utilities (Phase 177)
// Utility functions for settings comparison

use super::types::CompareResult;

/// Format comparison result
pub fn format_compare_result(result: &CompareResult) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "Comparing: {} → {}\n",
        result.source_label, result.target_label
    ));
    output.push_str(&format!("  Added: {}\n", result.added));
    output.push_str(&format!("  Removed: {}\n", result.removed));
    output.push_str(&format!("  Modified: {}\n", result.modified));
    output.push_str(&format!("  Unchanged: {}\n", result.unchanged));

    for diff in &result.diffs {
        if diff.is_change() {
            output.push_str(&format!(
                "  [{}] {} ({})\n",
                diff.diff_type, diff.key, diff.category
            ));
        }
    }

    output
}

/// Check if query is about comparator
pub fn is_comparator_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("compare")
        || lower.contains("diff")
        || lower.contains("difference")
}

/// Fun fact about comparator
pub fn comparator_fun_fact() -> &'static str {
    "Anna can compare settings between snapshots to show exactly what changed!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_comparator_query() {
        assert!(is_comparator_query("compare settings"));
        assert!(is_comparator_query("show diff"));
        assert!(!is_comparator_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = comparator_fun_fact();
        assert!(fact.contains("compare"));
    }
}
