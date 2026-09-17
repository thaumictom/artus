//! Warframe.market API proxy commands.
//!
//! All requests go through the shared [`reqwest::Client`] stored in [`AppState`]
//! to reuse connections and respect keep-alive.

use serde_json::Value;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Base URL for warframe.market API v2.
const MARKET_API_V2: &str = "https://api.warframe.market/v2";

/// Base URL for warframe.market API v1 (statistics endpoint).
const MARKET_API_V1: &str = "https://api.warframe.market/v1";

/// Fetches JSON from a URL with the `Language: en` header.
async fn fetch_json(client: &reqwest::Client, url: &str) -> AppResult<Value> {
    let response = client
        .get(url)
        .header("Language", "en")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|e| AppError::msg(e.to_string()))?;

    response
        .json::<Value>()
        .await
        .map_err(|e| AppError::msg(e.to_string()))
}

/// Fetches the search dictionary outside the webview's CORS restrictions.
#[tauri::command]
pub async fn get_market_dictionary(state: State<'_, AppState>) -> AppResult<Value> {
    fetch_json(
        &state.http_client,
        "https://api.thaumictom.de/warframe/v2/wfm-items",
    )
    .await
}

/// Fetches item details from warframe.market.
#[tauri::command]
pub async fn get_market_item(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    let url = format!("{MARKET_API_V2}/item/{slug}");
    fetch_json(&state.http_client, &url).await
}

/// Fetches current buy/sell orders for an item.
#[tauri::command]
pub async fn get_market_orders(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    let url = format!("{MARKET_API_V2}/orders/item/{slug}");
    fetch_json(&state.http_client, &url).await
}

/// Fetches historical price statistics for an item.
#[tauri::command]
pub async fn get_market_statistics(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    let url = format!("{MARKET_API_V1}/items/{slug}/statistics");
    fetch_json(&state.http_client, &url).await
}
