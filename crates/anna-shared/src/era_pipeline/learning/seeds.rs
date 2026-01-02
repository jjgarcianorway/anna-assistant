//! Seed mappings - Predefined intent-to-fact mappings for common intents.

use crate::era_pipeline::pipeline::AnswerType;

use super::types::IntentFactMapping;

/// Add seed mappings for common intents to a vector.
pub fn create_seed_mappings() -> Vec<IntentFactMapping> {
    vec![
        // Memory intents
        IntentFactMapping::new(
            "memory.free",
            vec!["memory.free_gib", "memory.total_gib"],
            AnswerType::Numeric,
        )
        .with_primary("memory.free_gib"),
        IntentFactMapping::new("memory.usage", vec!["memory.used_pct"], AnswerType::Numeric)
            .with_primary("memory.used_pct"),
        // Boot intents
        IntentFactMapping::new("boot.time", vec!["boot.total_time_s"], AnswerType::Numeric)
            .with_primary("boot.total_time_s"),
        IntentFactMapping::new(
            "boot.slow_service",
            vec!["boot.blame", "boot.slowest_service"],
            AnswerType::Entity,
        )
        .with_primary("boot.slowest_service"),
        IntentFactMapping::new("boot.blame_list", vec!["boot.blame"], AnswerType::List)
            .with_primary("boot.blame"),
        // CPU intents
        IntentFactMapping::new("cpu.model", vec!["cpu.model"], AnswerType::Entity)
            .with_primary("cpu.model"),
        IntentFactMapping::new("cpu.temperature", vec!["cpu.temp_c"], AnswerType::Numeric)
            .with_primary("cpu.temp_c"),
        IntentFactMapping::new("cpu.load", vec!["cpu.load_1m"], AnswerType::Numeric)
            .with_primary("cpu.load_1m"),
        // Disk intents
        IntentFactMapping::new("disk.free", vec!["disk.root_free_gib"], AnswerType::Numeric)
            .with_primary("disk.root_free_gib"),
        IntentFactMapping::new(
            "disk.usage",
            vec!["disk.root_used_pct"],
            AnswerType::Numeric,
        )
        .with_primary("disk.root_used_pct"),
        IntentFactMapping::new("disk.trim", vec!["disk.trim_enabled"], AnswerType::Boolean)
            .with_primary("disk.trim_enabled"),
        // GPU intents
        IntentFactMapping::new("gpu.model", vec!["gpu.model"], AnswerType::Entity)
            .with_primary("gpu.model"),
        IntentFactMapping::new("gpu.driver", vec!["gpu.driver"], AnswerType::Entity)
            .with_primary("gpu.driver"),
        // Service intents
        IntentFactMapping::new(
            "services.failed",
            vec!["services.failed_list", "services.failed_count"],
            AnswerType::List,
        )
        .with_primary("services.failed_list"),
        IntentFactMapping::new(
            "services.failed_count",
            vec!["services.failed_count"],
            AnswerType::Numeric,
        )
        .with_primary("services.failed_count"),
    ]
}
