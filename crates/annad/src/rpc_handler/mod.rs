//! RPC request handlers with deterministic routing, triage, and fallback (v0.0.291).
//!
//! v0.0.166: Integrated stage modules for modularization.
//! v0.0.167: Integrated routing_stage module for further modularization.
//! v0.0.200: Modularized into domain-focused submodules.
//! v0.0.291: Extracted verification_stage for further modularization.

use anna_shared::rpc::params::WebSearchParams;
use anna_shared::rpc::result::{WebSearchItem, WebSearchResult};
use anna_shared::rpc::RpcResponse;
use anyhow::Error;
use serde_json;
use tracing::{info, warn}; // For the dummy search_results type

mod deterministic_handlers;
mod dispatcher;
mod fast_path_stage;
mod formatting;
mod helpers;
mod llm_request;
mod probe_handler;
mod request_helpers;
mod routing_handler;
mod triage_handler;
mod verification_stage;

// Re-export main handler
pub use dispatcher::handle_request;

pub async fn handle_web_search(id: String, params: Option<serde_json::Value>) -> RpcResponse {
    let params: WebSearchParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(p) => p,
            Err(e) => return RpcResponse::error(id, -32602, format!("Invalid params: {}", e)),
        },
        None => return RpcResponse::error(id, -32602, "Missing params".to_string()),
    };

    let search_results: Result<String, Error> = Ok(serde_json::json!({
        "organic_results": [
            {
                "title": "Dummy Search Result 1",
                "link": "https://example.com/result1",
                "snippet": "This is a dummy snippet for result 1 related to: "
            },
            {
                "title": "Dummy Search Result 2",
                "link": "https://example.com/result2",
                "snippet": "Another dummy snippet for result 2, about: "
            }
        ]
    })
    .to_string());

    match search_results {
        Ok(results_json) => {
            let mut web_search_results: Vec<WebSearchItem> = Vec::new();
            // Parse the JSON string from google_web_search into a serde_json::Value
            let parsed_results: serde_json::Value = serde_json::from_str(&results_json)
                .unwrap_or_else(|e| {
                    warn!("Failed to parse Google Web Search results: {}", e);
                    serde_json::json!({})
                });

            // Extract the "organic_results" array
            if let Some(organic_results) = parsed_results["organic_results"].as_array() {
                for item in organic_results {
                    let title = item["title"].as_str().unwrap_or("").to_string();
                    let url = item["link"].as_str().unwrap_or("").to_string();
                    let snippet = item["snippet"].as_str().unwrap_or("").to_string();
                    web_search_results.push(WebSearchItem {
                        title,
                        url,
                        snippet,
                    });
                }
            }

            let result = WebSearchResult {
                query: params.query,
                results: web_search_results,
            };
            RpcResponse::success(id, serde_json::to_value(result).unwrap())
        }
        Err(e) => RpcResponse::error(id, -32000, format!("Web search failed: {}", e)),
    }
}
