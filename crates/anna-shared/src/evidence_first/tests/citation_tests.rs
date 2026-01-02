//! Citation and evidence management tests.
//!
//! Tests for citation storage, verification, and evidence ID formatting.

#[cfg(test)]
mod tests {
    use crate::evidence_first::{
        citations::{Citation, CitationStore, EvidenceId},
        sources::KnowledgeSource,
    };

    /// Test 5: Citation store and verification.
    #[test]
    fn test_citation_store() {
        let mut store = CitationStore::new();

        // Add evidence
        let id = EvidenceId::probe("test.probe");
        store.add_evidence(
            id.clone(),
            KnowledgeSource::ProbeOutput("test.probe".to_string()),
            "This is the raw output with important data",
        );

        // Add citation
        store.add_citation(Citation::new(
            id.clone(),
            "probe:test.probe",
            "important data",
        ));

        // Verify evidence exists
        assert!(store.has_evidence(&id));

        // Verify citation
        let citations = store.citations_for(&id);
        assert_eq!(citations.len(), 1);

        // Verify citation content is in raw evidence
        let citation = &citations[0];
        assert!(
            store.verify_citation(citation),
            "Citation should be verifiable"
        );
    }

    /// Test 11: Evidence ID formatting.
    #[test]
    fn test_evidence_id_formatting() {
        let probe = EvidenceId::probe("sys.boot.analyze");
        assert_eq!(probe.0, "probe:sys.boot.analyze");

        let man = EvidenceId::man("systemctl");
        assert_eq!(man.0, "man:systemctl");

        let wiki = EvidenceId::wiki("Systemd");
        assert_eq!(wiki.0, "wiki:Systemd");

        let help = EvidenceId::help("git");
        assert_eq!(help.0, "help:git");
    }
}
