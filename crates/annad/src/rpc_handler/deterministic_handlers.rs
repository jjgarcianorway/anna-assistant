//! Deterministic handlers for specific query classes (v0.0.291).
//!
//! Extracted from llm_request.rs to keep files under 400 lines.
//! Handles ConfigureEditor, DesktopWallpaper, SystemUpdate, ConfigureShell.

use anna_shared::rpc::{ProbeResult, RpcResponse, SpecialistDomain, TranslatorTicket};
use anna_shared::transcript::Transcript;

use crate::configure_editor::{handle_configure_editor, ConfigureEditorResult};
use crate::configure_shell::{handle_configure_shell, ConfigureShellResult};
use crate::desktop_wallpaper::{handle_desktop_wallpaper, DesktopWallpaperResult};
use crate::router::{self, QueryClass};
use crate::system_update::{handle_system_update, SystemUpdateResult};

pub enum DeterministicHandlerResult {
    Handled(RpcResponse),
    NotHandled,
}

/// Try all deterministic handlers in sequence.
pub fn try_all_deterministic_handlers(
    id: &str,
    request_id: &str,
    query_class: &QueryClass,
    query: &str,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> DeterministicHandlerResult {
    // Try ConfigureEditor
    if let DeterministicHandlerResult::Handled(response) = try_handle_configure_editor(
        id,
        request_id,
        query_class,
        query,
        ticket,
        probe_results,
        transcript.clone(),
        classified_domain,
    ) {
        return DeterministicHandlerResult::Handled(response);
    }

    // Try DesktopWallpaper
    if let DeterministicHandlerResult::Handled(response) = try_handle_desktop_wallpaper(
        id,
        request_id,
        query_class,
        query,
        ticket,
        probe_results,
        transcript.clone(),
    ) {
        return DeterministicHandlerResult::Handled(response);
    }

    // Try SystemUpdate
    if let DeterministicHandlerResult::Handled(response) = try_handle_system_update(
        id,
        request_id,
        query_class,
        query,
        ticket,
        probe_results,
        transcript.clone(),
        classified_domain,
    ) {
        return DeterministicHandlerResult::Handled(response);
    }

    // Try ConfigureShell
    if let DeterministicHandlerResult::Handled(response) = try_handle_configure_shell(
        id,
        request_id,
        query_class,
        query,
        ticket,
        probe_results,
        transcript,
        classified_domain,
    ) {
        return DeterministicHandlerResult::Handled(response);
    }

    DeterministicHandlerResult::NotHandled
}

/// Try to handle ConfigureEditor query class.
pub fn try_handle_configure_editor(
    id: &str,
    request_id: &str,
    query_class: &QueryClass,
    query: &str,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> DeterministicHandlerResult {
    if *query_class != router::QueryClass::ConfigureEditor {
        return DeterministicHandlerResult::NotHandled;
    }

    let editor_result = handle_configure_editor(
        request_id.to_string(),
        query,
        ticket.clone(),
        probe_results,
        transcript,
        classified_domain,
    );

    match editor_result {
        ConfigureEditorResult::Handled(result) => {
            match serde_json::to_value(result) {
                Ok(v) => DeterministicHandlerResult::Handled(RpcResponse::success(id.to_string(), v)),
                Err(e) => DeterministicHandlerResult::Handled(RpcResponse::error(
                    id.to_string(),
                    -32603,
                    format!("Serialization error: {}", e),
                )),
            }
        }
        ConfigureEditorResult::NotApplicable => DeterministicHandlerResult::NotHandled,
    }
}

/// Try to handle DesktopWallpaper query class.
pub fn try_handle_desktop_wallpaper(
    id: &str,
    request_id: &str,
    query_class: &QueryClass,
    query: &str,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
) -> DeterministicHandlerResult {
    if *query_class != router::QueryClass::DesktopWallpaper {
        return DeterministicHandlerResult::NotHandled;
    }

    let wallpaper_result = handle_desktop_wallpaper(
        request_id.to_string(),
        query,
        ticket.clone(),
        probe_results,
        transcript,
    );

    match wallpaper_result {
        DesktopWallpaperResult::Handled(result) => {
            match serde_json::to_value(result) {
                Ok(v) => DeterministicHandlerResult::Handled(RpcResponse::success(id.to_string(), v)),
                Err(e) => DeterministicHandlerResult::Handled(RpcResponse::error(
                    id.to_string(),
                    -32603,
                    format!("Serialization error: {}", e),
                )),
            }
        }
        DesktopWallpaperResult::NotApplicable => DeterministicHandlerResult::NotHandled,
    }
}

/// Try to handle SystemUpdate query class.
pub fn try_handle_system_update(
    id: &str,
    request_id: &str,
    query_class: &QueryClass,
    query: &str,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> DeterministicHandlerResult {
    if *query_class != router::QueryClass::SystemUpdate {
        return DeterministicHandlerResult::NotHandled;
    }

    let update_result = handle_system_update(
        request_id.to_string(),
        query,
        ticket.clone(),
        probe_results,
        transcript,
        classified_domain,
    );

    match update_result {
        SystemUpdateResult::Handled(result) => {
            match serde_json::to_value(result) {
                Ok(v) => DeterministicHandlerResult::Handled(RpcResponse::success(id.to_string(), v)),
                Err(e) => DeterministicHandlerResult::Handled(RpcResponse::error(
                    id.to_string(),
                    -32603,
                    format!("Serialization error: {}", e),
                )),
            }
        }
        SystemUpdateResult::NotApplicable => DeterministicHandlerResult::NotHandled,
    }
}

/// Try to handle ConfigureShell query class.
pub fn try_handle_configure_shell(
    id: &str,
    request_id: &str,
    query_class: &QueryClass,
    query: &str,
    ticket: &TranslatorTicket,
    probe_results: &[ProbeResult],
    transcript: Transcript,
    classified_domain: SpecialistDomain,
) -> DeterministicHandlerResult {
    if *query_class != router::QueryClass::ConfigureShell {
        return DeterministicHandlerResult::NotHandled;
    }

    let shell_result = handle_configure_shell(
        request_id.to_string(),
        query,
        ticket.clone(),
        probe_results,
        transcript,
        classified_domain,
    );

    match shell_result {
        ConfigureShellResult::Handled(result) => {
            match serde_json::to_value(result) {
                Ok(v) => DeterministicHandlerResult::Handled(RpcResponse::success(id.to_string(), v)),
                Err(e) => DeterministicHandlerResult::Handled(RpcResponse::error(
                    id.to_string(),
                    -32603,
                    format!("Serialization error: {}", e),
                )),
            }
        }
        ConfigureShellResult::NotApplicable => DeterministicHandlerResult::NotHandled,
    }
}
