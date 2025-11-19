//! Template Library - Pre-built, tested recipes for common operations
//!
//! Templates are parametric command recipes that have been:
//! - Verified against Arch Wiki documentation
//! - Tested on real Arch Linux systems
//! - Validated for safety and correctness
//!
//! Instead of generating arbitrary shell commands, the LLM should:
//! 1. Choose appropriate templates
//! 2. Fill in parameters (package names, file paths, etc.)
//! 3. Only fall back to raw commands when no template exists

use crate::command_recipe::{
    CommandCategory, CommandRecipe, OutputValidation, SafetyLevel,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A template definition - parametric recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    /// Template identifier (e.g., "check_swap_status")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// What this template does
    pub description: String,

    /// Required parameters
    pub parameters: Vec<TemplateParameter>,

    /// Command pattern with {{param}} placeholders
    pub command_pattern: String,

    /// Safety classification
    pub category: CommandCategory,

    /// Arch Wiki source
    pub wiki_source: String,

    /// Expected output validation pattern
    pub validation_pattern: Option<OutputValidation>,

    /// Example usage
    pub example: String,
}

/// Template parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    /// Parameter name (for {{name}} substitution)
    pub name: String,

    /// Human description
    pub description: String,

    /// Regex validation for parameter value
    pub validation_regex: Option<String>,

    /// Whether this parameter is required
    pub required: bool,
}

impl Template {
    /// Instantiate template with given parameters
    pub fn instantiate(&self, params: &HashMap<String, String>) -> Result<CommandRecipe, String> {
        // Validate required parameters
        for param_def in &self.parameters {
            if param_def.required && !params.contains_key(&param_def.name) {
                return Err(format!("Missing required parameter: {}", param_def.name));
            }

            // Validate parameter value if present
            if let Some(value) = params.get(&param_def.name) {
                if let Some(regex_pattern) = &param_def.validation_regex {
                    let re = regex::Regex::new(regex_pattern)
                        .map_err(|e| format!("Invalid validation regex: {}", e))?;
                    if !re.is_match(value) {
                        return Err(format!(
                            "Parameter '{}' value '{}' does not match pattern '{}'",
                            param_def.name, value, regex_pattern
                        ));
                    }
                }
            }
        }

        // Substitute parameters in command pattern
        let mut command = self.command_pattern.clone();
        for (param_name, param_value) in params {
            let placeholder = format!("{{{{{}}}}}", param_name);
            command = command.replace(&placeholder, param_value);
        }

        // Check for remaining unsubstituted placeholders
        if command.contains("{{") {
            return Err(format!("Unsubstituted parameters in command: {}", command));
        }

        Ok(CommandRecipe {
            id: format!("{}_instance", self.id),
            command,
            category: self.category,
            safety_level: if matches!(self.category, CommandCategory::ReadOnly) {
                SafetyLevel::Safe
            } else {
                SafetyLevel::NeedsConfirmation
            },
            capture_output: true,
            expected_validation: self.validation_pattern.clone(),
            explanation: self.description.clone(),
            doc_sources: vec![self.wiki_source.clone()],
            rollback_command: None,
            template_id: Some(self.id.clone()),
            template_params: params.clone(),
        })
    }
}

/// Library of all available templates
pub struct TemplateLibrary {
    templates: HashMap<String, Template>,
}

impl TemplateLibrary {
    /// Create library with built-in templates
    pub fn new() -> Self {
        let mut library = Self {
            templates: HashMap::new(),
        };

        // Register all built-in templates
        library.register(Self::check_swap_status());
        library.register(Self::check_package_installed());
        library.register(Self::check_service_status());
        library.register(Self::check_gpu_memory());
        library.register(Self::enable_vim_syntax());
        library.register(Self::check_kernel_version());
        library.register(Self::check_disk_space());

        library
    }

    fn register(&mut self, template: Template) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get template by ID
    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.get(id)
    }

    /// Get all template IDs
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Search templates by description
    pub fn search(&self, query: &str) -> Vec<&Template> {
        let query_lower = query.to_lowercase();
        self.templates
            .values()
            .filter(|t| {
                t.description.to_lowercase().contains(&query_lower)
                    || t.name.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    // ===== BUILT-IN TEMPLATE DEFINITIONS =====

    fn check_swap_status() -> Template {
        Template {
            id: "check_swap_status".to_string(),
            name: "Check Swap Status".to_string(),
            description: "Check if swap is enabled and how much is configured".to_string(),
            parameters: vec![],
            command_pattern: "swapon --show".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Swap".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no swap)
                stdout_must_not_match: Some("error".to_string()),
                stderr_must_match: None,
                validation_description: "Command succeeds, output shows swap devices or is empty"
                    .to_string(),
            }),
            example: "swapon --show".to_string(),
        }
    }

    fn check_package_installed() -> Template {
        Template {
            id: "check_package_installed".to_string(),
            name: "Check Package Installed".to_string(),
            description: "Check if a specific package is installed".to_string(),
            parameters: vec![TemplateParameter {
                name: "package".to_string(),
                description: "Package name to check".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._+-]+$".to_string()),
                required: true,
            }],
            command_pattern: "pacman -Qi {{package}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Name\\s+:\\s+{{package}}".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Package info is displayed".to_string(),
            }),
            example: "pacman -Qi vim".to_string(),
        }
    }

    fn check_service_status() -> Template {
        Template {
            id: "check_service_status".to_string(),
            name: "Check Service Status".to_string(),
            description: "Check systemd service status".to_string(),
            parameters: vec![TemplateParameter {
                name: "service".to_string(),
                description: "Service name (e.g., sshd, NetworkManager)".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._@-]+$".to_string()),
                required: true,
            }],
            command_pattern: "systemctl status {{service}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("{{service}}\\.service".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Service status is displayed".to_string(),
            }),
            example: "systemctl status sshd".to_string(),
        }
    }

    fn check_gpu_memory() -> Template {
        Template {
            id: "check_gpu_memory".to_string(),
            name: "Check GPU Memory".to_string(),
            description: "Check total GPU memory (NVIDIA)".to_string(),
            parameters: vec![],
            command_pattern: "nvidia-smi --query-gpu=memory.total --format=csv,noheader"
                .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/NVIDIA".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some(r"\d+ MiB".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Output shows memory in MiB".to_string(),
            }),
            example: "nvidia-smi --query-gpu=memory.total --format=csv,noheader".to_string(),
        }
    }

    fn enable_vim_syntax() -> Template {
        Template {
            id: "enable_vim_syntax".to_string(),
            name: "Enable Vim Syntax Highlighting".to_string(),
            description: "Enable syntax highlighting in vim configuration".to_string(),
            parameters: vec![TemplateParameter {
                name: "vimrc_path".to_string(),
                description: "Path to vimrc file".to_string(),
                validation_regex: Some(r"^[/~].*\.?vimrc$".to_string()),
                required: true,
            }],
            command_pattern: "echo 'syntax on' >> {{vimrc_path}}".to_string(),
            category: CommandCategory::UserWrite,
            wiki_source: "https://wiki.archlinux.org/title/Vim".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: Some("error".to_string()),
                stderr_must_match: None,
                validation_description: "Command completes without error".to_string(),
            }),
            example: "echo 'syntax on' >> ~/.vimrc".to_string(),
        }
    }

    fn check_kernel_version() -> Template {
        Template {
            id: "check_kernel_version".to_string(),
            name: "Check Kernel Version".to_string(),
            description: "Get current kernel version".to_string(),
            parameters: vec![],
            command_pattern: "uname -r".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Kernel".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some(r"\d+\.\d+\.\d+".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Output shows kernel version number".to_string(),
            }),
            example: "uname -r".to_string(),
        }
    }

    fn check_disk_space() -> Template {
        Template {
            id: "check_disk_space".to_string(),
            name: "Check Disk Space".to_string(),
            description: "Check available disk space".to_string(),
            parameters: vec![],
            command_pattern: "df -h /".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/File_systems".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Filesystem".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Disk space table is displayed".to_string(),
            }),
            example: "df -h /".to_string(),
        }
    }
}

impl Default for TemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_instantiation() {
        let library = TemplateLibrary::new();
        let template = library.get("check_package_installed").unwrap();

        let mut params = HashMap::new();
        params.insert("package".to_string(), "vim".to_string());

        let recipe = template.instantiate(&params).unwrap();
        assert_eq!(recipe.command, "pacman -Qi vim");
    }

    #[test]
    fn test_missing_required_param() {
        let library = TemplateLibrary::new();
        let template = library.get("check_package_installed").unwrap();

        let params = HashMap::new(); // Empty - missing required param
        assert!(template.instantiate(&params).is_err());
    }

    #[test]
    fn test_swap_template() {
        let library = TemplateLibrary::new();
        let template = library.get("check_swap_status").unwrap();

        let params = HashMap::new(); // No params needed
        let recipe = template.instantiate(&params).unwrap();

        assert_eq!(recipe.command, "swapon --show");
        assert_eq!(recipe.category, CommandCategory::ReadOnly);
        assert!(recipe.is_auto_executable());
    }
}
