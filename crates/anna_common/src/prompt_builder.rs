//! LLM Prompt Builder for Safe Change Recipes
//!
//! Phase 9: Constructs structured, deterministic prompts for LLM recipe generation
//!
//! The LLM never gets free reign. We provide:
//! - Machine state (OS, packages, services, hardware)
//! - Ethical constraints and forbidden zones
//! - Explicit output schema requirement
//! - Safety rules and rollback requirements
//!
//! The LLM must respond with valid JSON matching ChangeRecipe schema.

use crate::change_recipe::*;
use crate::telemetry::SystemTelemetry;
#[cfg(test)]
use serde_json::json;

/// Safety context for LLM prompting
#[derive(Debug, Clone)]
pub struct SafetyContext {
    /// User who initiated the request
    pub user: String,

    /// Whether to allow system-wide changes
    pub allow_system_changes: bool,

    /// Whether to allow package installations
    pub allow_package_operations: bool,

    /// Maximum risk level allowed
    pub max_risk: ChangeRisk,

    /// Forbidden paths (absolute no-go zones)
    pub forbidden_paths: Vec<String>,
}

impl SafetyContext {
    /// Create a conservative safety context (user-space only)
    pub fn conservative(user: String) -> Self {
        Self {
            user,
            allow_system_changes: false,
            allow_package_operations: false,
            max_risk: ChangeRisk::Medium,
            forbidden_paths: Self::default_forbidden_paths(),
        }
    }

    /// Create a permissive safety context (allows system changes with approval)
    pub fn permissive(user: String) -> Self {
        Self {
            user,
            allow_system_changes: true,
            allow_package_operations: true,
            max_risk: ChangeRisk::High,
            forbidden_paths: Self::default_forbidden_paths(),
        }
    }

    /// Get default forbidden paths (always forbidden regardless of context)
    pub fn default_forbidden_paths() -> Vec<String> {
        vec![
            "/boot".to_string(),
            "/boot/grub".to_string(),
            "/etc/fstab".to_string(),
            "/etc/crypttab".to_string(),
            "/etc/mkinitcpio.conf".to_string(),
            "/etc/default/grub".to_string(),
            "/sys".to_string(),
            "/proc".to_string(),
            "/dev".to_string(),
        ]
    }
}

/// Build LLM prompt for recipe generation
pub struct PromptBuilder {
    safety_context: SafetyContext,
}

impl PromptBuilder {
    /// Create a new prompt builder with safety context
    pub fn new(safety_context: SafetyContext) -> Self {
        Self { safety_context }
    }

    /// Build complete prompt for LLM recipe generation
    ///
    /// Returns a structured prompt that includes:
    /// - System context (telemetry)
    /// - User request
    /// - Safety constraints
    /// - Output schema requirements
    pub fn build_prompt(&self, user_request: &str, telemetry: &SystemTelemetry) -> String {
        let system_prompt = self.build_system_prompt();
        let context = self.build_context_section(telemetry);
        let safety_rules = self.build_safety_rules_section();
        let schema = self.build_output_schema();
        let user_section = format!("# User Request\n\n{}\n", user_request);

        format!(
            "{}\n\n{}\n\n{}\n\n{}\n\n{}",
            system_prompt, context, safety_rules, schema, user_section
        )
    }

    /// Build system prompt (role and constraints)
    fn build_system_prompt(&self) -> String {
        format!(
            "# System Role\n\n\
             You are Anna, a safe system assistant for Arch Linux.\n\
             You ONLY generate change recipes in JSON format.\n\
             You NEVER execute commands directly.\n\
             You NEVER invent system state - only use the provided telemetry.\n\
             You MUST follow all safety constraints.\n\
             \n\
             Current user: {}\n\
             Max allowed risk: {:?}",
            self.safety_context.user, self.safety_context.max_risk
        )
    }

    /// Build context section from telemetry
    fn build_context_section(&self, telemetry: &SystemTelemetry) -> String {
        let mut context = String::from("# System Context\n\n");

        // OS and hardware
        context.push_str("## Platform\n");
        context.push_str("- OS: Arch Linux\n");
        context.push_str(&format!(
            "- CPU: {} ({} cores)\n",
            telemetry.hardware.cpu_model, telemetry.cpu.cores
        ));
        context.push_str(&format!(
            "- RAM: {} MB total, {} MB available\n",
            telemetry.memory.total_mb, telemetry.memory.available_mb
        ));
        context.push_str(&format!(
            "- Machine Type: {}\n",
            match telemetry.hardware.machine_type {
                crate::telemetry::MachineType::Laptop => "Laptop",
                crate::telemetry::MachineType::Desktop => "Desktop",
                crate::telemetry::MachineType::Server => "Server",
            }
        ));
        context.push('\n');

        // Packages
        context.push_str("## Package Status\n");
        context.push_str(&format!(
            "- Total installed: {}\n",
            telemetry.packages.total_installed
        ));
        if telemetry.packages.updates_available > 0 {
            context.push_str(&format!(
                "- Updates available: {}\n",
                telemetry.packages.updates_available
            ));
        }
        if telemetry.packages.orphaned > 0 {
            context.push_str(&format!(
                "- Orphaned packages: {}\n",
                telemetry.packages.orphaned
            ));
        }
        context.push('\n');

        // Desktop environment
        if let Some(ref desktop) = telemetry.desktop {
            context.push_str("## Desktop Environment\n");
            if let Some(ref de) = desktop.de_name {
                context.push_str(&format!("- DE: {}\n", de));
            }
            if let Some(ref wm) = desktop.wm_name {
                context.push_str(&format!("- WM: {}\n", wm));
            }
            if let Some(ref display) = desktop.display_server {
                context.push_str(&format!("- Display: {}\n", display));
            }
            context.push('\n');
        }

        // Services
        context.push_str("## System Services\n");
        context.push_str(&format!(
            "- Total units: {}\n",
            telemetry.services.total_units
        ));
        if !telemetry.services.failed_units.is_empty() {
            context.push_str(&format!(
                "- Failed units: {}\n",
                telemetry.services.failed_units.len()
            ));
        }
        context.push('\n');

        context
    }

    /// Build safety rules section
    fn build_safety_rules_section(&self) -> String {
        let mut rules = String::from("# Safety Constraints\n\n");

        rules.push_str("## FORBIDDEN (Never Allowed)\n\n");
        rules.push_str("You MUST REJECT requests that involve:\n");
        rules.push_str("- Bootloader modifications (/boot/grub, grub.cfg)\n");
        rules.push_str("- Filesystem tables (/etc/fstab, /etc/crypttab)\n");
        rules.push_str("- Initramfs changes (/etc/mkinitcpio.conf)\n");
        rules.push_str("- Kernel parameters\n");
        rules.push_str("- Partition operations (fdisk, parted, mkfs)\n");
        rules.push_str("- Firewall rewrites (iptables flush, nftables delete)\n");
        rules.push_str("- Mass delete operations (rm -rf /*, find -delete)\n");
        rules.push_str("- Network stack rewrites\n");
        rules.push_str("- System-critical package removal (systemd, kernel, pacman)\n");
        rules.push('\n');

        rules.push_str("## Forbidden Paths\n\n");
        for path in &self.safety_context.forbidden_paths {
            rules.push_str(&format!("- {}\n", path));
        }
        rules.push('\n');

        rules.push_str("## Risk Levels\n\n");
        rules.push_str("- **Low**: Cosmetic changes (wallpaper, terminal colors)\n");
        rules.push_str("  - No sudo required\n");
        rules.push_str("  - Worst case: User doesn't like the look\n");
        rules.push('\n');
        rules.push_str("- **Medium**: User config changes (vim, zsh, git)\n");
        rules.push_str("  - No sudo required (user files only)\n");
        rules.push_str("  - Worst case: Application fails to start until config fixed\n");
        rules.push('\n');
        rules.push_str("- **High**: System config or packages\n");
        rules.push_str("  - Sudo required\n");
        rules.push_str("  - Worst case: System services fail, manual recovery needed\n");
        rules.push('\n');

        if !self.safety_context.allow_system_changes {
            rules.push_str("## Restrictions for Current Context\n\n");
            rules.push_str("- System-wide changes: DISABLED\n");
            rules.push_str("- Only user-space modifications allowed\n");
            rules.push('\n');
        }

        if !self.safety_context.allow_package_operations {
            rules.push_str("- Package operations: DISABLED\n");
            rules.push_str("- No pacman/yay commands\n");
            rules.push('\n');
        }

        rules.push_str("## Required Safety Measures\n\n");
        rules.push_str("- Every action must have clear rollback notes\n");
        rules.push_str("- Sudo requirements must be explicit per action\n");
        rules.push_str("- File modifications require backup strategy\n");
        rules.push_str("- Destructive edits (ReplaceEntire) need strong justification\n");
        rules.push('\n');

        rules
    }

    /// Build output schema section
    fn build_output_schema(&self) -> String {
        let schema = r#"# Output Schema

You MUST respond with ONLY valid JSON matching this schema:

```json
{
  "title": "Short title (max 60 chars)",
  "summary": "One sentence description",
  "why_it_matters": "Why this change is useful",
  "rollback_notes": "How to undo these changes",
  "actions": [
    {
      "kind": "EditFile | AppendToFile | InstallPackages | RemovePackages | EnableService | DisableService | SetWallpaper",
      "description": "Human-readable description",
      "estimated_impact": "What this action does",
      "details": { /* Kind-specific fields */ }
    }
  ]
}
```

## Action Kind Examples

### EditFile
```json
{
  "kind": "EditFile",
  "description": "Enable syntax highlighting in vim",
  "estimated_impact": "Makes code more readable",
  "details": {
    "path": "/home/user/.vimrc",
    "strategy": {
      "type": "AppendIfMissing",
      "lines": ["syntax on", "set number"]
    }
  }
}
```

### InstallPackages
```json
{
  "kind": "InstallPackages",
  "description": "Install vim plugins",
  "estimated_impact": "Adds color scheme support",
  "details": {
    "packages": ["vim-runtime"]
  }
}
```

### SetWallpaper
```json
{
  "kind": "SetWallpaper",
  "description": "Set desktop wallpaper",
  "estimated_impact": "Changes background image",
  "details": {
    "image_path": "/home/user/Pictures/wallpaper.jpg"
  }
}
```

## Strategy Types for EditFile

1. **AppendIfMissing**: Add lines if not present
2. **ReplaceSection**: Replace content between markers
3. **ReplaceEntire**: Replace entire file (use sparingly!)

IMPORTANT:
- If the request is forbidden or too dangerous, respond with:
  ```json
  {
    "error": "FORBIDDEN",
    "reason": "Explanation of why this is not allowed"
  }
  ```
- If you need more information, respond with:
  ```json
  {
    "error": "INSUFFICIENT_INFO",
    "reason": "What information is needed"
  }
  ```
"#;
        schema.to_string()
    }

    /// Build a prompt for a specific use case (e.g., wallpaper)
    pub fn build_focused_prompt(&self, use_case: &str, params: &serde_json::Value) -> String {
        match use_case {
            "wallpaper" => self.build_wallpaper_prompt(params),
            "vim_config" => self.build_vim_config_prompt(params),
            "service_enable" => self.build_service_prompt(params),
            _ => format!("Unknown use case: {}", use_case),
        }
    }

    fn build_wallpaper_prompt(&self, params: &serde_json::Value) -> String {
        format!(
            "Generate a ChangeRecipe to set wallpaper.\n\
             Image path: {}\n\
             Desktop environment: {}\n\
             \n\
             Use SetWallpaper action kind.\n\
             Risk level: Low (cosmetic only)\n\
             No sudo required.",
            params["image_path"].as_str().unwrap_or(""),
            params["de"].as_str().unwrap_or("unknown")
        )
    }

    fn build_vim_config_prompt(&self, params: &serde_json::Value) -> String {
        format!(
            "Generate a ChangeRecipe to configure vim.\n\
             User request: {}\n\
             \n\
             Use EditFile action kind for ~/.vimrc\n\
             Use AppendIfMissing strategy to avoid duplicates\n\
             Risk level: Medium (user config)\n\
             No sudo required.",
            params["request"].as_str().unwrap_or("")
        )
    }

    fn build_service_prompt(&self, params: &serde_json::Value) -> String {
        format!(
            "Generate a ChangeRecipe to enable a systemd service.\n\
             Service: {}\n\
             User service: {}\n\
             \n\
             Use EnableService action kind\n\
             Risk level: High if system service, Low if user service\n\
             Sudo required for system services.",
            params["service"].as_str().unwrap_or(""),
            params["user_service"].as_bool().unwrap_or(false)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_telemetry() -> SystemTelemetry {
        use crate::telemetry::*;

        SystemTelemetry {
            timestamp: chrono::Utc::now(),
            hardware: HardwareInfo {
                cpu_model: "Intel Core i7-8550U".to_string(),
                total_ram_mb: 16384,
                machine_type: MachineType::Laptop,
                has_battery: true,
                has_gpu: true,
                gpu_info: Some("Intel UHD Graphics 620".to_string()),
            },
            disks: vec![DiskInfo {
                mount_point: "/".to_string(),
                total_mb: 512000,
                used_mb: 256000,
                usage_percent: 50.0,
                fs_type: "ext4".to_string(),
                smart_status: Some(SmartStatus::Healthy),
            }],
            memory: MemoryInfo {
                total_mb: 16384,
                available_mb: 8192,
                used_mb: 8192,
                swap_total_mb: 8192,
                swap_used_mb: 0,
                usage_percent: 50.0,
            },
            cpu: CpuInfo {
                cores: 8,
                load_avg_1min: 1.2,
                load_avg_5min: 1.5,
                usage_percent: Some(25.0),
            },
            packages: PackageInfo {
                total_installed: 1200,
                updates_available: 5,
                orphaned: 3,
                cache_size_mb: 1024.0,
                last_update: Some(chrono::Utc::now()),
            },
            services: ServiceInfo {
                total_units: 250,
                failed_units: vec![],
                recently_restarted: vec![],
            },
            network: NetworkInfo {
                is_connected: true,
                primary_interface: Some("wlan0".to_string()),
                firewall_active: true,
                firewall_type: Some("ufw".to_string()),
            },
            security: SecurityInfo {
                failed_ssh_attempts: 0,
                auto_updates_enabled: false,
                audit_warnings: vec![],
            },
            desktop: Some(DesktopInfo {
                de_name: Some("Hyprland".to_string()),
                wm_name: Some("Hyprland".to_string()),
                display_server: Some("Wayland".to_string()),
                monitor_count: 2,
            }),
            boot: Some(BootInfo {
                last_boot_time_secs: 15.5,
                avg_boot_time_secs: Some(16.0),
                trend: Some(BootTrend::Stable),
            }),
            audio: AudioTelemetry {
                has_sound_hardware: true,
                pipewire_running: true,
                wireplumber_running: true,
                pipewire_pulse_running: true,
            },
        }
    }

    #[test]
    fn test_conservative_safety_context() {
        let ctx = SafetyContext::conservative("alice".to_string());
        assert_eq!(ctx.user, "alice");
        assert!(!ctx.allow_system_changes);
        assert!(!ctx.allow_package_operations);
        assert_eq!(ctx.max_risk, ChangeRisk::Medium);
        assert!(!ctx.forbidden_paths.is_empty());
    }

    #[test]
    fn test_permissive_safety_context() {
        let ctx = SafetyContext::permissive("bob".to_string());
        assert_eq!(ctx.user, "bob");
        assert!(ctx.allow_system_changes);
        assert!(ctx.allow_package_operations);
        assert_eq!(ctx.max_risk, ChangeRisk::High);
    }

    #[test]
    fn test_forbidden_paths_include_boot() {
        let paths = SafetyContext::default_forbidden_paths();
        assert!(paths.contains(&"/boot".to_string()));
        assert!(paths.contains(&"/boot/grub".to_string()));
        assert!(paths.contains(&"/etc/fstab".to_string()));
        assert!(paths.contains(&"/etc/crypttab".to_string()));
    }

    #[test]
    fn test_build_system_prompt_includes_user() {
        let ctx = SafetyContext::conservative("testuser".to_string());
        let builder = PromptBuilder::new(ctx);
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("testuser"));
        assert!(prompt.contains("Anna"));
        assert!(prompt.contains("safe system assistant"));
    }

    #[test]
    fn test_build_system_prompt_includes_max_risk() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let prompt = builder.build_system_prompt();

        assert!(prompt.contains("Medium")); // max_risk is Medium for conservative
    }

    #[test]
    fn test_build_context_includes_telemetry() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let telemetry = mock_telemetry();
        let context = builder.build_context_section(&telemetry);

        assert!(context.contains("Arch Linux"));
        assert!(context.contains("Intel Core i7-8550U"));
        assert!(context.contains("8 cores"));
        assert!(context.contains("16384 MB"));
    }

    #[test]
    fn test_build_context_includes_packages() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let telemetry = mock_telemetry();
        let context = builder.build_context_section(&telemetry);

        assert!(context.contains("1200")); // total_installed
    }

    #[test]
    fn test_build_context_includes_desktop_environment() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let telemetry = mock_telemetry();
        let context = builder.build_context_section(&telemetry);

        assert!(context.contains("Hyprland"));
    }

    #[test]
    fn test_safety_rules_include_forbidden_operations() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let rules = builder.build_safety_rules_section();

        assert!(rules.contains("Bootloader"));
        assert!(rules.contains("fstab"));
        assert!(rules.contains("Partition operations"));
        assert!(rules.contains("Mass delete"));
    }

    #[test]
    fn test_safety_rules_include_forbidden_paths() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let rules = builder.build_safety_rules_section();

        assert!(rules.contains("/boot"));
        assert!(rules.contains("/etc/fstab"));
        assert!(rules.contains("/etc/crypttab"));
    }

    #[test]
    fn test_safety_rules_include_risk_levels() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let rules = builder.build_safety_rules_section();

        assert!(rules.contains("Low"));
        assert!(rules.contains("Medium"));
        assert!(rules.contains("High"));
        assert!(rules.contains("wallpaper"));
        assert!(rules.contains("vim"));
    }

    #[test]
    fn test_conservative_context_restricts_system_changes() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let rules = builder.build_safety_rules_section();

        assert!(rules.contains("System-wide changes: DISABLED"));
        assert!(rules.contains("user-space"));
    }

    #[test]
    fn test_conservative_context_restricts_packages() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let rules = builder.build_safety_rules_section();

        assert!(rules.contains("Package operations: DISABLED"));
    }

    #[test]
    fn test_output_schema_includes_action_kinds() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let schema = builder.build_output_schema();

        assert!(schema.contains("EditFile"));
        assert!(schema.contains("InstallPackages"));
        assert!(schema.contains("SetWallpaper"));
        assert!(schema.contains("AppendIfMissing"));
    }

    #[test]
    fn test_output_schema_includes_error_handling() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let schema = builder.build_output_schema();

        assert!(schema.contains("FORBIDDEN"));
        assert!(schema.contains("INSUFFICIENT_INFO"));
    }

    #[test]
    fn test_full_prompt_construction() {
        let ctx = SafetyContext::conservative("alice".to_string());
        let builder = PromptBuilder::new(ctx);
        let telemetry = mock_telemetry();
        let prompt = builder.build_prompt("enable syntax highlighting in vim", &telemetry);

        assert!(prompt.contains("Anna"));
        assert!(prompt.contains("alice"));
        assert!(prompt.contains("Arch Linux"));
        assert!(prompt.contains("Hyprland"));
        assert!(prompt.contains("FORBIDDEN"));
        assert!(prompt.contains("enable syntax highlighting in vim"));
    }

    #[test]
    fn test_wallpaper_focused_prompt() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let params = json!({
            "image_path": "/home/user/pic.jpg",
            "de": "Hyprland"
        });
        let prompt = builder.build_focused_prompt("wallpaper", &params);

        assert!(prompt.contains("SetWallpaper"));
        assert!(prompt.contains("/home/user/pic.jpg"));
        assert!(prompt.contains("Hyprland"));
        assert!(prompt.contains("Low"));
    }

    #[test]
    fn test_vim_config_focused_prompt() {
        let ctx = SafetyContext::conservative("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let params = json!({
            "request": "enable syntax highlighting"
        });
        let prompt = builder.build_focused_prompt("vim_config", &params);

        assert!(prompt.contains("EditFile"));
        assert!(prompt.contains(".vimrc"));
        assert!(prompt.contains("AppendIfMissing"));
        assert!(prompt.contains("Medium"));
    }

    #[test]
    fn test_service_focused_prompt() {
        let ctx = SafetyContext::permissive("user".to_string());
        let builder = PromptBuilder::new(ctx);
        let params = json!({
            "service": "nginx",
            "user_service": false
        });
        let prompt = builder.build_focused_prompt("service_enable", &params);

        assert!(prompt.contains("EnableService"));
        assert!(prompt.contains("nginx"));
        assert!(prompt.contains("High"));
        assert!(prompt.contains("Sudo required"));
    }
}
