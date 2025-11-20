//! Model Catalog and Selection System
//!
//! Beta.53: Intelligent model selection based on hardware capabilities
//! Provides recommendations and auto-installation for optimal LLM models

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Model definition in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub family: String,
    pub params_billion: f64,
    pub disk_size_gb: f64,
    pub ram_requirement_gb: f64,
    pub quality_score: u32,
    pub latency_ms: u32,
    pub tags: Vec<String>,
    pub description: String,
}

/// Get the hardcoded model catalog
pub fn get_model_catalog() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "llama3.2:3b".to_string(),
            family: "llama3.2".to_string(),
            params_billion: 3.0,
            disk_size_gb: 2.0,
            ram_requirement_gb: 4.0,
            quality_score: 60,
            latency_ms: 200,
            tags: vec![
                "fast".to_string(),
                "interactive".to_string(),
                "minimal".to_string(),
            ],
            description: "Minimal model for quick chats - NOT recommended for system admin"
                .to_string(),
        },
        ModelInfo {
            id: "llama3.1:8b".to_string(),
            family: "llama3.1".to_string(),
            params_billion: 8.0,
            disk_size_gb: 4.5,
            ram_requirement_gb: 8.0,
            quality_score: 80,
            latency_ms: 500,
            tags: vec!["balanced".to_string(), "recommended".to_string()],
            description: "Recommended for most systems - good balance of speed and intelligence"
                .to_string(),
        },
        ModelInfo {
            id: "qwen2.5:14b".to_string(),
            family: "qwen2.5".to_string(),
            params_billion: 14.0,
            disk_size_gb: 8.5,
            ram_requirement_gb: 16.0,
            quality_score: 90,
            latency_ms: 1000,
            tags: vec![
                "deep".to_string(),
                "code".to_string(),
                "analysis".to_string(),
            ],
            description: "Best for code analysis and complex reasoning".to_string(),
        },
        ModelInfo {
            id: "deepseek-r1:8b".to_string(),
            family: "deepseek-r1".to_string(),
            params_billion: 8.0,
            disk_size_gb: 4.8,
            ram_requirement_gb: 8.0,
            quality_score: 85,
            latency_ms: 600,
            tags: vec!["reasoning".to_string(), "recommended".to_string()],
            description: "Excellent reasoning capabilities for troubleshooting".to_string(),
        },
    ]
}

/// Get list of installed models from Ollama
pub fn get_installed_models() -> Result<Vec<String>> {
    let output = Command::new("ollama").arg("list").output()?;

    if !output.status.success() {
        bail!("Failed to list Ollama models");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let models: Vec<String> = stdout
        .lines()
        .skip(1) // Skip header
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            parts.first().map(|s| s.to_string())
        })
        .collect();

    Ok(models)
}

/// Select best model based on hardware specs
pub fn select_best_model(host_ram_gb: f64, _gpu_vram_gb: Option<f64>) -> ModelInfo {
    let catalog = get_model_catalog();

    // Selection logic based on available RAM
    let selected = if host_ram_gb >= 16.0 {
        // High-end: prefer qwen2.5:14b for best quality
        catalog.iter().find(|m| m.id == "qwen2.5:14b")
    } else if host_ram_gb >= 8.0 {
        // Mid-range: prefer llama3.1:8b for balance
        catalog.iter().find(|m| m.id == "llama3.1:8b")
    } else {
        // Low-end: fall back to llama3.2:3b but warn
        catalog.iter().find(|m| m.id == "llama3.2:3b")
    };

    selected.cloned().unwrap_or_else(|| catalog[1].clone())
}

/// Get alternative models for the user to choose from
pub fn get_alternatives(primary: &ModelInfo, host_ram_gb: f64) -> Vec<ModelInfo> {
    let catalog = get_model_catalog();

    catalog
        .into_iter()
        .filter(|m| {
            m.id != primary.id && m.ram_requirement_gb <= host_ram_gb * 0.8 // Leave 20% RAM free
        })
        .collect()
}

/// Check if a model is installed
pub fn is_model_installed(model_id: &str) -> bool {
    get_installed_models()
        .map(|models| models.iter().any(|m| m.starts_with(model_id)))
        .unwrap_or(false)
}

/// Pull/install a model from Ollama
pub fn install_model(model_id: &str) -> Result<()> {
    println!("\n📥 Installing model: {}", model_id);
    println!("This may take a few minutes depending on your connection...\n");

    let status = Command::new("ollama").arg("pull").arg(model_id).status()?;

    if !status.success() {
        bail!("Failed to install model {}", model_id);
    }

    Ok(())
}

/// Get model recommendation text for runtime prompt
pub fn get_model_suggestion(current_model: &str, host_ram_gb: f64) -> String {
    let recommended = select_best_model(host_ram_gb, None);

    if current_model == recommended.id {
        "Current model is optimal for your hardware".to_string()
    } else if current_model == "llama3.2:3b" {
        format!(
            "⚠️  Current model (llama3.2:3b) is too small for system administration. \
            Consider upgrading to {} for better results.",
            recommended.id
        )
    } else {
        format!(
            "💡 Consider {} for better performance on your hardware",
            recommended.id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_selection_high_ram() {
        let model = select_best_model(32.0, None);
        assert_eq!(model.id, "qwen2.5:14b");
    }

    #[test]
    fn test_model_selection_mid_ram() {
        let model = select_best_model(12.0, None);
        assert_eq!(model.id, "llama3.1:8b");
    }

    #[test]
    fn test_model_selection_low_ram() {
        let model = select_best_model(4.0, None);
        assert_eq!(model.id, "llama3.2:3b");
    }
}
