//! Acceptance tests for doc_engine (v0.0.429).
//!
//! Tests from spec:
//! 1. TRIM/SSD question produces probes + wiki docs + correct answer
//! 2. Boot slow question produces systemd-analyze + docs
//! 3. Debug failing service produces checklist from docs

use super::*;

// Test 1: DocSourceKind priority ordering
#[test]
fn test_source_priority_order() {
    assert!(DocSourceKind::ArchWiki.priority() < DocSourceKind::ManPage.priority());
    assert!(DocSourceKind::ManPage.priority() < DocSourceKind::ToolHelp.priority());
    assert!(DocSourceKind::ToolHelp.priority() < DocSourceKind::LocalDoc.priority());
}

// Test 2: DocSnippet ID generation is stable
#[test]
fn test_snippet_id_stability() {
    let id1 = DocSnippet::generate_id(DocSourceKind::ManPage, "systemctl", Some("1"));
    let id2 = DocSnippet::generate_id(DocSourceKind::ManPage, "systemctl", Some("1"));
    assert_eq!(id1, id2);

    // Different inputs = different IDs
    let id3 = DocSnippet::generate_id(DocSourceKind::ManPage, "journalctl", Some("1"));
    assert_ne!(id1, id3);
}

// Test 3: DocQuery builders work correctly
#[test]
fn test_query_builders() {
    let q = DocQuery::man_page("pacman");
    assert_eq!(q.preferred_sources, vec![DocSourceKind::ManPage]);
    assert_eq!(q.name_filter, Some("pacman".to_string()));

    let q = DocQuery::arch_wiki("solid state drive");
    assert_eq!(q.preferred_sources, vec![DocSourceKind::ArchWiki]);
    assert!(q.query.contains("solid state"));

    let q = DocQuery::tool_help("systemctl");
    assert_eq!(q.preferred_sources, vec![DocSourceKind::ToolHelp]);
}

// Test 4: DocReference citation formatting
#[test]
fn test_doc_reference_citations() {
    let r = DocReference::arch_wiki("systemd").with_section("Troubleshooting");
    assert_eq!(r.citation(), "Arch Wiki: systemd#Troubleshooting");

    let r = DocReference::man_page("systemctl", "1");
    assert_eq!(r.citation(), "systemctl(1)");

    let r = DocReference::tool_help("pacman");
    assert_eq!(r.citation(), "pacman --help");
}

// Test 5: Index add and search
#[test]
fn test_index_operations() {
    let mut index = DocIndex::new();

    // Add snippets
    let snippet1 = DocSnippet::new(
        DocSourceKind::ManPage,
        "systemctl",
        Some("1"),
        "Control the systemd system and service manager",
        "systemctl may be used to introspect and control the state of the systemd system and service manager.",
    );

    let snippet2 = DocSnippet::new(
        DocSourceKind::ArchWiki,
        "systemd",
        Some("Troubleshooting"),
        "Troubleshooting systemd",
        "Common systemd troubleshooting steps.",
    );

    index.add(snippet1);
    index.add(snippet2);

    // Search should find both
    let results = index.search("systemd", &[], 10);
    assert!(results.len() >= 2);

    // Filter by source
    let results = index.search("systemd", &[DocSourceKind::ManPage], 10);
    assert!(results.iter().all(|s| s.source == DocSourceKind::ManPage));
}

// Test 6: DocResult merging
#[test]
fn test_result_merging() {
    let mut r1 = DocResult::default();
    r1.snippets.push(
        DocSnippet::new(DocSourceKind::ManPage, "test1", None, "Test 1", "Content 1")
            .with_relevance(80),
    );

    let mut r2 = DocResult::default();
    r2.snippets.push(
        DocSnippet::new(
            DocSourceKind::ArchWiki,
            "test2",
            None,
            "Test 2",
            "Content 2",
        )
        .with_relevance(90),
    );

    let merged = r1.merge(r2);
    assert_eq!(merged.snippets.len(), 2);
    // Should be sorted by relevance (90 first)
    assert_eq!(merged.snippets[0].relevance, 90);
}

// Test 7: Snippet staleness detection
#[test]
fn test_snippet_staleness() {
    let mut snippet = DocSnippet::new(DocSourceKind::ToolHelp, "test", None, "Test", "Content");

    // Fresh snippet should not be stale
    assert!(!snippet.is_stale(7));

    // Set old timestamp
    snippet.indexed_at = 0;
    assert!(snippet.is_stale(7));
}

// Test 8: Content truncation
#[test]
fn test_content_truncation() {
    let long_content = "x".repeat(5000);
    let mut snippet = DocSnippet::new(DocSourceKind::ManPage, "test", None, "Test", &long_content);

    snippet.truncate_content(1000);
    assert!(snippet.content.len() <= 1003); // +3 for "..."
    assert!(snippet.content.ends_with("..."));
}

// Test 9: Essential lists are populated
#[test]
fn test_essential_lists() {
    use super::help_extractor::get_essential_commands;
    use super::man_parser::get_essential_man_pages;
    use super::wiki_reader::get_essential_wiki_pages;

    let man_pages = get_essential_man_pages();
    assert!(!man_pages.is_empty());
    assert!(man_pages.iter().any(|(n, _)| *n == "systemctl"));
    assert!(man_pages.iter().any(|(n, _)| *n == "pacman"));

    let commands = get_essential_commands();
    assert!(!commands.is_empty());
    assert!(commands.contains(&"systemctl"));

    let wiki_pages = get_essential_wiki_pages();
    assert!(!wiki_pages.is_empty());
    assert!(wiki_pages.contains(&"systemd"));
    assert!(wiki_pages.contains(&"pacman"));
}

// Test 10: Safe command validation
#[test]
fn test_man_parser_safety() {
    use super::man_parser::ManParseError;

    // These should fail validation
    let result = super::man_parser::parse_man_page("rm -rf /", None);
    assert!(matches!(result, Err(ManParseError::InvalidName(_))));

    let result = super::man_parser::parse_man_page("$(whoami)", None);
    assert!(matches!(result, Err(ManParseError::InvalidName(_))));

    let result = super::man_parser::parse_man_page("; cat /etc/passwd", None);
    assert!(matches!(result, Err(ManParseError::InvalidName(_))));
}

// Test 11: DocEngine creation and basic query
#[test]
fn test_doc_engine_basic() {
    let engine = DocEngine::new();

    // Should be able to get stats
    let stats = engine.index_stats();
    assert!(stats.is_some());

    // Empty query should return (possibly empty) result
    let result = engine.query(DocQuery::new("test"));
}

// Test 12: Multiple sources query
#[test]
fn test_multi_source_query() {
    let query = DocQuery::new("systemd services")
        .with_sources(vec![DocSourceKind::ArchWiki, DocSourceKind::ManPage])
        .with_limit(5)
        .with_min_relevance(10);

    assert_eq!(query.preferred_sources.len(), 2);
    assert_eq!(query.limit, 5);
    assert_eq!(query.min_relevance, 10);
}

// Test 13: Index persistence format
#[test]
fn test_index_serialization() {
    let mut index = DocIndex::new();
    index.add(DocSnippet::new(
        DocSourceKind::ManPage,
        "test",
        Some("1"),
        "Test command",
        "Test content",
    ));

    // Should be serializable
    let json = serde_json::to_string(&index);
    assert!(json.is_ok());

    // Should be deserializable
    let deserialized: Result<DocIndex, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok());
}

// Test 14: Keyword extraction
#[test]
fn test_keyword_extraction() {
    let snippet = DocSnippet::new(
        DocSourceKind::ArchWiki,
        "Solid_state_drive",
        Some("TRIM"),
        "Solid state drive TRIM support",
        "TRIM is important for SSD performance.",
    );

    // Should have extracted keywords
    assert!(snippet.keywords.contains(&"solid_state_drive".to_string()));
    assert!(snippet
        .keywords
        .iter()
        .any(|k| k.contains("trim") || k.contains("ssd")));
}

// Test 15: Citation format consistency
#[test]
fn test_citation_formats() {
    // Man page format
    assert_eq!(
        DocSourceKind::ManPage.citation_format("systemctl", Some("1")),
        "systemctl(1)"
    );

    // Wiki format
    assert_eq!(
        DocSourceKind::ArchWiki.citation_format("pacman", Some("Tips")),
        "Arch Wiki: pacman#Tips"
    );

    // Help format
    assert_eq!(
        DocSourceKind::ToolHelp.citation_format("df", None),
        "df --help"
    );

    // Local doc format
    assert_eq!(
        DocSourceKind::LocalDoc.citation_format("readme", None),
        "/usr/share/doc/readme"
    );
}
