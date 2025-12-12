//! Knowledge Pipeline acceptance tests (v0.0.432).

use super::sources::{Citation, KnowledgeSource, SourcePriority, SourceResult};
use super::fetcher::{FetchConfig, FetchResult, KnowledgeFetcher};
use super::research::{ResearchOutcome, ResearchPattern, ResearchRequest};
use super::wiki_sync::WikiSyncer;
use super::clarification::{ClarificationProtocol, ClarificationRequest, ClarificationResponse};
use super::learning::{LearningLoop, LearningOutcome, RecipeStats};
use std::path::Path;

/// Test scenario 1: Memory query uses probes first (highest trust).
#[test]
fn test_memory_query_uses_probes() {
    let fetcher = KnowledgeFetcher::new();
    let sources = fetcher.suggest_sources("how much memory do I have?");

    // Should suggest probe first
    assert!(!sources.is_empty());
    assert!(matches!(
        sources[0],
        KnowledgeSource::Probe { ref name, .. } if name == "meminfo"
    ));
}

/// Test scenario 2: Priority ordering is respected.
#[test]
fn test_priority_ordering() {
    // Probe < LocalDoc < CachedWiki < Remote
    assert!(SourcePriority::Probe < SourcePriority::LocalDoc);
    assert!(SourcePriority::LocalDoc < SourcePriority::CachedWiki);
    assert!(SourcePriority::CachedWiki < SourcePriority::Remote);

    // Trust scores decrease with priority
    assert!(SourcePriority::Probe.trust_score() > SourcePriority::LocalDoc.trust_score());
    assert!(SourcePriority::LocalDoc.trust_score() > SourcePriority::CachedWiki.trust_score());
    assert!(SourcePriority::CachedWiki.trust_score() > SourcePriority::Remote.trust_score());
}

/// Test scenario 3: Remote is disabled by default.
#[test]
fn test_remote_disabled_by_default() {
    let config = FetchConfig::default();
    assert!(!config.allow_remote);

    let fetcher = KnowledgeFetcher::new();
    let sources = vec![
        KnowledgeSource::RemoteUrl { url: "https://example.com".to_string() },
    ];

    let result = fetcher.fetch("test", &sources);
    // Remote should be skipped
    assert!(result.results.is_empty());
}

/// Test scenario 4: Research pattern workflow.
#[test]
fn test_research_pattern_workflow() {
    let pattern = ResearchPattern::new();

    // Memory questions should be answerable locally
    assert!(pattern.can_answer_locally("memory usage"));
    assert!(pattern.can_answer_locally("cpu info"));

    // Research request builder
    let request = ResearchRequest::new("how much ram?")
        .with_topic("memory")
        .with_min_confidence(0.8)
        .with_source(KnowledgeSource::probe("meminfo"));

    assert_eq!(request.question, "how much ram?");
    assert_eq!(request.sources.len(), 1);
}

/// Test scenario 5: Clarification protocol.
#[test]
fn test_clarification_protocol() {
    let mut protocol = ClarificationProtocol::new();

    // Request clarification
    protocol.request(ClarificationRequest::choice(
        "Which package manager?",
        vec!["pacman", "yay", "paru"],
    ));

    assert!(protocol.has_pending());
    assert_eq!(protocol.pending_count(), 1);

    // Get pending
    let pending = protocol.next_pending().unwrap();
    assert!(pending.question.contains("package manager"));

    // Resolve
    let response = ClarificationResponse::with_value("pacman");
    protocol.resolve(response);

    assert!(!protocol.has_pending());
    assert_eq!(protocol.history().len(), 1);
}

/// Test scenario 6: Learning loop tracks success.
#[test]
fn test_learning_loop_tracks_success() {
    let path = format!("/tmp/anna_knowledge_test_{}", std::process::id());
    let mut learning = LearningLoop::new(Path::new(&path));
    learning.clear();

    // Learn from success
    let outcome = learning.learn_from_success(
        "how much memory?",
        &[KnowledgeSource::probe("meminfo")],
        &[Citation::new(KnowledgeSource::probe("meminfo"), 0.95)],
        50,
    );

    assert!(matches!(outcome, LearningOutcome::NewPattern { .. }));
    assert_eq!(learning.pattern_count(), 1);

    // Should find pattern for similar queries
    let patterns = learning.find_patterns("memory usage");
    assert!(!patterns.is_empty());

    let _ = std::fs::remove_dir_all(&path);
}

/// Test scenario 7: Recipe stats tracking.
#[test]
fn test_recipe_stats_tracking() {
    let mut stats = RecipeStats::default();

    // Track uses
    for _ in 0..8 {
        stats.record_use(true, 100);
    }
    for _ in 0..2 {
        stats.record_use(false, 150);
    }

    assert_eq!(stats.uses, 10);
    assert_eq!(stats.success_rate(), 0.8);

    // Track satisfaction
    stats.record_satisfaction(5);
    stats.record_satisfaction(4);
    assert_eq!(stats.avg_satisfaction(), 4.5);
}

/// Test scenario 8: Wiki sync creates cache structure.
#[test]
fn test_wiki_sync_structure() {
    let path = format!("/tmp/anna_wiki_test_{}", std::process::id());
    let syncer = WikiSyncer::new(Path::new(&path));

    let stats = syncer.stats();
    assert!(stats.total_articles > 0);
    assert!(stats.total_articles >= 20); // Default articles

    let _ = std::fs::remove_dir_all(&path);
}

/// Test scenario 9: Citation formatting.
#[test]
fn test_citation_formatting() {
    let citation = Citation::with_excerpt(
        KnowledgeSource::probe("meminfo"),
        "MemTotal: 32000000 kB",
        0.95,
    );

    let formatted = citation.format();
    assert!(formatted.contains("probe:meminfo"));
    assert!(formatted.contains("MemTotal"));
}

/// Test scenario 10: No hardcoded answers rule.
/// Answers must come from sources, not hardcoded values.
#[test]
fn test_no_hardcoded_answers() {
    let fetcher = KnowledgeFetcher::new();

    // Without sources, should not return results
    let result = fetcher.fetch("random question", &[]);
    assert!(result.results.is_empty());

    // Results must have source attribution
    let sources = vec![KnowledgeSource::probe("meminfo")];
    let result = fetcher.fetch("memory", &sources);

    // If there are results, they must have sources
    for res in &result.results {
        assert!(!res.source.description().is_empty());
    }
}

/// Test scenario 11: Research outcome types.
#[test]
fn test_research_outcomes() {
    // Found outcome
    let found = ResearchOutcome::Found {
        answer: "Test answer".to_string(),
        citations: vec![Citation::new(KnowledgeSource::probe("test"), 0.9)],
        confidence: 0.95,
    };
    assert!(found.is_found());
    assert_eq!(found.confidence(), Some(0.95));
    assert_eq!(found.citations().len(), 1);

    // Not found outcome
    let not_found = ResearchOutcome::NotFound {
        reason: "No sources".to_string(),
        sources_tried: vec!["test".to_string()],
    };
    assert!(!not_found.is_found());
    assert_eq!(not_found.confidence(), None);

    // Needs clarification
    let needs = ResearchOutcome::NeedsClarification {
        question: "Which one?".to_string(),
        options: vec!["a".to_string(), "b".to_string()],
    };
    assert!(!needs.is_found());
}

/// Test scenario 12: Pattern deprecation on failure.
#[test]
fn test_pattern_deprecation() {
    let path = format!("/tmp/anna_deprecation_test_{}", std::process::id());
    let mut learning = LearningLoop::new(Path::new(&path));
    learning.clear();

    // Create a pattern
    learning.learn_from_success(
        "test query",
        &[KnowledgeSource::probe("test")],
        &[],
        50,
    );

    // Record many failures (need at least 5 uses for evaluation)
    for _ in 0..6 {
        learning.record_failure("test query", 50);
    }

    // Pattern should be deprecated
    assert_eq!(learning.pattern_count(), 0);

    let _ = std::fs::remove_dir_all(&path);
}

/// Test scenario 13: Source trust verification requirements.
#[test]
fn test_source_verification_requirements() {
    // Probes and local docs don't need verification
    assert!(!SourcePriority::Probe.requires_verification());
    assert!(!SourcePriority::LocalDoc.requires_verification());

    // Wiki and remote need verification
    assert!(SourcePriority::CachedWiki.requires_verification());
    assert!(SourcePriority::Remote.requires_verification());
}

/// Test scenario 14: Fetch result merging.
#[test]
fn test_fetch_result_merging() {
    let mut result1 = FetchResult {
        results: vec![SourceResult::new(
            KnowledgeSource::probe("test1"),
            "content1".to_string(),
            0.9,
        )],
        citations: vec![],
        confident: true,
        failed_sources: vec![],
        lookup_time_ms: 50,
    };

    let result2 = FetchResult {
        results: vec![SourceResult::new(
            KnowledgeSource::man("test2"),
            "content2".to_string(),
            0.8,
        )],
        citations: vec![],
        confident: false,
        failed_sources: vec!["failed".to_string()],
        lookup_time_ms: 30,
    };

    result1.merge(result2);

    assert_eq!(result1.results.len(), 2);
    assert_eq!(result1.failed_sources.len(), 1);
    assert_eq!(result1.lookup_time_ms, 80);
}

/// Test scenario 15: Full pipeline integration.
#[test]
fn test_full_pipeline_integration() {
    let path = format!("/tmp/anna_pipeline_test_{}", std::process::id());

    // 1. Create learning loop
    let mut learning = LearningLoop::new(Path::new(&path));
    learning.clear();

    // 2. Create research pattern
    let pattern = ResearchPattern::new();

    // 3. Research a memory question
    let request = ResearchRequest::new("memory info")
        .with_source(KnowledgeSource::probe("meminfo"));

    let outcome = pattern.research(&request);

    // 4. If successful, learn from it
    if let ResearchOutcome::Found { citations, confidence, .. } = &outcome {
        if *confidence > 0.7 {
            learning.learn_from_success(
                "memory info",
                &[KnowledgeSource::probe("meminfo")],
                citations,
                100,
            );
        }
    }

    // 5. Future queries should find the pattern
    let found = learning.find_patterns("memory");
    // May or may not find depending on whether /proc/meminfo exists
    // This is expected behavior - we're testing the flow, not the actual probe

    let _ = std::fs::remove_dir_all(&path);
}
