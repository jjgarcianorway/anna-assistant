//! Recipe store types and structures (v0.0.423).

use std::collections::HashMap;
use std::path::PathBuf;

use super::{RecipeDomain, RecipeV3};

/// Recipe store with file-based persistence
pub struct RecipeStore {
    /// Base directory for recipes
    pub(super) base_dir: PathBuf,
    /// Global recipes directory
    pub(super) global_dir: PathBuf,
    /// User recipes directory
    pub(super) user_dir: PathBuf,
    /// In-memory cache of recipes
    pub(super) recipes: HashMap<String, RecipeV3>,
    /// Index by domain
    pub(super) by_domain: HashMap<RecipeDomain, Vec<String>>,
    /// Index by tag
    pub(super) by_tag: HashMap<String, Vec<String>>,
    /// Whether store has been loaded
    pub(super) loaded: bool,
}

impl Default for RecipeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Store errors
#[derive(Debug, Clone)]
pub enum StoreError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    NotFound(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}
