use std::collections::HashSet;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State};

use crate::state::AppState;

const MARKET_API_URL: &str = "https://api.warframe.market/v2/orders/item/{slug}";
const MARKET_HTTP_TIMEOUT_SECS: u64 = 10;

#[derive(Debug, Clone, Serialize)]
pub struct MarketDictionaryItem {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketItemResultPayload {
    pub slug: String,
    pub payload: serde_json::Value,
}

#[tauri::command]
pub fn get_market_dictionary_items(
    state: State<'_, AppState>,
) -> Result<Vec<MarketDictionaryItem>, String> {
    let dictionary = state
        .ocr_dictionary
        .lock()
        .map_err(|_| "failed to read OCR dictionary".to_string())?;

    let mut seen_slugs = HashSet::new();
    let mut items = Vec::with_capacity(dictionary.len());

    for entry in dictionary.iter() {
        let name = entry.name.trim();
        let slug = entry.slug.trim();

        if name.is_empty() || slug.is_empty() {
            continue;
        }

        if seen_slugs.insert(slug.to_string()) {
            items.push(MarketDictionaryItem {
                name: name.to_string(),
                slug: slug.to_string(),
            });
        }
    }

    items.sort_by(|left, right| {
        left.name
            .to_ascii_lowercase()
            .cmp(&right.name.to_ascii_lowercase())
    });

    Ok(items)
}

#[tauri::command]
pub fn fetch_market_item_by_slug<R: Runtime>(
    app: AppHandle<R>,
    slug: String,
) -> Result<serde_json::Value, String> {
    let slug_key = slug.trim().to_ascii_lowercase();
    if slug_key.is_empty() {
        return Err("slug cannot be empty".to_string());
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(MARKET_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build market client: {err}"))?;

    let request_url = MARKET_API_URL.replace("{slug}", &slug_key);
    let response = client
        .get(request_url)
        .send()
        .map_err(|err| format!("failed to fetch market item: {err}"))?
        .error_for_status()
        .map_err(|err| format!("market request failed: {err}"))?;

    let payload = response
        .json::<serde_json::Value>()
        .map_err(|err| format!("failed to parse market response: {err}"))?;

    app.emit(
        "market_item_result",
        MarketItemResultPayload {
            slug: slug_key,
            payload: payload.clone(),
        },
    )
    .map_err(|err| format!("failed to emit market response: {err}"))?;

    Ok(payload)
}
