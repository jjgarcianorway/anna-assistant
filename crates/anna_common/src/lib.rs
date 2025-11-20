//! Anna Common - Shared types and utilities
//!
//! This crate contains data models and utilities shared between
//! the daemon (annad) and CLI client (annactl).

pub mod action_plan; // Beta.66: Secure ACTION_PLAN execution with validation
pub mod action_plan_v3; // Beta.143: JSON runtime contract for LLM
pub mod advice_cache;
pub mod answer_validator; // Beta.87: Multi-pass answer validation - zero hallucination guarantee
pub mod audio; // Audio system detection (PipeWire, Pulse, ALSA)
pub mod backup_detection; // Backup tool detection (timeshift, snapper, borg, restic)
pub mod beautiful;
pub mod boot; // Boot system detection (UEFI/BIOS, Secure Boot, bootloader)
pub mod caretaker_brain; // Core analysis engine - ties everything together
pub mod categories;
pub mod change_log; // Phase 5.1: Change logging and rollback
pub mod change_log_db; // Phase 5.1: SQLite persistence for change logs
pub mod change_recipe; // Phase 7: Safe change recipes with strict guardrails
pub mod change_recipe_display; // Phase 7: UI display for change recipes
pub mod command_meta;
pub mod command_recipe; // Beta.90: Command recipe system - executable knowledge
pub mod config;
pub mod config_file; // Desktop config file parsing (Hyprland, i3, Sway)
pub mod config_parser;
pub mod container_virt_perf; // Container and virtualization performance (broken containers, resource limits, nested virt)
pub mod context;
pub mod cpu_performance; // CPU performance detection (governors, microcode, flags)
pub mod cpu_throttling; // CPU throttling and power state detection (throttle events, C-states)
pub mod desktop; // Desktop environment detection (Hyprland, i3, KDE, etc.)
pub mod desktop_automation; // Desktop automation helpers (wallpaper, config updates, reload)
pub mod disk_analysis;
pub mod display;
pub mod display_issues; // Display driver and multi-monitor issue detection
pub mod file_backup; // File backup system with SHA256 verification
pub mod file_index; // Beta.84: File-level indexing - track every file on the system
pub mod filesystem; // Filesystem features detection (TRIM, LUKS, Btrfs)
pub mod filesystem_health; // Filesystem health detection (Ext4, XFS, ZFS fsck/scrub status)
pub mod github_releases;
pub mod gpu_compute; // GPU compute capabilities (CUDA, OpenCL, ROCm, oneAPI)
pub mod gpu_throttling; // GPU throttling detection (NVIDIA, AMD, Intel thermal/power limits)
pub mod graphics; // Graphics and display detection (Vulkan, OpenGL, session type)
pub mod hardware_capability; // Hardware capability detection for local LLM
pub mod historian; // Historian - Long-term memory and trend analysis system
pub mod ignore_filters;
pub mod initramfs; // Initramfs configuration detection (mkinitcpio/dracut, hooks, modules, compression)
pub mod insights; // Phase 5.2: Behavioral insights engine
pub mod installation_source;
pub mod ipc;
pub mod kernel_modules; // Kernel and boot detection (installed kernels, modules, boot config)
pub mod language; // Language system with natural configuration
pub mod learning;
pub mod llm; // Task 12: LLM abstraction layer
pub mod llm_benchmark; // Beta.68: LLM benchmarking harness
pub mod llm_context; // LLM Contextualization - Synthesizes all detection data for caretaker brain
pub mod llm_upgrade; // Step 3: Hardware upgrade detection for brain improvements
pub mod memory_usage; // Memory usage detection (RAM, swap, OOM events)
pub mod model_profiles; // Data-driven model selection with upgrade paths
pub mod network_config; // Network configuration detection (DNS, NetworkManager, Wi-Fi)
pub mod network_monitoring; // Network monitoring (interfaces, latency, packet loss, routes, firewall)
pub mod ollama_installer; // Automatic local LLM bootstrap
pub mod orphaned_packages; // Orphaned package detection (pacman -Qtd, size tracking)
pub mod package_health; // Package health detection (unowned files, conflicts, database corruption)
pub mod package_mgmt; // Package management configuration (pacman.conf, mirrorlist, AUR)
pub mod paths;
pub mod personality; // Phase 5.1: Conversational personality controls
pub mod power; // Power and battery detection (health, cycles, AC status, TLP)
pub mod prediction;
pub mod prediction_actions;
pub mod profile;
pub mod prompt_builder; // Phase 9: LLM prompt construction with safety
pub mod qa_scenarios; // Beta.67: Real-world QA scenarios (vim, hardware, LLM upgrade)
pub mod recipe_executor; // Beta.90: Safe command execution with validation
pub mod recipe_planner; // Beta.90: Planner/Critic LLM loop
pub mod recipe_validator; // Phase 9: LLM response parsing and validation
pub mod reddit_qa_validator; // Beta.78: Reddit-based validation (500-1000 real questions)
pub mod rollback;
pub mod security; // Security configuration (firewall, SSH config, umask)
pub mod security_features; // Security features (SELinux, AppArmor, Polkit, sudo, kernel lockdown)
pub mod self_healing;
pub mod sensors; // Hardware sensors detection (CPU/GPU temps, fan speeds)
pub mod storage; // Storage detection (SSD/HDD, SMART status, health, I/O errors, alignment)
pub mod suggestion_engine; // Task 8: Deep Caretaker v0.1 - Rule-based suggestion generation
pub mod suggestions; // Phase 5.1: Suggestion engine with Arch Wiki integration
pub mod system_health; // System health detection (load averages, daemon crashes, uptime)
pub mod systemd_health; // Systemd health detection (failed units, timers, journal)
pub mod telemetry; // Telemetry structures from annad
pub mod template_library; // Beta.90: Pre-built templates for common operations
pub mod terminal_format; // Phase 8: Beautiful terminal formatting
pub mod trend_detectors; // Milestone 1.3: Trend-based detection and proactive warnings
pub mod types;
pub mod updater;
pub mod user_behavior; // User behavior pattern detection (commands, resources, development, gaming, security)
pub mod virtualization;
pub mod voltage_monitoring; // Voltage monitoring and anomaly detection // Virtualization and containerization (KVM, Docker, libvirt)

pub use advice_cache::*;
pub use beautiful::*;
pub use categories::*;
pub use config::*;
pub use config_parser::*;
pub use file_index::*; // Beta.84: File-level indexing
pub use ignore_filters::*;
pub use ipc::*;
pub use paths::*;
pub use profile::*;
pub use rollback::*;
pub use types::*;
pub use updater::*;
