//! Model selection logic (v0.0.223).

use std::collections::HashMap;

use super::catalog::model_catalog;
use super::config::ModelSelectorConfig;
use super::types::{ModelBenchmark, ModelCandidate, ModelFamily, ModelRole, ModelSelection};

/// Select best model for a role from available models
pub fn select_model(
    role: ModelRole,
    available: &[String],
    config: &ModelSelectorConfig,
    benchmarks: &HashMap<String, ModelBenchmark>,
) -> Option<ModelSelection> {
    let catalog = model_catalog();

    // Filter to models that are available and support the role
    let mut candidates: Vec<&ModelCandidate> = catalog
        .iter()
        .filter(|c| c.roles.contains(&role))
        .filter(|c| available.iter().any(|a| model_matches(&c.name, a)))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Sort by: family preference, then priority, then benchmark if available
    candidates.sort_by(|a, b| {
        // Family preference: Qwen3VL > Qwen25 > Llama32 > Other
        let family_order = |f: &ModelFamily| match f {
            ModelFamily::Qwen3VL if config.prefer_qwen3_vl => 0,
            ModelFamily::Qwen25 => 1,
            ModelFamily::Llama32 => 2,
            ModelFamily::Qwen3VL => 1,
            ModelFamily::Other => 3,
        };

        let a_family = family_order(&a.family);
        let b_family = family_order(&b.family);

        if a_family != b_family {
            return a_family.cmp(&b_family);
        }

        // Same family: check benchmark if available
        let a_tps = benchmarks
            .get(&a.name)
            .map(|b| b.tokens_per_sec)
            .unwrap_or(0.0);
        let b_tps = benchmarks
            .get(&b.name)
            .map(|b| b.tokens_per_sec)
            .unwrap_or(0.0);

        if a_tps > 0.0 && b_tps > 0.0 {
            // Higher TPS is better
            return b_tps
                .partial_cmp(&a_tps)
                .unwrap_or(std::cmp::Ordering::Equal);
        }

        // Fallback to priority
        a.priority.cmp(&b.priority)
    });

    let selected = candidates.first()?;
    let is_preferred = selected.family == ModelFamily::Qwen3VL && config.prefer_qwen3_vl;
    let is_fallback = !is_preferred && config.prefer_qwen3_vl;

    let reason = if is_preferred {
        format!("preferred: {} available", selected.family_display())
    } else if is_fallback {
        format!(
            "fallback: {} (Qwen3-VL not available)",
            selected.family_display()
        )
    } else {
        format!("selected: {}", selected.family_display())
    };

    Some(ModelSelection {
        model: selected.name.clone(),
        family: selected.family,
        reason,
        is_preferred,
        is_fallback,
    })
}

/// Check if a catalog model name matches an available model
pub fn model_matches(catalog_name: &str, available_name: &str) -> bool {
    let normalize = |s: &str| s.to_lowercase().replace(['-', '_'], "");
    let c = normalize(catalog_name);
    let a = normalize(available_name);

    // Exact match
    if c == a {
        return true;
    }

    // Prefix match (qwen3vl:4b matches qwen3-vl:4b-q4_k_m)
    if a.starts_with(&c) {
        return true;
    }

    // Check if base model matches (without quantization suffix)
    let a_base = a.split(':').next().unwrap_or(&a);
    let c_base = c.split(':').next().unwrap_or(&c);

    a_base == c_base || a_base.starts_with(c_base)
}

/// Detect model family from model name
pub fn detect_family(model_name: &str) -> ModelFamily {
    let name_lower = model_name.to_lowercase();

    if name_lower.contains("qwen3-vl") || name_lower.contains("qwen3vl") {
        ModelFamily::Qwen3VL
    } else if name_lower.contains("qwen2.5") || name_lower.contains("qwen25") {
        ModelFamily::Qwen25
    } else if name_lower.contains("llama3.2") || name_lower.contains("llama32") {
        ModelFamily::Llama32
    } else {
        ModelFamily::Other
    }
}
