use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_store::StoreExt;

use crate::state::{AppState, OcrDictionaryEntry, TradeablePriceEntry};

use super::{
    OcrWord, OcrThemeOption, SETTINGS_STORE_PATH, OVERLAY_DURATION_STORE_KEY,
    OVERLAY_TOGGLE_MODE_STORE_KEY, OCR_DICTIONARY_MAPPING_ENABLED_STORE_KEY,
    OCR_DICTIONARY_MATCH_THRESHOLD_STORE_KEY, OCR_THEME_STORE_KEY,
    DEFAULT_OVERLAY_TOGGLE_MODE, DEFAULT_OCR_DICTIONARY_MAPPING_ENABLED,
    DEFAULT_OCR_DICTIONARY_MATCH_THRESHOLD, MIN_OCR_DICTIONARY_MATCH_THRESHOLD,
    MAX_OCR_DICTIONARY_MATCH_THRESHOLD, OCR_DICTIONARY_HTTP_TIMEOUT_SECS,
    OCR_DICTIONARY_API_URL, TRADEABLE_ITEMS_API_URL, CUSTOM_OCR_DICTIONARY_ITEMS,
    THEME_COLORS_TOML,
};

#[derive(Debug, Deserialize)]
pub struct ThemeColorsToml {
    #[serde(default)]
    pub primary: BTreeMap<String, [u8; 3]>,
}

#[derive(Debug, Deserialize)]
pub struct DictionaryApiResponse {
    #[serde(default)]
    pub tradeable_items: Vec<DictionaryApiItem>,
}

#[derive(Debug, Deserialize)]
pub struct DictionaryApiItem {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub ducats: Option<u64>,
    #[serde(default)]
    pub vaulted: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TradeableItemsApiResponse {
    #[serde(default)]
    pub tradeable_items: Vec<TradeableItemApiItem>,
}

#[derive(Debug, Deserialize)]
pub struct TradeableItemApiItem {
    pub slug: String,
    #[serde(default)]
    pub statistics_today: Vec<TradeableItemStats>,
    #[serde(default)]
    pub current_offers: Vec<TradeableItemStats>,
    #[serde(default)]
    pub ducats: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct TradeableItemStats {
    pub median: Option<f64>,
    pub volume: Option<f64>,
    pub moving_avg: Option<f64>,
}

pub fn load_primary_theme_options<R: Runtime>(
    _app: &AppHandle<R>,
) -> Result<Vec<OcrThemeOption>, String> {
    let parsed: ThemeColorsToml = toml::from_str(THEME_COLORS_TOML)
        .map_err(|err| format!("failed to parse embedded theme colors: {err}"))?;

    let themes = parsed
        .primary
        .into_iter()
        .map(|(name, rgb)| OcrThemeOption { name, rgb })
        .collect::<Vec<_>>();

    if themes.is_empty() {
        return Err("primary section in theme_colors.toml is empty".to_string());
    }

    Ok(themes)
}

pub fn load_ocr_dictionary<R: Runtime>(app: &AppHandle<R>) -> Result<usize, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OCR_DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build dictionary client: {err}"))?;

    let response = client
        .get(OCR_DICTIONARY_API_URL)
        .send()
        .map_err(|err| format!("failed to fetch dictionary: {err}"))?
        .error_for_status()
        .map_err(|err| format!("dictionary request failed: {err}"))?;

    let payload: DictionaryApiResponse = response
        .json()
        .map_err(|err| format!("failed to parse dictionary response: {err}"))?;

    let mut dictionary_entries = payload
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
        .collect::<Vec<_>>();

    for name in CUSTOM_OCR_DICTIONARY_ITEMS {
        if let Some(entry) = build_custom_dictionary_entry(name) {
            dictionary_entries.push(entry);
        }
    }

    dictionary_entries.sort_by(|left, right| left.normalized_name.cmp(&right.normalized_name));
    dictionary_entries.dedup_by(|left, right| left.normalized_name == right.normalized_name);

    let dictionary_len = dictionary_entries.len();
    let app_state = app.state::<AppState>();
    let mut dictionary = app_state
        .ocr_dictionary
        .lock()
        .map_err(|_| "failed to store OCR dictionary".to_string())?;
    *dictionary = dictionary_entries;

    Ok(dictionary_len)
}

pub fn load_tradeable_item_prices<R: Runtime>(app: &AppHandle<R>) -> Result<usize, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(OCR_DICTIONARY_HTTP_TIMEOUT_SECS))
        .build()
        .map_err(|err| format!("failed to build tradeable item client: {err}"))?;

    let response = client
        .get(TRADEABLE_ITEMS_API_URL)
        .send()
        .map_err(|err| format!("failed to fetch tradeable items: {err}"))?
        .error_for_status()
        .map_err(|err| format!("tradeable items request failed: {err}"))?;

    let payload: TradeableItemsApiResponse = response
        .json()
        .map_err(|err| format!("failed to parse tradeable items response: {err}"))?;

    let mut prices_by_slug = HashMap::new();

    for item in payload.tradeable_items {
        let slug = item.slug.trim();
        if slug.is_empty() {
            continue;
        }

        let statistics_today_median = item
            .statistics_today
            .first()
            .and_then(|entry| entry.median)
            .filter(|median| median.is_finite());

        let current_offers_median = item
            .current_offers
            .first()
            .and_then(|entry| entry.median)
            .filter(|median| median.is_finite());

        let trades_24h = item
            .statistics_today
            .first()
            .and_then(|entry| entry.volume)
            .filter(|value| value.is_finite());
        let moving_avg = item
            .statistics_today
            .first()
            .and_then(|entry| entry.moving_avg)
            .filter(|value| value.is_finite());
        let ducats = item.ducats;

        let Some((median, used_fallback)) = statistics_today_median
            .map(|value| (value, false))
            .or_else(|| current_offers_median.map(|value| (value, true)))
        else {
            continue;
        };

        prices_by_slug.insert(
            slug.to_string(),
            TradeablePriceEntry {
                median,
                used_current_offer_fallback: used_fallback,
                trades_24h,
                moving_avg,
                ducats,
            },
        );
    }

    let count = prices_by_slug.len();
    let app_state = app.state::<AppState>();
    let mut prices_guard = app_state
        .ocr_tradeable_prices
        .lock()
        .map_err(|_| "failed to store tradeable prices".to_string())?;
    *prices_guard = prices_by_slug;

    Ok(count)
}

pub fn map_words_to_dictionary<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
    threshold: f64,
) -> Vec<OcrWord> {
    if words.is_empty() {
        return vec![];
    }

    let app_state = app.state::<AppState>();
    let dictionary_guard = match app_state.ocr_dictionary.lock() {
        Ok(guard) => guard,
        Err(_) => return words.to_vec(),
    };

    if dictionary_guard.is_empty() {
        return words.to_vec();
    }

    let needs_price_reload = app_state
        .ocr_tradeable_prices
        .lock()
        .map(|prices| prices.is_empty())
        .unwrap_or(false);

    if needs_price_reload {
        match load_tradeable_item_prices(app) {
            Ok(count) => println!("[ocr] lazy-loaded tradeable item prices: {count}"),
            Err(err) => eprintln!("[ocr] failed to lazy-load tradeable item prices: {err}"),
        }
    }

    let prices_guard = app_state.ocr_tradeable_prices.lock().ok();
    let prices_by_slug = prices_guard.as_deref();

    words
        .iter()
        .filter_map(|word| {
            map_word_to_dictionary(word, &dictionary_guard, prices_by_slug, threshold)
        })
        .collect()
}

pub fn map_word_to_dictionary(
    word: &OcrWord,
    dictionary: &[OcrDictionaryEntry],
    prices_by_slug: Option<&HashMap<String, TradeablePriceEntry>>,
    threshold: f64,
) -> Option<OcrWord> {
    let normalized_word = normalize_dictionary_text(&word.text);
    if normalized_word.is_empty() {
        return None;
    }
    let normalized_tokens = normalized_word.split_whitespace().collect::<Vec<_>>();

    let mut best_match: Option<(&OcrDictionaryEntry, f64)> = None;

    for candidate in dictionary {
        let tag_bonus = candidate
            .tags
            .iter()
            .filter_map(|tag| {
                let normalized_tag = normalize_dictionary_text(tag);
                if normalized_tag.is_empty() {
                    None
                } else {
                    Some(normalized_tag)
                }
            })
            .filter(|normalized_tag| {
                normalized_tokens
                    .iter()
                    .any(|token| *token == normalized_tag.as_str())
            })
            .count() as f64
            * 0.02;

        let score = (dictionary_similarity_score(&normalized_word, &candidate.normalized_name)
            + tag_bonus)
            .min(1.0);
        match best_match {
            Some((_, best_score)) if score <= best_score => {}
            _ => best_match = Some((candidate, score)),
        }
    }

    match best_match {
        Some((candidate, score)) if score >= threshold => {
            let mut mapped = word.clone();
            mapped.text = candidate.name.clone();
            mapped.slug = Some(candidate.slug.clone());
            mapped.mapping_confidence = Some(score);
            mapped.ducats = candidate.ducats;
            mapped.vaulted = candidate.vaulted;
            mapped.is_custom = Some(candidate.is_custom);

            if let Some(prices_lookup) = prices_by_slug {
                if let Some(price_entry) = prices_lookup.get(&candidate.slug) {
                    mapped.market_median = Some(price_entry.median);
                    mapped.market_median_from_current_offers =
                        Some(price_entry.used_current_offer_fallback);
                    if mapped.ducats.is_none() {
                        mapped.ducats = price_entry.ducats;
                    }
                    mapped.trades_24h = price_entry.trades_24h;
                    mapped.moving_avg = price_entry.moving_avg;
                }
            }

            Some(mapped)
        }
        _ => None,
    }
}

pub fn dictionary_similarity_score(left: &str, right: &str) -> f64 {
    if left == right {
        return 1.0;
    }

    let max_len = left.len().max(right.len());
    if max_len == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(left.as_bytes(), right.as_bytes());
    let levenshtein_score = 1.0 - distance as f64 / max_len as f64;
    let overlap_score = token_overlap_score(left, right);
    (levenshtein_score * 0.85 + overlap_score * 0.15).clamp(0.0, 1.0)
}

pub fn token_overlap_score(left: &str, right: &str) -> f64 {
    let left_tokens = left.split_whitespace().collect::<Vec<_>>();
    let right_tokens = right.split_whitespace().collect::<Vec<_>>();
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }

    let shared_count = left_tokens
        .iter()
        .filter(|token| right_tokens.contains(token))
        .count();
    shared_count as f64 / left_tokens.len().max(right_tokens.len()) as f64
}

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

pub fn build_custom_dictionary_entry(name: &str) -> Option<OcrDictionaryEntry> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized_name = normalize_dictionary_text(trimmed);
    if normalized_name.is_empty() {
        return None;
    }

    Some(OcrDictionaryEntry {
        name: trimmed.to_string(),
        slug: normalized_name.replace(' ', "_"),
        tags: Vec::new(),
        normalized_name,
        ducats: None,
        vaulted: None,
        is_custom: true,
    })
}

pub fn levenshtein_distance(left: &[u8], right: &[u8]) -> usize {
    if left.is_empty() {
        return right.len();
    }
    if right.is_empty() {
        return left.len();
    }

    let mut previous_row: Vec<usize> = (0..=right.len()).collect();
    let mut current_row = vec![0usize; right.len() + 1];

    for (left_index, left_byte) in left.iter().enumerate() {
        current_row[0] = left_index + 1;

        for (right_index, right_byte) in right.iter().enumerate() {
            let substitution_cost = if left_byte == right_byte { 0 } else { 1 };
            let delete_cost = previous_row[right_index + 1] + 1;
            let insert_cost = current_row[right_index] + 1;
            let substitute_cost = previous_row[right_index] + substitution_cost;

            current_row[right_index + 1] = delete_cost.min(insert_cost).min(substitute_cost);
        }

        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[right.len()]
}
