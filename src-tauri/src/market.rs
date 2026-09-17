//! Warframe.market API proxy commands.
//!
//! All requests go through the shared [`reqwest::Client`] stored in [`AppState`]
//! to reuse connections and respect keep-alive.

use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Base URL for warframe.market API v2.
const MARKET_API_V2: &str = "https://api.warframe.market/v2";

/// Base URL for warframe.market API v1 (statistics endpoint).
const MARKET_API_V1: &str = "https://api.warframe.market/v1";

fn catalog_path(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(AppError::msg)?
        .join("market-items.json"))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CatalogCacheMetadata {
    etag: String,
    item_count: usize,
}

/// Revalidate once during setup. No catalog is retained in backend application state.
pub fn refresh_item_catalog(app: &AppHandle) -> AppResult<usize> {
    let path = catalog_path(app)?;
    let metadata_path = path.with_extension("meta.json");
    // Never send a validator without the corresponding local catalog.
    let cached = std::fs::metadata(&path)
        .ok()
        .filter(|file| file.is_file() && file.len() > 0)
        .and_then(|_| std::fs::read(&metadata_path).ok())
        .and_then(|bytes| serde_json::from_slice::<CatalogCacheMetadata>(&bytes).ok())
        .filter(|metadata| reqwest::header::HeaderValue::from_str(&metadata.etag).is_ok());
    let mut request = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .get("https://api.thaumictom.de/warframe/v2/items");
    if let Some(metadata) = &cached {
        request = request.header(reqwest::header::IF_NONE_MATCH, &metadata.etag);
    }
    let response = request.send()?.error_for_status()?;
    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        let metadata = cached
            .ok_or_else(|| AppError::msg("catalog returned 304 without a local validator"))?;
        log::info!("market item catalog unchanged; using local cache");
        return Ok(metadata.item_count);
    }
    let etag = response
        .headers()
        .get(reqwest::header::ETAG)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    // Some servers return 200 even when the ETag has not changed.
    if let Some(metadata) = &cached {
        if etag.as_deref() == Some(metadata.etag.as_str()) {
            return Ok(metadata.item_count);
        }
    }
    let bytes = response.bytes()?;
    let catalog: serde_json::Map<String, Value> = serde_json::from_slice(&bytes)?;
    if catalog.is_empty()
        || catalog
            .iter()
            .any(|(key, item)| !key.starts_with("/Lotus/") || !item.is_object())
    {
        return Err(AppError::msg(
            "invalid item catalog; keeping the previous cache",
        ));
    }
    let count = catalog.len();
    drop(catalog);
    std::fs::create_dir_all(
        path.parent()
            .ok_or_else(|| AppError::msg("invalid cache path"))?,
    )?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, &bytes)?;
    // Publish only a complete, validated download; failed refreshes leave the old file intact.
    // Invalidate the old validator before replacing its data, so an interrupted
    // update cannot leave an ETag associated with the wrong catalog.
    match std::fs::remove_file(&metadata_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    std::fs::rename(temporary, path)?;
    if let Some(etag) = etag {
        let metadata = CatalogCacheMetadata {
            etag,
            item_count: count,
        };
        let temporary_metadata = metadata_path.with_extension("json.tmp");
        std::fs::write(&temporary_metadata, serde_json::to_vec(&metadata)?)?;
        std::fs::rename(temporary_metadata, metadata_path)?;
    }
    Ok(count)
}

/// Read only the local copy. The caller owns its lifetime while Market is mounted.
#[tauri::command]
pub async fn get_cached_market_items(app: AppHandle) -> AppResult<Value> {
    let path = catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || -> AppResult<Value> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(path)?,
        ))?)
    })
    .await
    .map_err(AppError::msg)?
}

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
