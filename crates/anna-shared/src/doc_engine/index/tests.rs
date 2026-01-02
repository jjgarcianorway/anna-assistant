//! Index tests (v0.0.429).

#[cfg(test)]
mod tests {
    use crate::doc_engine::{DocIndex, DocSnippet, DocSourceKind};

    fn make_snippet(name: &str, source: DocSourceKind) -> DocSnippet {
        DocSnippet::new(
            source,
            name,
            Some("1"),
            &format!("{} command description", name),
            &format!("Content about {} and how to use it.", name),
        )
    }

    #[test]
    fn test_index_add_and_get() {
        let mut index = DocIndex::new();
        let snippet = make_snippet("systemctl", DocSourceKind::ManPage);
        let id = snippet.id.clone();

        index.add(snippet);

        assert!(index.contains(&id));
        assert_eq!(index.len(), 1);

        let retrieved = index.get(&id).unwrap();
        assert_eq!(retrieved.name, "systemctl");
    }

    #[test]
    fn test_index_search() {
        let mut index = DocIndex::new();
        index.add(make_snippet("systemctl", DocSourceKind::ManPage));
        index.add(make_snippet("journalctl", DocSourceKind::ManPage));
        index.add(make_snippet("systemd", DocSourceKind::ArchWiki));

        // Search for "systemctl"
        let results = index.search("systemctl", &[], 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "systemctl");

        // Search for "system" should match multiple
        let results = index.search("system", &[], 5);
        assert!(results.len() >= 2);

        // Filter by source
        let results = index.search("system", &[DocSourceKind::ArchWiki], 5);
        assert!(results.iter().all(|s| s.source == DocSourceKind::ArchWiki));
    }

    #[test]
    fn test_index_search_by_name() {
        let mut index = DocIndex::new();
        index.add(make_snippet("pacman", DocSourceKind::ManPage));
        index.add(make_snippet("pacman", DocSourceKind::ArchWiki));

        let results = index.search_by_name("pacman", None);
        assert_eq!(results.len(), 2);

        let results = index.search_by_name("pacman", Some(DocSourceKind::ManPage));
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_index_remove() {
        let mut index = DocIndex::new();
        let snippet = make_snippet("test", DocSourceKind::ManPage);
        let id = snippet.id.clone();

        index.add(snippet);
        assert!(index.contains(&id));

        let removed = index.remove(&id);
        assert!(removed.is_some());
        assert!(!index.contains(&id));
    }

    #[test]
    fn test_count_by_source() {
        let mut index = DocIndex::new();
        index.add(make_snippet("cmd1", DocSourceKind::ManPage));
        index.add(make_snippet("cmd2", DocSourceKind::ManPage));
        index.add(make_snippet("topic1", DocSourceKind::ArchWiki));

        let counts = index.count_by_source();
        assert_eq!(counts.get(&DocSourceKind::ManPage), Some(&2));
        assert_eq!(counts.get(&DocSourceKind::ArchWiki), Some(&1));
    }
}
