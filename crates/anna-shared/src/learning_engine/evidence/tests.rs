//! Tests for evidence cache system.

#[cfg(test)]
mod tests {
    use super::super::cache::EvidenceCache;
    use super::super::evidence_types::{EvidenceEntry, EvidenceType};
    use super::super::utils::{extract_keywords, infer_domain};

    #[test]
    fn test_evidence_entry_creation() {
        let entry =
            EvidenceEntry::probe_output("free", "Mem: 16000 8000 8000", "performance.memory");
        assert!(entry.id.starts_with("probe:free:"));
        assert_eq!(entry.evidence_type, EvidenceType::ProbeOutput);
        assert!(entry.citation.unwrap().contains("probe:free"));
    }

    #[test]
    fn test_cache_add_and_prune() {
        let mut cache = EvidenceCache::new(5);

        for i in 0..10 {
            cache.add(EvidenceEntry::probe_output(
                &format!("probe{}", i),
                "output",
                "test",
            ));
        }

        // Should have pruned to max 5
        assert_eq!(cache.len(), 5);
    }

    #[test]
    fn test_cache_search() {
        let mut cache = EvidenceCache::new(100);
        cache.add(EvidenceEntry::probe_output(
            "free",
            "memory available 8000",
            "memory",
        ));
        cache.add(EvidenceEntry::probe_output("df", "disk usage 50%", "disk"));

        let results = cache.search(&["memory"]);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("memory"));
    }

    #[test]
    fn test_keyword_extraction() {
        let keywords = extract_keywords("The systemd service failed to start");
        assert!(keywords.contains(&"systemd".to_string()));
        assert!(keywords.contains(&"service".to_string()));
        assert!(keywords.contains(&"failed".to_string()));
        assert!(!keywords.contains(&"the".to_string())); // Stopword
    }

    #[test]
    fn test_domain_inference() {
        assert_eq!(infer_domain("systemctl"), "services.systemd");
        assert_eq!(infer_domain("pacman"), "packages");
        assert_eq!(infer_domain("free"), "performance.memory");
        assert_eq!(infer_domain("unknown"), "general");
    }
}
