//! Corpus-driven router tests.
//!
//! Validates deterministic router against golden expectations in query_corpus.tsv.
//! Ensures >= 80% deterministic coverage (not QueryClass::Unknown).

use std::fs;
use std::path::PathBuf;

// Import from the crate - note: annad is a binary crate, so we test via integration tests
// For unit tests, see src/router_tests.rs

/// Parsed corpus entry
#[derive(Debug)]
struct CorpusEntry {
    query: String,
    expected_class: String,
    expected_domain: String,
    expected_probes: Vec<String>,
    line_num: usize,
}

/// Parse the query corpus TSV file
fn parse_corpus() -> Vec<CorpusEntry> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(manifest_dir)
        .join("tests")
        .join("fixtures")
        .join("query_corpus.tsv");

    let content = fs::read_to_string(&path).expect("Failed to read query_corpus.tsv");

    let mut entries = Vec::new();
    let mut in_header = true;

    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip header line
        if in_header && line.starts_with("query\t") {
            in_header = false;
            continue;
        }
        in_header = false;

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            panic!("Line {}: expected at least 3 columns, got {}", line_num, parts.len());
        }

        let query = parts[0].to_string();
        let expected_class = parts[1].to_string();
        let expected_domain = parts[2].to_string();
        let expected_probes = if parts.len() > 3 && !parts[3].is_empty() {
            parts[3].split(',').map(|s| s.trim().to_string()).collect()
        } else {
            vec![]
        };

        entries.push(CorpusEntry {
            query,
            expected_class,
            expected_domain,
            expected_probes,
            line_num,
        });
    }

    entries
}

#[test]
fn test_corpus_minimum_size() {
    let entries = parse_corpus();
    assert!(
        entries.len() >= 30,
        "Corpus must have >= 30 entries, got {}",
        entries.len()
    );
}

#[test]
fn test_corpus_deterministic_coverage() {
    let entries = parse_corpus();
    let total = entries.len();
    let deterministic = entries
        .iter()
        .filter(|e| e.expected_class != "unknown")
        .count();

    let coverage = (deterministic as f64 / total as f64) * 100.0;
    assert!(
        coverage >= 80.0,
        "Deterministic coverage must be >= 80%, got {:.1}% ({}/{})",
        coverage,
        deterministic,
        total
    );
}

#[test]
fn test_corpus_parses_correctly() {
    let entries = parse_corpus();

    // Spot check a few known entries
    let cpu_entry = entries.iter().find(|e| e.query == "what cpu do i have?");
    assert!(cpu_entry.is_some(), "Missing CPU query in corpus");
    let cpu = cpu_entry.unwrap();
    assert_eq!(cpu.expected_class, "cpu_info");
    assert_eq!(cpu.expected_domain, "system");
    assert!(cpu.expected_probes.is_empty());

    let disk_entry = entries.iter().find(|e| e.query == "disk usage");
    assert!(disk_entry.is_some(), "Missing disk usage query in corpus");
    let disk = disk_entry.unwrap();
    assert_eq!(disk.expected_class, "disk_usage");
    assert_eq!(disk.expected_domain, "storage");
    assert_eq!(disk.expected_probes, vec!["df"]);

    let service_entry = entries.iter().find(|e| e.query == "is nginx running");
    assert!(service_entry.is_some(), "Missing service query in corpus");
    let service = service_entry.unwrap();
    assert_eq!(service.expected_class, "service_status");
    assert_eq!(service.expected_domain, "system");
    assert_eq!(service.expected_probes, vec!["systemctl"]);
}

// Note: Full router integration tests would require exposing the router module
// from the annad binary crate. For now, unit tests in src/router_tests.rs
// cover the classify_query and get_route functions directly.
//
// To enable full integration testing:
// 1. Create a lib.rs in annad that re-exports router
// 2. Add [[lib]] section to Cargo.toml
// 3. Import and test classify_query/get_route here
