use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const PRICES_ENDPOINT: &str = "https://api.warframestat.us/wfinfo/prices/";
const CACHE_FILE_NAME: &str = "warframe_prices.json";
const MAX_CACHE_AGE_SECS: u64 = 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarframePrice {
    pub name: String,
    #[serde(deserialize_with = "deserialize_u32_from_string")]
    pub yesterday_vol: u32,
    #[serde(deserialize_with = "deserialize_u32_from_string")]
    pub today_vol: u32,
    #[serde(deserialize_with = "deserialize_f32_from_string")]
    pub custom_avg: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCache {
    pub fetched_at_unix_secs: u64,
    pub prices: Vec<WarframePrice>,
}

impl PriceCache {
    fn is_stale_at(&self, now_unix_secs: u64) -> bool {
        now_unix_secs.saturating_sub(self.fetched_at_unix_secs) >= MAX_CACHE_AGE_SECS
    }
}

pub fn refresh_prices_on_start<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        if let Err(err) = refresh_prices_if_stale(&app) {
            eprintln!("[prices] {err}");
        }
    });
}

pub fn refresh_prices_if_stale<R: Runtime>(app: &AppHandle<R>) -> Result<PriceCache, String> {
    let cache_path = cache_path(app)?;
    let now_unix_secs = current_unix_secs()?;

    println!("[prices] cache path: {}", cache_path.display());

    if let Some(cache) = load_cache(&cache_path)? {
        if !cache.is_stale_at(now_unix_secs) {
            println!("[prices] using cached prices from {}", cache_path.display());
            return Ok(cache);
        }
    }

    let prices = fetch_prices()?;
    let cache = PriceCache {
        fetched_at_unix_secs: now_unix_secs,
        prices,
    };

    write_cache(&cache_path, &cache)?;
    println!("[prices] cached fresh prices to {}", cache_path.display());

    Ok(cache)
}

pub fn load_cached_prices<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PriceCache>, String> {
    let cache_path = cache_path(app)?;
    load_cache(&cache_path)
}

fn fetch_prices() -> Result<Vec<WarframePrice>, String> {
    reqwest::blocking::Client::new()
        .get(PRICES_ENDPOINT)
        .send()
        .map_err(|err| format!("failed to fetch prices: {err}"))?
        .error_for_status()
        .map_err(|err| format!("price endpoint returned error status: {err}"))?
        .json::<Vec<WarframePrice>>()
        .map_err(|err| format!("failed to parse prices response: {err}"))
}

fn cache_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data directory: {err}"))?;

    fs::create_dir_all(&data_dir)
        .map_err(|err| format!("failed to create app data directory: {err}"))?;

    Ok(data_dir.join(CACHE_FILE_NAME))
}

fn load_cache(cache_path: &Path) -> Result<Option<PriceCache>, String> {
    match fs::read_to_string(cache_path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map(Some)
            .map_err(|err| format!("failed to parse cached prices JSON: {err}")),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("failed to read cached prices file: {err}")),
    }
}

fn write_cache(cache_path: &Path, cache: &PriceCache) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(cache)
        .map_err(|err| format!("failed to serialize cached prices JSON: {err}"))?;
    fs::write(cache_path, contents)
        .map_err(|err| format!("failed to write cached prices file: {err}"))
}

fn current_unix_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| format!("system clock is before unix epoch: {err}"))
}

fn deserialize_u32_from_string<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_number_from_string(deserializer)
}

fn deserialize_f32_from_string<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_number_from_string(deserializer)
}

fn deserialize_number_from_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    struct NumberOrStringVisitor<T>(std::marker::PhantomData<T>);

    impl<'de, T> Visitor<'de> for NumberOrStringVisitor<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a number or a string containing a number")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            value.parse::<T>().map_err(E::custom)
        }

        fn visit_string<E>(self, value: String) -> Result<T, E>
        where
            E: de::Error,
        {
            value.parse::<T>().map_err(E::custom)
        }

        fn visit_u64<E>(self, value: u64) -> Result<T, E>
        where
            E: de::Error,
        {
            value.to_string().parse::<T>().map_err(E::custom)
        }

        fn visit_i64<E>(self, value: i64) -> Result<T, E>
        where
            E: de::Error,
        {
            value.to_string().parse::<T>().map_err(E::custom)
        }

        fn visit_f64<E>(self, value: f64) -> Result<T, E>
        where
            E: de::Error,
        {
            value.to_string().parse::<T>().map_err(E::custom)
        }
    }

    deserializer.deserialize_any(NumberOrStringVisitor(std::marker::PhantomData))
}

#[cfg(test)]
mod tests {
    use super::{PriceCache, MAX_CACHE_AGE_SECS};

    #[test]
    fn cache_is_fresh_before_one_hour() {
        let cache = PriceCache {
            fetched_at_unix_secs: 1_000,
            prices: vec![],
        };

        assert!(!cache.is_stale_at(1_000 + MAX_CACHE_AGE_SECS - 1));
    }

    #[test]
    fn cache_is_stale_at_one_hour() {
        let cache = PriceCache {
            fetched_at_unix_secs: 1_000,
            prices: vec![],
        };

        assert!(cache.is_stale_at(1_000 + MAX_CACHE_AGE_SECS));
    }
}
