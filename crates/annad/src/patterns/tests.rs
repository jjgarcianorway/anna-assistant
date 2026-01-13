//! Tests for pattern matching.

use super::*;
use super::normalize::normalize_query;
use super::synonyms::expand_with_synonyms;
use super::typos::{edit_distance, fix_typos, fuzzy_correct_query};

#[test]
fn test_pacman_database_locked() {
    let result = match_common_pattern("pacman says database is locked");
    assert!(result.is_some());
    let u = result.unwrap();
    assert_eq!(u.confidence, 0.95);
    assert!(!u.needs_confirmation);
}

#[test]
fn test_deleted_usr_bin() {
    let result = match_common_pattern("I accidentally deleted /usr/bin");
    assert!(result.is_some());
    assert!(!result.unwrap().needs_confirmation);
}

#[test]
fn test_fan_idle() {
    let result = match_common_pattern("why does my fan spin up when the system is idle");
    assert!(result.is_some());
}

#[test]
fn test_no_match() {
    let result = match_common_pattern("what is the meaning of life");
    assert!(result.is_none());
}

#[test]
fn test_contains_word() {
    assert!(contains_word("my id", "id"));
    assert!(contains_word("show id", "id"));
    assert!(!contains_word("bandwidth", "id"));
    assert!(!contains_word("idle system", "id"));
    assert!(contains_word("what at jobs", "at"));
    assert!(!contains_word("what jobs", "at"));
    assert!(contains_word("show kernel version", "kernel"));
    assert!(contains_word("kernel", "kernel"));
}

// Factual pattern tests
#[test]
fn test_factual_disk_usage() {
    assert!(match_common_pattern("what is my disk usage").is_some());
    assert!(match_common_pattern("show disk space").is_some());
}

#[test]
fn test_factual_ram() {
    assert!(match_common_pattern("how much ram do I have").is_some());
    assert!(match_common_pattern("total memory").is_some());
}

#[test]
fn test_factual_gpu() {
    assert!(match_common_pattern("what gpu do I have").is_some());
    assert!(match_common_pattern("which graphics card").is_some());
}

#[test]
fn test_factual_ip() {
    assert!(match_common_pattern("what is my ip address").is_some());
    assert!(match_common_pattern("show my ip").is_some());
}

#[test]
fn test_factual_kernel() {
    assert!(match_common_pattern("what kernel am I running").is_some());
    assert!(match_common_pattern("kernel version").is_some());
}

#[test]
fn test_factual_services() {
    assert!(match_common_pattern("list failed services").is_some());
    assert!(match_common_pattern("show running services").is_some());
}

// Development pattern tests
#[test]
fn test_dev_git() {
    assert!(match_common_pattern("git status").is_some());
    assert!(match_common_pattern("show git log").is_some());
}

#[test]
fn test_dev_docker() {
    assert!(match_common_pattern("list docker containers").is_some());
    assert!(match_common_pattern("docker images").is_some());
}

#[test]
fn test_dev_build_tools() {
    assert!(match_common_pattern("cargo version").is_some());
    assert!(match_common_pattern("node version").is_some());
}

// Security pattern tests
#[test]
fn test_sec_firewall() {
    assert!(match_common_pattern("firewall status").is_some());
    assert!(match_common_pattern("ufw status").is_some());
}

#[test]
fn test_sec_users() {
    assert!(match_common_pattern("list all users").is_some());
    assert!(match_common_pattern("who has sudo access").is_some());
}

#[test]
fn test_sec_ssh() {
    assert!(match_common_pattern("ssh key").is_some());
    assert!(match_common_pattern("ssh status").is_some());
}

// Desktop pattern tests
#[test]
fn test_desktop_display_server() {
    assert!(match_common_pattern("wayland or x11").is_some());
    assert!(match_common_pattern("which desktop am I running").is_some());
}

#[test]
fn test_desktop_gnome() {
    assert!(match_common_pattern("gnome version").is_some());
    assert!(match_common_pattern("gnome extensions").is_some());
}

#[test]
fn test_desktop_kde() {
    assert!(match_common_pattern("plasma version").is_some());
    assert!(match_common_pattern("kde settings").is_some());
}

#[test]
fn test_desktop_monitors() {
    assert!(match_common_pattern("list connected monitors").is_some());
    assert!(match_common_pattern("screen resolution").is_some());
}

// HowTo pattern tests
#[test]
fn test_howto_install_package() {
    assert!(match_common_pattern("how do I install a package").is_some());
    assert!(match_common_pattern("install package").is_some());
}

#[test]
fn test_howto_update_system() {
    assert!(match_common_pattern("how to update system").is_some());
    assert!(match_common_pattern("upgrade system").is_some());
}

#[test]
fn test_howto_enable_service() {
    assert!(match_common_pattern("how to enable a service").is_some());
    assert!(match_common_pattern("how to restart service").is_some());
}

#[test]
fn test_howto_add_user() {
    assert!(match_common_pattern("how to add a user").is_some());
    assert!(match_common_pattern("give sudo access").is_some());
}

#[test]
fn test_howto_file_permissions() {
    assert!(match_common_pattern("how to change permissions").is_some());
    assert!(match_common_pattern("make file executable").is_some());
}

#[test]
fn test_howto_system_config() {
    assert!(match_common_pattern("how to change hostname").is_some());
    assert!(match_common_pattern("how to reboot").is_some());
}

// Network pattern tests
#[test]
fn test_network_connection() {
    assert!(match_common_pattern("am i connected").is_some());
    assert!(match_common_pattern("wifi status").is_some());
}

#[test]
fn test_network_ip() {
    assert!(match_common_pattern("what is my ip").is_some());
    assert!(match_common_pattern("public ip").is_some());
}

#[test]
fn test_network_dns() {
    assert!(match_common_pattern("dns servers").is_some());
    assert!(match_common_pattern("flush dns cache").is_some());
}

#[test]
fn test_network_ports() {
    assert!(match_common_pattern("open ports").is_some());
    assert!(match_common_pattern("listening ports").is_some());
}

// Hardware pattern tests
#[test]
fn test_hardware_temperature() {
    assert!(match_common_pattern("cpu temperature").is_some());
    assert!(match_common_pattern("gpu temp").is_some());
}

#[test]
fn test_hardware_battery() {
    assert!(match_common_pattern("battery status").is_some());
    assert!(match_common_pattern("battery level").is_some());
}

#[test]
fn test_hardware_cpu() {
    assert!(match_common_pattern("cpu frequency").is_some());
    assert!(match_common_pattern("cpu usage").is_some());
}

#[test]
fn test_hardware_devices() {
    assert!(match_common_pattern("usb devices").is_some());
    assert!(match_common_pattern("pci devices").is_some());
}

// Gaming pattern tests
#[test]
fn test_gaming_steam() {
    assert!(match_common_pattern("steam installation").is_some());
    assert!(match_common_pattern("steam games").is_some());
}

#[test]
fn test_gaming_wine_proton() {
    assert!(match_common_pattern("wine version").is_some());
    assert!(match_common_pattern("proton version").is_some());
}

#[test]
fn test_gaming_controllers() {
    assert!(match_common_pattern("controller detect").is_some());
    assert!(match_common_pattern("xbox controller").is_some());
}

#[test]
fn test_gaming_graphics() {
    assert!(match_common_pattern("vulkan support").is_some());
    assert!(match_common_pattern("opengl version").is_some());
}

// Boot pattern tests
#[test]
fn test_boot_grub() {
    assert!(match_common_pattern("grub config").is_some());
    assert!(match_common_pattern("update grub").is_some());
}

#[test]
fn test_boot_efi() {
    assert!(match_common_pattern("efi boot entry").is_some());
    assert!(match_common_pattern("boot order").is_some());
}

#[test]
fn test_boot_kernel() {
    assert!(match_common_pattern("kernel version").is_some());
    assert!(match_common_pattern("kernel parameters").is_some());
}

#[test]
fn test_boot_issues() {
    assert!(match_common_pattern("boot time").is_some());
    assert!(match_common_pattern("boot errors").is_some());
}

// Synonym expansion tests
#[test]
fn test_synonym_expansion() {
    assert!(match_common_pattern("how much ram").is_some());
    assert!(match_common_pattern("processor temperature").is_some());
    assert!(match_common_pattern("graphics temp").is_some());
    assert!(match_common_pattern("wireless status").is_some());
}

#[test]
fn test_expanded_synonyms() {
    assert!(match_common_pattern("sound status").is_some() ||
            match_common_pattern("audio status").is_some());
    assert!(match_common_pattern("screen resolution").is_some() ||
            match_common_pattern("display resolution").is_some());
    assert!(match_common_pattern("task manager").is_some() ||
            match_common_pattern("running processes").is_some());
}

#[test]
fn test_expand_with_synonyms() {
    let expanded = expand_with_synonyms("check my ram usage");
    assert!(expanded.contains("memory"));
    let expanded2 = expand_with_synonyms("processor info");
    assert!(expanded2.contains("cpu"));
}

// Query normalization tests
#[test]
fn test_normalize_query() {
    let norm = normalize_query("what is my disk usage?");
    assert!(!norm.contains("?"));
    let norm2 = normalize_query("please show me disk usage");
    assert!(!norm2.contains("please"));
    let norm3 = normalize_query("disk    usage");
    assert_eq!(norm3, "disk usage");
}

#[test]
fn test_normalized_pattern_matching() {
    assert!(match_common_pattern("Please check my disk usage?").is_some());
    assert!(match_common_pattern("Can you show me the cpu temperature?").is_some());
    assert!(match_common_pattern("Help me, I need to check battery status!").is_some());
}

// Fuzzy matching tests
#[test]
fn test_edit_distance() {
    assert_eq!(edit_distance("disk", "disk"), 0);
    assert_eq!(edit_distance("disk", "dsk"), 1);
    assert_eq!(edit_distance("memory", "memroy"), 2);
    assert_eq!(edit_distance("kernel", "kernal"), 1);
}

#[test]
fn test_fix_typos() {
    assert!(fix_typos("pacaman").contains("pacman"));
    assert!(fix_typos("kernal version").contains("kernel"));
    assert!(fix_typos("systemclt status").contains("systemctl"));
    assert!(fix_typos("memroy usage").contains("memory"));
}

#[test]
fn test_fuzzy_correct_query() {
    let corrected = fuzzy_correct_query("diks usage");
    assert!(corrected.is_some());
    assert!(corrected.unwrap().contains("disk"));
    let corrected2 = fuzzy_correct_query("memry usage");
    assert!(corrected2.is_some());
    assert!(corrected2.unwrap().contains("memory"));
    let corrected3 = fuzzy_correct_query("disk usage");
    assert!(corrected3.is_none());
}

#[test]
fn test_fuzzy_pattern_matching() {
    assert!(match_common_pattern("kernal version").is_some());
    assert!(match_common_pattern("what is my diks usage").is_some());
    assert!(match_common_pattern("memry usage").is_some());
    assert!(match_common_pattern("packman database locked").is_some());
}

#[test]
fn test_typo_pattern_matching() {
    assert!(match_common_pattern("baterry status").is_some());
    assert!(match_common_pattern("temperture check").is_some());
    assert!(match_common_pattern("firwall status").is_some());
    assert!(match_common_pattern("netwrok connection").is_some());
}

// Container pattern tests
#[test]
fn test_container_docker() {
    assert!(match_common_pattern("docker containers").is_some());
    assert!(match_common_pattern("docker images").is_some());
    assert!(match_common_pattern("docker version").is_some());
}

#[test]
fn test_container_podman() {
    assert!(match_common_pattern("podman containers").is_some());
    assert!(match_common_pattern("podman images").is_some());
    assert!(match_common_pattern("podman pods").is_some());
}

#[test]
fn test_container_vms() {
    assert!(match_common_pattern("list vms").is_some());
    assert!(match_common_pattern("running vms").is_some());
    assert!(match_common_pattern("virtualization support").is_some());
}

// Log pattern tests
#[test]
fn test_logs_journalctl() {
    assert!(match_common_pattern("recent logs").is_some());
    assert!(match_common_pattern("boot logs").is_some());
    assert!(match_common_pattern("error logs").is_some());
    assert!(match_common_pattern("kernel logs").is_some());
}

#[test]
fn test_logs_dmesg() {
    assert!(match_common_pattern("dmesg").is_some());
    assert!(match_common_pattern("dmesg errors").is_some());
}

#[test]
fn test_logs_analysis() {
    assert!(match_common_pattern("crash logs").is_some());
    assert!(match_common_pattern("what happened").is_some());
    assert!(match_common_pattern("sudo logs").is_some());
}

// Audio pattern tests
#[test]
fn test_audio_general() {
    assert!(match_common_pattern("no sound").is_some());
    assert!(match_common_pattern("audio devices").is_some());
    assert!(match_common_pattern("volume level").is_some());
}

#[test]
fn test_audio_pipewire() {
    assert!(match_common_pattern("pipewire status").is_some());
    assert!(match_common_pattern("pipewire version").is_some());
}

#[test]
fn test_audio_alsa() {
    assert!(match_common_pattern("alsa devices").is_some());
    assert!(match_common_pattern("alsa mixer").is_some());
}

// Power pattern tests
#[test]
fn test_power_battery() {
    assert!(match_common_pattern("battery status").is_some());
    assert!(match_common_pattern("battery level").is_some());
    assert!(match_common_pattern("charging status").is_some());
}

#[test]
fn test_power_suspend() {
    assert!(match_common_pattern("suspend mode").is_some());
    assert!(match_common_pattern("sleep modes").is_some());
}

#[test]
fn test_power_laptop() {
    assert!(match_common_pattern("screen brightness").is_some());
    assert!(match_common_pattern("fan speed").is_some());
    assert!(match_common_pattern("cpu governor").is_some());
}
