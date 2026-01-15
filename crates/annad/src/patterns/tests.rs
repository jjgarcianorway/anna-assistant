//! Tests for pattern matching.

use super::*;
use super::normalize::normalize_query;
use super::synonyms::expand_with_synonyms;
use super::typos::{edit_distance, fix_typos, fuzzy_correct_query};

#[test]
fn test_common_errors() {
    let result = match_common_pattern("pacman says database is locked");
    assert!(result.is_some());
    assert_eq!(result.as_ref().unwrap().confidence, 0.95);
    assert!(!result.unwrap().needs_confirmation);
    assert!(match_common_pattern("I accidentally deleted /usr/bin").is_some());
    assert!(match_common_pattern("why does my fan spin up when the system is idle").is_some());
    assert!(match_common_pattern("what is the meaning of life").is_none());
}

#[test]
fn test_contains_word() {
    assert!(contains_word("my id", "id") && contains_word("show id", "id"));
    assert!(!contains_word("bandwidth", "id") && !contains_word("idle system", "id"));
    assert!(contains_word("what at jobs", "at") && !contains_word("what jobs", "at"));
    assert!(contains_word("show kernel version", "kernel") && contains_word("kernel", "kernel"));
}

#[test]
fn test_factual_patterns() {
    for q in ["what is my disk usage", "show disk space", "how much ram do I have",
              "total memory", "what gpu do I have", "which graphics card",
              "what is my ip address", "kernel version", "list failed services"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_dev_patterns() {
    for q in ["git status", "show git log", "list docker containers",
              "docker images", "cargo version", "node version"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_security_patterns() {
    for q in ["firewall status", "ufw status", "list all users",
              "who has sudo access", "ssh key", "ssh status"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_desktop_patterns() {
    for q in ["wayland or x11", "which desktop am I running", "gnome version",
              "gnome extensions", "plasma version", "list connected monitors"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_howto_patterns() {
    for q in ["how do I install a package", "how to update system",
              "how to enable a service", "how to add a user",
              "how to change permissions", "how to change hostname"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_network_patterns() {
    for q in ["am i connected", "wifi status", "what is my ip", "public ip",
              "dns servers", "flush dns cache", "open ports", "listening ports"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_hardware_patterns() {
    for q in ["cpu temperature", "gpu temp", "battery status", "battery level",
              "cpu frequency", "cpu usage", "usb devices", "pci devices"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_gaming_patterns() {
    for q in ["steam installation", "steam games", "wine version", "proton version",
              "controller detect", "xbox controller", "vulkan support", "opengl version"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

#[test]
fn test_boot_patterns() {
    for q in ["grub config", "update grub", "efi boot entry", "boot order",
              "kernel version", "kernel parameters", "boot time", "boot errors"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

// Synonym and normalization tests
#[test]
fn test_synonyms_and_normalization() {
    // Synonym expansion
    assert!(match_common_pattern("how much ram").is_some());
    assert!(match_common_pattern("processor temperature").is_some());
    assert!(match_common_pattern("graphics temp").is_some());
    assert!(match_common_pattern("wireless status").is_some());
    let expanded = expand_with_synonyms("check my ram usage");
    assert!(expanded.contains("memory"));
    // Normalization
    let norm = normalize_query("what is my disk usage?");
    assert!(!norm.contains("?"));
    assert!(!normalize_query("please show me disk usage").contains("please"));
    assert_eq!(normalize_query("disk    usage"), "disk usage");
    // Normalized matching
    assert!(match_common_pattern("Please check my disk usage?").is_some());
    assert!(match_common_pattern("Can you show me the cpu temperature?").is_some());
}

// Fuzzy matching tests
#[test]
fn test_fuzzy_matching() {
    assert_eq!(edit_distance("disk", "disk"), 0);
    assert_eq!(edit_distance("disk", "dsk"), 1);
    assert_eq!(edit_distance("kernel", "kernal"), 1);
    assert!(fix_typos("pacaman").contains("pacman"));
    assert!(fix_typos("kernal version").contains("kernel"));
    assert!(fix_typos("systemclt status").contains("systemctl"));
    let corrected = fuzzy_correct_query("diks usage");
    assert!(corrected.is_some() && corrected.unwrap().contains("disk"));
    assert!(fuzzy_correct_query("disk usage").is_none());
}

#[test]
fn test_typo_pattern_matching() {
    for q in ["kernal version", "what is my diks usage", "memry usage",
              "packman database locked", "baterry status", "temperture check",
              "firwall status", "netwrok connection"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

// Container pattern tests
#[test]
fn test_container_patterns() {
    for q in ["docker containers", "docker images", "docker version",
              "podman containers", "podman images", "podman pods",
              "list vms", "running vms", "virtualization support"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

// Log pattern tests
#[test]
fn test_log_patterns() {
    for q in ["recent logs", "boot logs", "error logs", "kernel logs",
              "dmesg", "dmesg errors", "crash logs", "what happened", "sudo logs"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

// Audio pattern tests
#[test]
fn test_audio_patterns() {
    for q in ["no sound", "audio devices", "volume level",
              "pipewire status", "pipewire version", "alsa devices", "alsa mixer"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}

// Power pattern tests
#[test]
fn test_power_patterns() {
    for q in ["battery status", "battery level", "charging status",
              "suspend mode", "sleep modes", "screen brightness",
              "fan speed", "cpu governor"] {
        assert!(match_common_pattern(q).is_some(), "Failed: {}", q);
    }
}
