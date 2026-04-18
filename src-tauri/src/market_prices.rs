use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Runtime};

use crate::dictionary;

const MARKET_STATISTICS_ENDPOINT: &str =
    "http://api.thaumictom.de/warframe/v1/market_statistics.json";
const HTTP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/thaumictom/artus)"
);
const MARKET_HTTP_TIMEOUT_SECS: u64 = 20;
const AUTO_REFRESH_INTERVAL_SECS: u64 = 3 * 60 * 60;
const MANUAL_REFRESH_COOLDOWN_SECS: u64 = 30;
const MARKET_PRICES_STATUS_EVENT: &str = "market_prices_status_updated";

#[derive(Debug, Deserialize)]
struct MarketStatisticsRoot {
    #[serde(default)]
    item_statistics: Vec<MarketStatisticsItem>,
}

#[derive(Debug, Deserialize)]
struct MarketStatisticsItem {
    #[serde(default)]
    item: String,
    #[serde(default)]
    last_fetched_at: String,
    #[serde(default)]
    statistics_today: MarketMedianContainer,
    #[serde(default)]
    current_offers: MarketMedianContainer,
}

#[derive(Debug, Default, Deserialize)]
struct MarketMedianContainer {
    median: Option<f64>,
}

#[derive(Debug, Default)]
struct MarketPricesCache {
    prices: HashMap<String, String>,
    endpoint_last_fetched_at_unix_secs: Option<u64>,
    last_fetched_data_from: Option<String>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketPricesStatusPayload {
    pub endpoint_last_fetched_at_unix_secs: Option<u64>,
    pub last_fetched_data_from: Option<String>,
    pub manual_refresh_cooldown_secs_left: u64,
    pub last_error: Option<String>,
    pub loaded_price_count: usize,
}

static MARKET_PRICES_CACHE: OnceLock<RwLock<MarketPricesCache>> = OnceLock::new();
static DICTIONARY_NAME_TO_SLUG: OnceLock<Result<HashMap<String, String>, String>> = OnceLock::new();
static LAST_MANUAL_REFRESH_REQUEST_UNIX_SECS: OnceLock<Mutex<Option<u64>>> = OnceLock::new();

pub fn initialize_market_prices_on_start<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        if let Err(err) = refresh_market_prices_cache(&app) {
            eprintln!("[market_prices] startup refresh failed: {err}");
        }

        loop {
            std::thread::sleep(Duration::from_secs(AUTO_REFRESH_INTERVAL_SECS));

            if let Err(err) = refresh_market_prices_cache(&app) {
                eprintln!("[market_prices] scheduled refresh failed: {err}");
            }
        }
    });
}

#[tauri::command]
pub fn get_market_prices_status() -> Result<MarketPricesStatusPayload, String> {
    build_status_payload()
}

#[tauri::command]
pub fn refresh_market_prices<R: Runtime>(
    app: AppHandle<R>,
) -> Result<MarketPricesStatusPayload, String> {
    let now_unix_secs = current_unix_secs()?;
    let cooldown_secs_left = mark_manual_refresh_request_and_get_cooldown(now_unix_secs)?;
    if cooldown_secs_left > 0 {
        return Err(format!(
            "refresh is on cooldown ({}s remaining)",
            cooldown_secs_left
        ));
    }

    if let Err(err) = refresh_market_prices_cache(&app) {
        eprintln!("[market_prices] manual refresh failed: {err}");
    }

    build_status_payload()
}

pub fn lookup_price_for_name<R: Runtime>(app: &AppHandle<R>, name: &str) -> Option<String> {
    let name_to_slug = match load_name_to_slug_map(app) {
        Ok(name_to_slug) => name_to_slug,
        Err(err) => {
            eprintln!("[market_prices] failed to load dictionary cache: {err}");
            return None;
        }
    };

    let normalized_name = normalize_lookup_name(name);
    let slug = name_to_slug.get(&normalized_name)?;

    market_prices_cache()
        .read()
        .ok()
        .and_then(|cache| cache.prices.get(slug).cloned())
}

fn load_name_to_slug_map<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<&'static HashMap<String, String>, String> {
    let mapping = DICTIONARY_NAME_TO_SLUG.get_or_init(|| {
        let items = dictionary::load_cached_dictionary_items(app)?;
        let mut name_to_slug = HashMap::new();

        for item in items {
            let normalized_name = normalize_lookup_name(&item.name);
            if normalized_name.is_empty() {
                continue;
            }
            name_to_slug.insert(normalized_name, item.slug);
        }

        Ok(name_to_slug)
    });

    match mapping {
        Ok(value) => Ok(value),
        Err(err) => Err(err.clone()),
    }
}

fn refresh_market_prices_cache<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let parsed = fetch_market_statistics()?;
    let fetched_at_unix_secs = current_unix_secs()?;

    let mut prices = HashMap::new();
    let mut last_fetched_data_from = None;

    for entry in parsed.item_statistics {
        if entry.item.is_empty() {
            continue;
        }

        let formatted_price = entry
            .statistics_today
            .median
            .map(format_market_median)
            .or_else(|| {
                entry
                    .current_offers
                    .median
                    .map(|median| format!("{}*", format_market_median(median)))
            });

        if let Some(price) = formatted_price {
            prices.insert(entry.item, price);
        }

        if !entry.last_fetched_at.is_empty() {
            last_fetched_data_from = Some(match last_fetched_data_from {
                Some(previous) if previous > entry.last_fetched_at => previous,
                _ => entry.last_fetched_at,
            });
        }
    }

    {
        let mut cache = market_prices_cache()
            .write()
            .map_err(|_| "failed to update market prices cache".to_string())?;
        cache.prices = prices;
        cache.endpoint_last_fetched_at_unix_secs = Some(fetched_at_unix_secs);
        cache.last_fetched_data_from = last_fetched_data_from;
        cache.last_error = None;
    }

    emit_status_update(app);
    Ok(())
}

fn build_status_payload() -> Result<MarketPricesStatusPayload, String> {
    let now_unix_secs = current_unix_secs()?;
    let cache = market_prices_cache()
        .read()
        .map_err(|_| "failed to read market prices cache".to_string())?;
    let last_manual_refresh_unix_secs = *last_manual_refresh_request_unix_secs()
        .lock()
        .map_err(|_| "failed to read market refresh cooldown state".to_string())?;

    Ok(MarketPricesStatusPayload {
        endpoint_last_fetched_at_unix_secs: cache.endpoint_last_fetched_at_unix_secs,
        last_fetched_data_from: cache.last_fetched_data_from.clone(),
        manual_refresh_cooldown_secs_left: cooldown_secs_left(
            now_unix_secs,
            last_manual_refresh_unix_secs,
        ),
        last_error: cache.last_error.clone(),
        loaded_price_count: cache.prices.len(),
    })
}

fn mark_manual_refresh_request_and_get_cooldown(now_unix_secs: u64) -> Result<u64, String> {
    let mut last_manual_refresh_unix_secs = last_manual_refresh_request_unix_secs()
        .lock()
        .map_err(|_| "failed to update market refresh cooldown state".to_string())?;

    let cooldown_secs_left = cooldown_secs_left(now_unix_secs, *last_manual_refresh_unix_secs);
    if cooldown_secs_left > 0 {
        return Ok(cooldown_secs_left);
    }

    *last_manual_refresh_unix_secs = Some(now_unix_secs);
    Ok(0)
}

fn cooldown_secs_left(now_unix_secs: u64, last_refresh_unix_secs: Option<u64>) -> u64 {
    let Some(last_refresh_unix_secs) = last_refresh_unix_secs else {
        return 0;
    };

    let elapsed = now_unix_secs.saturating_sub(last_refresh_unix_secs);
    MANUAL_REFRESH_COOLDOWN_SECS.saturating_sub(elapsed)
}

fn market_prices_cache() -> &'static RwLock<MarketPricesCache> {
    MARKET_PRICES_CACHE.get_or_init(|| RwLock::new(MarketPricesCache::default()))
}

fn last_manual_refresh_request_unix_secs() -> &'static Mutex<Option<u64>> {
    LAST_MANUAL_REFRESH_REQUEST_UNIX_SECS.get_or_init(|| Mutex::new(None))
}

fn emit_status_update<R: Runtime>(app: &AppHandle<R>) {
    match build_status_payload() {
        Ok(payload) => {
            let _ = app.emit(MARKET_PRICES_STATUS_EVENT, payload);
        }
        Err(err) => {
            eprintln!("[market_prices] failed to emit status update: {err}");
        }
    }
}

fn normalize_lookup_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn fetch_market_statistics() -> Result<MarketStatisticsRoot, String> {
    build_market_http_client()?
        .get(MARKET_STATISTICS_ENDPOINT)
        .send()
        .map_err(|err| format!("failed to fetch market statistics: {err}"))?
        .error_for_status()
        .map_err(|err| format!("market statistics endpoint returned error status: {err}"))?
        .json::<MarketStatisticsRoot>()
        .map_err(|err| format!("failed to parse market statistics response: {err}"))
}

fn build_market_http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .user_agent(HTTP_USER_AGENT)
        .timeout(Duration::from_secs(MARKET_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build market statistics HTTP client: {err}"))
}

fn current_unix_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| format!("system clock is before unix epoch: {err}"))
}

fn format_market_median(median: f64) -> String {
    if median.fract() == 0.0 {
        format!("{}", median as i64)
    } else {
        format!("{median:.1}")
    }
}
