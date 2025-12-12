//! Cross-reference claims with external sources (v0.0.448).

use crate::rpc_handler::handle_web_search; // Updated import
use crate::state::SharedState; // New import
use anna_shared::rpc::params::WebSearchParams;
use anna_shared::rpc::result::WebSearchResult;
use anna_shared::truth_ledger::{Claim, Source, TrustScore, TruthLedger, Veracity};
use anyhow::Result;
use tracing::{info, warn};
use url::Url;
use uuid; // Need to import uuid crate

/// Performs a web search and updates the veracity of a claim in the TruthLedger.
pub async fn cross_reference_claim(
    state: SharedState, // Modified signature
    truth_ledger: &mut TruthLedger,
    claim_text: &str,
) -> Result<()> {
    info!("Cross-referencing claim: '{}' with web search", claim_text);

    let params = WebSearchParams {
        query: claim_text.to_string(),
    };

    // Generate a unique ID for this internal call
    let id = uuid::Uuid::new_v4().to_string();

    let response: anna_shared::rpc::RpcResponse =
        handle_web_search(id, Some(serde_json::to_value(params)?)).await;

    if response.is_success() {
        let search_result: WebSearchResult = serde_json::from_value(response.result.unwrap())?;

        let (verified_count, disputed_count) = analyze_search_results(&search_result);

        // Update the claim in the ledger based on web search results
        if verified_count > disputed_count && verified_count > 0 {
            info!(
                "Claim '{}' corroborated by web search. Verified: {} Disputed: {}",
                claim_text, verified_count, disputed_count
            );
            truth_ledger.verify_claim(claim_text);
        } else if disputed_count > verified_count && disputed_count > 0 {
            warn!(
                "Claim '{}' disputed by web search. Verified: {} Disputed: {}",
                claim_text, verified_count, disputed_count
            );
            truth_ledger.dispute_claim(claim_text);
        } else {
            info!(
                "Web search for '{}' inconclusive. Verified: {} Disputed: {}",
                claim_text, verified_count, disputed_count
            );
        }

        // Add web search results as sources
        for item in search_result.results {
            if let Ok(url) = Url::parse(&item.url) {
                truth_ledger.add_claim(
                    Claim {
                        text: item.snippet, // Store the snippet as a related claim
                    },
                    Source::Url(url),
                    TrustScore::Unknown, // Initial trust score for web sources
                    0.5,                 // Initial confidence for web snippets
                    None,                // Default to Unverified
                );
            }
        }
    } else {
        warn!(
            "Web search for claim '{}' failed: {}",
            claim_text,
            response
                .error_message()
                .unwrap_or("unknown error".to_string())
        );
    }

    Ok(())
}

/// Analyzes web search results to determine corroboration or dispute.
fn analyze_search_results(search_result: &WebSearchResult) -> (usize, usize) {
    let mut verified_count = 0;
    let mut disputed_count = 0;

    // This is a very basic analysis. In a real system, this would involve NLP,
    // sentiment analysis, and more sophisticated reasoning.
    for item in &search_result.results {
        let snippet_lower = item.snippet.to_lowercase();
        let query_lower = search_result.query.to_lowercase();

        if snippet_lower.contains(&query_lower) {
            // Very basic: if snippet contains the query, it's likely corroborating
            verified_count += 1;
        } else {
            // Placeholder for negative indicators
            // if snippet_lower.contains("false") || snippet_lower.contains("incorrect") {
            //     disputed_count += 1;
            // }
        }
    }

    (verified_count, disputed_count)
}
