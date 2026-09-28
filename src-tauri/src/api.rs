//! Shared catalog refreshes. UI commands read these snapshots or the local item file.

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::{market, ocr, state::AppState};

pub const USER_AGENT: &str = concat!("Artus/", env!("CARGO_PKG_VERSION"), " (+https://github.com/thaumictom/artus)");
const WFM_ITEMS: &str = "https://api.thaumictom.de/warframe/v2/wfm-items";
const TRADEABLE_ITEMS: &str = "https://api.thaumictom.de/warframe/v2/tradeable-items";
const MARKET_ITEMS: &str = "https://api.warframe.market/v2/items";
const REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);

#[derive(Default)]
pub struct CatalogCache {
    pub last_fetched_at: Option<u64>,
    pub wfm_items: Option<Value>,
    pub wfm_by_slug: HashMap<String, Value>,
    pub wfm_by_id: HashMap<String, Value>,
    pub tradeable_items: Option<Value>,
    pub market_by_slug: HashMap<String, Value>,
}

/// Drop all API data that must be fetched again in a new session.
/// The ETag-backed Thaumictom /items file and its metadata remain on disk.
pub fn clear_session_caches(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    *state.catalogs.lock()? = CatalogCache::default();
    state.ocr_dictionary.lock()?.clear();
    state.ocr_tradeable_prices.lock()?.clear();
    state.mastery_dictionary.lock()?.clear();
    market::clear_response_caches(app)
}

pub fn cached_wfm_item(state: &AppState, slug: &str) -> AppResult<Value> {
    let catalogs = state.catalogs.lock()?;
    let base = slug.strip_suffix("_intact").or_else(|| slug.strip_suffix("_radiant")).unwrap_or(slug);
    catalogs.wfm_by_slug.get(slug).or_else(|| catalogs.wfm_by_slug.get(base))
        .cloned()
        .ok_or_else(|| AppError::msg(format!("market item unavailable: {slug}")))
}

pub fn cached_market_item(state: &AppState, slug: &str) -> AppResult<Value> {
    let catalogs = state.catalogs.lock()?;
    let base = slug.strip_suffix("_intact").or_else(|| slug.strip_suffix("_radiant")).unwrap_or(slug);
    catalogs.market_by_slug.get(slug).or_else(|| catalogs.market_by_slug.get(base))
        .cloned()
        .ok_or_else(|| AppError::msg(format!("market item unavailable: {slug}")))
}

fn fetch(client: &reqwest::blocking::Client, url: &str, key: &str) -> AppResult<Value> {
    let value: Value = client.get(url).send()?.error_for_status()?.json()?;
    if value.get(key).and_then(Value::as_array).is_none_or(Vec::is_empty) {
        return Err(AppError::msg(format!("invalid catalog from {url}")));
    }
    Ok(value)
}

/// Run one serialized refresh cycle. A failed endpoint retains its previous snapshot.
pub fn refresh_catalogs(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let _refresh = state.catalog_refresh.lock()?;
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()?;
    let mut errors = Vec::new();
    let mut fetched = false;

    match fetch(&client, WFM_ITEMS, "items") {
        Ok(value) => {
            let items = value["items"].as_array().expect("validated item array");
            let mut by_slug = HashMap::new();
            let mut by_id = HashMap::new();
            for item in items {
                if let (Some(slug), Some(id)) = (item["slug"].as_str(), item["id"].as_str()) {
                    if !slug.is_empty() && !id.is_empty() {
                        by_slug.insert(slug.to_owned(), item.clone());
                        by_id.insert(id.to_owned(), item.clone());
                    }
                }
            }
            if by_slug.len() != items.len() || by_id.len() != items.len() {
                errors.push("wfm-items: missing or duplicate item IDs/slugs".into());
            } else {
                match ocr::load_ocr_dictionary(app, value.clone()) {
                    Ok(count) => {
                        let mut cache = state.catalogs.lock()?;
                        cache.wfm_items = Some(value);
                        cache.wfm_by_slug = by_slug;
                        cache.wfm_by_id = by_id;
                        fetched = true;
                        log::info!("loaded {count} OCR dictionary entries");
                    }
                    Err(error) => errors.push(format!("wfm-items: {error}")),
                }
            }
        }
        Err(error) => errors.push(format!("wfm-items: {error}")),
    }
    match fetch(&client, TRADEABLE_ITEMS, "items") {
        Ok(value) => match ocr::load_tradeable_item_prices(app, value.clone()) {
            Ok(count) => {
                state.catalogs.lock()?.tradeable_items = Some(value);
                fetched = true;
                log::info!("loaded {count} tradeable prices");
            }
            Err(error) => errors.push(format!("tradeable-items: {error}")),
        },
        Err(error) => errors.push(format!("tradeable-items: {error}")),
    }
    match market::refresh_item_catalog(app) {
        Ok(count) => {
            fetched = true;
            log::info!("loaded {count} catalog items");
            if let Err(error) = ocr::load_mastery_dictionary(app) {
                errors.push(format!("mastery catalog: {error}"));
            }
        }
        Err(error) => errors.push(format!("items: {error}")),
    }
    match fetch(&client, MARKET_ITEMS, "data") {
        Ok(value) => {
            let mut by_slug = HashMap::new();
            for item in value["data"].as_array().into_iter().flatten() {
                if let (Some(slug), Some(id)) = (item["slug"].as_str(), item["id"].as_str()) {
                    if id.is_empty() { continue; }
                    by_slug.insert(slug.to_owned(), item.clone());
                }
            }
            if by_slug.is_empty() {
                errors.push("market items: no valid items".into());
            } else {
                let mut cache = state.catalogs.lock()?;
                cache.market_by_slug = by_slug;
                fetched = true;
            }
        }
        Err(error) => errors.push(format!("market items: {error}")),
    }

    if fetched {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(AppError::msg)?.as_millis() as u64;
        state.catalogs.lock()?.last_fetched_at = Some(timestamp);
        if let Err(error) = app.emit("api_catalogs_fetched", timestamp) {
            log::warn!("could not notify windows of catalog fetch: {error}");
        }
    }

    if errors.is_empty() { Ok(()) } else { Err(AppError::msg(errors.join("; "))) }
}

pub fn start_refresh_loop(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(REFRESH_INTERVAL).await;
            let handle = app.clone();
            match tauri::async_runtime::spawn_blocking(move || refresh_catalogs(&handle)).await {
                Ok(Ok(())) => log::info!("catalog refresh complete"),
                Ok(Err(error)) => log::warn!("catalog refresh incomplete: {error}"),
                Err(error) => log::warn!("catalog refresh failed: {error}"),
            }
        }
    });
}

#[tauri::command]
pub fn get_api_catalogs_last_fetched(app: AppHandle) -> AppResult<Option<u64>> {
    Ok(app.state::<AppState>().catalogs.lock()?.last_fetched_at)
}

#[tauri::command]
pub async fn refresh_api_catalogs(app: AppHandle) -> AppResult<()> {
    tauri::async_runtime::spawn_blocking(move || refresh_catalogs(&app))
        .await
        .map_err(AppError::msg)?
}
