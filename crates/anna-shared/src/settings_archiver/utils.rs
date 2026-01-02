// v0.0.658: Settings Archiver Utilities (Phase 234)
// Utility functions for archiver

/// Check if query is about archiver
pub fn is_archiver_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("archiver") || lower.contains("archive settings") || lower.contains("backup settings")
}

/// Fun fact about archiver
pub fn archiver_fun_fact() -> &'static str {
    "Anna's settings archivers create safe backups of your configs!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_archiver_query() {
        assert!(is_archiver_query("settings archiver"));
        assert!(!is_archiver_query("hello world"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = archiver_fun_fact();
        assert!(fact.contains("archiver"));
    }
}
