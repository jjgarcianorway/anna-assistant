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

use crate::command_recipe::{CommandCategory, CommandRecipe, OutputValidation, SafetyLevel};
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

        // Beta.103: Systemd Boot Analysis templates (700 questions - systemd diagnostics)
        library.register(Self::analyze_boot_time());
        library.register(Self::check_boot_errors());
        library.register(Self::show_boot_log());
        library.register(Self::analyze_boot_critical_chain());
        library.register(Self::check_systemd_timers());
        library.register(Self::analyze_journal_size());
        library.register(Self::check_systemd_version());
        library.register(Self::show_recent_journal_errors());

        // Beta.104: CPU & Performance Profiling templates (700 questions - performance diagnostics)
        library.register(Self::check_cpu_frequency());
        library.register(Self::check_cpu_governor());
        library.register(Self::analyze_cpu_usage());
        library.register(Self::check_cpu_temperature());
        library.register(Self::detect_cpu_throttling());
        library.register(Self::show_top_cpu_processes());
        library.register(Self::check_load_average());
        library.register(Self::analyze_context_switches());

        // Beta.105: Memory & Swap diagnostics (700 questions - memory issues)
        library.register(Self::check_memory_usage());
        library.register(Self::check_swap_usage());
        library.register(Self::analyze_memory_pressure());
        library.register(Self::show_top_memory_processes());
        library.register(Self::check_oom_killer());
        library.register(Self::analyze_swap_activity());
        library.register(Self::check_huge_pages());
        library.register(Self::show_memory_info());

        // Beta.106: GPU diagnostics (700 questions - GPU issues)
        library.register(Self::check_gpu_info());
        library.register(Self::check_gpu_drivers());
        library.register(Self::check_nvidia_status());
        library.register(Self::check_amd_gpu());
        library.register(Self::check_gpu_processes());
        library.register(Self::check_gpu_temperature());
        library.register(Self::check_gpu_errors());
        library.register(Self::analyze_graphics_performance());

        // Beta.107: Desktop Environment diagnostics (700 questions - desktop/DE issues)
        library.register(Self::check_display_server());
        library.register(Self::check_desktop_environment());
        library.register(Self::check_display_manager());
        library.register(Self::analyze_xorg_errors());
        library.register(Self::check_wayland_compositor());
        library.register(Self::check_desktop_session());
        library.register(Self::analyze_desktop_performance());
        library.register(Self::check_window_manager());

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
            description:
                "List packages that were installed as dependencies but are no longer required"
                    .to_string(),
            parameters: vec![],
            command_pattern: "pacman -Qdt".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Removing_packages".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no orphans)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Lists orphaned packages or shows empty if none"
                    .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Cleaning_the_package_cache"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            description: "List packages with available updates (official repos and AUR)".to_string(),
            parameters: vec![],
            command_pattern: "yay -Qu 2>/dev/null || pacman -Qu 2>/dev/null || checkupdates 2>/dev/null || echo 'Unable to check for updates'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance#Check_for_updates"
                .to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None, // Empty output is valid (no updates)
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description:
                    "Lists available updates, shows empty if up to date, or reports unable to check".to_string(),
            }),
            example: "yay -Qu".to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration#Routing_table"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Wireless#Get_the_name_of_the_interface"
                .to_string(),
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
            command_pattern: "systemctl list-units --type=service --state=running --no-pager"
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Improving_performance/Boot_process"
                .to_string(),
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
            command_pattern: r#"sudo cp {{filepath}} {{filepath}}.backup.$(date +%Y%m%d_%H%M%S)"#
                .to_string(),
            category: CommandCategory::SystemWrite,
            wiki_source: "https://wiki.archlinux.org/title/System_maintenance".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: Some("cannot".to_string()),
                stderr_must_match: None,
                validation_description: "Backup created successfully".to_string(),
            }),
            example: r#"sudo cp /etc/pacman.conf /etc/pacman.conf.backup.$(date +%Y%m%d_%H%M%S)"#
                .to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Network_configuration#Set_the_hostname"
                .to_string(),
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
            description: "Check if Pacman is working correctly and display current configuration"
                .to_string(),
            parameters: vec![],
            command_pattern:
                "pacman --version && echo && grep -v '^#' /etc/pacman.conf | grep -v '^$'"
                    .to_string(),
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
            description: "Check for available package updates (both official repos and AUR)".to_string(),
            parameters: vec![],
            command_pattern:
                "yay -Qu 2>/dev/null || pacman -Qu 2>/dev/null || checkupdates 2>/dev/null || echo 'Unable to check for updates'"
                    .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Pacman#Querying_package_databases"
                .to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Available updates listed or unable to check message".to_string(),
            }),
            example: "yay -Qu".to_string(),
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
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Basic_systemctl_usage"
                .to_string(),
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

    // ============================================================================
    // Beta.103: Systemd Boot Analysis Templates (700-question test suite)
    // ============================================================================

    fn analyze_boot_time() -> Template {
        Template {
            id: "analyze_boot_time".to_string(),
            name: "Analyze Boot Time".to_string(),
            description: "Show systemd boot time analysis with service breakdown".to_string(),
            parameters: vec![],
            command_pattern: "systemd-analyze && echo && systemd-analyze blame | head -20"
                .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Analyzing_the_boot_process"
                .to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Startup finished".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Boot time analysis displayed".to_string(),
            }),
            example: "systemd-analyze blame".to_string(),
        }
    }

    fn check_boot_errors() -> Template {
        Template {
            id: "check_boot_errors".to_string(),
            name: "Check Boot Errors".to_string(),
            description: "Show boot-time errors and warnings from journal".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -b -p err..warning --no-pager | head -50".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Boot errors displayed".to_string(),
            }),
            example: "journalctl -b -p err".to_string(),
        }
    }

    fn show_boot_log() -> Template {
        Template {
            id: "show_boot_log".to_string(),
            name: "Show Boot Log".to_string(),
            description: "Display detailed boot log with kernel messages".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -b --no-pager | head -100".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("kernel".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Boot log displayed".to_string(),
            }),
            example: "journalctl -b".to_string(),
        }
    }

    fn analyze_boot_critical_chain() -> Template {
        Template {
            id: "analyze_boot_critical_chain".to_string(),
            name: "Analyze Boot Critical Chain".to_string(),
            description: "Show critical boot path and time-critical units".to_string(),
            parameters: vec![],
            command_pattern: "systemd-analyze critical-chain".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd#Analyzing_the_boot_process"
                .to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("graphical.target".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Critical chain displayed".to_string(),
            }),
            example: "systemd-analyze critical-chain".to_string(),
        }
    }

    fn check_systemd_timers() -> Template {
        Template {
            id: "check_systemd_timers".to_string(),
            name: "Check Systemd Timers".to_string(),
            description: "List all systemd timers and their next execution time".to_string(),
            parameters: vec![],
            command_pattern: "systemctl list-timers --all".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Timers".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("NEXT".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Timers listed".to_string(),
            }),
            example: "systemctl list-timers".to_string(),
        }
    }

    fn analyze_journal_size() -> Template {
        Template {
            id: "analyze_journal_size".to_string(),
            name: "Analyze Journal Size".to_string(),
            description: "Show journal disk usage and configuration".to_string(),
            parameters: vec![],
            command_pattern: "journalctl --disk-usage && echo && du -sh /var/log/journal/*"
                .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal#Journal_size_limit"
                .to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Archived and active journals".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Journal size displayed".to_string(),
            }),
            example: "journalctl --disk-usage".to_string(),
        }
    }

    fn check_systemd_version() -> Template {
        Template {
            id: "check_systemd_version".to_string(),
            name: "Check Systemd Version".to_string(),
            description: "Show systemd version and compiled features".to_string(),
            parameters: vec![],
            command_pattern: "systemctl --version".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("systemd".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Systemd version displayed".to_string(),
            }),
            example: "systemctl --version".to_string(),
        }
    }

    fn show_recent_journal_errors() -> Template {
        Template {
            id: "show_recent_journal_errors".to_string(),
            name: "Show Recent Journal Errors".to_string(),
            description: "Display recent system errors from the last hour".to_string(),
            parameters: vec![],
            command_pattern: "journalctl --since '1 hour ago' -p err --no-pager | tail -50"
                .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Systemd/Journal".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Recent errors displayed".to_string(),
            }),
            example: "journalctl -p err --since today".to_string(),
        }
    }

    // ============================================================================
    // Beta.104: CPU & Performance Profiling Templates (700-question test suite)
    // ============================================================================

    fn check_cpu_frequency() -> Template {
        Template {
            id: "check_cpu_frequency".to_string(),
            name: "Check CPU Frequency".to_string(),
            description: "Show current CPU frequency and available scaling frequencies".to_string(),
            parameters: vec![],
            command_pattern: "cat /proc/cpuinfo | grep MHz | head -20 && echo && ls /sys/devices/system/cpu/cpu0/cpufreq/ 2>/dev/null | head -10".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/CPU_frequency_scaling".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("MHz".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "CPU frequency displayed".to_string(),
            }),
            example: "grep MHz /proc/cpuinfo".to_string(),
        }
    }

    fn check_cpu_governor() -> Template {
        Template {
            id: "check_cpu_governor".to_string(),
            name: "Check CPU Governor".to_string(),
            description: "Show active CPU frequency scaling governor for all cores".to_string(),
            parameters: vec![],
            command_pattern: "for cpu in /sys/devices/system/cpu/cpu[0-9]*; do echo \"$cpu: $(cat $cpu/cpufreq/scaling_governor 2>/dev/null || echo 'N/A')\"; done | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/CPU_frequency_scaling#Scaling_governors".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("cpu".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "CPU governors displayed".to_string(),
            }),
            example: "cat /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor".to_string(),
        }
    }

    fn analyze_cpu_usage() -> Template {
        Template {
            id: "analyze_cpu_usage".to_string(),
            name: "Analyze CPU Usage".to_string(),
            description: "Show per-core CPU utilization breakdown".to_string(),
            parameters: vec![],
            command_pattern: "mpstat -P ALL 1 1 2>/dev/null || top -bn1 | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Core_utilities".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("CPU".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "CPU usage displayed".to_string(),
            }),
            example: "mpstat -P ALL".to_string(),
        }
    }

    fn check_cpu_temperature() -> Template {
        Template {
            id: "check_cpu_temperature".to_string(),
            name: "Check CPU Temperature".to_string(),
            description: "Display CPU temperature from sensors".to_string(),
            parameters: vec![],
            command_pattern: "sensors 2>/dev/null | grep -E '(Core|Package|Tctl|Tdie|CPU)' | head -20 || cat /sys/class/thermal/thermal_zone*/temp 2>/dev/null | head -10".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Lm_sensors".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "CPU temperature displayed".to_string(),
            }),
            example: "sensors | grep Core".to_string(),
        }
    }

    fn detect_cpu_throttling() -> Template {
        Template {
            id: "detect_cpu_throttling".to_string(),
            name: "Detect CPU Throttling".to_string(),
            description: "Check for thermal throttling events in system journal".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -b --no-pager | grep -iE '(throttl|thermal|overheat|temperature)' | tail -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/CPU_frequency_scaling".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Throttling events displayed".to_string(),
            }),
            example: "journalctl | grep throttl".to_string(),
        }
    }

    fn show_top_cpu_processes() -> Template {
        Template {
            id: "show_top_cpu_processes".to_string(),
            name: "Show Top CPU Processes".to_string(),
            description: "Display processes consuming the most CPU".to_string(),
            parameters: vec![],
            command_pattern: "ps aux --sort=-%cpu | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Core_utilities".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("CPU".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Top CPU processes displayed".to_string(),
            }),
            example: "ps aux --sort=-%cpu | head -10".to_string(),
        }
    }

    fn check_load_average() -> Template {
        Template {
            id: "check_load_average".to_string(),
            name: "Check Load Average".to_string(),
            description: "Show system load average and interpretation".to_string(),
            parameters: vec![],
            command_pattern: "uptime && echo && cat /proc/loadavg && echo && nproc".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Core_utilities".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("load average".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Load average displayed".to_string(),
            }),
            example: "uptime".to_string(),
        }
    }

    fn analyze_context_switches() -> Template {
        Template {
            id: "analyze_context_switches".to_string(),
            name: "Analyze Context Switches".to_string(),
            description: "Show context switch rate as performance indicator".to_string(),
            parameters: vec![],
            command_pattern: "vmstat 1 5 | tail -6".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Core_utilities".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("cs".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Context switches displayed".to_string(),
            }),
            example: "vmstat 1 5".to_string(),
        }
    }

    // ===== Beta.105: Memory & Swap Diagnostics (700-question test suite) =====

    fn check_memory_usage() -> Template {
        Template {
            id: "check_memory_usage".to_string(),
            name: "Check Memory Usage".to_string(),
            description:
                "Show current memory usage overview (total, used, free, available, cached)"
                    .to_string(),
            parameters: vec![],
            command_pattern: "free -h && echo && cat /proc/meminfo | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Memory".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Mem:".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Memory usage displayed".to_string(),
            }),
            example: "free -h".to_string(),
        }
    }

    fn check_swap_usage() -> Template {
        Template {
            id: "check_swap_usage".to_string(),
            name: "Check Swap Usage".to_string(),
            description: "Show swap usage and configuration".to_string(),
            parameters: vec![],
            command_pattern:
                "free -h | grep -i swap && echo && swapon --show && echo && cat /proc/swaps"
                    .to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Swap".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Swap".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Swap usage displayed".to_string(),
            }),
            example: "swapon --show".to_string(),
        }
    }

    fn analyze_memory_pressure() -> Template {
        Template {
            id: "analyze_memory_pressure".to_string(),
            name: "Analyze Memory Pressure".to_string(),
            description: "Detect memory pressure and OOM (Out-Of-Memory) events".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -p warning -g 'Out of memory|OOM|memory pressure' --since '1 hour ago' | head -30 || echo 'No memory pressure detected in last hour'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Memory".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("memory".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Memory pressure analysis displayed".to_string(),
            }),
            example: "journalctl -p warning -g OOM".to_string(),
        }
    }

    fn show_top_memory_processes() -> Template {
        Template {
            id: "show_top_memory_processes".to_string(),
            name: "Show Top Memory Processes".to_string(),
            description: "Show top memory-consuming processes sorted by memory usage".to_string(),
            parameters: vec![],
            command_pattern: "ps aux --sort=-%mem | head -15".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Core_utilities".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("%MEM".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Top memory processes displayed".to_string(),
            }),
            example: "ps aux --sort=-%mem | head".to_string(),
        }
    }

    fn check_oom_killer() -> Template {
        Template {
            id: "check_oom_killer".to_string(),
            name: "Check OOM Killer".to_string(),
            description: "Check for OOM (Out-Of-Memory) killer events from system journal".to_string(),
            parameters: vec![],
            command_pattern: "journalctl -k -g 'Out of memory|oom_reaper|Kill process' --since '7 days ago' | tail -50 || echo 'No OOM killer events in last 7 days'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Out_of_memory".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "OOM killer events displayed".to_string(),
            }),
            example: "journalctl -k -g 'Out of memory'".to_string(),
        }
    }

    fn analyze_swap_activity() -> Template {
        Template {
            id: "analyze_swap_activity".to_string(),
            name: "Analyze Swap Activity".to_string(),
            description: "Show swap in/out activity via vmstat".to_string(),
            parameters: vec![],
            command_pattern: "vmstat -s | grep -i swap && echo && vmstat 1 5 | tail -6".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Swap".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("swap".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Swap activity displayed".to_string(),
            }),
            example: "vmstat -s | grep swap".to_string(),
        }
    }

    fn check_huge_pages() -> Template {
        Template {
            id: "check_huge_pages".to_string(),
            name: "Check Huge Pages".to_string(),
            description: "Show huge pages configuration and usage".to_string(),
            parameters: vec![],
            command_pattern: "grep -i huge /proc/meminfo && echo && cat /sys/kernel/mm/transparent_hugepage/enabled 2>/dev/null || echo 'Transparent huge pages info not available'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Memory".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("Huge".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Huge pages info displayed".to_string(),
            }),
            example: "grep Huge /proc/meminfo".to_string(),
        }
    }

    fn show_memory_info() -> Template {
        Template {
            id: "show_memory_info".to_string(),
            name: "Show Memory Hardware Info".to_string(),
            description: "Show detailed memory hardware information from DMI/SMBIOS".to_string(),
            parameters: vec![],
            command_pattern: "sudo dmidecode -t memory 2>/dev/null | head -100 || lshw -short -C memory 2>/dev/null | head -20 || echo 'Memory hardware info requires root access (sudo dmidecode -t memory)'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Memory".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: Some("memory".to_string()),
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Memory hardware info displayed".to_string(),
            }),
            example: "sudo dmidecode -t memory".to_string(),
        }
    }

    // ============================================================================
    // Beta.106: GPU Diagnostics (700 questions - GPU issues)
    // ============================================================================

    fn check_gpu_info() -> Template {
        Template {
            id: "check_gpu_info".to_string(),
            name: "Check GPU Info".to_string(),
            description: "Show GPU hardware information (vendor, model, PCI ID)".to_string(),
            parameters: vec![],
            command_pattern: "lspci -nn | grep -i 'VGA\\|3D\\|Display' && echo && lspci -v -s $(lspci | grep -i 'VGA\\|3D\\|Display' | cut -d' ' -f1) 2>/dev/null | head -20 || echo 'No GPU detected'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/GPU".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "GPU info displayed".to_string(),
            }),
            example: "lspci | grep -i vga".to_string(),
        }
    }

    fn check_gpu_drivers() -> Template {
        Template {
            id: "check_gpu_drivers".to_string(),
            name: "Check GPU Drivers".to_string(),
            description: "Show loaded GPU driver modules (nvidia, amdgpu, i915, nouveau, radeon)".to_string(),
            parameters: vec![],
            command_pattern: "lsmod | grep -i 'nvidia\\|amdgpu\\|i915\\|nouveau\\|radeon' && echo && for mod in nvidia amdgpu i915 nouveau radeon; do if lsmod | grep -q \"^$mod \"; then echo \"=== $mod driver info ===\"  && modinfo $mod 2>/dev/null | grep -E '^(version|description|author|license|firmware):' | head -10; fi; done".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Kernel_module".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "GPU drivers displayed".to_string(),
            }),
            example: "lsmod | grep nvidia".to_string(),
        }
    }

    fn check_nvidia_status() -> Template {
        Template {
            id: "check_nvidia_status".to_string(),
            name: "Check NVIDIA GPU Status".to_string(),
            description: "Show NVIDIA GPU status (memory, utilization, temperature) via nvidia-smi".to_string(),
            parameters: vec![],
            command_pattern: "if command -v nvidia-smi >/dev/null 2>&1; then nvidia-smi && echo && nvidia-smi --query-gpu=gpu_name,driver_version,memory.total,memory.used,memory.free,temperature.gpu,utilization.gpu,power.draw --format=csv; else echo 'nvidia-smi not found. Install nvidia-utils package or not using NVIDIA GPU'; fi".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/NVIDIA".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "NVIDIA GPU status displayed".to_string(),
            }),
            example: "nvidia-smi".to_string(),
        }
    }

    fn check_amd_gpu() -> Template {
        Template {
            id: "check_amd_gpu".to_string(),
            name: "Check AMD GPU Status".to_string(),
            description: "Show AMD GPU status (sensors, DRM info from sysfs)".to_string(),
            parameters: vec![],
            command_pattern: "if lsmod | grep -q amdgpu; then echo '=== AMD GPU Detected ===' && cat /sys/class/drm/card*/device/pp_dpm_* 2>/dev/null | head -50 && echo && sensors 2>/dev/null | grep -A 5 'amdgpu\\|radeon' && echo && find /sys/class/drm/card* -name 'mem_info_*' -o -name 'gpu_busy_percent' 2>/dev/null | xargs -I {} sh -c 'echo {} && cat {} 2>/dev/null' | head -30; else echo 'AMD GPU not detected or amdgpu driver not loaded'; fi".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/AMDGPU".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "AMD GPU status displayed".to_string(),
            }),
            example: "cat /sys/class/drm/card0/device/gpu_busy_percent".to_string(),
        }
    }

    fn check_gpu_processes() -> Template {
        Template {
            id: "check_gpu_processes".to_string(),
            name: "Check GPU Processes".to_string(),
            description: "Show processes using GPU (via nvidia-smi or ps filtering graphics processes)".to_string(),
            parameters: vec![],
            command_pattern: "if command -v nvidia-smi >/dev/null 2>&1; then echo '=== NVIDIA GPU Processes ===' && nvidia-smi pmon -c 1 2>/dev/null || nvidia-smi --query-compute-apps=pid,process_name,used_memory --format=csv 2>/dev/null; fi && echo && echo '=== Graphics-Related Processes ===' && ps aux | grep -E 'Xorg|wayland|gnome-shell|kwin|plasmashell|chrome|firefox' | grep -v grep | head -20".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/GPU".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "GPU processes displayed".to_string(),
            }),
            example: "nvidia-smi pmon".to_string(),
        }
    }

    fn check_gpu_temperature() -> Template {
        Template {
            id: "check_gpu_temperature".to_string(),
            name: "Check GPU Temperature".to_string(),
            description: "Show GPU temperature from sensors and vendor-specific tools".to_string(),
            parameters: vec![],
            command_pattern: "if command -v sensors >/dev/null 2>&1; then echo '=== GPU Temperature (sensors) ===' && sensors 2>/dev/null | grep -i 'nvidia\\|amdgpu\\|radeon' -A 5; fi && if command -v nvidia-smi >/dev/null 2>&1; then echo && echo '=== NVIDIA GPU Temperature ===' && nvidia-smi --query-gpu=temperature.gpu,temperature.memory,fan.speed --format=csv,noheader,nounits 2>/dev/null; fi && echo && echo '=== GPU Thermal Zones ===' && find /sys/class/hwmon -name 'temp*_label' 2>/dev/null | while read label; do echo \"$(cat \"$label\" 2>/dev/null): $(cat \"${label%_*}_input\" 2>/dev/null | awk '{print $1/1000}')°C\"; done | grep -i 'gpu\\|vga'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Lm_sensors".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "GPU temperature displayed".to_string(),
            }),
            example: "sensors | grep -i gpu".to_string(),
        }
    }

    fn check_gpu_errors() -> Template {
        Template {
            id: "check_gpu_errors".to_string(),
            name: "Check GPU Errors".to_string(),
            description: "Check for GPU errors in dmesg and system journal".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Recent GPU Errors (dmesg) ===' && dmesg -T | grep -i 'nvidia\\|amdgpu\\|radeon\\|i915\\|nouveau\\|gpu' | grep -iE 'error|fail|warn|timeout|hang|reset' | tail -30 && echo && echo '=== GPU Errors (journal - last 7 days) ===' && journalctl -k -p err -g 'nvidia|amdgpu|radeon|i915|nouveau|gpu' --since '7 days ago' | tail -30 || echo 'No GPU errors found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/GPU".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "GPU errors displayed".to_string(),
            }),
            example: "dmesg | grep -i nvidia | grep -i error".to_string(),
        }
    }

    fn analyze_graphics_performance() -> Template {
        Template {
            id: "analyze_graphics_performance".to_string(),
            name: "Analyze Graphics Performance".to_string(),
            description: "Show graphics stack information (Wayland/X11, compositor, OpenGL/Vulkan)".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Display Server ===' && echo \"$XDG_SESSION_TYPE\" && ps aux | grep -E 'Xorg|wayland' | grep -v grep | head -5 && echo && echo '=== OpenGL Renderer ===' && glxinfo 2>/dev/null | grep -E 'OpenGL renderer|OpenGL version|Direct rendering' | head -5 || echo 'glxinfo not found (install mesa-utils)' && echo && echo '=== Vulkan ===' && vulkaninfo --summary 2>/dev/null | head -20 || echo 'vulkaninfo not found (install vulkan-tools)' && echo && echo '=== Compositor ===' && ps aux | grep -E 'picom|compton|kwin|gnome-shell|mutter' | grep -v grep | head -5".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Xorg".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Graphics stack info displayed".to_string(),
            }),
            example: "glxinfo | grep 'OpenGL renderer'".to_string(),
        }
    }

    // ============================================================================
    // Beta.107: Desktop Environment Diagnostics (700 questions - DE/desktop issues)
    // ============================================================================

    fn check_display_server() -> Template {
        Template {
            id: "check_display_server".to_string(),
            name: "Check Display Server".to_string(),
            description: "Detect display server type (Wayland or X11) and session information".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Display Server Type ===' && echo \"Session Type: $XDG_SESSION_TYPE\" && echo \"Display: $DISPLAY\" && echo \"Wayland Display: $WAYLAND_DISPLAY\" && echo && echo '=== Running Display Server ===' && ps aux | grep -E 'Xorg|X |wayland|weston|sway' | grep -v grep | head -10 && echo && echo '=== loginctl Session ===' && loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') 2>/dev/null | grep -E 'Type=|Class=|State=' || echo 'No loginctl session found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Wayland".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Display server info displayed".to_string(),
            }),
            example: "echo $XDG_SESSION_TYPE".to_string(),
        }
    }

    fn check_desktop_environment() -> Template {
        Template {
            id: "check_desktop_environment".to_string(),
            name: "Check Desktop Environment".to_string(),
            description: "Identify desktop environment or window manager (KDE, GNOME, Xfce, i3, etc.)".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Desktop Environment ===' && echo \"Desktop: $XDG_CURRENT_DESKTOP\" && echo \"Session Desktop: $XDG_SESSION_DESKTOP\" && echo \"Desktop Session: $DESKTOP_SESSION\" && echo && echo '=== Running DE/WM Processes ===' && ps aux | grep -E 'plasma|plasmashell|kwin|gnome-shell|gnome-session|xfce4-session|mate-session|cinnamon|lxqt-session|i3|sway|bspwm|awesome|openbox|fluxbox' | grep -v grep | awk '{print $11}' | sort -u && echo && echo '=== Desktop Files ===' && ls -la ~/.config/autostart/*.desktop 2>/dev/null | head -5 || echo 'No autostart files found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Desktop_environment".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Desktop environment info displayed".to_string(),
            }),
            example: "echo $XDG_CURRENT_DESKTOP".to_string(),
        }
    }

    fn check_display_manager() -> Template {
        Template {
            id: "check_display_manager".to_string(),
            name: "Check Display Manager".to_string(),
            description: "Identify display manager (SDDM, GDM, LightDM, etc.) and status".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Display Manager Processes ===' && ps aux | grep -E 'sddm|gdm|lightdm|kdm|xdm|lxdm|slim|entrance' | grep -v grep && echo && echo '=== Display Manager Services ===' && systemctl list-units --type=service --state=running | grep -E 'sddm|gdm|lightdm|kdm|xdm|lxdm' && echo && echo '=== Display Manager Configs ===' && ls -la /etc/sddm.conf /etc/gdm/custom.conf /etc/lightdm/lightdm.conf 2>/dev/null | head -10 || echo 'No display manager configs found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Display_manager".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Display manager info displayed".to_string(),
            }),
            example: "ps aux | grep sddm".to_string(),
        }
    }

    fn analyze_xorg_errors() -> Template {
        Template {
            id: "analyze_xorg_errors".to_string(),
            name: "Analyze Xorg Errors".to_string(),
            description: "Check for X11/Xorg errors and crashes from logs".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Recent Xorg Errors (Xorg log) ===' && if [ -f ~/.local/share/xorg/Xorg.0.log ]; then grep -iE 'error|fail|fatal|warning' ~/.local/share/xorg/Xorg.0.log | tail -30; elif [ -f /var/log/Xorg.0.log ]; then grep -iE 'error|fail|fatal|warning' /var/log/Xorg.0.log | tail -30; else echo 'No Xorg log found'; fi && echo && echo '=== Xorg Crashes (journal - last 7 days) ===' && journalctl -b --since '7 days ago' | grep -iE 'xorg.*error|xorg.*crash|xorg.*segfault|x server.*error' | tail -20 || echo 'No Xorg crashes found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Xorg".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Xorg errors displayed".to_string(),
            }),
            example: "grep -i error ~/.local/share/xorg/Xorg.0.log".to_string(),
        }
    }

    fn check_wayland_compositor() -> Template {
        Template {
            id: "check_wayland_compositor".to_string(),
            name: "Check Wayland Compositor".to_string(),
            description: "Show Wayland compositor status and configuration".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Wayland Compositor ===' && if [ -n \"$WAYLAND_DISPLAY\" ]; then echo \"Wayland Display: $WAYLAND_DISPLAY\"; ps aux | grep -E 'weston|sway|kwin_wayland|gnome-shell.*wayland|mutter.*wayland' | grep -v grep | head -5; else echo 'Not running Wayland'; fi && echo && echo '=== Wayland Errors (journal - last 7 days) ===' && journalctl -b --since '7 days ago' | grep -iE 'wayland.*error|wayland.*crash|compositor.*error' | tail -20 || echo 'No Wayland errors found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Wayland".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Wayland compositor status displayed".to_string(),
            }),
            example: "echo $WAYLAND_DISPLAY".to_string(),
        }
    }

    fn check_desktop_session() -> Template {
        Template {
            id: "check_desktop_session".to_string(),
            name: "Check Desktop Session".to_string(),
            description: "Show current desktop session information and environment variables".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Desktop Session Variables ===' && env | grep -E 'XDG_|DESKTOP|DISPLAY|WAYLAND' | sort && echo && echo '=== Session Processes ===' && ps aux | grep -E 'session|dbus-daemon' | grep $(whoami) | grep -v grep | head -10 && echo && echo '=== Active Sessions ===' && loginctl list-sessions && echo && echo '=== Current Session Details ===' && loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') 2>/dev/null | head -20 || echo 'No session details available'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Desktop_environment".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Desktop session info displayed".to_string(),
            }),
            example: "env | grep XDG".to_string(),
        }
    }

    fn analyze_desktop_performance() -> Template {
        Template {
            id: "analyze_desktop_performance".to_string(),
            name: "Analyze Desktop Performance".to_string(),
            description: "Check desktop performance issues (compositing, vsync, tearing)".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Compositor Status ===' && ps aux | grep -E 'picom|compton|xcompmgr|kwin|mutter|muffin' | grep -v grep | head -5 && echo && echo '=== Compositor Config ===' && ls -la ~/.config/picom.conf ~/.config/compton.conf 2>/dev/null | head -5 || echo 'No compositor config found' && echo && echo '=== Desktop CPU Usage ===' && ps aux | grep -E 'kwin|plasmashell|gnome-shell|xfwm4|mutter' | grep -v grep | awk '{print $3, $11}' | sort -rn | head -5 && echo && echo '=== Vsync/Tearing Info ===' && glxinfo 2>/dev/null | grep -i 'sync\\|tear\\|vblank' | head -5 || echo 'glxinfo not available'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Picom".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Desktop performance info displayed".to_string(),
            }),
            example: "ps aux | grep compositor".to_string(),
        }
    }

    fn check_window_manager() -> Template {
        Template {
            id: "check_window_manager".to_string(),
            name: "Check Window Manager".to_string(),
            description: "Identify window manager and show configuration files".to_string(),
            parameters: vec![],
            command_pattern: "echo '=== Running Window Manager ===' && ps aux | grep -E 'i3|sway|bspwm|awesome|openbox|fluxbox|dwm|xmonad|herbstluftwm|kwin|mutter|xfwm4|muffin|marco' | grep -v grep | head -10 && echo && echo '=== Window Manager Configs ===' && ls -la ~/.config/i3/config ~/.config/sway/config ~/.config/bspwm/bspwmrc ~/.config/awesome/rc.lua ~/.config/openbox/rc.xml ~/.xmonad/xmonad.hs 2>/dev/null | head -10 || echo 'No WM configs found' && echo && echo '=== WM Errors (journal - last 24h) ===' && journalctl --user -b --since '24 hours ago' | grep -iE 'window manager|wm.*error|i3.*error|sway.*error' | tail -10 || echo 'No WM errors found'".to_string(),
            category: CommandCategory::ReadOnly,
            wiki_source: "https://wiki.archlinux.org/title/Window_manager".to_string(),
            validation_pattern: Some(OutputValidation {
                exit_code: 0,
                stdout_must_match: None,
                stdout_must_not_match: None,
                stderr_must_match: None,
                validation_description: "Window manager info displayed".to_string(),
            }),
            example: "ps aux | grep i3".to_string(),
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

    // v6.37.0: Tests for reliable update detection
    #[test]
    fn test_update_check_uses_yay_first() {
        let library = TemplateLibrary::new();
        let template = library.get("check_pending_updates").unwrap();

        // Should use yay -Qu as first option
        assert!(template.command_pattern.starts_with("yay -Qu"));
    }

    #[test]
    fn test_update_check_has_pacman_fallback() {
        let library = TemplateLibrary::new();
        let template = library.get("check_pending_updates").unwrap();

        // Should have pacman -Qu fallback
        assert!(template.command_pattern.contains("pacman -Qu"));
    }

    #[test]
    fn test_update_check_has_checkupdates_fallback() {
        let library = TemplateLibrary::new();
        let template = library.get("check_pending_updates").unwrap();

        // Should have checkupdates as final fallback
        assert!(template.command_pattern.contains("checkupdates"));
    }

    #[test]
    fn test_update_check_handles_failure_gracefully() {
        let library = TemplateLibrary::new();
        let template = library.get("check_pending_updates").unwrap();

        // Should have honest failure message, not "no updates"
        assert!(template.command_pattern.contains("Unable to check for updates"));
    }
}
