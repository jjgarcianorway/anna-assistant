use anyhow::Result;
use tracing::{info, warn};

/// Perform a web search using the google_web_search tool.
pub async fn google_web_search(query: String) -> Result<String> {
    info!("Performing web search for query: {}", query);

    // Call the google_web_search tool
    let search_results = crate::google_web_search(query.clone()).await?;

    info!("Web search completed for query: {}", query);

    Ok(search_results)
}