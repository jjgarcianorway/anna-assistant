//! Intent dispatcher - routes intents to appropriate handlers.

use std::collections::HashMap;

use super::disk::handle_check_disk_usage;
use super::memory::{handle_check_free_ram, handle_check_swap_presence, handle_list_top_memory_processes};
use super::packages::{handle_check_package_count, handle_check_package_installed};
use super::services::handle_check_failed_services;
use super::system::{handle_check_boot_time, handle_check_uptime};
use super::types::HandlerResult;

/// Dispatch to appropriate handler based on intent
pub fn dispatch_handler(
    ticket_id: &str,
    intent: &str,
    probes: &HashMap<String, String>,
    question: &str,
) -> HandlerResult {
    match intent {
        "check_free_ram" | "query_metric"
            if question.to_lowercase().contains("ram")
                || question.to_lowercase().contains("memory") =>
        {
            handle_check_free_ram(ticket_id, probes)
        }
        "check_swap_presence" | "check_swap" => handle_check_swap_presence(ticket_id, probes),
        "check_disk_usage" | "query_metric"
            if question.to_lowercase().contains("disk")
                || question.to_lowercase().contains("space") =>
        {
            handle_check_disk_usage(ticket_id, probes)
        }
        "check_failed_services" | "check_status"
            if question.to_lowercase().contains("failed")
                && question.to_lowercase().contains("service") =>
        {
            handle_check_failed_services(ticket_id, probes)
        }
        "check_boot_time" | "query_metric" if question.to_lowercase().contains("boot") => {
            handle_check_boot_time(ticket_id, probes)
        }
        "check_package_count" | "query_metric"
            if question.to_lowercase().contains("package")
                && question.to_lowercase().contains("count") =>
        {
            handle_check_package_count(ticket_id, probes)
        }
        "check_uptime" | "query_metric" if question.to_lowercase().contains("uptime") => {
            handle_check_uptime(ticket_id, probes)
        }
        "list_top_memory_processes" | "list"
            if question.to_lowercase().contains("memory")
                && question.to_lowercase().contains("process") =>
        {
            handle_list_top_memory_processes(ticket_id, probes)
        }
        _ => {
            // Check if we can infer intent from probes available
            if probes.contains_key("memory_info")
                && (question.contains("ram") || question.contains("memory"))
            {
                return handle_check_free_ram(ticket_id, probes);
            }
            if probes.contains_key("disk_usage")
                && (question.contains("disk") || question.contains("space"))
            {
                return handle_check_disk_usage(ticket_id, probes);
            }
            if probes.contains_key("failed_services") && question.contains("failed") {
                return handle_check_failed_services(ticket_id, probes);
            }
            if probes.contains_key("boot_time") && question.contains("boot") {
                return handle_check_boot_time(ticket_id, probes);
            }
            if probes.contains_key("uptime") && question.contains("uptime") {
                return handle_check_uptime(ticket_id, probes);
            }

            HandlerResult::NeedsSpecialist {
                reason: format!(
                    "No deterministic handler for intent '{}' with question '{}'",
                    intent, question
                ),
            }
        }
    }
}
