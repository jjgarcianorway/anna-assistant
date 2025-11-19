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
        library.register(Self::check_memory());

        // Beta.93: Additional telemetry templates
        library.register(Self::check_uptime());
        library.register(Self::check_cpu_model());
        library.register(Self::check_cpu_load());
        library.register(Self::check_distro());
        library.register(Self::check_failed_services());
        library.register(Self::check_journal_errors());

        // Beta.96: Critical fix for hallucination issue
        library.register(Self::system_weak_points_diagnostic());

        // Beta.96: Package Management templates
        library.register(Self::list_orphaned_packages());
        library.register(Self::check_package_integrity());
        library.register(Self::clean_package_cache());
        library.register(Self::list_package_files());
        library.register(Self::find_file_owner());
        library.register(Self::list_explicit_packages());
        library.register(Self::check_package_updates());
        library.register(Self::list_aur_packages());
        library.register(Self::package_depends());
        library.register(Self::package_reverse_depends());

        // Beta.96: Network Diagnostics templates
        library.register(Self::check_dns_resolution());
        library.register(Self::check_network_interfaces());
        library.register(Self::check_routing_table());
        library.register(Self::check_firewall_rules());
        library.register(Self::test_port_connectivity());
        library.register(Self::check_wifi_signal());
        library.register(Self::check_network_latency());
        library.register(Self::check_listening_ports());

        // Beta.97: Service Management templates
        library.register(Self::restart_service());
        library.register(Self::enable_service());
        library.register(Self::disable_service());
        library.register(Self::check_service_logs());
        library.register(Self::list_enabled_services());
        library.register(Self::list_running_services());

        // Beta.97: System Diagnostics templates
        library.register(Self::check_boot_time());
        library.register(Self::check_dmesg_errors());
        library.register(Self::check_disk_health());
        library.register(Self::check_temperature());
        library.register(Self::check_usb_devices());
        library.register(Self::check_pci_devices());

        // Beta.97: Configuration Management templates
        library.register(Self::backup_config_file());
        library.register(Self::show_config_file());
        library.register(Self::check_config_syntax());
        library.register(Self::list_loaded_modules());
        library.register(Self::check_hostname());

        // Beta.98: WiFi Troubleshooting templates (User reported WiFi issue ignored)
        library.register(Self::wifi_diagnostics());
        library.register(Self::check_networkmanager_status());
        library.register(Self::check_recent_kernel_updates());

        // Beta.102: Pacman Diagnostic templates (User's 200 questions - highest priority)
        library.register(Self::check_pacman_status());
        library.register(Self::check_pacman_locks());
        library.register(Self::check_dependency_conflicts());
        library.register(Self::check_pacman_cache_size());
        library.register(Self::show_recent_pacman_operations());
        library.register(Self::check_pending_updates());
        library.register(Self::check_pacman_mirrors());
        library.register(Self::check_archlinux_keyring());
        library.register(Self::check_failed_systemd_units());

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

    fn check_memory() -> Template {
        Template {
            id: "check_memory".to_string(),
            name: "Check System Memory".to_string(),
            description: "Check total and available system RAM".to_string(),
            parameters: vec![],
            command_pattern: "free -h".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Mem:".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Memory information is displayed".to_string(),
            }),
            example: "free -h".to_string(),
        }
    }

    // Beta.93: Additional telemetry templates

    fn check_uptime() -> Template {
        Template {
            id: "check_uptime".to_string(),
            name: "Check System Uptime".to_string(),
            description: "Check how long the system has been running".to_string(),
            parameters: vec![],
            command_pattern: "uptime".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("up".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Uptime displayed with load averages".to_string(),
            }),
            example: "uptime".to_string(),
        }
    }

    fn check_cpu_model() -> Template {
        Template {
            id: "check_cpu_model".to_string(),
            name: "Check CPU Model".to_string(),
            description: "Get CPU model name from /proc/cpuinfo".to_string(),
            parameters: vec![],
            command_pattern: "grep 'model name' /proc/cpuinfo | head -n 1".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/CPU".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("model name".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "CPU model name is displayed".to_string(),
            }),
            example: "grep 'model name' /proc/cpuinfo | head -n 1".to_string(),
        }
    }

    fn check_cpu_load() -> Template {
        Template {
            id: "check_cpu_load".to_string(),
            name: "Check CPU Load".to_string(),
            description: "Check current CPU load averages from /proc/loadavg".to_string(),
            parameters: vec![],
            command_pattern: "cat /proc/loadavg".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some(r"\d+\.\d+".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Load averages are displayed".to_string(),
            }),
            example: "cat /proc/loadavg".to_string(),
        }
    }

    fn check_distro() -> Template {
        Template {
            id: "check_distro".to_string(),
            name: "Check Distribution Info".to_string(),
            description: "Get Linux distribution information from /etc/os-release".to_string(),
            parameters: vec![],
            command_pattern: "cat /etc/os-release".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Arch_Linux".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("NAME=".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Distribution info is displayed".to_string(),
            }),
            example: "cat /etc/os-release".to_string(),
        }
    }

    fn check_failed_services() -> Template {
        Template {
            id: "check_failed_services".to_string(),
            name: "Check Failed Services".to_string(),
            description: "List failed systemd services".to_string(),
            parameters: vec![],
            command_pattern: "systemctl --failed".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no failures)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Failed services are listed, or none if all OK".to_string(),
            }),
            example: "systemctl --failed".to_string(),
        }
    }

    fn check_journal_errors() -> Template {
        Template {
            id: "check_journal_errors".to_string(),
            name: "Check Journal Errors".to_string(),
            description: "Show recent error-level messages from system journal".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -p 3 -xb".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no errors)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Journal errors are displayed if any exist".to_string(),
            }),
            example: "journalctl -p 3 -xb".to_string(),
        }
    }

    // Beta.96: Critical template to fix hallucination issue reported 2025-11-19
    // User reported: "Asked about weak points and it hallucinated '0% storage free space'"
    fn system_weak_points_diagnostic() -> Template {
        Template {
            id: "system_weak_points_diagnostic".to_string(),
            name: "System Weak Points Diagnostic".to_string(),
            description: "Comprehensive system diagnostic to identify actual weak points and issues - no hallucination, real data only".to_string(),
            parameters: vec![],
            command_pattern: r#"printf "=== STORAGE ===\n" && df -h / && printf "\n=== MEMORY ===\n" && free -h && printf "\n=== CPU LOAD ===\n" && uptime && printf "\n=== FAILED SERVICES ===\n" && systemctl --failed --no-pager && printf "\n=== RECENT ERRORS (last 20) ===\n" && journalctl -p err -b --no-pager -n 20"#.to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("STORAGE".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Diagnostic output includes all system health checks".to_string(),
            }),
            example: r#"printf "=== STORAGE ===\n" && df -h / && printf "\n=== MEMORY ===\n" && free -h && ..."#.to_string(),
        }
    }

    // ===== BETA.96: PACKAGE MANAGEMENT TEMPLATES =====

    fn list_orphaned_packages() -> Template {
        Template {
            id: "list_orphaned_packages".to_string(),
            name: "List Orphaned Packages".to_string(),
            description: "List packages that were installed as dependencies but are no longer required".to_string(),
            parameters: vec![],
            command_pattern: "pacman -Qdt".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Removing_packages".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no orphans)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Lists orphaned packages or shows empty if none".to_string(),
            }),
            example: "pacman -Qdt".to_string(),
        }
    }

    fn check_package_integrity() -> Template {
        Template {
            id: "check_package_integrity".to_string(),
            name: "Check Package File Integrity".to_string(),
            description: "Verify integrity of installed package files".to_string(),
            parameters: vec![TemplateParameter {
                name: "package".to_string(),
                description: "Package name to verify".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._+-]+$".to_string()),
                required: true,
            }],
            command_pattern: "pacman -Qk {{package}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("{{package}}".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Package integrity check results displayed".to_string(),
            }),
            example: "pacman -Qk linux".to_string(),
        }
    }

    fn clean_package_cache() -> Template {
        Template {
            id: "clean_package_cache".to_string(),
            name: "Clean Package Cache".to_string(),
            description: "Remove all cached package files to free disk space".to_string(),
            parameters: vec![],
            command_pattern: "sudo pacman -Scc --noconfirm".to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: Some("error".to_string()),
                stderr_must_match: None,
                validation_description: "Cache cleaned successfully".to_string(),
            }),
            example: "sudo pacman -Scc --noconfirm".to_string(),
        }
    }

    fn list_package_files() -> Template {
        Template {
            id: "list_package_files".to_string(),
            name: "List Package Files".to_string(),
            description: "List all files installed by a package".to_string(),
            parameters: vec![TemplateParameter {
                name: "package".to_string(),
                description: "Package name to list files for".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._+-]+$".to_string()),
                required: true,
            }],
            command_pattern: "pacman -Ql {{package}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("{{package}}".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "List of files displayed".to_string(),
            }),
            example: "pacman -Ql vim".to_string(),
        }
    }

    fn find_file_owner() -> Template {
        Template {
            id: "find_file_owner".to_string(),
            name: "Find File Owner Package".to_string(),
            description: "Find which package owns a specific file".to_string(),
            parameters: vec![TemplateParameter {
                name: "filepath".to_string(),
                description: "Full path to the file".to_string(),
                validation_regex: Some(r"^/.*".to_string()),
                required: true,
            }],
            command_pattern: "pacman -Qo {{filepath}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("owned by".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Shows which package owns the file".to_string(),
            }),
            example: "pacman -Qo /usr/bin/vim".to_string(),
        }
    }

    fn list_explicit_packages() -> Template {
        Template {
            id: "list_explicit_packages".to_string(),
            name: "List Explicitly Installed Packages".to_string(),
            description: "List packages that were explicitly installed by the user".to_string(),
            parameters: vec![],
            command_pattern: "pacman -Qe".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "List of explicitly installed packages".to_string(),
            }),
            example: "pacman -Qe".to_string(),
        }
    }

    fn check_package_updates() -> Template {
        Template {
            id: "check_package_updates".to_string(),
            name: "Check for Package Updates".to_string(),
            description: "List packages with available updates".to_string(),
            parameters: vec![],
            command_pattern: "checkupdates".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance#Check_for_updates".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no updates)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Lists available updates or shows empty if system is up to date".to_string(),
            }),
            example: "checkupdates".to_string(),
        }
    }

    fn list_aur_packages() -> Template {
        Template {
            id: "list_aur_packages".to_string(),
            name: "List AUR Packages".to_string(),
            description: "List packages installed from AUR (foreign packages)".to_string(),
            parameters: vec![],
            command_pattern: "pacman -Qm".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Arch_User_Repository".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no AUR packages)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Lists foreign/AUR packages".to_string(),
            }),
            example: "pacman -Qm".to_string(),
        }
    }

    fn package_depends() -> Template {
        Template {
            id: "package_depends".to_string(),
            name: "Show Package Dependencies".to_string(),
            description: "Show all dependencies of a package".to_string(),
            parameters: vec![TemplateParameter {
                name: "package".to_string(),
                description: "Package name to check dependencies for".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._+-]+$".to_string()),
                required: true,
            }],
            command_pattern: "pactree {{package}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("{{package}}".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Dependency tree displayed".to_string(),
            }),
            example: "pactree vim".to_string(),
        }
    }

    fn package_reverse_depends() -> Template {
        Template {
            id: "package_reverse_depends".to_string(),
            name: "Show Packages Depending On This".to_string(),
            description: "Show which packages depend on this package".to_string(),
            parameters: vec![TemplateParameter {
                name: "package".to_string(),
                description: "Package name to check reverse dependencies for".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._+-]+$".to_string()),
                required: true,
            }],
            command_pattern: "pactree -r {{package}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("{{package}}".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Reverse dependency tree displayed".to_string(),
            }),
            example: "pactree -r glibc".to_string(),
        }
    }

    // ===== BETA.96: NETWORK DIAGNOSTICS TEMPLATES =====

    fn check_dns_resolution() -> Template {
        Template {
            id: "check_dns_resolution".to_string(),
            name: "Check DNS Resolution".to_string(),
            description: "Test DNS resolution for a domain".to_string(),
            parameters: vec![TemplateParameter {
                name: "domain".to_string(),
                description: "Domain name to resolve (e.g., archlinux.org)".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string()),
                required: true,
            }],
            command_pattern: "nslookup {{domain}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_Debugging".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Address".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "DNS resolution successful".to_string(),
            }),
            example: "nslookup archlinux.org".to_string(),
        }
    }

    fn check_network_interfaces() -> Template {
        Template {
            id: "check_network_interfaces".to_string(),
            name: "Check Network Interfaces".to_string(),
            description: "List all network interfaces and their status".to_string(),
            parameters: vec![],
            command_pattern: "ip -br addr".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Network interfaces listed with IP addresses".to_string(),
            }),
            example: "ip -br addr".to_string(),
        }
    }

    fn check_routing_table() -> Template {
        Template {
            id: "check_routing_table".to_string(),
            name: "Check Routing Table".to_string(),
            description: "Display the kernel routing table".to_string(),
            parameters: vec![],
            command_pattern: "ip route".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration#Routing_table".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Routing table displayed".to_string(),
            }),
            example: "ip route".to_string(),
        }
    }

    fn check_firewall_rules() -> Template {
        Template {
            id: "check_firewall_rules".to_string(),
            name: "Check Firewall Rules".to_string(),
            description: "List current firewall rules (iptables)".to_string(),
            parameters: vec![],
            command_pattern: "sudo iptables -L -n -v".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Iptables".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Chain".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Firewall rules listed".to_string(),
            }),
            example: "sudo iptables -L -n -v".to_string(),
        }
    }

    fn test_port_connectivity() -> Template {
        Template {
            id: "test_port_connectivity".to_string(),
            name: "Test Port Connectivity".to_string(),
            description: "Test if a remote port is accessible".to_string(),
            parameters: vec![
                TemplateParameter {
                    name: "host".to_string(),
                    description: "Hostname or IP address".to_string(),
                    validation_regex: Some(r"^[a-zA-Z0-9.-]+$".to_string()),
                    required: true,
                },
                TemplateParameter {
                    name: "port".to_string(),
                    description: "Port number (e.g., 80, 443, 22)".to_string(),
                    validation_regex: Some(r"^\d{1,5}$".to_string()),
                    required: true,
                },
            ],
            command_pattern: "nc -zv {{host}} {{port}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_Debugging".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: Some("succeeded".to_string()),
                validation_description: "Port is open and accessible".to_string(),
            }),
            example: "nc -zv archlinux.org 443".to_string(),
        }
    }

    fn check_wifi_signal() -> Template {
        Template {
            id: "check_wifi_signal".to_string(),
            name: "Check WiFi Signal Strength".to_string(),
            description: "Show WiFi signal strength and quality".to_string(),
            parameters: vec![],
            command_pattern: "iwconfig 2>/dev/null | grep -E 'Quality|Signal'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Wireless#Get_the_name_of_the_interface".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty if no WiFi
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "WiFi signal information or empty if no WiFi".to_string(),
            }),
            example: "iwconfig 2>/dev/null | grep -E 'Quality|Signal'".to_string(),
        }
    }

    fn check_network_latency() -> Template {
        Template {
            id: "check_network_latency".to_string(),
            name: "Check Network Latency".to_string(),
            description: "Test network latency with ping".to_string(),
            parameters: vec![TemplateParameter {
                name: "host".to_string(),
                description: "Host to ping (e.g., 8.8.8.8, archlinux.org)".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9.-]+$".to_string()),
                required: true,
            }],
            command_pattern: "ping -c 4 {{host}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_Debugging".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("packets transmitted".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Ping statistics displayed".to_string(),
            }),
            example: "ping -c 4 8.8.8.8".to_string(),
        }
    }

    fn check_listening_ports() -> Template {
        Template {
            id: "check_listening_ports".to_string(),
            name: "Check Listening Ports".to_string(),
            description: "List all ports that services are listening on".to_string(),
            parameters: vec![],
            command_pattern: "ss -tulpn".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_Debugging".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("State".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Listening ports and services displayed".to_string(),
            }),
            example: "ss -tulpn".to_string(),
        }
    }

    // ===== BETA.97: SERVICE MANAGEMENT TEMPLATES =====

    fn restart_service() -> Template {
        Template {
            id: "restart_service".to_string(),
            name: "Restart Service".to_string(),
            description: "Restart a systemd service".to_string(),
            parameters: vec![TemplateParameter {
                name: "service".to_string(),
                description: "Service name (e.g., sshd, NetworkManager)".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._@-]+$".to_string()),
                required: true,
            }],
            command_pattern: "sudo systemctl restart {{service}}".to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Using_units".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: Some("failed".to_string()),
                stderr_must_match: None,
                validation_description: "Service restarts successfully".to_string(),
            }),
            example: "sudo systemctl restart sshd".to_string(),
        }
    }

    fn enable_service() -> Template {
        Template {
            id: "enable_service".to_string(),
            name: "Enable Service at Boot".to_string(),
            description: "Enable a service to start automatically at boot".to_string(),
            parameters: vec![TemplateParameter {
                name: "service".to_string(),
                description: "Service name to enable".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._@-]+$".to_string()),
                required: true,
            }],
            command_pattern: "sudo systemctl enable {{service}}".to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Using_units".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Created symlink".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Service enabled successfully".to_string(),
            }),
            example: "sudo systemctl enable sshd".to_string(),
        }
    }

    fn disable_service() -> Template {
        Template {
            id: "disable_service".to_string(),
            name: "Disable Service at Boot".to_string(),
            description: "Disable a service from starting automatically at boot".to_string(),
            parameters: vec![TemplateParameter {
                name: "service".to_string(),
                description: "Service name to disable".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._@-]+$".to_string()),
                required: true,
            }],
            command_pattern: "sudo systemctl disable {{service}}".to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Using_units".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Removed".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Service disabled successfully".to_string(),
            }),
            example: "sudo systemctl disable bluetooth".to_string(),
        }
    }

    fn check_service_logs() -> Template {
        Template {
            id: "check_service_logs".to_string(),
            name: "Check Service Logs".to_string(),
            description: "View recent logs for a systemd service".to_string(),
            parameters: vec![TemplateParameter {
                name: "service".to_string(),
                description: "Service name to check logs for".to_string(),
                validation_regex: Some(r"^[a-zA-Z0-9._@-]+$".to_string()),
                required: true,
            }],
            command_pattern: "journalctl -u {{service}} -n 50 --no-pager".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Service logs displayed".to_string(),
            }),
            example: "journalctl -u sshd -n 50 --no-pager".to_string(),
        }
    }

    fn list_enabled_services() -> Template {
        Template {
            id: "list_enabled_services".to_string(),
            name: "List Enabled Services".to_string(),
            description: "List all services enabled to start at boot".to_string(),
            parameters: vec![],
            command_pattern: "systemctl list-unit-files --state=enabled --no-pager".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Using_units".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("UNIT FILE".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "List of enabled services displayed".to_string(),
            }),
            example: "systemctl list-unit-files --state=enabled --no-pager".to_string(),
        }
    }

    fn list_running_services() -> Template {
        Template {
            id: "list_running_services".to_string(),
            name: "List Running Services".to_string(),
            description: "List all currently running services".to_string(),
            parameters: vec![],
            command_pattern: "systemctl list-units --type=service --state=running --no-pager".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Using_units".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("UNIT".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "List of running services displayed".to_string(),
            }),
            example: "systemctl list-units --type=service --state=running --no-pager".to_string(),
        }
    }

    // ===== BETA.97: SYSTEM DIAGNOSTICS TEMPLATES =====

    fn check_boot_time() -> Template {
        Template {
            id: "check_boot_time".to_string(),
            name: "Check Boot Time".to_string(),
            description: "Analyze system boot time and identify slow services".to_string(),
            parameters: vec![],
            command_pattern: "systemd-analyze blame | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Improving_performance/Boot_process".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("ms".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Boot time analysis displayed".to_string(),
            }),
            example: "systemd-analyze blame | head -20".to_string(),
        }
    }

    fn check_dmesg_errors() -> Template {
        Template {
            id: "check_dmesg_errors".to_string(),
            name: "Check Kernel Messages for Errors".to_string(),
            description: "Check kernel ring buffer for error messages".to_string(),
            parameters: vec![],
            command_pattern: "dmesg --level=err,warn --human --nopager | tail -50".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Kernel".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Kernel error messages displayed".to_string(),
            }),
            example: "dmesg --level=err,warn --human --nopager | tail -50".to_string(),
        }
    }

    fn check_disk_health() -> Template {
        Template {
            id: "check_disk_health".to_string(),
            name: "Check Disk Health (SMART)".to_string(),
            description: "Check disk health status using SMART".to_string(),
            parameters: vec![TemplateParameter {
                name: "device".to_string(),
                description: "Device path (e.g., /dev/sda, /dev/nvme0n1)".to_string(),
                validation_regex: Some(r"^/dev/[a-zA-Z0-9]+$".to_string()),
                required: true,
            }],
            command_pattern: "sudo smartctl -H {{device}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/S.M.A.R.T.".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("SMART".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Disk health status displayed".to_string(),
            }),
            example: "sudo smartctl -H /dev/sda".to_string(),
        }
    }

    fn check_temperature() -> Template {
        Template {
            id: "check_temperature".to_string(),
            name: "Check System Temperature".to_string(),
            description: "Check CPU and system temperatures using sensors".to_string(),
            parameters: vec![],
            command_pattern: "sensors".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Lm_sensors".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Temperature sensors displayed".to_string(),
            }),
            example: "sensors".to_string(),
        }
    }

    fn check_usb_devices() -> Template {
        Template {
            id: "check_usb_devices".to_string(),
            name: "List USB Devices".to_string(),
            description: "List all connected USB devices".to_string(),
            parameters: vec![],
            command_pattern: "lsusb".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/USB".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Bus".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "USB devices listed".to_string(),
            }),
            example: "lsusb".to_string(),
        }
    }

    fn check_pci_devices() -> Template {
        Template {
            id: "check_pci_devices".to_string(),
            name: "List PCI Devices".to_string(),
            description: "List all PCI devices (graphics, network, etc)".to_string(),
            parameters: vec![],
            command_pattern: "lspci".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/PCI".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "PCI devices listed".to_string(),
            }),
            example: "lspci".to_string(),
        }
    }

    // ===== BETA.97: CONFIGURATION MANAGEMENT TEMPLATES =====

    fn backup_config_file() -> Template {
        Template {
            id: "backup_config_file".to_string(),
            name: "Backup Configuration File".to_string(),
            description: "Create a timestamped backup of a configuration file".to_string(),
            parameters: vec![TemplateParameter {
                name: "filepath".to_string(),
                description: "Path to config file to backup".to_string(),
                validation_regex: Some(r"^/.*".to_string()),
                required: true,
            }],
            command_pattern: r#"sudo cp {{filepath}} {{filepath}}.backup.$(date +%Y%m%d_%H%M%S)"#.to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: Some("cannot".to_string()),
                stderr_must_match: None,
                validation_description: "Backup created successfully".to_string(),
            }),
            example: r#"sudo cp /etc/pacman.conf /etc/pacman.conf.backup.$(date +%Y%m%d_%H%M%S)"#.to_string(),
        }
    }

    fn show_config_file() -> Template {
        Template {
            id: "show_config_file".to_string(),
            name: "Show Configuration File".to_string(),
            description: "Display contents of a configuration file".to_string(),
            parameters: vec![TemplateParameter {
                name: "filepath".to_string(),
                description: "Path to config file".to_string(),
                validation_regex: Some(r"^/.*".to_string()),
                required: true,
            }],
            command_pattern: "cat {{filepath}}".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Config file contents displayed".to_string(),
            }),
            example: "cat /etc/pacman.conf".to_string(),
        }
    }

    fn check_config_syntax() -> Template {
        Template {
            id: "check_config_syntax".to_string(),
            name: "Check Config Syntax (nginx)".to_string(),
            description: "Test nginx configuration syntax".to_string(),
            parameters: vec![],
            command_pattern: "sudo nginx -t".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Nginx".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: Some("syntax is ok".to_string()),
                validation_description: "Config syntax is valid".to_string(),
            }),
            example: "sudo nginx -t".to_string(),
        }
    }

    fn list_loaded_modules() -> Template {
        Template {
            id: "list_loaded_modules".to_string(),
            name: "List Loaded Kernel Modules".to_string(),
            description: "List all currently loaded kernel modules".to_string(),
            parameters: vec![],
            command_pattern: "lsmod".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Kernel_module".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Module".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Kernel modules listed".to_string(),
            }),
            example: "lsmod".to_string(),
        }
    }

    fn check_hostname() -> Template {
        Template {
            id: "check_hostname".to_string(),
            name: "Check System Hostname".to_string(),
            description: "Display current system hostname".to_string(),
            parameters: vec![],
            command_pattern: "hostnamectl".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration#Set_the_hostname".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("hostname".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Hostname information displayed".to_string(),
            }),
            example: "hostnamectl".to_string(),
        }
    }

    // Beta.98: WiFi Troubleshooting Templates (Critical - User reported WiFi slowness ignored)
    fn wifi_diagnostics() -> Template {
        Template {
            id: "wifi_diagnostics".to_string(),
            name: "WiFi Performance Diagnostics".to_string(),
            description: "Comprehensive WiFi diagnostics: signal strength, link speed, errors, driver info".to_string(),
            parameters: vec![],
            command_pattern: r#"printf "=== WIFI DIAGNOSTICS ===\n\n" && printf "Signal & Speed:\n" && iwconfig 2>/dev/null | grep -A 10 "IEEE 802.11" && printf "\n\nNetwork Interfaces:\n" && ip addr show | grep -A 5 "wl" && printf "\n\nRecent WiFi Errors (last 20):\n" && journalctl -u NetworkManager --no-pager -n 20 | grep -i "wifi\|wlan" && printf "\n\nDriver Info:\n" && lspci -k | grep -A 3 -i "network\|wireless""#.to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration/Wireless".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("WIFI DIAGNOSTICS".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "WiFi diagnostics completed".to_string(),
            }),
            example: "wifi_diagnostics".to_string(),
        }
    }

    fn check_networkmanager_status() -> Template {
        Template {
            id: "check_networkmanager_status".to_string(),
            name: "Check NetworkManager Status".to_string(),
            description: "Check NetworkManager service status and recent logs for network issues".to_string(),
            parameters: vec![],
            command_pattern: "systemctl status NetworkManager --no-pager -l && printf '\n\n=== Recent Errors ===\n' && journalctl -u NetworkManager -p err --no-pager -n 10".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/NetworkManager".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("NetworkManager".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "NetworkManager status checked".to_string(),
            }),
            example: "systemctl status NetworkManager".to_string(),
        }
    }

    fn check_recent_kernel_updates() -> Template {
        Template {
            id: "check_recent_kernel_updates".to_string(),
            name: "Check Recent Kernel/Driver Updates".to_string(),
            description: "Check for recent kernel or driver updates that might affect WiFi performance".to_string(),
            parameters: vec![],
            command_pattern: r"pacman -Qi linux | grep -E 'Name|Version|Install Date' && printf '\n' && pacman -Ql linux | grep -i 'wireless\|wifi' | head -5".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Kernel".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("linux".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Kernel update info displayed".to_string(),
            }),
            example: "pacman -Qi linux".to_string(),
        }
    }

    // ===== BETA.102: PACMAN DIAGNOSTIC TEMPLATES =====
    // User's 200 practical questions showed Pacman issues in 20+ questions
    // Priority 1: Most common user problems need actionable diagnostics

    fn check_pacman_status() -> Template {
        Template {
            id: "check_pacman_status".to_string(),
            name: "Check Pacman Status".to_string(),
            description: "Check if Pacman is working correctly and display current configuration".to_string(),
            parameters: vec![],
            command_pattern: "pacman --version && echo && grep -v '^#' /etc/pacman.conf | grep -v '^$'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Pacman".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Pacman version and config displayed".to_string(),
            }),
            example: "pacman --version".to_string(),
        }
    }

    fn check_pacman_locks() -> Template {
        Template {
            id: "check_pacman_locks".to_string(),
            name: "Check Pacman Lock Files".to_string(),
            description: "Check for stale Pacman lock files that prevent package operations".to_string(),
            parameters: vec![],
            command_pattern: "ls -lh /var/lib/pacman/db.lck 2>/dev/null && echo 'Lock exists - remove with: sudo rm /var/lib/pacman/db.lck' || echo 'No lock file - Pacman is available'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman/Troubleshooting#Failed_to_init_transaction_(unable_to_lock_database)".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("lock".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Lock file status displayed".to_string(),
            }),
            example: "ls -lh /var/lib/pacman/db.lck".to_string(),
        }
    }

    fn check_dependency_conflicts() -> Template {
        Template {
            id: "check_dependency_conflicts".to_string(),
            name: "Check Dependency Conflicts".to_string(),
            description: "Check for broken dependencies and package conflicts".to_string(),
            parameters: vec![],
            command_pattern: "pacman -Qk 2>&1 | head -30".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman/Troubleshooting".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Dependency check results displayed".to_string(),
            }),
            example: "pacman -Qk".to_string(),
        }
    }

    fn check_pacman_cache_size() -> Template {
        Template {
            id: "check_pacman_cache_size".to_string(),
            name: "Check Pacman Cache Size".to_string(),
            description: "Show size of Pacman package cache".to_string(),
            parameters: vec![],
            command_pattern: "du -sh /var/cache/pacman/pkg/ && echo && ls /var/cache/pacman/pkg/*.pkg.tar.* 2>/dev/null | wc -l && echo 'cached packages'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("pkg".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Cache size displayed".to_string(),
            }),
            example: "du -sh /var/cache/pacman/pkg/".to_string(),
        }
    }

    fn show_recent_pacman_operations() -> Template {
        Template {
            id: "show_recent_pacman_operations".to_string(),
            name: "Show Recent Pacman Operations".to_string(),
            description: "Display recent package installations, updates, and removals".to_string(),
            parameters: vec![],
            command_pattern: "grep -E '\\[ALPM\\] (installed|upgraded|removed)' /var/log/pacman.log 2>/dev/null | tail -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Pacman_log".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("ALPM".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Recent operations displayed".to_string(),
            }),
            example: "grep 'ALPM' /var/log/pacman.log | tail -20".to_string(),
        }
    }

    fn check_pending_updates() -> Template {
        Template {
            id: "check_pending_updates".to_string(),
            name: "Check Pending Updates".to_string(),
            description: "Check for available package updates".to_string(),
            parameters: vec![],
            command_pattern: "checkupdates || echo 'No updates available (or checkupdates not installed)'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Available updates listed".to_string(),
            }),
            example: "checkupdates".to_string(),
        }
    }

    fn check_pacman_mirrors() -> Template {
        Template {
            id: "check_pacman_mirrors".to_string(),
            name: "Check Pacman Mirror Configuration".to_string(),
            description: "Display configured Pacman mirrors".to_string(),
            parameters: vec![],
            command_pattern: "grep '^Server' /etc/pacman.d/mirrorlist | head -10".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Mirrors".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Server".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Mirror list displayed".to_string(),
            }),
            example: "grep Server /etc/pacman.d/mirrorlist".to_string(),
        }
    }

    fn check_archlinux_keyring() -> Template {
        Template {
            id: "check_archlinux_keyring".to_string(),
            name: "Check Arch Linux Keyring Status".to_string(),
            description: "Check GPG keyring status and version".to_string(),
            parameters: vec![],
            command_pattern: "pacman -Q archlinux-keyring && echo && pacman-key --list-keys | grep -E 'pub|uid' | head -10".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman/Package_signing".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("keyring".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Keyring status displayed".to_string(),
            }),
            example: "pacman -Q archlinux-keyring".to_string(),
        }
    }

    fn check_failed_systemd_units() -> Template {
        Template {
            id: "check_failed_systemd_units".to_string(),
            name: "Check Failed Systemd Units".to_string(),
            description: "List all failed systemd units".to_string(),
            parameters: vec![],
            command_pattern: "systemctl list-units --failed --all".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Basic_systemctl_usage".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("UNIT".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Failed units listed".to_string(),
            }),
            example: "systemctl --failed".to_string(),
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
