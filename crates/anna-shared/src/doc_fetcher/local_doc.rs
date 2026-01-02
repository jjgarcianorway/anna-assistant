//! Local documentation (/usr/share/doc) fetching functionality.

use crate::evidence_engine::{DocSnippet, DocSource};
use std::fs;
use std::path::Path;

/// Max snippet length
const MAX_SNIPPET: usize = 500;

/// Search /usr/share/doc for a topic
pub fn fetch_local_doc(topic: &str) -> Option<DocSnippet> {
    let doc_dirs = ["/usr/share/doc", "/usr/share/help"];

    for doc_dir in &doc_dirs {
        let base = Path::new(doc_dir);
        if !base.exists() {
            continue;
        }

        // Look for directories matching topic
        if let Ok(entries) = fs::read_dir(base) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&topic.to_lowercase()) {
                    // Look for README or similar
                    let readme_names = ["README", "README.md", "README.txt", "index.html"];
                    for readme in &readme_names {
                        let readme_path = entry.path().join(readme);
                        if readme_path.exists() {
                            if let Ok(content) = fs::read_to_string(&readme_path) {
                                return Some(
                                    DocSnippet::new(
                                        DocSource::LocalDoc,
                                        &format!("doc: {}", entry.file_name().to_string_lossy()),
                                        &super::utils::extract_relevant_section(&content, topic, MAX_SNIPPET),
                                        &readme_path.display().to_string(),
                                    )
                                    .with_relevance(60),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
