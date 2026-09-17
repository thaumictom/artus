//! Remote dictionary fetching, fuzzy matching, and tradeable-item price lookups.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use log::{info, warn};
use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};

use super::{
    OcrThemeOption, OcrWord, CUSTOM_OCR_DICTIONARY_ITEMS, OCR_DICTIONARY_API_URL,
    OCR_DICTIONARY_HTTP_TIMEOUT_SECS, THEME_COLORS_TOML, TRADEABLE_ITEMS_API_URL,
};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

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
    pub is_relic: bool,
    pub subtype: Option<String>,
    pub set_slug: Option<String>,
    pub set_ducats: Option<u64>,
    pub max_rank: Option<u64>,
}

pub struct TradeablePriceEntry {
    pub median: f64,
    pub used_current_offer_fallback: bool,
    pub relic_price_is_fallback: bool,
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
    items: Vec<DictionaryApiItem>,
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
    #[serde(default)]
    set_slug: Option<String>,
    #[serde(default, rename = "maxRank")]
    max_rank: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemsApiResponse {
    #[serde(default)]
    items: Vec<TradeableItemApiItem>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemApiItem {
    slug: String,
    #[serde(default, deserialize_with = "deserialize_statistics")]
    statistics_today: Vec<TradeableItemStats>,
    #[serde(default, deserialize_with = "deserialize_statistics")]
    statistics_live: Vec<TradeableItemStats>,
    #[serde(default)]
    ducats: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemStats {
    median: Option<f64>,
    volume: Option<f64>,
    moving_avg: Option<f64>,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    mod_rank: Option<u64>,
}

// ── HTTP helper ───────────────────────────────────────────────────────────────

// Feeds and individual variants may be null when there are no trades.
fn deserialize_statistics<'de, D>(deserializer: D) -> Result<Vec<TradeableItemStats>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let entries = Option::<Vec<Option<TradeableItemStats>>>::deserialize(deserializer)?;
    Ok(entries.unwrap_or_default().into_iter().flatten().collect())
}

fn resolve_price(
    stats: Option<&TradeableItemStats>,
    live: Option<&TradeableItemStats>,
    ducats: Option<u64>,
) -> Option<TradeablePriceEntry> {
    let (median, used_current_offer_fallback) = stats
        .and_then(|s| s.median)
        .filter(|v| v.is_finite())
        .map(|v| (v, false))
        .or_else(|| {
            live.and_then(|s| s.median)
                .filter(|v| v.is_finite())
                .map(|v| (v, true))
        })?;
    Some(TradeablePriceEntry {
        median,
        used_current_offer_fallback,
        relic_price_is_fallback: false,
        trades_24h: stats.and_then(|s| s.volume).filter(|v| v.is_finite()),
        moving_avg: stats.and_then(|s| s.moving_avg).filter(|v| v.is_finite()),
        ducats,
    })
}

impl TradeablePriceEntry {
    fn display_price(&self) -> f64 {
        if self.used_current_offer_fallback {
            self.median
        } else {
            self.moving_avg.unwrap_or(self.median)
        }
    }
}

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

    // Set entries carry the total ducats, but are excluded from OCR matching below.
    let set_ducats: HashMap<String, u64> = payload
        .items
        .iter()
        .filter(|item| item.tags.iter().any(|tag| tag == "set"))
        .filter_map(|item| item.ducats.map(|ducats| (item.slug.clone(), ducats)))
        .collect();

    let mut entries: Vec<OcrDictionaryEntry> = payload
        .items
        .into_iter()
        .flat_map(|item| {
            let name = item.name.trim();
            let slug = item.slug.trim();
            if name.is_empty() || slug.is_empty() {
                return vec![];
            }

            // Remove sets from dictionary, not a valid item to trade
            if item.tags.contains(&"set".to_string()) {
                return vec![];
            }

            let mut mapped_entries = vec![];

            if (name.ends_with("Relic") || item.tags.contains(&"relic".to_string()))
                && !item.tags.contains(&"requiem".to_string())
            {
                // Helper to create the 4 different relic subtype entries
                let mut add_relic = |suffix: &str, slug_suffix: &str, subtype: &str| {
                    let entry_name = if suffix.is_empty() {
                        name.to_string()
                    } else {
                        format!("{} {}", name, suffix)
                    };

                    let normalized = normalize_dictionary_text(&entry_name);
                    if !normalized.is_empty() {
                        mapped_entries.push(OcrDictionaryEntry {
                            name: entry_name,
                            slug: format!("{}_{}", slug, slug_suffix),
                            tags: item.tags.clone(),
                            normalized_name: normalized,
                            ducats: item.ducats,
                            vaulted: item.vaulted,
                            is_custom: false,
                            is_relic: true,
                            subtype: Some(subtype.to_string()),
                            set_slug: None,
                            set_ducats: None,
                            max_rank: None,
                        });
                    }
                };

                add_relic("", "intact", "Intact");
                add_relic("[Exceptional]", "intact", "Exceptional");
                add_relic("[Flawless]", "intact", "Flawless");
                add_relic("[Radiant]", "radiant", "Radiant");
            } else {
                let normalized_name = normalize_dictionary_text(name);
                if !normalized_name.is_empty() {
                    mapped_entries.push(OcrDictionaryEntry {
                        name: name.to_string(),
                        slug: slug.to_string(),
                        tags: item.tags,
                        normalized_name,
                        ducats: item.ducats,
                        vaulted: item.vaulted,
                        is_custom: false,
                        is_relic: false,
                        subtype: None,
                        set_ducats: item
                            .set_slug
                            .as_ref()
                            .and_then(|slug| set_ducats.get(slug))
                            .copied(),
                        set_slug: item.set_slug,
                        max_rank: item.max_rank,
                    });
                }
            }

            mapped_entries
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

    let prices = build_tradeable_prices(payload);
    let count = prices.len();
    *app.state::<AppState>().ocr_tradeable_prices.lock()? = prices;
    Ok(count)
}

fn build_tradeable_prices(
    payload: TradeableItemsApiResponse,
) -> HashMap<String, TradeablePriceEntry> {
    let mut prices = HashMap::new();

    for item in payload.items {
        let slug = item.slug.trim();
        if slug.is_empty() {
            continue;
        }

        if slug.ends_with("_relic") && !slug.contains("requiem") {
            let get_stat = |subtype_target: &str| -> Option<&TradeableItemStats> {
                item.statistics_today
                    .iter()
                    .find(|s| s.subtype.as_deref() == Some(subtype_target))
            };

            let get_offer = |subtype_target: &str| -> Option<&TradeableItemStats> {
                item.statistics_live
                    .iter()
                    .find(|s| s.subtype.as_deref() == Some(subtype_target))
            };

            let intact_stats = get_stat("intact");
            let intact_offers = get_offer("intact");
            let radiant_stats = get_stat("radiant");
            let radiant_offers = get_offer("radiant");

            // Resolve Intact prices
            let intact_median_data = intact_stats
                .and_then(|s| s.median)
                .filter(|m| m.is_finite())
                .map(|m| (m, false, false)) // (median, used_current_offer_fallback, used_subtype_fallback)
                .or_else(|| {
                    intact_offers
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, true, false))
                })
                .or_else(|| {
                    radiant_stats
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, false, true))
                })
                .or_else(|| {
                    radiant_offers
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, true, true))
                });

            if let Some((median, used_offer, used_subtype)) = intact_median_data {
                let source_stat = if !used_subtype {
                    intact_stats
                } else {
                    radiant_stats
                };
                prices.insert(
                    format!("{}_intact", slug),
                    TradeablePriceEntry {
                        median,
                        used_current_offer_fallback: used_offer,
                        relic_price_is_fallback: used_subtype,
                        trades_24h: source_stat.and_then(|s| s.volume).filter(|v| v.is_finite()),
                        moving_avg: source_stat
                            .and_then(|s| s.moving_avg)
                            .filter(|v| v.is_finite()),
                        ducats: item.ducats,
                    },
                );
            }

            // Resolve Radiant prices
            let radiant_median_data = radiant_stats
                .and_then(|s| s.median)
                .filter(|m| m.is_finite())
                .map(|m| (m, false, false))
                .or_else(|| {
                    radiant_offers
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, true, false))
                })
                .or_else(|| {
                    intact_stats
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, false, true))
                })
                .or_else(|| {
                    intact_offers
                        .and_then(|s| s.median)
                        .filter(|m| m.is_finite())
                        .map(|m| (m, true, true))
                });

            if let Some((median, used_offer, used_subtype)) = radiant_median_data {
                let source_stat = if !used_subtype {
                    radiant_stats
                } else {
                    intact_stats
                };
                prices.insert(
                    format!("{}_radiant", slug),
                    TradeablePriceEntry {
                        median,
                        used_current_offer_fallback: used_offer,
                        relic_price_is_fallback: used_subtype,
                        trades_24h: source_stat.and_then(|s| s.volume).filter(|v| v.is_finite()),
                        moving_avg: source_stat
                            .and_then(|s| s.moving_avg)
                            .filter(|v| v.is_finite()),
                        ducats: item.ducats,
                    },
                );
            }
        } else {
            // Rank order is not stable. Cache each variant independently.
            let ranks: std::collections::BTreeSet<_> = item
                .statistics_today
                .iter()
                .chain(&item.statistics_live)
                .filter_map(|s| s.mod_rank)
                .collect();
            for rank in &ranks {
                let stats = item
                    .statistics_today
                    .iter()
                    .find(|s| s.mod_rank == Some(*rank));
                let live = item
                    .statistics_live
                    .iter()
                    .find(|s| s.mod_rank == Some(*rank));
                if let Some(price) = resolve_price(stats, live, item.ducats) {
                    prices.insert(format!("{slug}_rank_{rank}"), price);
                }
            }
            let base_rank = if ranks.is_empty() { None } else { Some(0) };
            let stats_today = item
                .statistics_today
                .iter()
                .find(|s| s.mod_rank == base_rank);
            let offers = item
                .statistics_live
                .iter()
                .find(|s| s.mod_rank == base_rank);

            let stats_median = stats_today.and_then(|s| s.median).filter(|m| m.is_finite());
            let offers_median = offers.and_then(|s| s.median).filter(|m| m.is_finite());

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
                    relic_price_is_fallback: false,
                    trades_24h: stats_today.and_then(|s| s.volume).filter(|v| v.is_finite()),
                    moving_avg: stats_today
                        .and_then(|s| s.moving_avg)
                        .filter(|v| v.is_finite()),
                    ducats: item.ducats,
                },
            );
        }
    }

    prices
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
    let best = dictionary
        .iter()
        .map(|candidate| {
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

            let score =
                (similarity_score(&normalized, &candidate.normalized_name) + tag_bonus).min(1.0);
            (candidate, score)
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;

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
    mapped.is_relic = Some(candidate.is_relic);
    // Dictionary tags identify mods regardless of the OCR color filter used.
    mapped.is_mod = Some(candidate.tags.iter().any(|tag| tag == "mod"));
    mapped.subtype = candidate.subtype.clone();

    // Enrich with price data
    if let Some(prices_map) = prices {
        let base_key = candidate
            .max_rank
            .map(|_| format!("{}_rank_0", candidate.slug));
        if let Some(price) = prices_map.get(base_key.as_ref().unwrap_or(&candidate.slug)) {
            mapped.market_median = Some(price.median);
            mapped.market_median_from_current_offers = Some(price.used_current_offer_fallback);
            mapped.relic_price_is_fallback = Some(price.relic_price_is_fallback);
            if mapped.ducats.is_none() {
                mapped.ducats = price.ducats;
            }
            mapped.trades_24h = price.trades_24h;
            mapped.moving_avg = price.moving_avg;
        }
        if let Some(price) = candidate
            .max_rank
            .filter(|rank| *rank > 0)
            .and_then(|rank| prices_map.get(&format!("{}_rank_{rank}", candidate.slug)))
        {
            // Keep the prepared overlay field names for mods as well as arcanes.
            mapped.maxed_arcane_price = Some(price.display_price());
            mapped.maxed_arcane_trades_24h = price.trades_24h;
            mapped.maxed_arcane_price_from_current_offers = Some(price.used_current_offer_fallback);
        }
        if let Some(price) = candidate
            .set_slug
            .as_ref()
            .and_then(|slug| prices_map.get(slug))
        {
            mapped.prime_set_price = Some(price.display_price());
            mapped.prime_set_trades_24h = price.trades_24h;
            mapped.prime_set_price_from_current_offers = Some(price.used_current_offer_fallback);
            mapped.prime_set_ducats = candidate.set_ducats.or(price.ducats);
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
        is_relic: false,
        subtype: None,
        set_slug: None,
        set_ducats: None,
        max_rank: None,
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
            curr[ri + 1] = (prev[ri + 1] + 1).min(curr[ri] + 1).min(prev[ri] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}
