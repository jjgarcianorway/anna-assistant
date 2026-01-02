//! Store metadata wire format.

use serde::{Deserialize, Serialize};

/// Wire format for store metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMetadata {
    pub version: u32,
    pub seq: u64,
    pub doc_count: usize,
}
