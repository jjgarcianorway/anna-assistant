//! Tests for configuration module.

use super::types::Config;

#[test]
fn test_default_config() {
    let config = Config::default();
    // v0.0.397: Translator needs 3B+ for reliable JSON output
    assert_eq!(config.llm.translator_model, "qwen2.5:3b-instruct");
    // v0.0.277: tiered model hierarchy
    assert_eq!(config.llm.junior_model, "qwen2.5:3b-instruct");
    assert_eq!(config.llm.senior_model, "qwen2.5:7b-instruct");
    // Legacy specialist maps to junior
    assert_eq!(config.llm.specialist_model, "qwen2.5:3b-instruct");
    // v0.0.398: 10s timeout for 3B+ models
    assert_eq!(config.llm.translator_timeout_secs, 10);
}

#[test]
fn test_required_models_dedup() {
    let config = Config::default();
    // v0.0.397: translator/supervisor/junior all 3b now, senior 7b
    let models = config.required_models();
    assert_eq!(models.len(), 2); // translator/supervisor/junior (3b), senior (7b)
}

#[test]
fn test_parse_toml() {
    let toml_str = r#"
[llm]
translator_model = "custom:1b"
specialist_model = "custom:7b"
translator_timeout_secs = 8
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.llm.translator_model, "custom:1b");
    assert_eq!(config.llm.specialist_model, "custom:7b");
    assert_eq!(config.llm.translator_timeout_secs, 8);
    // v0.0.140: specialist timeout increased to 12 for reliability
    assert_eq!(config.llm.specialist_timeout_secs, 12);
}

#[test]
fn test_model_registry_parse_toml() {
    let toml_str = r#"
[model_registry]
translator = "qwen3-vl:2b"
specialist_default = "qwen3-vl:4b"
preferred_family = "qwen3-vl"

[model_registry.specialist_overrides]
network = "qwen3-vl:8b"
"security:senior" = "qwen3-vl:14b"
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert_eq!(config.model_registry.translator, "qwen3-vl:2b");
    assert_eq!(config.model_registry.specialist_default, "qwen3-vl:4b");
    assert!(config.model_registry.prefers_qwen3_vl());
    assert_eq!(
        config.model_registry.get_specialist("network", None),
        "qwen3-vl:8b"
    );
}
