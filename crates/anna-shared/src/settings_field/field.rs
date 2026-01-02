// v0.0.762: Settings Field (Phase 338)
// Main settings field implementation

use super::config::FieldConfig;
use super::crop::FieldCrop;
use super::farmer::FieldFarmer;
use super::stats::FieldStats;

/// Settings field
#[derive(Debug, Clone, Default)]
pub struct SettingsField {
    /// Config
    config: FieldConfig,
    /// Crops
    crops: Vec<FieldCrop>,
    /// Farmers
    farmers: Vec<FieldFarmer>,
    /// Stats
    stats: FieldStats,
}

impl SettingsField {
    /// Create new field system
    pub fn new(config: FieldConfig) -> Self {
        Self {
            config,
            crops: Vec::new(),
            farmers: Vec::new(),
            stats: FieldStats::default(),
        }
    }

    /// Add crop
    pub fn add_crop(&mut self, crop: FieldCrop) -> bool {
        if self.crops.len() >= self.config.max_crops {
            return false;
        }
        self.crops.push(crop);
        self.update_stats();
        true
    }

    /// Get crop
    pub fn get_crop(&self, id: &str) -> Option<&FieldCrop> {
        self.crops.iter().find(|c| c.id == id)
    }

    /// Get crop mut
    pub fn get_crop_mut(&mut self, id: &str) -> Option<&mut FieldCrop> {
        self.crops.iter_mut().find(|c| c.id == id)
    }

    /// Add farmer
    pub fn add_farmer(&mut self, farmer: FieldFarmer) {
        self.farmers.push(farmer);
    }

    /// Update stats
    fn update_stats(&mut self) {
        self.stats.update(&self.crops, self.config.field_type);
    }

    /// Get stats
    pub fn stats(&self) -> &FieldStats {
        &self.stats
    }

    /// Crop count
    pub fn crop_count(&self) -> usize {
        self.crops.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_new() {
        let f = SettingsField::new(FieldConfig::default());
        assert_eq!(f.crop_count(), 0);
    }

    #[test]
    fn test_field_add_crop() {
        let mut f = SettingsField::new(FieldConfig::default());
        f.add_crop(FieldCrop::new("c1", "Title", "Content"));
        assert_eq!(f.crop_count(), 1);
    }
}
