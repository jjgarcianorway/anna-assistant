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
        // Family preference: Qwen3VL > DeepSeekR1 > Qwen25 > Llama32 > Other
        let family_order = |f: &ModelFamily| match f {
            ModelFamily::Qwen3VL if config.prefer_qwen3_vl => 0,
            ModelFamily::DeepSeekR1 => 1, // v0.0.267: Strong reasoning
            ModelFamily::Qwen25 => 2,
            ModelFamily::Llama32 => 3,
            ModelFamily::Qwen3VL => 2,
            ModelFamily::Other => 4,
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
/// v0.0.393: Fixed to require size match (4b != 2b) for accurate selection
pub fn model_matches(catalog_name: &str, available_name: &str) -> bool {
    let normalize = |s: &str| s.to_lowercase().replace(['-', '_'], "");
    let c = normalize(catalog_name);
    let a = normalize(available_name);

    // Exact match
    if c == a {
        return true;
    }

    // Prefix match (qwen3vl:4b matches qwen3-vl:4b-q4_k_m)
    // This handles quantization suffixes like -q4_k_m
    if a.starts_with(&c) {
        return true;
    }

    // v0.0.393: IMPORTANT - must match size too!
    // Extract base and size from "qwen3vl:4binstruct" -> ("qwen3vl", "4b")
    fn extract_size(s: &str) -> Option<String> {
        let rest = s.split(':').nth(1)?;
        // Extract size - first part with digits + 'b' like "4b", "7b", "3b"
        let size: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == 'b').collect();
        if size.ends_with('b') && size.len() > 1 {
            Some(size)
        } else {
            None
        }
    }

    fn extract_base(s: &str) -> String {
        s.split(':').next().unwrap_or(s).to_string()
    }

    let c_base = extract_base(&c);
    let a_base = extract_base(&a);
    let c_size = extract_size(&c);
    let a_size = extract_size(&a);

    // Base must match
    if c_base != a_base && !a_base.starts_with(&c_base) {
        return false;
    }

    // If catalog specifies size, available must have same size
    if let Some(ref cs) = c_size {
        if let Some(ref avs) = a_size {
            return cs == avs;
        }
        // Available doesn't specify size - no match for sized catalog entry
        return false;
    }

    // Catalog doesn't specify size - any size matches
    true
}

/// Detect model family from model name
/// v0.0.267: Added DeepSeek-R1 detection
pub fn detect_family(model_name: &str) -> ModelFamily {
    let name_lower = model_name.to_lowercase();

    if name_lower.contains("qwen3-vl") || name_lower.contains("qwen3vl") {
        ModelFamily::Qwen3VL
    } else if name_lower.contains("deepseek-r1") || name_lower.contains("deepseekr1") {
        ModelFamily::DeepSeekR1
    } else if name_lower.contains("qwen2.5") || name_lower.contains("qwen25") {
        ModelFamily::Qwen25
    } else if name_lower.contains("llama3.2") || name_lower.contains("llama32") {
        ModelFamily::Llama32
    } else {
        ModelFamily::Other
    }
}
