//! Query classification patterns for deterministic routing (v0.0.810).
//!
//! This module contains the pattern matching logic that classifies user queries
//! into known QueryClass categories for deterministic probe selection.
//!
//! Classification is organized by domain:
//! - `classify_core`: Help, meta, triage, health summary
//! - `classify_hardware`: CPU, GPU, memory, disk, audio, sensors
//! - `classify_network`: Interfaces, ports, DNS, gateway, connectivity
//! - `classify_services`: Systemd, docker, crontab, timers
//! - `classify_system`: Uptime, load, users, hostname, OS, architecture
//! - `classify_storage`: Block devices, LVM, RAID, ZFS, mounts
//! - `classify_security`: Firewall, SELinux, SSH, logins, sudoers
//! - `classify_config`: Editor, shell, git, packages, kernel modules

mod classify_config;
mod classify_core;
mod classify_hardware;
mod classify_network;
mod classify_security;
mod classify_services;
mod classify_storage;
mod classify_system;
mod helpers;
pub mod patterns;

use crate::router::QueryClass;
use helpers::strip_greetings;

/// Classify query to a known class.
///
/// Delegates to domain-specific classifiers in priority order.
/// Returns `QueryClass::Unknown` if no pattern matches.
pub fn classify_query(query: &str) -> QueryClass {
    let q = query.to_lowercase();
    let stripped = strip_greetings(query);

    // Core queries (help, meta, triage, health) - highest priority
    if let Some(class) = classify_core::classify_core(&q, &stripped) {
        return class;
    }

    // Hardware queries (CPU, GPU, memory, disk, etc.)
    if let Some(class) = classify_hardware::classify_hardware(&q) {
        return class;
    }

    // Service queries (systemd, docker, etc.)
    if let Some(class) = classify_services::classify_services(&q) {
        return class;
    }

    // Network queries
    if let Some(class) = classify_network::classify_network(&q) {
        return class;
    }

    // Storage queries (block devices, LVM, RAID, ZFS)
    if let Some(class) = classify_storage::classify_storage(&q) {
        return class;
    }

    // Security queries (firewall, SELinux, SSH, logins)
    if let Some(class) = classify_security::classify_security(&q) {
        return class;
    }

    // System queries (uptime, users, hostname, etc.)
    if let Some(class) = classify_system::classify_system(&q) {
        return class;
    }

    // Configuration queries (editor, shell, git, packages)
    if let Some(class) = classify_config::classify_config(&q) {
        return class;
    }

    QueryClass::Unknown
}
