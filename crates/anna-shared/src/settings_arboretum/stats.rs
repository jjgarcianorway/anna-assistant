// v0.0.772: Settings Arboretum Stats (Phase 348)
// Statistics tracking for arboretum

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ArboretumType;
use super::specimen::ArboretumSpecimen;

/// Arboretum stats
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArboretumStats {
    /// Total specimens
    pub total_specimens: usize,
    /// Cataloged specimens
    pub cataloged: usize,
    /// Active count
    pub active_count: usize,
    /// By type
    pub by_type: HashMap<String, usize>,
}

impl ArboretumStats {
    /// Update from specimens
    pub fn update(&mut self, specimens: &[ArboretumSpecimen], arboretum_type: ArboretumType) {
        self.total_specimens = specimens.len();
        self.cataloged = specimens.iter().filter(|s| s.cataloged).count();
        *self.by_type.entry(arboretum_type.to_string()).or_insert(0) += 1;
    }

    /// Catalog rate
    pub fn catalog_rate(&self) -> f64 {
        if self.total_specimens == 0 { 0.0 } else { self.cataloged as f64 / self.total_specimens as f64 * 100.0 }
    }
}
