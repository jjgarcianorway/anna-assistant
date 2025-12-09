//! Deterministic router - routes queries and enforces probe spine (v0.0.172).
//!
//! v0.45.x stabilization: LLM-first reasoning with probe spine.
//! Deterministic code selects tools and enforces safety, but does NOT invent answers.
//!
//! Key policy:
//! - can_answer_deterministically = true ONLY for narrow typed queries with extractable claims
//! - All other queries go to LLM specialist with probe evidence
//! - Probe spine enforces minimum probes when evidence is required

mod query_class;
mod query_class_impl;
mod route_types;
mod routes_config;
mod routes_core;
mod routes_hardware;
mod routes_kernel;
mod routes_network;
mod routes_packages;
mod routes_security;
mod routes_services;
mod routes_storage;
mod routes_system;

pub use query_class::QueryClass;
pub use route_types::DeterministicRoute;

use anna_shared::probe_spine::RouteCapability;
use anna_shared::rpc::{QueryIntent, SpecialistDomain, TranslatorTicket};
use tracing::info;

// Re-export classify_query from the patterns module
pub use crate::query_classify::classify_query;

/// Get deterministic route for a query
pub fn get_route(query: &str) -> DeterministicRoute {
    let class = classify_query(query);
    build_route(class)
}

/// Build route from query class - delegates to specialized route modules
fn build_route(class: QueryClass) -> DeterministicRoute {
    // Try each route builder in order
    if let Some(route) = routes_core::build_core_route(class) {
        return route;
    }
    if let Some(route) = routes_hardware::build_hardware_route(class) {
        return route;
    }
    if let Some(route) = routes_system::build_system_route(class) {
        return route;
    }
    if let Some(route) = routes_network::build_network_route(class) {
        return route;
    }
    if let Some(route) = routes_storage::build_storage_route(class) {
        return route;
    }
    if let Some(route) = routes_services::build_services_route(class) {
        return route;
    }
    if let Some(route) = routes_security::build_security_route(class) {
        return route;
    }
    if let Some(route) = routes_packages::build_packages_route(class) {
        return route;
    }
    if let Some(route) = routes_kernel::build_kernel_route(class) {
        return route;
    }
    if let Some(route) = routes_config::build_config_route(class) {
        return route;
    }
    if let Some(route) = routes_config::build_diagnostic_route(class) {
        return route;
    }

    // Unknown - full LLM path
    DeterministicRoute {
        class,
        domain: SpecialistDomain::System,
        intent: QueryIntent::Question,
        probes: vec![],
        capability: RouteCapability {
            can_answer_deterministically: false,
            evidence_required: false,
            required_evidence: vec![],
            spine_probes: vec![],
        },
    }
}

/// Apply deterministic router, overriding LLM ticket for known classes
pub fn apply_deterministic_routing(
    query: &str,
    llm_ticket: Option<TranslatorTicket>,
) -> TranslatorTicket {
    let route = get_route(query);

    if route.class == QueryClass::Unknown {
        return llm_ticket.unwrap_or_else(|| create_default_ticket(&route));
    }

    info!(
        "Deterministic router: class={:?}, domain={}, probes={:?}, can_det={}",
        route.class,
        route.domain,
        route.probes,
        route.can_answer_deterministically()
    );

    TranslatorTicket {
        intent: route.intent,
        domain: route.domain,
        entities: vec![],
        needs_probes: route.probes,
        clarification_question: None,
        confidence: 1.0,
        answer_contract: None,
    }
}

/// Create default ticket from route
fn create_default_ticket(route: &DeterministicRoute) -> TranslatorTicket {
    TranslatorTicket {
        intent: route.intent,
        domain: route.domain,
        entities: vec![],
        needs_probes: route.probes.clone(),
        clarification_question: None,
        confidence: 0.5,
        answer_contract: None,
    }
}

/// Check if query class can be answered deterministically
#[allow(dead_code)]
pub fn can_answer_deterministically(query: &str) -> bool {
    get_route(query).can_answer_deterministically()
}
