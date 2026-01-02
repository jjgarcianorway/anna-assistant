//! Stabilization tests for v0.0.74.

fn golden_v074_model_selector_prefers_qwen3_vl() {
    use anna_shared::model_selector::{select_model, ModelFamily, ModelRole, ModelSelectorConfig};
    use std::collections::HashMap;

    let available = vec![
        "qwen3-vl:4b".to_string(),
        "qwen2.5:3b".to_string(),
        "llama3.2:3b".to_string(),
    ];
    let config = ModelSelectorConfig::default();
    let benchmarks = HashMap::new();

    let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
    assert!(selection.is_some());
    let sel = selection.unwrap();
    assert_eq!(sel.family, ModelFamily::Qwen3VL);
    assert!(sel.is_preferred);
}

/// v0.0.74: Model selector falls back when Qwen3-VL not available.
#[test]
fn golden_v074_model_selector_fallback() {
    use anna_shared::model_selector::{select_model, ModelFamily, ModelRole, ModelSelectorConfig};
    use std::collections::HashMap;

    let available = vec!["qwen2.5:3b".to_string(), "llama3.2:3b".to_string()];
    let config = ModelSelectorConfig::default();
    let benchmarks = HashMap::new();

    let selection = select_model(ModelRole::Specialist, &available, &config, &benchmarks);
    assert!(selection.is_some());
    let sel = selection.unwrap();
    assert_ne!(sel.family, ModelFamily::Qwen3VL);
    assert!(sel.is_fallback);
}

/// v0.0.74: Answer contract extracts requested fields from query.
#[test]
fn golden_v074_answer_contract_from_query() {
    use anna_shared::answer_contract::{AnswerContract, RequestedField, Verbosity};

    let contract = AnswerContract::from_query("how many cores does my CPU have?");
    assert!(contract
        .requested_fields
        .contains(&RequestedField::CpuCores));

    let contract2 = AnswerContract::from_query("just tell me free RAM");
    assert!(contract2
        .requested_fields
        .contains(&RequestedField::RamFree));
    assert_eq!(contract2.verbosity, Verbosity::Minimal);
}

/// v0.0.74: Answer contract allows extra context in teaching mode.
#[test]
fn golden_v074_answer_contract_teaching_mode() {
    use anna_shared::answer_contract::AnswerContract;

    let contract = AnswerContract::from_query("explain how many cores I have");
    assert!(contract.teaching_mode);
    assert!(contract.allows_extra_context());
}

/// v0.0.74: Editor recipe exists for vim syntax highlighting.
#[test]
fn golden_v074_editor_recipe_vim_syntax() {
    use anna_shared::editor_recipes::{get_recipe, ConfigFeature, Editor};

    let recipe = get_recipe(Editor::Vim, ConfigFeature::SyntaxHighlighting);
    assert!(recipe.is_some());
    let r = recipe.unwrap();
    assert!(r.lines[0].line.contains("syntax on"));
}

/// v0.0.74: Editor recipe is idempotent.
#[test]
fn golden_v074_editor_recipe_idempotent() {
    use anna_shared::editor_recipes::{apply_recipe, get_recipe, ConfigFeature, Editor};

    let recipe = get_recipe(Editor::Vim, ConfigFeature::LineNumbers).unwrap();
    let existing = "\" My vimrc\nset number\n";

    // Apply should not duplicate
    let result = apply_recipe(existing, &recipe);
    let count = result.matches("set number").count();
    assert_eq!(count, 1, "Recipe must be idempotent");
}

/// v0.0.74: Editor from tool name parsing.
#[test]
fn golden_v074_editor_from_tool_name() {
    use anna_shared::editor_recipes::Editor;

    assert_eq!(Editor::from_tool_name("vim"), Some(Editor::Vim));
    assert_eq!(Editor::from_tool_name("nvim"), Some(Editor::Neovim));
    assert_eq!(Editor::from_tool_name("hx"), Some(Editor::Helix));
    assert_eq!(Editor::from_tool_name("nano"), Some(Editor::Nano));
    assert_eq!(Editor::from_tool_name("unknown_editor"), None);
}

/// v0.0.74: Version module consistency check.
#[test]
fn golden_v074_version_sanity() {
    use anna_shared::version::{is_newer_version, VersionInfo, VERSION};

    // VERSION must be semver format
    let parts: Vec<&str> = VERSION.split('.').collect();
    assert_eq!(parts.len(), 3);

    // VersionInfo::current() must work
    let info = VersionInfo::current();
    assert_eq!(info.version, VERSION);

    // Version comparison must work
    assert!(is_newer_version("0.0.73", "0.0.74"));
    assert!(!is_newer_version("0.0.74", "0.0.74"));
}
