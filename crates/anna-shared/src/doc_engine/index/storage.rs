//! Index storage and persistence (v0.0.429).

use super::types::{DocIndex, IndexError};
use std::fs;
use std::path::{Path, PathBuf};

impl DocIndex {
    /// Load index from disk
    pub fn load(path: &Path) -> Result<Self, IndexError> {
        let index_file = path.join("doc_index.json");

        if !index_file.exists() {
            return Ok(Self::new());
        }

        let content =
            fs::read_to_string(&index_file).map_err(|e| IndexError::IoError(e.to_string()))?;

        let index: Self =
            serde_json::from_str(&content).map_err(|e| IndexError::ParseError(e.to_string()))?;

        // Version check
        if index.version != Self::VERSION {
            return Err(IndexError::VersionMismatch {
                expected: Self::VERSION,
                found: index.version,
            });
        }

        Ok(index)
    }

    /// Save index to disk
    pub fn save(&self, path: &Path) -> Result<(), IndexError> {
        fs::create_dir_all(path).map_err(|e| IndexError::IoError(e.to_string()))?;

        let index_file = path.join("doc_index.json");
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| IndexError::ParseError(e.to_string()))?;

        fs::write(&index_file, content).map_err(|e| IndexError::IoError(e.to_string()))?;

        Ok(())
    }
}

/// Get the doc storage path (tries system, falls back to user)
pub fn get_storage_path() -> PathBuf {
    let system_path = PathBuf::from(crate::doc_engine::DOC_STORAGE_PATH);
    if system_path.exists() || fs::create_dir_all(&system_path).is_ok() {
        return system_path;
    }

    // Fall back to user directory
    if let Some(home) = dirs::home_dir() {
        let user_path = home.join(".anna").join("docs");
        let _ = fs::create_dir_all(&user_path);
        return user_path;
    }

    system_path
}

/// Get the wiki cache path
pub fn get_wiki_cache_path() -> PathBuf {
    let system_path = PathBuf::from(crate::doc_engine::WIKI_CACHE_PATH);
    if system_path.exists() {
        return system_path;
    }

    // Fall back to user directory
    if let Some(home) = dirs::home_dir() {
        let user_path = home.join(".anna").join("wiki-cache");
        if user_path.exists() {
            return user_path;
        }
    }

    system_path
}
