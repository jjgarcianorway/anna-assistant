// v0.0.531: LLM Model Registry (Phase 107)
// Tracks installed LLM models and their assignments to specialists per VISION.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model capability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ModelCapability {
    Light,      // Fast, low resource (for juniors)
    Standard,   // Balanced
    Heavy,      // Deep thinking (for seniors)
    Multimodal, // Vision/audio capable
}

impl Default for ModelCapability {
    fn default() -> Self {
        Self::Standard
    }
}

impl std::fmt::Display for ModelCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "Light"),
            Self::Standard => write!(f, "Standard"),
            Self::Heavy => write!(f, "Heavy"),
            Self::Multimodal => write!(f, "Multimodal"),
        }
    }
}

/// Model installation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ModelStatus {
    #[default]
    Available,
    Downloading,
    Installing,
    Ready,
    Failed,
    Removed,
}

impl std::fmt::Display for ModelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Available => write!(f, "Available"),
            Self::Downloading => write!(f, "Downloading"),
            Self::Installing => write!(f, "Installing"),
            Self::Ready => write!(f, "Ready"),
            Self::Failed => write!(f, "Failed"),
            Self::Removed => write!(f, "Removed"),
        }
    }
}

/// Who installed the model
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InstalledBy {
    #[default]
    User,
    Anna,
    System,
}

impl std::fmt::Display for InstalledBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Anna => write!(f, "Anna"),
            Self::System => write!(f, "System"),
        }
    }
}

/// Individual model record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub name: String,
    pub capability: ModelCapability,
    pub status: ModelStatus,
    pub installed_by: InstalledBy,
    pub size_gb: f64,
    pub vram_required_gb: f64,
    pub assigned_specialists: Vec<String>,
    pub usage_count: u32,
    pub avg_response_ms: u64,
    pub installed_at: Option<String>,
}

impl ModelRecord {
    /// Create a new model record
    pub fn new(name: &str, capability: ModelCapability, size_gb: f64, vram_gb: f64) -> Self {
        Self {
            name: name.to_string(),
            capability,
            status: ModelStatus::Available,
            installed_by: InstalledBy::User,
            size_gb,
            vram_required_gb: vram_gb,
            assigned_specialists: Vec::new(),
            usage_count: 0,
            avg_response_ms: 0,
            installed_at: None,
        }
    }

    /// Install model
    pub fn install(&mut self, by: InstalledBy, timestamp: &str) {
        self.status = ModelStatus::Ready;
        self.installed_by = by;
        self.installed_at = Some(timestamp.to_string());
    }

    /// Assign to specialist
    pub fn assign(&mut self, specialist_id: &str) {
        if !self.assigned_specialists.contains(&specialist_id.to_string()) {
            self.assigned_specialists.push(specialist_id.to_string());
        }
    }

    /// Unassign from specialist
    pub fn unassign(&mut self, specialist_id: &str) {
        self.assigned_specialists.retain(|s| s != specialist_id);
    }

    /// Record usage
    pub fn record_use(&mut self, response_ms: u64) {
        self.usage_count += 1;
        let total = self.avg_response_ms * (self.usage_count - 1) as u64 + response_ms;
        self.avg_response_ms = total / self.usage_count as u64;
    }

    /// Is model ready for use?
    pub fn is_ready(&self) -> bool {
        self.status == ModelStatus::Ready
    }
}

/// LLM Model Registry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmModelRegistry {
    models: HashMap<String, ModelRecord>,
}

impl LlmModelRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
        }
    }

    /// Register a model
    pub fn register(&mut self, model: ModelRecord) {
        self.models.insert(model.name.clone(), model);
    }

    /// Get model by name
    pub fn get(&self, name: &str) -> Option<&ModelRecord> {
        self.models.get(name)
    }

    /// Get mutable model
    pub fn get_mut(&mut self, name: &str) -> Option<&mut ModelRecord> {
        self.models.get_mut(name)
    }

    /// Get ready models
    pub fn ready(&self) -> Vec<&ModelRecord> {
        self.models.values().filter(|m| m.is_ready()).collect()
    }

    /// Get models by capability
    pub fn by_capability(&self, cap: ModelCapability) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.capability == cap && m.is_ready())
            .collect()
    }

    /// Get models installed by Anna
    pub fn installed_by_anna(&self) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.installed_by == InstalledBy::Anna)
            .collect()
    }

    /// Get model for specialist
    pub fn for_specialist(&self, specialist_id: &str) -> Vec<&ModelRecord> {
        self.models
            .values()
            .filter(|m| m.assigned_specialists.contains(&specialist_id.to_string()))
            .collect()
    }

    /// Get best available model for capability
    pub fn best_for(&self, cap: ModelCapability) -> Option<&ModelRecord> {
        self.by_capability(cap)
            .into_iter()
            .min_by_key(|m| m.avg_response_ms)
    }

    /// Total VRAM used by ready models
    pub fn total_vram_gb(&self) -> f64 {
        self.ready().iter().map(|m| m.vram_required_gb).sum()
    }

    /// Total disk used
    pub fn total_disk_gb(&self) -> f64 {
        self.ready().iter().map(|m| m.size_gb).sum()
    }

    /// Model count
    pub fn total(&self) -> usize {
        self.models.len()
    }

    /// Ready count
    pub fn ready_count(&self) -> usize {
        self.ready().len()
    }

    /// All models
    pub fn all(&self) -> Vec<&ModelRecord> {
        self.models.values().collect()
    }
}

/// Format model for display
pub fn format_model(model: &ModelRecord) -> String {
    format!(
        "{} [{}]\n  Status: {} | Installed by: {}\n  Size: {:.1}GB | VRAM: {:.1}GB\n  Usage: {} calls | Avg: {}ms\n  Assigned to: {}",
        model.name,
        model.capability,
        model.status,
        model.installed_by,
        model.size_gb,
        model.vram_required_gb,
        model.usage_count,
        model.avg_response_ms,
        if model.assigned_specialists.is_empty() {
            "None".to_string()
        } else {
            model.assigned_specialists.join(", ")
        }
    )
}

/// Format model compact
pub fn format_model_compact(model: &ModelRecord) -> String {
    format!(
        "{} [{}] - {:.1}GB ({} calls)",
        model.name, model.capability, model.size_gb, model.usage_count
    )
}

/// Format model oneline
pub fn format_model_oneline(model: &ModelRecord) -> String {
    format!("{} [{}]", model.name, model.status)
}

/// Format registry summary
pub fn format_registry_summary(registry: &LlmModelRegistry) -> String {
    let mut output = String::new();
    output.push_str("=== LLM Model Registry ===\n\n");

    output.push_str(&format!(
        "Total: {} | Ready: {}\n",
        registry.total(),
        registry.ready_count()
    ));
    output.push_str(&format!("VRAM Used: {:.1}GB\n", registry.total_vram_gb()));
    output.push_str(&format!("Disk Used: {:.1}GB\n\n", registry.total_disk_gb()));

    output.push_str("--- Ready Models ---\n");
    for model in registry.ready() {
        output.push_str(&format!("  {}\n", format_model_compact(model)));
    }

    let anna_models = registry.installed_by_anna();
    if !anna_models.is_empty() {
        output.push_str("\n--- Installed by Anna ---\n");
        for model in anna_models {
            output.push_str(&format!("  {}\n", model.name));
        }
    }

    output
}

/// Check if query is model-related
pub fn is_model_query(query: &str) -> bool {
    let lower = query.to_lowercase();
    lower.contains("model")
        || lower.contains("llm")
        || lower.contains("ollama")
        || lower.contains("qwen")
        || lower.contains("llama")
        || lower.contains("vram")
}

/// Fun fact about models
pub fn model_fun_fact() -> &'static str {
    "Modern LLMs can contain billions of parameters - GPT-4 is estimated to have over 1.7 trillion parameters, while smaller models like Qwen 3B fit in just 2GB!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = ModelRecord::new("qwen2.5:3b", ModelCapability::Light, 2.0, 3.0);
        assert_eq!(model.name, "qwen2.5:3b");
        assert_eq!(model.capability, ModelCapability::Light);
        assert_eq!(model.status, ModelStatus::Available);
    }

    #[test]
    fn test_model_install() {
        let mut model = ModelRecord::new("test", ModelCapability::Standard, 5.0, 8.0);
        model.install(InstalledBy::Anna, "2024-01-01");
        assert!(model.is_ready());
        assert_eq!(model.installed_by, InstalledBy::Anna);
    }

    #[test]
    fn test_model_assign() {
        let mut model = ModelRecord::new("test", ModelCapability::Heavy, 14.0, 16.0);
        model.assign("senior-1");
        model.assign("senior-2");
        assert_eq!(model.assigned_specialists.len(), 2);
    }

    #[test]
    fn test_record_use() {
        let mut model = ModelRecord::new("test", ModelCapability::Light, 2.0, 3.0);
        model.install(InstalledBy::User, "ts");
        model.record_use(1000);
        model.record_use(3000);
        assert_eq!(model.usage_count, 2);
        assert_eq!(model.avg_response_ms, 2000);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = LlmModelRegistry::new();
        let model = ModelRecord::new("qwen", ModelCapability::Light, 2.0, 3.0);
        registry.register(model);
        assert_eq!(registry.total(), 1);
    }

    #[test]
    fn test_ready_filter() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("m1", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::User, "ts");
        let m2 = ModelRecord::new("m2", ModelCapability::Light, 2.0, 3.0);
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.ready_count(), 1);
    }

    #[test]
    fn test_by_capability() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("light", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::User, "ts");
        let mut m2 = ModelRecord::new("heavy", ModelCapability::Heavy, 14.0, 16.0);
        m2.install(InstalledBy::User, "ts");
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.by_capability(ModelCapability::Light).len(), 1);
    }

    #[test]
    fn test_installed_by_anna() {
        let mut registry = LlmModelRegistry::new();
        let mut m1 = ModelRecord::new("m1", ModelCapability::Light, 2.0, 3.0);
        m1.install(InstalledBy::Anna, "ts");
        let mut m2 = ModelRecord::new("m2", ModelCapability::Light, 2.0, 3.0);
        m2.install(InstalledBy::User, "ts");
        registry.register(m1);
        registry.register(m2);
        assert_eq!(registry.installed_by_anna().len(), 1);
    }

    #[test]
    fn test_is_model_query() {
        assert!(is_model_query("Which models are installed?"));
        assert!(is_model_query("Show LLM status"));
        assert!(is_model_query("Check Ollama models"));
        assert!(!is_model_query("Install vim"));
    }

    #[test]
    fn test_fun_fact() {
        let fact = model_fun_fact();
        assert!(fact.contains("billion") || fact.contains("trillion"));
    }
}
