//! Recipe Validator - Parses and validates LLM-generated recipes
//!
//! Phase 9: LLM Recipe Engine
//!
//! This module is the safety gatekeeper between LLM output and system execution.
//! It enforces strict validation rules to prevent unsafe operations.
//!
//! **Safety Principles**:
//! - Parse JSON responses into structured ChangeRecipe objects
//! - Validate against safety context (forbidden paths, risk levels)
//! - Reject any recipe containing forbidden operations
//! - Enforce maximum risk level constraints
//! - Validate all paths are safe and accessible
//!
//! **Error Handling**:
//! - Invalid JSON → Clear parse error
//! - Missing required fields → Schema validation error
//! - Forbidden operations → Safety violation error
//! - Risk too high → Risk level exceeded error

use crate::change_recipe::*;
use crate::prompt_builder::SafetyContext;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// LLM response envelope
///
/// The LLM can return either a valid recipe or an error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum LlmResponse {
    #[serde(rename = "success")]
    Success { recipe: RecipeJson },

    #[serde(rename = "error")]
    Error {
        error_type: String,
        message: String,
        suggestion: Option<String>,
    },
}

/// JSON representation of a ChangeRecipe
///
/// This matches the schema we provide to the LLM in prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeJson {
    pub title: String,
    pub summary: String,
    pub why_it_matters: String,
    pub actions: Vec<ActionJson>,
    pub rollback_notes: String,
    pub model_profile_id: String,
    pub user_query: String,
}

/// JSON representation of a ChangeAction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionJson {
    pub kind: String, // "edit_file", "install_packages", etc.
    pub description: String,
    pub estimated_impact: String,

    // EditFile fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>, // "append_if_missing", "replace_section", "replace_entire"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_marker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_content: Option<String>,

    // AppendToFile fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    // Package fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<String>>,

    // Service fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_service: Option<bool>,

    // SetWallpaper fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,

    // RunReadOnlyCommand fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
}

/// Parse and validate an LLM response
///
/// This is the main entry point for recipe validation.
///
/// **Steps**:
/// 1. Parse JSON string into LlmResponse
/// 2. If error response, return appropriate error
/// 3. If success, convert RecipeJson to ChangeRecipe
/// 4. Validate recipe against safety context
/// 5. Return validated ChangeRecipe or error
pub fn validate_llm_response(
    json: &str,
    context: &SafetyContext,
) -> Result<ChangeRecipe> {
    // Step 1: Parse JSON
    let response: LlmResponse = serde_json::from_str(json)
        .map_err(|e| anyhow!("Invalid JSON response from LLM: {}", e))?;

    // Step 2: Handle error responses
    match response {
        LlmResponse::Error {
            error_type,
            message,
            suggestion,
        } => {
            let mut error_msg = format!("LLM returned error ({}): {}", error_type, message);
            if let Some(sug) = suggestion {
                error_msg.push_str(&format!("\nSuggestion: {}", sug));
            }
            Err(anyhow!(error_msg))
        }
        LlmResponse::Success { recipe } => {
            // Step 3: Convert to ChangeRecipe
            let change_recipe = convert_recipe_json(recipe)?;

            // Step 4: Validate safety
            validate_recipe_safety(&change_recipe, context)?;

            // Step 5: Return validated recipe
            Ok(change_recipe)
        }
    }
}

/// Convert RecipeJson to ChangeRecipe
fn convert_recipe_json(json: RecipeJson) -> Result<ChangeRecipe> {
    let mut actions = Vec::new();

    for action_json in json.actions {
        let action = convert_action_json(action_json)?;
        actions.push(action);
    }

    let recipe = ChangeRecipe::new(
        json.title,
        json.summary,
        json.why_it_matters,
        actions,
        json.rollback_notes,
        ChangeRecipeSource::LlmPlanned {
            model_profile_id: json.model_profile_id,
            user_query: json.user_query,
        },
    );

    // Validate the recipe structure
    recipe.validate()?;

    Ok(recipe)
}

/// Convert ActionJson to ChangeAction
fn convert_action_json(json: ActionJson) -> Result<ChangeAction> {
    let kind = match json.kind.as_str() {
        "edit_file" => {
            let path = PathBuf::from(
                json.path
                    .as_ref()
                    .ok_or_else(|| anyhow!("edit_file action missing 'path' field"))?
                    .clone(),
            );
            let strategy = parse_edit_strategy(&json)?;
            ChangeActionKind::EditFile { path, strategy }
        }
        "append_to_file" => {
            let path = PathBuf::from(
                json.path
                    .as_ref()
                    .ok_or_else(|| anyhow!("append_to_file action missing 'path' field"))?
                    .clone(),
            );
            let content = json
                .content
                .clone()
                .ok_or_else(|| anyhow!("append_to_file action missing 'content' field"))?;
            ChangeActionKind::AppendToFile { path, content }
        }
        "install_packages" => {
            let packages = json
                .packages
                .clone()
                .ok_or_else(|| anyhow!("install_packages action missing 'packages' field"))?;
            ChangeActionKind::InstallPackages { packages }
        }
        "remove_packages" => {
            let packages = json
                .packages
                .clone()
                .ok_or_else(|| anyhow!("remove_packages action missing 'packages' field"))?;
            ChangeActionKind::RemovePackages { packages }
        }
        "enable_service" => {
            let service_name = json
                .service_name
                .clone()
                .ok_or_else(|| anyhow!("enable_service action missing 'service_name' field"))?;
            let user_service = json.user_service.unwrap_or(false);
            ChangeActionKind::EnableService {
                service_name,
                user_service,
            }
        }
        "disable_service" => {
            let service_name = json
                .service_name
                .clone()
                .ok_or_else(|| anyhow!("disable_service action missing 'service_name' field"))?;
            let user_service = json.user_service.unwrap_or(false);
            ChangeActionKind::DisableService {
                service_name,
                user_service,
            }
        }
        "set_wallpaper" => {
            let image_path = PathBuf::from(
                json.image_path
                    .as_ref()
                    .ok_or_else(|| anyhow!("set_wallpaper action missing 'image_path' field"))?
                    .clone(),
            );
            ChangeActionKind::SetWallpaper { image_path }
        }
        "run_readonly_command" => {
            let command = json
                .command
                .clone()
                .ok_or_else(|| anyhow!("run_readonly_command action missing 'command' field"))?;
            let args = json.args.clone().unwrap_or_default();
            ChangeActionKind::RunReadOnlyCommand { command, args }
        }
        unknown => {
            return Err(anyhow!("Unknown action kind: {}", unknown));
        }
    };

    let action = ChangeAction::new(kind, json.description, json.estimated_impact);
    Ok(action)
}

/// Parse EditStrategy from ActionJson
fn parse_edit_strategy(json: &ActionJson) -> Result<EditStrategy> {
    let strategy_type = json
        .strategy
        .as_ref()
        .ok_or_else(|| anyhow!("edit_file action missing 'strategy' field"))?;

    match strategy_type.as_str() {
        "append_if_missing" => {
            let lines = json
                .lines
                .clone()
                .ok_or_else(|| anyhow!("append_if_missing strategy missing 'lines' field"))?;
            Ok(EditStrategy::AppendIfMissing { lines })
        }
        "replace_section" => {
            let start_marker = json
                .start_marker
                .clone()
                .ok_or_else(|| anyhow!("replace_section strategy missing 'start_marker' field"))?;
            let end_marker = json
                .end_marker
                .clone()
                .ok_or_else(|| anyhow!("replace_section strategy missing 'end_marker' field"))?;
            let new_content = json
                .new_content
                .clone()
                .ok_or_else(|| anyhow!("replace_section strategy missing 'new_content' field"))?;
            Ok(EditStrategy::ReplaceSection {
                start_marker,
                end_marker,
                new_content,
            })
        }
        "replace_entire" => {
            let new_content = json
                .new_content
                .clone()
                .ok_or_else(|| anyhow!("replace_entire strategy missing 'new_content' field"))?;
            Ok(EditStrategy::ReplaceEntire { new_content })
        }
        unknown => Err(anyhow!("Unknown edit strategy: {}", unknown)),
    }
}

/// Validate recipe against safety context
///
/// **Safety checks**:
/// - No forbidden paths
/// - Risk level within allowed maximum
/// - System changes only if allowed
/// - Package operations only if allowed
pub fn validate_recipe_safety(recipe: &ChangeRecipe, context: &SafetyContext) -> Result<()> {
    // Check overall risk level
    if recipe.overall_risk > context.max_risk {
        return Err(anyhow!(
            "Recipe risk level ({:?}) exceeds maximum allowed ({:?})",
            recipe.overall_risk,
            context.max_risk
        ));
    }

    // Validate each action
    for action in &recipe.actions {
        validate_action_safety(action, context)?;
    }

    Ok(())
}

/// Validate individual action against safety context
fn validate_action_safety(action: &ChangeAction, context: &SafetyContext) -> Result<()> {
    // Check if action needs sudo (system change)
    if action.kind.needs_sudo() && !context.allow_system_changes {
        return Err(anyhow!(
            "Action '{}' requires system changes, but safety context does not allow it",
            action.description
        ));
    }

    // Check package operations
    if matches!(
        action.kind,
        ChangeActionKind::InstallPackages { .. } | ChangeActionKind::RemovePackages { .. }
    ) && !context.allow_package_operations
    {
        return Err(anyhow!(
            "Action '{}' requires package operations, but safety context does not allow it",
            action.description
        ));
    }

    // Check forbidden paths
    if let Some(path) = action.kind.get_path() {
        validate_path_safety(path, context)?;
    }

    Ok(())
}

/// Validate path against forbidden paths
fn validate_path_safety(path: &PathBuf, context: &SafetyContext) -> Result<()> {
    let path_str = path.to_string_lossy();

    for forbidden in &context.forbidden_paths {
        if path_str.contains(forbidden) {
            return Err(anyhow!(
                "Path '{}' contains forbidden segment '{}'. This operation is not allowed for safety reasons.",
                path_str,
                forbidden
            ));
        }
    }

    Ok(())
}

impl ChangeActionKind {
    /// Get the path associated with this action, if any
    fn get_path(&self) -> Option<&PathBuf> {
        match self {
            ChangeActionKind::EditFile { path, .. } => Some(path),
            ChangeActionKind::AppendToFile { path, .. } => Some(path),
            ChangeActionKind::SetWallpaper { image_path } => Some(image_path),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_conservative_context() -> SafetyContext {
        SafetyContext::conservative("testuser".to_string())
    }

    fn make_permissive_context() -> SafetyContext {
        SafetyContext::permissive("testuser".to_string())
    }

    // ==================== JSON Parsing Tests ====================

    #[test]
    fn test_parse_valid_success_response() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Enable Vim Syntax",
                "summary": "Add syntax highlighting to vim",
                "why_it_matters": "Makes code easier to read",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Add syntax on to .vimrc",
                        "estimated_impact": "Vim will show colors",
                        "path": "/home/testuser/.vimrc",
                        "strategy": "append_if_missing",
                        "lines": ["syntax on", "set number"]
                    }
                ],
                "rollback_notes": "Remove the lines from .vimrc",
                "model_profile_id": "ollama-llama3.2-3b",
                "user_query": "enable syntax highlighting in vim"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.title, "Enable Vim Syntax");
        assert_eq!(recipe.actions.len(), 1);
    }

    #[test]
    fn test_parse_error_response() {
        let json = r#"{
            "status": "error",
            "error_type": "FORBIDDEN",
            "message": "Cannot modify bootloader configuration",
            "suggestion": "This is too dangerous to automate"
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("FORBIDDEN"));
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "{ invalid json }";

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
    }

    #[test]
    fn test_parse_missing_required_field() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Test",
                        "estimated_impact": "Test"
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing 'path'"));
    }

    #[test]
    fn test_parse_unknown_action_kind() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "delete_everything",
                        "description": "Test",
                        "estimated_impact": "Test"
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unknown action kind"));
    }

    // ==================== EditFile Strategy Parsing Tests ====================

    #[test]
    fn test_parse_append_if_missing_strategy() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Test",
                        "estimated_impact": "Test",
                        "path": "/home/testuser/.vimrc",
                        "strategy": "append_if_missing",
                        "lines": ["line1", "line2"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        if let ChangeActionKind::EditFile { strategy, .. } = &recipe.actions[0].kind {
            assert!(matches!(strategy, EditStrategy::AppendIfMissing { .. }));
        } else {
            panic!("Expected EditFile action");
        }
    }

    #[test]
    fn test_parse_replace_section_strategy() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Test",
                        "estimated_impact": "Test",
                        "path": "/home/testuser/.config/test.conf",
                        "strategy": "replace_section",
                        "start_marker": "BEGIN_CONFIG",
                        "end_marker": "END_CONFIG",
                        "new_content": "option_value"
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        if let ChangeActionKind::EditFile { strategy, .. } = &recipe.actions[0].kind {
            assert!(matches!(strategy, EditStrategy::ReplaceSection { .. }));
        } else {
            panic!("Expected EditFile action");
        }
    }

    #[test]
    fn test_parse_replace_entire_strategy() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Test",
                        "estimated_impact": "Test",
                        "path": "/home/testuser/.config/test.conf",
                        "strategy": "replace_entire",
                        "new_content": "new file content"
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        if let ChangeActionKind::EditFile { strategy, .. } = &recipe.actions[0].kind {
            assert!(matches!(strategy, EditStrategy::ReplaceEntire { .. }));
        } else {
            panic!("Expected EditFile action");
        }
    }

    // ==================== Other Action Types ====================

    #[test]
    fn test_parse_install_packages_action() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "install_packages",
                        "description": "Install vim",
                        "estimated_impact": "Adds vim editor",
                        "packages": ["vim", "vim-runtime"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert!(matches!(
            recipe.actions[0].kind,
            ChangeActionKind::InstallPackages { .. }
        ));
    }

    #[test]
    fn test_parse_set_wallpaper_action() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "set_wallpaper",
                        "description": "Set wallpaper",
                        "estimated_impact": "Changes desktop background",
                        "image_path": "/home/testuser/Pictures/wallpaper.jpg"
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert!(matches!(
            recipe.actions[0].kind,
            ChangeActionKind::SetWallpaper { .. }
        ));
    }

    // ==================== Safety Validation Tests ====================

    #[test]
    fn test_reject_forbidden_path_boot() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Modify grub config",
                        "estimated_impact": "Changes boot config",
                        "path": "/boot/grub/grub.cfg",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("/boot"));
    }

    #[test]
    fn test_reject_forbidden_path_fstab() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Modify fstab",
                        "estimated_impact": "Changes mounts",
                        "path": "/etc/fstab",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("fstab"));
    }

    #[test]
    fn test_reject_system_change_in_conservative_mode() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Modify system config",
                        "estimated_impact": "Changes system settings",
                        "path": "/etc/pacman.conf",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        // Conservative context has max_risk Medium, but /etc/pacman.conf is High
        assert!(err_msg.contains("risk level") && err_msg.contains("exceeds"));
    }

    #[test]
    fn test_reject_package_install_in_conservative_mode() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "install_packages",
                        "description": "Install vim",
                        "estimated_impact": "Adds vim",
                        "packages": ["vim"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        // Package install needs sudo, so it's caught as a system change
        assert!(err_msg.contains("system changes"));
    }

    #[test]
    fn test_accept_user_config_in_conservative_mode() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Configure vim",
                        "estimated_impact": "Adds syntax highlighting",
                        "path": "/home/testuser/.vimrc",
                        "strategy": "append_if_missing",
                        "lines": ["syntax on"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_high_risk_when_max_is_medium() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Modify system file",
                        "estimated_impact": "System changes",
                        "path": "/etc/pacman.conf",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let mut context = make_conservative_context();
        context.allow_system_changes = true; // Allow sudo, but max risk is still Medium
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("risk level"));
    }

    #[test]
    fn test_accept_system_change_in_permissive_mode() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "install_packages",
                        "description": "Install vim",
                        "estimated_impact": "Adds vim",
                        "packages": ["vim"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());
    }

    // ==================== Multi-Action Recipe Tests ====================

    #[test]
    fn test_validate_multi_action_recipe() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Configure Development Environment",
                "summary": "Set up vim and git",
                "why_it_matters": "Improves productivity",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Configure vim",
                        "estimated_impact": "Syntax highlighting",
                        "path": "/home/testuser/.vimrc",
                        "strategy": "append_if_missing",
                        "lines": ["syntax on"]
                    },
                    {
                        "kind": "edit_file",
                        "description": "Configure git",
                        "estimated_impact": "Sets username",
                        "path": "/home/testuser/.gitconfig",
                        "strategy": "append_if_missing",
                        "lines": ["[user]", "name = Test User"]
                    }
                ],
                "rollback_notes": "Remove lines from config files",
                "model_profile_id": "test",
                "user_query": "configure my dev environment"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());

        let recipe = result.unwrap();
        assert_eq!(recipe.actions.len(), 2);
    }

    #[test]
    fn test_reject_recipe_with_any_forbidden_action() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "edit_file",
                        "description": "Safe action",
                        "estimated_impact": "Safe",
                        "path": "/home/testuser/.vimrc",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    },
                    {
                        "kind": "edit_file",
                        "description": "Forbidden action",
                        "estimated_impact": "Dangerous",
                        "path": "/boot/grub/grub.cfg",
                        "strategy": "append_if_missing",
                        "lines": ["test"]
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("/boot"));
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_actions_list() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_conservative_context();
        let result = validate_llm_response(json, &context);
        // Empty actions should be rejected by recipe validation
        assert!(result.is_err());
    }

    #[test]
    fn test_service_actions() {
        let json = r#"{
            "status": "success",
            "recipe": {
                "title": "Test",
                "summary": "Test",
                "why_it_matters": "Test",
                "actions": [
                    {
                        "kind": "enable_service",
                        "description": "Enable service",
                        "estimated_impact": "Service starts on boot",
                        "service_name": "sshd",
                        "user_service": false
                    }
                ],
                "rollback_notes": "Test",
                "model_profile_id": "test",
                "user_query": "test"
            }
        }"#;

        let context = make_permissive_context();
        let result = validate_llm_response(json, &context);
        assert!(result.is_ok());
    }
}
