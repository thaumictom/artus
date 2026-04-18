use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const DICTIONARY_ENDPOINT: &str = "https://api.warframe.market/v2/items";
const HTTP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/thaumictom/artus)"
);
const DICTIONARY_HTTP_TIMEOUT_SECS: u64 = 20;
const CACHE_FILE_NAME: &str = "warframe_dictionary.json";
const MAX_CACHE_AGE_SECS: u64 = 72 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryItem {
    pub slug: String,
    pub tags: Vec<String>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryCache {
    pub fetched_at_unix_secs: u64,
    pub items: Vec<DictionaryItem>,
}

#[derive(Debug, Deserialize)]
struct MarketItemsResponse {
    data: Vec<MarketItem>,
}

#[derive(Debug, Deserialize)]
struct MarketItem {
    slug: String,
    #[serde(default)]
    tags: Vec<String>,
    i18n: MarketItemI18n,
}

#[derive(Debug, Deserialize)]
struct MarketItemI18n {
    en: MarketItemEnglish,
}

#[derive(Debug, Deserialize)]
struct MarketItemEnglish {
    name: String,
}

impl DictionaryCache {
    fn is_stale_at(&self, now_unix_secs: u64) -> bool {
        now_unix_secs.saturating_sub(self.fetched_at_unix_secs) >= MAX_CACHE_AGE_SECS
    }
}

pub fn refresh_dictionary_on_start<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        if let Err(err) = refresh_dictionary_if_stale(&app) {
            eprintln!("[dictionary] {err}");
        }
    });
}

pub fn refresh_dictionary_if_stale<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<DictionaryCache, String> {
    let cache_path = cache_path(app)?;
    let now_unix_secs = current_unix_secs()?;

    println!("[dictionary] cache path: {}", cache_path.display());

    if let Some(cache) = load_cache(&cache_path)? {
        if !cache.is_stale_at(now_unix_secs) {
            println!(
                "[dictionary] using cached dictionary from {}",
                cache_path.display()
            );
            return Ok(cache);
        }
    }

    let items = fetch_items()?;
    let cache = DictionaryCache {
        fetched_at_unix_secs: now_unix_secs,
        items,
    };

    write_cache(&cache_path, &cache)?;
    println!(
        "[dictionary] cached fresh dictionary to {}",
        cache_path.display()
    );

    Ok(cache)
}

pub fn load_cached_dictionary_names<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<String>, String> {
    let cache_path = cache_path(app)?;
    let cache = load_cache(&cache_path)?.ok_or_else(|| {
        "strict names enabled but warframe_dictionary.json has not been cached yet".to_string()
    })?;

    Ok(build_dictionary_names(&cache.items))
}

pub fn load_cached_dictionary_items<R: Runtime>(
    app: &AppHandle<R>,
) -> Result<Vec<DictionaryItem>, String> {
    let cache_path = cache_path(app)?;
    let cache = load_cache(&cache_path)?
        .ok_or_else(|| "warframe_dictionary.json has not been cached yet".to_string())?;

    Ok(cache.items)
}

fn fetch_items() -> Result<Vec<DictionaryItem>, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(HTTP_USER_AGENT)
        .timeout(Duration::from_secs(DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build dictionary HTTP client: {err}"))?;

    let response = client
        .get(DICTIONARY_ENDPOINT)
        .send()
        .map_err(|err| format!("failed to fetch dictionary: {err}"))?
        .error_for_status()
        .map_err(|err| format!("dictionary endpoint returned error status: {err}"))?
        .json::<MarketItemsResponse>()
        .map_err(|err| format!("failed to parse dictionary response: {err}"))?;

    Ok(response
        .data
        .into_iter()
        .map(|item| DictionaryItem {
            slug: item.slug,
            tags: item.tags,
            name: normalize_dictionary_name(&item.i18n.en.name),
        })
        .collect())
}

fn build_dictionary_names(items: &[DictionaryItem]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut dictionary = Vec::new();

    for item in items {
        let name = normalize_dictionary_name(&item.name);
        if name.is_empty() || name.ends_with(" Set") || name == "Set" {
            continue;
        }

        if seen.insert(name.clone()) {
            dictionary.push(name);
        }
    }

    dictionary
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

fn load_cache(cache_path: &Path) -> Result<Option<DictionaryCache>, String> {
    match fs::read_to_string(cache_path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map(Some)
            .map_err(|err| format!("failed to parse cached dictionary JSON: {err}")),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(format!("failed to read cached dictionary file: {err}")),
    }
}

fn write_cache(cache_path: &Path, cache: &DictionaryCache) -> Result<(), String> {
    let contents = serde_json::to_string_pretty(cache)
        .map_err(|err| format!("failed to serialize cached dictionary JSON: {err}"))?;
    fs::write(cache_path, contents)
        .map_err(|err| format!("failed to write cached dictionary file: {err}"))
}

fn normalize_dictionary_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn current_unix_secs() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|err| format!("system clock is before unix epoch: {err}"))
}
