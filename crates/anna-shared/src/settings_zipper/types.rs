// v0.0.683: Settings Zipper Types
// Core types for zipping and unzipping settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Zip mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ZipMode {
    /// Zip by matching keys
    #[default]
    ByKey,
    /// Zip by position
    ByPosition,
    /// Zip all combinations
    Cartesian,
    /// Zip with default for missing
    WithDefault,
}

impl std::fmt::Display for ZipMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByKey => write!(f, "by_key"),
            Self::ByPosition => write!(f, "by_position"),
            Self::Cartesian => write!(f, "cartesian"),
            Self::WithDefault => write!(f, "with_default"),
        }
    }
}

/// Unzip mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum UnzipMode {
    /// Split by key prefix
    #[default]
    ByPrefix,
    /// Split alternating
    Alternating,
    /// Split by predicate (odd/even index)
    ByIndex,
}

impl std::fmt::Display for UnzipMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ByPrefix => write!(f, "by_prefix"),
            Self::Alternating => write!(f, "alternating"),
            Self::ByIndex => write!(f, "by_index"),
        }
    }
}

/// Zipped pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZippedPair {
    /// Key
    pub key: String,
    /// First value
    pub first: String,
    /// Second value
    pub second: String,
}

impl ZippedPair {
    /// Create new pair
    pub fn new(key: impl Into<String>, first: impl Into<String>, second: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            first: first.into(),
            second: second.into(),
        }
    }

    /// To tuple
    pub fn to_tuple(&self) -> (&str, &str, &str) {
        (&self.key, &self.first, &self.second)
    }

    /// Combined value
    pub fn combined(&self, sep: &str) -> String {
        format!("{}{}{}", self.first, sep, self.second)
    }
}

/// Zip result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipResult {
    /// Zipped pairs
    pub pairs: Vec<ZippedPair>,
    /// Total pairs
    pub total_pairs: usize,
    /// Matched count
    pub matched: usize,
    /// Unmatched count
    pub unmatched: usize,
    /// Mode used
    pub mode: ZipMode,
}

impl ZipResult {
    /// Create new result
    pub fn new(pairs: Vec<ZippedPair>, matched: usize, unmatched: usize, mode: ZipMode) -> Self {
        let total_pairs = pairs.len();
        Self {
            pairs,
            total_pairs,
            matched,
            unmatched,
            mode,
        }
    }

    /// Get pair
    pub fn get(&self, index: usize) -> Option<&ZippedPair> {
        self.pairs.get(index)
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Match rate
    pub fn match_rate(&self) -> f64 {
        let total = self.matched + self.unmatched;
        if total == 0 {
            1.0
        } else {
            self.matched as f64 / total as f64
        }
    }
}

impl Default for ZipResult {
    fn default() -> Self {
        Self::new(Vec::new(), 0, 0, ZipMode::ByKey)
    }
}

/// Unzip result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnzipResult {
    /// First collection
    pub first: HashMap<String, String>,
    /// Second collection
    pub second: HashMap<String, String>,
    /// Total split
    pub total_split: usize,
}

impl UnzipResult {
    /// Create new result
    pub fn new(first: HashMap<String, String>, second: HashMap<String, String>) -> Self {
        let total_split = first.len() + second.len();
        Self {
            first,
            second,
            total_split,
        }
    }

    /// Is balanced
    pub fn is_balanced(&self) -> bool {
        let diff = if self.first.len() > self.second.len() {
            self.first.len() - self.second.len()
        } else {
            self.second.len() - self.first.len()
        };
        diff <= 1
    }
}

impl Default for UnzipResult {
    fn default() -> Self {
        Self::new(HashMap::new(), HashMap::new())
    }
}

/// Zipper stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ZipperStats {
    /// Total zip operations
    pub total_zips: usize,
    /// Total unzip operations
    pub total_unzips: usize,
    /// Total pairs created
    pub total_pairs: usize,
    /// By mode
    pub by_mode: HashMap<String, usize>,
}

impl ZipperStats {
    /// Record zip
    pub fn record_zip(&mut self, result: &ZipResult) {
        self.total_zips += 1;
        self.total_pairs += result.total_pairs;
        *self.by_mode.entry(result.mode.to_string()).or_insert(0) += 1;
    }

    /// Record unzip
    pub fn record_unzip(&mut self, result: &UnzipResult) {
        self.total_unzips += 1;
        self.total_pairs += result.total_split;
    }
}
