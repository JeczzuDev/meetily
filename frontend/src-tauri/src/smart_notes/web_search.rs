use crate::smart_notes::models::SearchResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

const TAVILY_SEARCH_URL: &str = "https://api.tavily.com/search";
const TAVILY_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_RESULTS: usize = 5;

/// Tavily Search API request
#[derive(Debug, Serialize)]
struct TavilySearchRequest<'a> {
    query: &'a str,
    max_results: usize,
    include_answer: bool,
}

/// Tavily Search API response structures
#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    results: Option<Vec<TavilyResult>>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
}

/// Search the web using Tavily Search API
///
/// Returns up to MAX_RESULTS search results, or an empty Vec on failure.
/// This function never fails hard — a web search failure should not prevent
/// the Smart Note from being generated (it just won't have sources).
pub async fn search_tavily(
    client: &Client,
    query: &str,
    api_key: &str,
) -> Result<Vec<SearchResult>, String> {
    info!("Tavily Search: querying '{}'", query);

    let body = TavilySearchRequest {
        query,
        max_results: MAX_RESULTS,
        include_answer: false,
    };

    let response = client
        .post(TAVILY_SEARCH_URL)
        .header("Content-Type", "application/json")
        .bearer_auth(api_key)
        .json(&body)
        .timeout(TAVILY_TIMEOUT)
        .send()
        .await
        .map_err(|e| format!("Tavily Search request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        warn!("Tavily Search returned {}: {}", status, body);
        return Err(format!("Tavily Search returned status {}", status));
    }

    let tavily_response: TavilySearchResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Tavily Search response: {}", e))?;

    let results: Vec<SearchResult> = tavily_response
        .results
        .map(|results| {
            results
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url,
                    snippet: r.content,
                })
                .collect()
        })
        .unwrap_or_default();

    info!("Tavily Search: got {} results for '{}'", results.len(), query);
    Ok(results)
}
