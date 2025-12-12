//! Model plan generation (v0.0.434).
//!
//! Creates a model plan based on hardware profile and catalog.

use super::catalog::{ModelCatalog, ModelRole};
use super::profile::{CapabilityTier, HardwareProfile};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A concrete model plan for this system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPlan {
    /// Catalog version used to create this plan.
    pub catalog_version: u32,
    /// Profile version used to create this plan.
    pub profile_version: u32,
    /// Hardware tier this plan was created for.
    pub tier: CapabilityTier,
    /// Translator model name.
    pub translator_model: String,
    /// Junior model name.
    pub junior_model: String,
    /// Senior model name.
    pub senior_model: String,
    /// Whether small models were preferred.
    pub prefer_small: bool,
    /// Estimated total disk usage in GB.
    pub estimated_disk_gb: u32,
    /// When this plan was created.
    pub created_at: String,
    /// Explanation of why these models were chosen.
    pub rationale: String,
}

impl ModelPlan {
    /// Load from file.
    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)
    }

    /// Check if plan needs update due to version mismatch.
    pub fn needs_update(&self, catalog: &ModelCatalog, profile: &HardwareProfile) -> bool {
        self.catalog_version != catalog.version
            || self.profile_version != profile.profile_version
            || self.tier != profile.tier
    }

    /// Get all unique model names in this plan.
    pub fn model_names(&self) -> Vec<&str> {
        let mut names = vec![
            self.translator_model.as_str(),
            self.junior_model.as_str(),
            self.senior_model.as_str(),
        ];
        names.sort();
        names.dedup();
        names
    }

    /// Format for display.
    pub fn format_summary(&self) -> String {
        format!(
            "Tier: {} | Translator: {} | Junior: {} | Senior: {} | Disk: ~{}GB",
            self.tier.label(),
            self.translator_model,
            self.junior_model,
            self.senior_model,
            self.estimated_disk_gb
        )
    }
}

/// Model planner that creates plans from profiles.
pub struct ModelPlanner {
    catalog: ModelCatalog,
    prefer_small: bool,
}

impl ModelPlanner {
    /// Create a new planner with default catalog.
    pub fn new() -> Self {
        Self {
            catalog: ModelCatalog::default_catalog(),
            prefer_small: false,
        }
    }

    /// Create with custom catalog.
    pub fn with_catalog(catalog: ModelCatalog) -> Self {
        Self {
            catalog,
            prefer_small: false,
        }
    }

    /// Set prefer_small option.
    pub fn prefer_small(mut self, prefer: bool) -> Self {
        self.prefer_small = prefer;
        self
    }

    /// Generate a model plan for the given hardware profile.
    pub fn generate_plan(&self, profile: &HardwareProfile) -> Result<ModelPlan, PlanError> {
        let available_ram = profile.ram_free_gb.max(profile.ram_total_gb * 0.7);

        // Select translator (always smallest suitable)
        let translator = self
            .catalog
            .select_model(ModelRole::Translator, profile.tier, available_ram, true)
            .ok_or(PlanError::NoSuitableModel(ModelRole::Translator))?;

        // Select junior
        let junior = self
            .catalog
            .select_model(ModelRole::Junior, profile.tier, available_ram, self.prefer_small)
            .ok_or(PlanError::NoSuitableModel(ModelRole::Junior))?;

        // Select senior
        let senior = self
            .catalog
            .select_model(ModelRole::Senior, profile.tier, available_ram, self.prefer_small)
            .ok_or(PlanError::NoSuitableModel(ModelRole::Senior))?;

        // Calculate disk usage (deduplicated)
        let model_names = [
            translator.name.as_str(),
            junior.name.as_str(),
            senior.name.as_str(),
        ];
        let estimated_disk_gb = self.catalog.total_disk_usage(&model_names);

        // Generate rationale
        let rationale = format!(
            "Based on {} tier ({:.1} GB RAM, {} GPU): \
             Translator {} for fast classification, \
             Junior {} for responsive answers, \
             Senior {} for complex analysis.",
            profile.tier.label(),
            profile.ram_total_gb,
            profile.gpu.vendor.label(),
            translator.name,
            junior.name,
            senior.name
        );

        Ok(ModelPlan {
            catalog_version: self.catalog.version,
            profile_version: profile.profile_version,
            tier: profile.tier,
            translator_model: translator.name.clone(),
            junior_model: junior.name.clone(),
            senior_model: senior.name.clone(),
            prefer_small: self.prefer_small,
            estimated_disk_gb,
            created_at: timestamp_now(),
            rationale,
        })
    }

    /// Check if a plan is valid for the current catalog and profile.
    pub fn validate_plan(&self, plan: &ModelPlan, profile: &HardwareProfile) -> PlanValidation {
        let mut issues = Vec::new();

        // Check versions
        if plan.catalog_version != self.catalog.version {
            issues.push(format!(
                "Catalog version mismatch: plan v{} vs current v{}",
                plan.catalog_version, self.catalog.version
            ));
        }

        if plan.profile_version != profile.profile_version {
            issues.push(format!(
                "Profile version mismatch: plan v{} vs current v{}",
                plan.profile_version, profile.profile_version
            ));
        }

        // Check tier match
        if plan.tier != profile.tier {
            issues.push(format!(
                "Tier mismatch: plan {} vs profile {}",
                plan.tier.label(),
                profile.tier.label()
            ));
        }

        // Check model availability
        for name in plan.model_names() {
            if self.catalog.get_model(name).is_none() {
                issues.push(format!("Model {} not in catalog", name));
            }
        }

        if issues.is_empty() {
            PlanValidation::Valid
        } else {
            PlanValidation::Invalid(issues)
        }
    }
}

impl Default for ModelPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Plan generation error.
#[derive(Debug, Clone)]
pub enum PlanError {
    /// No suitable model found for role.
    NoSuitableModel(ModelRole),
    /// Disk limit exceeded.
    DiskLimitExceeded { required: u32, available: u32 },
}

impl std::fmt::Display for PlanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSuitableModel(role) => {
                write!(f, "No suitable model found for {} role", role.label())
            }
            Self::DiskLimitExceeded { required, available } => {
                write!(
                    f,
                    "Disk limit exceeded: need {}GB but only {}GB available",
                    required, available
                )
            }
        }
    }
}

impl std::error::Error for PlanError {}

/// Plan validation result.
#[derive(Debug, Clone)]
pub enum PlanValidation {
    /// Plan is valid.
    Valid,
    /// Plan has issues.
    Invalid(Vec<String>),
}

impl PlanValidation {
    /// Whether the plan is valid.
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }
}

/// Get current timestamp.
fn timestamp_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::profile::{CpuInfo, GpuInfo, OsInfo, StorageInfo};

    fn mock_profile(tier: CapabilityTier, ram_gb: f32) -> HardwareProfile {
        HardwareProfile {
            profile_version: 1,
            last_profiled_at: "0".to_string(),
            cpu: CpuInfo::default(),
            ram_total_gb: ram_gb,
            ram_free_gb: ram_gb * 0.8,
            gpu: GpuInfo::default(),
            storage: StorageInfo::default(),
            os: OsInfo::default(),
            tier,
        }
    }

    #[test]
    fn test_generate_plan_tiny() {
        let planner = ModelPlanner::new();
        // 8GB RAM gives ~6.4GB available (enough for senior model)
        let profile = mock_profile(CapabilityTier::Tiny, 8.0);

        let plan = planner.generate_plan(&profile).unwrap();

        assert_eq!(plan.tier, CapabilityTier::Tiny);
        assert!(!plan.translator_model.is_empty());
        assert!(!plan.junior_model.is_empty());
        assert!(!plan.senior_model.is_empty());
    }

    #[test]
    fn test_generate_plan_large() {
        let planner = ModelPlanner::new();
        let profile = mock_profile(CapabilityTier::Large, 64.0);

        let plan = planner.generate_plan(&profile).unwrap();

        assert_eq!(plan.tier, CapabilityTier::Large);
        // Large tier should get higher-priority models
        assert!(plan.estimated_disk_gb > 0);
    }

    #[test]
    fn test_prefer_small() {
        let profile = mock_profile(CapabilityTier::Large, 64.0);

        let planner_default = ModelPlanner::new();
        let planner_small = ModelPlanner::new().prefer_small(true);

        let plan_default = planner_default.generate_plan(&profile).unwrap();
        let plan_small = planner_small.generate_plan(&profile).unwrap();

        // Small-preferred plan should have smaller or equal disk usage
        assert!(plan_small.estimated_disk_gb <= plan_default.estimated_disk_gb);
    }

    #[test]
    fn test_plan_validation() {
        let planner = ModelPlanner::new();
        let profile = mock_profile(CapabilityTier::Medium, 24.0);
        let plan = planner.generate_plan(&profile).unwrap();

        let validation = planner.validate_plan(&plan, &profile);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_plan_invalid_tier_mismatch() {
        let planner = ModelPlanner::new();
        let profile_medium = mock_profile(CapabilityTier::Medium, 24.0);
        let profile_small = mock_profile(CapabilityTier::Small, 12.0);

        let plan = planner.generate_plan(&profile_medium).unwrap();
        let validation = planner.validate_plan(&plan, &profile_small);

        assert!(!validation.is_valid());
    }

    #[test]
    fn test_model_names_dedup() {
        let planner = ModelPlanner::new();
        // 8GB RAM gives ~6.4GB available (enough for senior model)
        let profile = mock_profile(CapabilityTier::Tiny, 8.0);
        let plan = planner.generate_plan(&profile).unwrap();

        let names = plan.model_names();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();

        // No duplicates in returned names
        assert_eq!(names.len(), sorted.len());
    }
}
