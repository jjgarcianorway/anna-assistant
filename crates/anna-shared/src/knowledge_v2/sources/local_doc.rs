//! Local documentation fetching (v0.0.422).

use super::types::SourceFetchResult;
use super::utils::truncate_doc;

/// Fetch local documentation
pub fn fetch_local_doc(topic: &str) -> Option<SourceFetchResult> {
    let doc_paths = ["/usr/share/doc", "/usr/share/help", "/usr/local/share/doc"];

    let topic_lower = topic.to_lowercase();

    for doc_root in &doc_paths {
        let root_path = std::path::Path::new(doc_root);
        if !root_path.exists() {
            continue;
        }

        // Search for matching directory or file
        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains(&topic_lower) {
                    // If directory, look for README
                    if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                        let readme_names = ["README", "README.md", "README.txt", "readme"];
                        for readme in &readme_names {
                            let readme_path = entry.path().join(readme);
                            if readme_path.exists() {
                                if let Ok(content) = std::fs::read_to_string(&readme_path) {
                                    let truncated = truncate_doc(&content, 100);
                                    return Some(SourceFetchResult::new(
                                        truncated,
                                        readme_path.to_string_lossy().as_ref(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
