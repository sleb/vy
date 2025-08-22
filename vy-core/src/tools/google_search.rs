use anyhow::Result;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct GoogleSearchError(String);

impl std::fmt::Display for GoogleSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for GoogleSearchError {}

impl GoogleSearchError {
    fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

#[derive(Debug, Deserialize)]
pub struct GoogleSearchArgs {
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

#[derive(Debug, Serialize)]
pub struct GoogleSearchResponse {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub total_results: u64,
}

pub struct GoogleSearchTool {
    api_key: String,
    search_engine_id: String,
}

impl GoogleSearchTool {
    pub fn new(api_key: String, search_engine_id: String) -> Self {
        Self {
            api_key,
            search_engine_id,
        }
    }
}

impl Tool for GoogleSearchTool {
    const NAME: &'static str = "google_search";

    type Error = GoogleSearchError;
    type Args = GoogleSearchArgs;
    type Output = GoogleSearchResponse;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Search Google for information on any topic. Useful for finding current information, news, facts, or answers to questions.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    }
                },
                "required": ["query"]
            })
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // Validate that we have the required configuration
        if self.api_key.is_empty() {
            return Err(GoogleSearchError::new(
                "Google API key not configured. Run: vy config set google_api_key",
            ));
        }

        if self.search_engine_id.is_empty() {
            return Err(GoogleSearchError::new(
                "Google Search Engine ID not configured. Run: vy config set google_search_engine_id",
            ));
        }

        // Build the Google Custom Search API URL
        let url = format!(
            "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}&num={}",
            self.api_key,
            self.search_engine_id,
            urlencoding::encode(&args.query),
            10
        );

        // Make the HTTP request
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "Vy-AI-Assistant/1.0")
            .send()
            .await
            .map_err(|e| GoogleSearchError::new(format!("Failed to send request: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GoogleSearchError::new(format!(
                "Google Search API request failed with status {status}: {error_text}"
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| GoogleSearchError::new(format!("Failed to read response: {e}")))?;

        // Parse the JSON response
        let api_response: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| GoogleSearchError::new(format!("Failed to parse JSON: {e}")))?;

        // Extract search results
        let mut results = Vec::new();

        if let Some(items) = api_response.get("items").and_then(|v| v.as_array()) {
            for item in items.iter().take(10) {
                if let (Some(title), Some(link), Some(snippet)) = (
                    item.get("title").and_then(|v| v.as_str()),
                    item.get("link").and_then(|v| v.as_str()),
                    item.get("snippet").and_then(|v| v.as_str()),
                ) {
                    results.push(SearchResult {
                        title: title.to_string(),
                        link: link.to_string(),
                        snippet: snippet.to_string(),
                    });
                }
            }
        }

        // Get total results count
        let total_results = api_response
            .get("searchInformation")
            .and_then(|info| info.get("totalResults"))
            .and_then(|total| total.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(GoogleSearchResponse {
            query: args.query,
            results,
            total_results,
        })
    }
}
