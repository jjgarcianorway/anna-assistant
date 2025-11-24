//! ACTS v4 - Wiki Answer Engine Tests (6.16.0)
//!
//! Tests the deterministic, multi-step reasoning pipeline:
//! 1. Query Understanding - classification into 5 categories
//! 2. Wiki Retrieval - mapping to Arch Wiki topics
//! 3. System Tailoring - adaptation to actual system state
//! 4. Answer Assembly - formatting with citations
//!
//! 40 tests covering top sysadmin questions.

use anna_common::wiki_answer_engine::*;
use anna_common::system_knowledge::{SystemKnowledgeBase, HardwareProfile};
use anna_common::telemetry::*;

// ============================================================================
// Test Fixtures
// ============================================================================

fn create_test_telemetry() -> SystemTelemetry {
    SystemTelemetry {
        timestamp: chrono::Utc::now(),
        hardware: HardwareInfo {
            cpu_model: "Intel Core i7-11800H".to_string(),
            total_ram_mb: 32768,
            machine_type: MachineType::Laptop,
            has_battery: true,
            has_gpu: true,
            gpu_info: Some("NVIDIA GeForce RTX 3060".to_string()),
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
            total_mb: 32768,
            available_mb: 16384,
            used_mb: 16384,
            swap_total_mb: 8192,
            swap_used_mb: 0,
            usage_percent: 50.0,
        },
        cpu: CpuInfo {
            cores: 16,
            load_avg_1min: 1.5,
            load_avg_5min: 1.8,
            usage_percent: Some(30.0),
        },
        packages: PackageInfo {
            total_installed: 1500,
            updates_available: 10,
            orphaned: 5,
            cache_size_mb: 2048.0,
            last_update: Some(chrono::Utc::now()),
        },
        services: ServiceInfo {
            total_units: 300,
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
            last_boot_time_secs: 12.5,
            avg_boot_time_secs: Some(13.0),
            trend: Some(BootTrend::Improving),
        }),
        audio: AudioTelemetry {
            has_sound_hardware: true,
            pipewire_running: true,
            wireplumber_running: true,
            pipewire_pulse_running: true,
        },
    }
}

fn create_test_knowledge() -> SystemKnowledgeBase {
    let mut kb = SystemKnowledgeBase::default();
    kb.hardware = HardwareProfile {
        cpu_model: Some("Intel Core i7-11800H".to_string()),
        cpu_physical_cores: Some(8),
        cpu_logical_cores: Some(16),
        gpu_model: Some("NVIDIA GeForce RTX 3060".to_string()),
        gpu_type: Some("discrete".to_string()),
        sound_devices: vec!["USB Audio".to_string()],
        total_ram_bytes: Some(32 * 1024 * 1024 * 1024),
        machine_model: Some("Laptop Workstation".to_string()),
    };
    kb
}

// ============================================================================
// Query Understanding Tests (10 tests)
// ============================================================================

#[test]
fn test_classify_kernel_version() {
    let cat = understand_query("how do I check my kernel version?");
    assert!(matches!(cat, QueryCategory::CommandSynthesis(CommandGoal::CheckKernelVersion)));
}

#[test]
fn test_classify_wifi_list() {
    let cat = understand_query("how do I list WiFi networks?");
    assert!(matches!(cat, QueryCategory::CommandSynthesis(CommandGoal::ListWifiNetworks)));
}

#[test]
fn test_classify_pacman_config_path() {
    let cat = understand_query("where is the pacman config?");
    assert!(matches!(cat, QueryCategory::ConfigPathDiscovery(ConfigFile::Pacman)));
}

#[test]
fn test_classify_grub_config_path() {
    let cat = understand_query("where is grub configuration?");
    assert!(matches!(cat, QueryCategory::ConfigPathDiscovery(ConfigFile::Grub)));
}

#[test]
fn test_classify_wayland_check() {
    let cat = understand_query("am I running Wayland?");
    assert!(matches!(cat, QueryCategory::CapabilityCheck(CapabilityType::DisplayServer)));
}

#[test]
fn test_classify_desktop_check() {
    let cat = understand_query("what desktop am I using?");
    assert!(matches!(cat, QueryCategory::CapabilityCheck(CapabilityType::DesktopEnvironment)));
}

#[test]
fn test_classify_gpu_check() {
    let cat = understand_query("which GPU do I have?");
    assert!(matches!(cat, QueryCategory::CapabilityCheck(CapabilityType::Gpu)));
}

#[test]
fn test_classify_service_running() {
    let cat = understand_query("is sshd running?");
    assert!(matches!(cat, QueryCategory::SystemStateCheck(StateQuery::ServiceRunning(_))));
}

#[test]
fn test_classify_package_installed() {
    let cat = understand_query("do I have vim installed?");
    assert!(matches!(cat, QueryCategory::SystemStateCheck(StateQuery::PackageInstalled(_))));
}

#[test]
fn test_classify_disk_space_check() {
    let cat = understand_query("how do I check disk space?");
    assert!(matches!(cat, QueryCategory::CommandSynthesis(CommandGoal::CheckDiskSpace)));
}

// ============================================================================
// Answer Generation Tests - Capability Checks (10 tests)
// ============================================================================

#[test]
fn test_answer_desktop_environment() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DesktopEnvironment);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.explanation.contains("Hyprland"));
    assert!(!answer.citations.is_empty());
}

#[test]
fn test_answer_display_server() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DisplayServer);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.explanation.contains("Wayland"));
    assert!(answer.citations.iter().any(|c| c.url.contains("Wayland") || c.url.contains("Xorg")));
}

#[test]
fn test_answer_gpu() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::Gpu);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.explanation.contains("NVIDIA") || answer.explanation.contains("RTX"));
    assert!(!answer.commands.is_empty());
}

#[test]
fn test_format_capability_answer() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DesktopEnvironment);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    let formatted = format_answer(&answer);
    assert!(formatted.contains("Hyprland"));
    assert!(formatted.contains("References:"));
    assert!(formatted.contains("wiki.archlinux.org"));
}

#[test]
fn test_no_hallucination_in_capability() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::Gpu);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should mention actual GPU from telemetry
    assert!(answer.explanation.contains("NVIDIA") || answer.explanation.contains("GeForce"));
    // Should not mention AMD or Intel (not the actual GPU)
    assert!(!answer.explanation.contains("AMD Radeon"));
    assert!(!answer.explanation.contains("Intel UHD"));
}

#[test]
fn test_capability_answer_includes_verification_command() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::Gpu);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should include command to verify GPU
    assert!(!answer.commands.is_empty());
    assert!(answer.commands.iter().any(|cmd| cmd.command.contains("lspci")));
}

#[test]
fn test_capability_answer_wiki_citation() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DisplayServer);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Must have wiki citations
    assert!(!answer.citations.is_empty());
    assert!(answer.citations.iter().all(|c| c.url.starts_with("https://wiki.archlinux.org")));
}

#[test]
fn test_capability_no_filler_text() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DesktopEnvironment);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should be concise, no filler
    assert!(answer.explanation.len() < 200);
    assert!(!answer.explanation.contains("In conclusion"));
    assert!(!answer.explanation.contains("In summary"));
}

#[test]
fn test_desktop_answer_no_suggestions_if_known() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DesktopEnvironment);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Since we already know the DE, should not suggest "run this command to check"
    assert!(answer.commands.is_empty());
}

#[test]
fn test_wayland_answer_specific() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CapabilityCheck(CapabilityType::DisplayServer);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should be definitive
    assert!(answer.explanation.contains("Wayland") || answer.explanation.contains("running"));
    assert!(!answer.explanation.contains("might be") && !answer.explanation.contains("probably"));
}

// ============================================================================
// Command Synthesis Tests (10 tests)
// ============================================================================

#[test]
fn test_kernel_version_command() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckKernelVersion);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.commands.iter().any(|cmd| cmd.command.contains("uname -r")));
    assert!(!answer.citations.is_empty());
}

#[test]
fn test_wifi_list_command() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::ListWifiNetworks);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.commands.iter().any(|cmd| cmd.command.contains("nmcli")));
}

#[test]
fn test_service_status_command() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckServiceStatus("sshd".to_string()));
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.commands.iter().any(|cmd| cmd.command.contains("systemctl status sshd")));
    assert!(answer.citations.iter().any(|c| c.url.contains("Systemd")));
}

#[test]
fn test_disk_space_command() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckDiskSpace);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.commands.iter().any(|cmd| cmd.command.contains("df -h")));
}

#[test]
fn test_command_has_description() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckKernelVersion);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    for cmd in &answer.commands {
        assert!(!cmd.description.is_empty());
        assert!(cmd.description.len() > 5);
    }
}

#[test]
fn test_command_marked_safe() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckKernelVersion);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Inspection commands should not require root
    for cmd in &answer.commands {
        assert!(!cmd.requires_root);
    }
}

#[test]
fn test_command_package_requirement() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::ListWifiNetworks);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // nmcli requires networkmanager
    assert!(answer.commands.iter().any(|cmd| cmd.requires_package.is_some()));
}

#[test]
fn test_command_wiki_backed() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckServiceStatus("nginx".to_string()));
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Must have wiki citation
    assert!(!answer.citations.is_empty());
    assert!(answer.citations.iter().any(|c| c.url.contains("wiki.archlinux.org")));
}

#[test]
fn test_no_invented_commands() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckKernelVersion);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should use standard uname, not invented commands
    assert!(answer.commands.iter().any(|cmd| cmd.command == "uname -r"));
    assert!(!answer.commands.iter().any(|cmd| cmd.command.contains("get-kernel") || cmd.command.contains("show-version")));
}

#[test]
fn test_command_format_consistent() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::CommandSynthesis(CommandGoal::CheckDiskSpace);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    let formatted = format_answer(&answer);
    // Should have $ prefix
    assert!(formatted.contains("$ "));
    // Should have description indented
    assert!(formatted.contains("  "));
}

// ============================================================================
// Config Path Discovery Tests (5 tests)
// ============================================================================

#[test]
fn test_pacman_config_path() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::ConfigPathDiscovery(ConfigFile::Pacman);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.explanation.contains("/etc/pacman.conf") || answer.tailored_notes.iter().any(|n| n.contains("/etc/pacman.conf")));
}

#[test]
fn test_grub_config_path() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::ConfigPathDiscovery(ConfigFile::Grub);
    let answer = generate_answer(cat, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    assert!(answer.explanation.contains("/etc/default/grub") || answer.tailored_notes.iter().any(|n| n.contains("/etc/default/grub")));
}

#[test]
fn test_config_path_wiki_citation() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::ConfigPathDiscovery(ConfigFile::Pacman);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    assert!(!answer.citations.is_empty());
}

#[test]
fn test_config_path_no_hallucination() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::ConfigPathDiscovery(ConfigFile::Pacman);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Should not invent paths
    assert!(!answer.explanation.contains("/usr/local"));
    assert!(!answer.explanation.contains("/opt"));
}

#[test]
fn test_config_path_absolute() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let cat = QueryCategory::ConfigPathDiscovery(ConfigFile::Grub);
    let answer = generate_answer(cat, &knowledge, &telemetry).unwrap();

    // Paths must be absolute
    let has_absolute = answer.explanation.starts_with("/")
        || answer.tailored_notes.iter().any(|n| n.starts_with("/"))
        || answer.explanation.contains("/etc/");
    assert!(has_absolute);
}

// ============================================================================
// Integration Tests (5 tests)
// ============================================================================

#[test]
fn test_end_to_end_kernel_query() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let query = "how do I check my kernel version?";
    let category = understand_query(query);
    let answer = generate_answer(category, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();
    let formatted = format_answer(&answer);

    // Should have explanation
    assert!(formatted.contains("kernel") || formatted.contains("uname"));
    // Should have command
    assert!(formatted.contains("$ uname -r"));
    // Should have wiki reference
    assert!(formatted.contains("wiki.archlinux.org"));
}

#[test]
fn test_end_to_end_desktop_query() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let query = "what desktop am I using?";
    let category = understand_query(query);
    let answer = generate_answer(category, &knowledge, &telemetry);

    assert!(answer.is_some());
    let answer = answer.unwrap();

    // Should reflect actual system
    assert!(answer.explanation.contains("Hyprland"));
}

#[test]
fn test_end_to_end_config_query() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let query = "where is the pacman config?";
    let category = understand_query(query);
    let answer = generate_answer(category, &knowledge, &telemetry);

    assert!(answer.is_some());
    let formatted = format_answer(&answer.unwrap());
    assert!(formatted.contains("/etc/pacman.conf"));
}

#[test]
fn test_deterministic_answers() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let query = "how do I check my kernel version?";

    // Run same query 3 times
    let cat1 = understand_query(query);
    let ans1 = generate_answer(cat1.clone(), &knowledge, &telemetry);

    let cat2 = understand_query(query);
    let ans2 = generate_answer(cat2, &knowledge, &telemetry);

    let cat3 = understand_query(query);
    let ans3 = generate_answer(cat3, &knowledge, &telemetry);

    // All should produce same answer
    let fmt1 = format_answer(&ans1.unwrap());
    let fmt2 = format_answer(&ans2.unwrap());
    let fmt3 = format_answer(&ans3.unwrap());

    assert_eq!(fmt1, fmt2);
    assert_eq!(fmt2, fmt3);
}

#[test]
fn test_all_answers_have_citations() {
    let telemetry = create_test_telemetry();
    let knowledge = create_test_knowledge();

    let queries = vec![
        "how do I check my kernel version?",
        "what desktop am I using?",
        "where is the pacman config?",
        "am I running Wayland?",
    ];

    for query in queries {
        let category = understand_query(query);
        if let Some(answer) = generate_answer(category, &knowledge, &telemetry) {
            assert!(!answer.citations.is_empty(), "Query '{}' has no citations", query);
        }
    }
}
