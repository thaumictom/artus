//! Remote dictionary fetching, fuzzy matching, and tradeable-item price lookups.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use log::{info, warn};
use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use super::{
    OcrWord, OcrThemeOption,
    OCR_DICTIONARY_HTTP_TIMEOUT_SECS, OCR_DICTIONARY_API_URL, TRADEABLE_ITEMS_API_URL,
    CUSTOM_OCR_DICTIONARY_ITEMS, THEME_COLORS_TOML,
};

// ── Types owned by this module (moved from state.rs) ──────────────────────────

/// A single entry in the OCR dictionary used for fuzzy matching.
#[derive(Debug, Clone)]
pub struct OcrDictionaryEntry {
    pub name: String,
    pub slug: String,
    pub tags: Vec<String>,
    pub normalized_name: String,
    pub ducats: Option<u64>,
    pub vaulted: Option<bool>,
    pub is_custom: bool,
}

/// Median price data for a tradeable item, keyed by slug.
#[derive(Debug, Clone)]
pub struct TradeablePriceEntry {
    pub median: f64,
    pub used_current_offer_fallback: bool,
    pub trades_24h: Option<f64>,
    pub moving_avg: Option<f64>,
    pub ducats: Option<u64>,
}

// ── API response types ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ThemeColorsToml {
    #[serde(default)]
    primary: BTreeMap<String, [u8; 3]>,
}

#[derive(Debug, Deserialize)]
struct DictionaryApiResponse {
    #[serde(default)]
    tradeable_items: Vec<DictionaryApiItem>,
}

#[derive(Debug, Deserialize)]
struct DictionaryApiItem {
    name: String,
    slug: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    ducats: Option<u64>,
    #[serde(default)]
    vaulted: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemsApiResponse {
    #[serde(default)]
    tradeable_items: Vec<TradeableItemApiItem>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemApiItem {
    slug: String,
    #[serde(default)]
    statistics_today: Vec<TradeableItemStats>,
    #[serde(default)]
    current_offers: Vec<TradeableItemStats>,
    #[serde(default)]
    ducats: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemStats {
    median: Option<f64>,
    volume: Option<f64>,
    moving_avg: Option<f64>,
}

// ── HTTP helper ───────────────────────────────────────────────────────────────

/// Builds a blocking HTTP client with the configured timeout.
fn blocking_http_client() -> AppResult<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OCR_DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| AppError::msg(format!("failed to build HTTP client: {err}")))
}

// ── Theme loading ─────────────────────────────────────────────────────────────

/// Parses the embedded `theme_colors.toml` and returns the available themes.
pub fn load_primary_theme_options<R: Runtime>(
    _app: &AppHandle<R>,
) -> AppResult<Vec<OcrThemeOption>> {
    let parsed: ThemeColorsToml = toml::from_str(THEME_COLORS_TOML)?;

    let themes: Vec<OcrThemeOption> = parsed
        .primary
        .into_iter()
        .map(|(name, rgb)| OcrThemeOption { name, rgb })
        .collect();

    if themes.is_empty() {
        return Err(AppError::msg(
            "primary section in theme_colors.toml is empty",
        ));
    }

    Ok(themes)
}

// ── Dictionary loading ────────────────────────────────────────────────────────

/// Fetches the OCR dictionary from the remote API and stores it in [`AppState`].
/// Returns the number of entries loaded.
pub fn load_ocr_dictionary<R: Runtime>(app: &AppHandle<R>) -> AppResult<usize> {
    let client = blocking_http_client()?;

    let payload: DictionaryApiResponse = client
        .get(OCR_DICTIONARY_API_URL)
        .send()?
        .error_for_status()?
        .json()?;

    let mut entries: Vec<OcrDictionaryEntry> = payload
        .tradeable_items
        .into_iter()
        .filter_map(|item| {
            let name = item.name.trim();
            let slug = item.slug.trim();
            if name.is_empty() || slug.is_empty() {
                return None;
            }
            let normalized_name = normalize_dictionary_text(name);
            if normalized_name.is_empty() {
                return None;
            }
            Some(OcrDictionaryEntry {
                name: name.to_string(),
                slug: slug.to_string(),
                tags: item.tags,
                normalized_name,
                ducats: item.ducats,
                vaulted: item.vaulted,
                is_custom: false,
            })
        })
        .collect();

    // Append hard-coded custom items that aren't in the remote API
    for name in CUSTOM_OCR_DICTIONARY_ITEMS {
        if let Some(entry) = build_custom_dictionary_entry(name) {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| a.normalized_name.cmp(&b.normalized_name));
    entries.dedup_by(|a, b| a.normalized_name == b.normalized_name);

    let count = entries.len();
    *app.state::<AppState>().ocr_dictionary.lock()? = entries;
    Ok(count)
}

// ── Tradeable item prices ─────────────────────────────────────────────────────

/// Fetches tradeable item price statistics and stores them in [`AppState`].
/// Returns the number of items with valid median prices.
pub fn load_tradeable_item_prices<R: Runtime>(app: &AppHandle<R>) -> AppResult<usize> {
    let client = blocking_http_client()?;

    let payload: TradeableItemsApiResponse = client
        .get(TRADEABLE_ITEMS_API_URL)
        .send()?
        .error_for_status()?
        .json()?;

    let mut prices = HashMap::new();

    for item in payload.tradeable_items {
        let slug = item.slug.trim();
        if slug.is_empty() {
            continue;
        }

        let stats_today = item.statistics_today.first();
        let offers = item.current_offers.first();

        let stats_median = stats_today
            .and_then(|s| s.median)
            .filter(|m| m.is_finite());
        let offers_median = offers
            .and_then(|s| s.median)
            .filter(|m| m.is_finite());

        // Prefer today's stats; fall back to current offers
        let Some((median, used_fallback)) = stats_median
            .map(|v| (v, false))
            .or_else(|| offers_median.map(|v| (v, true)))
        else {
            continue;
        };

        prices.insert(
            slug.to_string(),
            TradeablePriceEntry {
                median,
                used_current_offer_fallback: used_fallback,
                trades_24h: stats_today.and_then(|s| s.volume).filter(|v| v.is_finite()),
                moving_avg: stats_today.and_then(|s| s.moving_avg).filter(|v| v.is_finite()),
                ducats: item.ducats,
            },
        );
    }

    let count = prices.len();
    *app.state::<AppState>().ocr_tradeable_prices.lock()? = prices;
    Ok(count)
}

// ── Dictionary matching ───────────────────────────────────────────────────────

/// Maps grouped OCR words to the closest dictionary entries, enriching each
/// word with slug, price, ducat, and vaulted metadata.
///
/// Words that don't meet the similarity `threshold` are dropped entirely.
pub fn map_words_to_dictionary<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
    threshold: f64,
) -> Vec<OcrWord> {
    if words.is_empty() {
        return Vec::new();
    }

    let state = app.state::<AppState>();
    let dict = match state.ocr_dictionary.lock() {
        Ok(guard) => guard,
        Err(_) => return words.to_vec(),
    };

    if dict.is_empty() {
        return words.to_vec();
    }

    // Lazy-load prices if they haven't been fetched yet
    let needs_prices = state
        .ocr_tradeable_prices
        .lock()
        .map(|p| p.is_empty())
        .unwrap_or(false);

    if needs_prices {
        match load_tradeable_item_prices(app) {
            Ok(count) => info!("lazy-loaded tradeable item prices: {count}"),
            Err(err) => warn!("failed to lazy-load tradeable item prices: {err}"),
        }
    }

    let prices = state.ocr_tradeable_prices.lock().ok();
    let prices_ref = prices.as_deref();

    words
        .iter()
        .filter_map(|word| match_single_word(word, &dict, prices_ref, threshold))
        .collect()
}

/// Finds the best dictionary match for a single word/block.
fn match_single_word(
    word: &OcrWord,
    dictionary: &[OcrDictionaryEntry],
    prices: Option<&HashMap<String, TradeablePriceEntry>>,
    threshold: f64,
) -> Option<OcrWord> {
    let normalized = normalize_dictionary_text(&word.text);
    if normalized.is_empty() {
        return None;
    }
    let tokens: Vec<&str> = normalized.split_whitespace().collect();

    // Find the highest-scoring candidate
    let best = dictionary.iter().map(|candidate| {
        // Bonus for tag overlap with OCR tokens
        let tag_bonus = candidate
            .tags
            .iter()
            .filter(|tag| {
                let nt = normalize_dictionary_text(tag);
                !nt.is_empty() && tokens.iter().any(|t| *t == nt.as_str())
            })
            .count() as f64
            * 0.02;

        let score = (similarity_score(&normalized, &candidate.normalized_name) + tag_bonus).min(1.0);
        (candidate, score)
    }).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;

    let (candidate, score) = best;
    if score < threshold {
        return None;
    }

    let mut mapped = word.clone();
    mapped.text = candidate.name.clone();
    mapped.slug = Some(candidate.slug.clone());
    mapped.mapping_confidence = Some(score);
    mapped.ducats = candidate.ducats;
    mapped.vaulted = candidate.vaulted;
    mapped.is_custom = Some(candidate.is_custom);

    // Enrich with price data
    if let Some(prices_map) = prices {
        if let Some(price) = prices_map.get(&candidate.slug) {
            mapped.market_median = Some(price.median);
            mapped.market_median_from_current_offers = Some(price.used_current_offer_fallback);
            if mapped.ducats.is_none() {
                mapped.ducats = price.ducats;
            }
            mapped.trades_24h = price.trades_24h;
            mapped.moving_avg = price.moving_avg;
        }
    }

    Some(mapped)
}

// ── String similarity ─────────────────────────────────────────────────────────

/// Combined similarity score: 85% Levenshtein distance + 15% token overlap.
fn similarity_score(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }

    let max_len = left.len().max(right.len());
    if max_len == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(left.as_bytes(), right.as_bytes());
    let lev_score = 1.0 - distance as f64 / max_len as f64;
    let overlap = token_overlap_score(left, right);
    (lev_score * 0.85 + overlap * 0.15).clamp(0.0, 1.0)
}

/// Fraction of tokens shared between two strings.
fn token_overlap_score(left: &str, right: &str) -> f64 {
    let lt: Vec<&str> = left.split_whitespace().collect();
    let rt: Vec<&str> = right.split_whitespace().collect();
    if lt.is_empty() || rt.is_empty() {
        return 0.0;
    }
    let shared = lt.iter().filter(|t| rt.contains(t)).count();
    shared as f64 / lt.len().max(rt.len()) as f64
}

/// Normalizes text for dictionary comparison: lowercase alphanumeric with
/// single spaces.
pub fn normalize_dictionary_text(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Creates a dictionary entry for a hard-coded custom item.
fn build_custom_dictionary_entry(name: &str) -> Option<OcrDictionaryEntry> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = normalize_dictionary_text(trimmed);
    if normalized.is_empty() {
        return None;
    }

    Some(OcrDictionaryEntry {
        name: trimmed.to_string(),
        slug: normalized.replace(' ', "_"),
        tags: Vec::new(),
        normalized_name: normalized,
        ducats: None,
        vaulted: None,
        is_custom: true,
    })
}

/// Classic two-row dynamic-programming Levenshtein distance.
fn levenshtein_distance(left: &[u8], right: &[u8]) -> usize {
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0usize; right.len() + 1];

    for (li, lb) in left.iter().enumerate() {
        curr[0] = li + 1;
        for (ri, rb) in right.iter().enumerate() {
            let cost = if lb == rb { 0 } else { 1 };
            curr[ri + 1] = (prev[ri + 1] + 1)
                .min(curr[ri] + 1)
                .min(prev[ri] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}
