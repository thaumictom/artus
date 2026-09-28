//! Cached item lookups and live order/statistics commands.

use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};

use crate::api;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Base URL for warframe.market API v2.
const MARKET_API_V2: &str = "https://api.warframe.market/v2";

/// Base URL for warframe.market API v1 (statistics endpoint).
const MARKET_API_V1: &str = "https://api.warframe.market/v1";
const ORDER_CACHE_LIFETIME: Duration = Duration::from_secs(60);
const STATISTICS_CACHE_LIFETIME: Duration = Duration::from_secs(10 * 60);

struct CachedResponse {
    fetched_at: Instant,
    value: Value,
}

#[derive(Default)]
pub struct MarketResponseCaches {
    orders: HashMap<String, CachedResponse>,
    orders_generation: u64,
    statistics: HashMap<String, CachedResponse>,
}

fn fresh_response(
    entries: &mut HashMap<String, CachedResponse>,
    slug: &str,
    lifetime: Duration,
) -> Option<Value> {
    entries.retain(|_, response| response.fetched_at.elapsed() < lifetime);
    entries.get(slug).map(|response| response.value.clone())
}

pub(crate) fn invalidate_order_cache(state: &AppState) -> AppResult<()> {
    let mut caches = state.market_responses.lock()?;
    caches.orders.clear();
    caches.orders_generation = caches.orders_generation.wrapping_add(1);
    Ok(())
}

pub(crate) fn clear_response_caches(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let mut caches = state.market_responses.lock()?;
    caches.orders.clear();
    caches.statistics.clear();
    Ok(())
}

pub(crate) fn catalog_path(app: &AppHandle) -> AppResult<std::path::PathBuf> {
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

/// Revalidate the local item catalog using its ETag.
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
        .user_agent(api::USER_AGENT)
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
    fetch_json_request(client.get(url)).await
}

async fn fetch_json_request(request: reqwest::RequestBuilder) -> AppResult<Value> {
    let response = request
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

/// Returns the search list already loaded from /wfm-items.
#[tauri::command]
pub fn get_market_dictionary(state: State<'_, AppState>) -> AppResult<Value> {
    state.catalogs.lock()?.wfm_items.clone()
        .ok_or_else(|| AppError::msg("market dictionary unavailable"))
}

/// Minimal landing-page entry from the tradeable item feed.
#[derive(serde::Deserialize, serde::Serialize)]
pub struct MostTradedItem {
    slug: String,
    name: String,
    liquidity: Option<f64>,
}

/// Rank the minimal item feed by liquidity for incremental display in the frontend.
#[tauri::command]
pub fn get_most_traded_items(state: State<'_, AppState>) -> AppResult<Vec<MostTradedItem>> {
    #[derive(serde::Deserialize)]
    struct Feed {
        items: Vec<MostTradedItem>,
    }

    let response = state.catalogs.lock()?.tradeable_items.clone()
        .ok_or_else(|| AppError::msg("tradeable items unavailable"))?;
    let mut items = serde_json::from_value::<Feed>(response)?.items;
    items.retain(|item| item.liquidity.is_some_and(|value| value.is_finite()));
    items.sort_by(|a, b| {
        b.liquidity
            .partial_cmp(&a.liquidity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.slug.cmp(&b.slug))
    });
    Ok(items)
}

/// Exposes the tradeable-items medians already loaded for OCR at startup.
#[derive(serde::Serialize)]
pub struct MasteryTradeablePrice {
    median: f64,
    from_current_offers: bool,
}

#[tauri::command]
pub fn get_mastery_tradeable_prices(
    state: State<'_, AppState>,
) -> AppResult<HashMap<String, MasteryTradeablePrice>> {
    let prices = state.ocr_tradeable_prices.lock()?;
    Ok(prices
        .iter()
        .map(|(slug, price)| {
            (
                slug.clone(),
                MasteryTradeablePrice {
                    median: price.median,
                    from_current_offers: price.used_current_offer_fallback,
                },
            )
        })
        .collect())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeableTodayStatistics {
    median: Option<f64>,
    weighted_average: Option<f64>,
    volume: Option<f64>,
}

/// Returns the cached trade statistics for today's trades, without offer fallbacks.
#[tauri::command]
pub fn get_tradeable_today_statistics(
    state: State<'_, AppState>,
    slug: String,
) -> AppResult<Option<TradeableTodayStatistics>> {
    let prices = state.ocr_tradeable_prices.lock()?;
    Ok(prices.get(&slug).map(|price| TradeableTodayStatistics {
        median: price.today_median,
        weighted_average: price.weighted_avg,
        volume: price.today_volume,
    }))
}

/// Returns item details from the cached full /v2/items catalog.
#[tauri::command]
pub fn get_cached_market_item(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    let item = api::cached_market_item(&state, &slug)?;
    Ok(serde_json::json!({ "apiVersion": "v2", "data": item, "error": null }))
}

/// Returns WFM item identity and variant fields without reading the market catalog.
#[tauri::command]
pub fn get_cached_wfm_item(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    let item = api::cached_wfm_item(&state, &slug)?;
    Ok(serde_json::json!({ "apiVersion": "v2", "data": item, "error": null }))
}

/// Minimal identities for the slugs recognized in one overlay capture.
#[tauri::command]
pub fn get_ocr_market_items(state: State<'_, AppState>, slugs: Vec<String>) -> AppResult<Value> {
    let cache = state.catalogs.lock()?;
    let items = slugs.into_iter().filter_map(|slug| {
        let base = slug.strip_suffix("_intact").or_else(|| slug.strip_suffix("_radiant")).unwrap_or(&slug);
        cache.wfm_by_slug.get(&slug).or_else(|| cache.wfm_by_slug.get(base))
            .map(|item| (slug, serde_json::json!({ "id": item["id"], "maxRank": item["maxRank"] })))
    }).collect::<serde_json::Map<String, Value>>();
    Ok(Value::Object(items))
}

/// Returns recent buy/sell orders for an item. Manual reloads bypass the cache.
#[tauri::command]
pub async fn get_market_orders(
    state: State<'_, AppState>,
    slug: String,
    force_refresh: Option<bool>,
) -> AppResult<Value> {
    fetch_market_orders(&state, &slug, force_refresh.unwrap_or(false)).await
}

pub(crate) async fn fetch_market_orders(state: &AppState, slug: &str, force_refresh: bool) -> AppResult<Value> {
    let generation = {
        let mut caches = state.market_responses.lock()?;
        if !force_refresh {
            if let Some(response) = fresh_response(&mut caches.orders, slug, ORDER_CACHE_LIFETIME) {
                return Ok(response);
            }
        }
        caches.orders_generation
    };
    let url = format!("{MARKET_API_V2}/orders/item/{slug}");
    let response = fetch_json_request(
        state
            .http_client
            .get(&url)
            .header("Platform", "pc")
            .header("Crossplay", "true"),
    )
    .await?;
    if !response.get("data").is_some_and(Value::is_array) {
        return Err(AppError::msg("Invalid item orders response"));
    }
    let mut caches = state.market_responses.lock()?;
    if caches.orders_generation == generation {
        caches.orders.insert(slug.to_owned(), CachedResponse {
            fetched_at: Instant::now(),
            value: response.clone(),
        });
    }
    Ok(response)
}

/// Returns historical price statistics cached for ten minutes per item.
#[tauri::command]
pub async fn get_market_statistics(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    if let Some(response) = fresh_response(
        &mut state.market_responses.lock()?.statistics,
        &slug,
        STATISTICS_CACHE_LIFETIME,
    ) {
        return Ok(response);
    }
    let url = format!("{MARKET_API_V1}/items/{slug}/statistics");
    let response = fetch_json(&state.http_client, &url).await?;
    if !response.get("payload").is_some_and(Value::is_object) {
        return Err(AppError::msg("Invalid item statistics response"));
    }
    state.market_responses.lock()?.statistics.insert(slug, CachedResponse {
        fetched_at: Instant::now(),
        value: response.clone(),
    });
    Ok(response)
}
