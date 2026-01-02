// v0.0.681: Iteration Item (Phase 257)
// Single item in iteration result

use serde::{Deserialize, Serialize};

/// Iteration item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationItem {
    /// Key
    pub key: String,
    /// Value
    pub value: String,
    /// Index
    pub index: usize,
}

impl IterationItem {
    /// Create new item
    pub fn new(key: impl Into<String>, value: impl Into<String>, index: usize) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            index,
        }
    }

    /// Value length
    pub fn value_length(&self) -> usize {
        self.value.len()
    }

    /// Is numeric
    pub fn is_numeric(&self) -> bool {
        self.value.parse::<f64>().is_ok()
    }
}
