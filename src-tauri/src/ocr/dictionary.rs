//! Remote dictionary fetching, fuzzy matching, and tradeable-item price lookups.

use std::collections::{BTreeMap, HashMap};
use std::time::Instant;

use log::info;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

use super::{
    OcrThemeOption, OcrWord, CUSTOM_OCR_DICTIONARY_ITEMS, THEME_COLORS_TOML,
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
    pub normalized_tags: Vec<String>,
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
    pub today_median: Option<f64>,
    pub weighted_avg: Option<f64>,
    pub today_volume: Option<f64>,
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
    #[serde(default)]
    highlight: BTreeMap<String, [u8; 3]>,
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
    #[serde(default, alias = "statistics_Today", deserialize_with = "deserialize_statistics")]
    statistics_today: Vec<TradeableItemStats>,
    #[serde(default, deserialize_with = "deserialize_statistics")]
    statistics_yesterday: Vec<TradeableItemStats>,
    #[serde(default, deserialize_with = "deserialize_statistics")]
    statistics_live: Vec<TradeableItemStats>,
    #[serde(default)]
    ducats: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TradeableItemStats {
    median: Option<f64>,
    wa_price: Option<f64>,
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

fn traded_stat(stats: Option<&TradeableItemStats>) -> Option<&TradeableItemStats> {
    stats.filter(|s| s.volume.is_some_and(|volume| volume.is_finite() && volume > 0.0))
}

fn positive_price(value: Option<f64>) -> Option<f64> {
    value.filter(|v| v.is_finite() && *v > 0.0)
}

fn valid_median(stats: Option<&TradeableItemStats>) -> Option<f64> {
    positive_price(stats.and_then(|s| s.median))
}

fn recent_price<'a>(
    today: Option<&'a TradeableItemStats>,
    yesterday: Option<&'a TradeableItemStats>,
    live: Option<&TradeableItemStats>,
) -> Option<(f64, bool, &'a TradeableItemStats)> {
    let today = traded_stat(today);
    let yesterday = traded_stat(yesterday);
    let source = today.or(yesterday)?;
    today.and_then(|stats| valid_median(Some(stats)).map(|median| (median, false, stats)))
        .or_else(|| yesterday.and_then(|stats| valid_median(Some(stats)).map(|median| (median, false, stats))))
        .or_else(|| valid_median(live).map(|median| (median, true, source)))
}

fn relic_price(
    median: f64,
    used_offer: bool,
    used_subtype: bool,
    source: &TradeableItemStats,
    target_today: Option<&TradeableItemStats>,
    source_today: Option<&TradeableItemStats>,
    ducats: Option<u64>,
) -> TradeablePriceEntry {
    let target_today = traded_stat(target_today);
    TradeablePriceEntry {
        median,
        today_median: valid_median(target_today),
        weighted_avg: positive_price(target_today.and_then(|s| s.wa_price)),
        today_volume: target_today.and_then(|s| s.volume),
        used_current_offer_fallback: used_offer,
        relic_price_is_fallback: used_subtype,
        trades_24h: traded_stat(source_today).and_then(|s| s.volume),
        moving_avg: positive_price(source.moving_avg),
        ducats,
    }
}

fn resolve_price(
    today: Option<&TradeableItemStats>,
    yesterday: Option<&TradeableItemStats>,
    live: Option<&TradeableItemStats>,
    ducats: Option<u64>,
) -> Option<TradeablePriceEntry> {
    let (median, used_current_offer_fallback, source) = recent_price(today, yesterday, live)?;
    let today = traded_stat(today);
    Some(TradeablePriceEntry {
        median,
        today_median: valid_median(today),
        weighted_avg: positive_price(today.and_then(|s| s.wa_price)),
        today_volume: today.and_then(|s| s.volume),
        used_current_offer_fallback,
        relic_price_is_fallback: false,
        trades_24h: today.and_then(|s| s.volume),
        moving_avg: positive_price(source.moving_avg),
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

// ── Theme loading ─────────────────────────────────────────────────────────────

/// Parses the embedded `theme_colors.toml` and returns the available themes.
pub fn load_primary_theme_options<R: Runtime>(
    _app: &AppHandle<R>,
) -> AppResult<Vec<OcrThemeOption>> {
    let parsed: ThemeColorsToml = toml::from_str(THEME_COLORS_TOML)?;

    let themes: Vec<OcrThemeOption> = parsed
        .primary
        .into_iter()
        .map(|(name, rgb)| {
            let highlight_rgb = parsed.highlight.get(&name).copied().ok_or_else(|| {
                AppError::msg(format!("missing highlight color for theme '{name}'"))
            })?;
            Ok(OcrThemeOption {
                name,
                rgb,
                highlight_rgb,
            })
        })
        .collect::<AppResult<_>>()?;

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
pub fn load_ocr_dictionary<R: Runtime>(app: &AppHandle<R>, value: Value) -> AppResult<usize> {
    let payload: DictionaryApiResponse = serde_json::from_value(value)?;

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
                            normalized_tags: Vec::new(),
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
                        normalized_tags: Vec::new(),
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

    for entry in &mut entries {
        entry.normalized_tags = entry
            .tags
            .iter()
            .map(|tag| normalize_dictionary_text(tag))
            .collect();
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
pub fn load_tradeable_item_prices<R: Runtime>(app: &AppHandle<R>, value: Value) -> AppResult<usize> {
    let payload: TradeableItemsApiResponse = serde_json::from_value(value)?;

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
            let get_today = |subtype_target: &str| -> Option<&TradeableItemStats> {
                item.statistics_today
                    .iter()
                    .find(|s| s.subtype.as_deref() == Some(subtype_target))
            };

            let get_yesterday = |subtype_target: &str| -> Option<&TradeableItemStats> {
                item.statistics_yesterday
                    .iter()
                    .find(|s| s.subtype.as_deref() == Some(subtype_target))
            };

            let get_offer = |subtype_target: &str| -> Option<&TradeableItemStats> {
                item.statistics_live
                    .iter()
                    .find(|s| s.subtype.as_deref() == Some(subtype_target))
            };

            let intact_stats = get_today("intact");
            let intact_yesterday = get_yesterday("intact");
            let intact_offers = get_offer("intact");
            let radiant_stats = get_today("radiant");
            let radiant_yesterday = get_yesterday("radiant");
            let radiant_offers = get_offer("radiant");

            let intact_median_data = recent_price(intact_stats, intact_yesterday, intact_offers)
                .map(|(median, used_offer, source)| (median, used_offer, false, source))
                .or_else(|| recent_price(radiant_stats, radiant_yesterday, radiant_offers)
                    .map(|(median, used_offer, source)| (median, used_offer, true, source)));

            if let Some((median, used_offer, used_subtype, source)) = intact_median_data {
                prices.insert(
                    format!("{}_intact", slug),
                    relic_price(median, used_offer, used_subtype, source, intact_stats,
                        if used_subtype { radiant_stats } else { intact_stats }, item.ducats),
                );
            }

            let radiant_median_data = recent_price(radiant_stats, radiant_yesterday, radiant_offers)
                .map(|(median, used_offer, source)| (median, used_offer, false, source))
                .or_else(|| recent_price(intact_stats, intact_yesterday, intact_offers)
                    .map(|(median, used_offer, source)| (median, used_offer, true, source)));

            if let Some((median, used_offer, used_subtype, source)) = radiant_median_data {
                prices.insert(
                    format!("{}_radiant", slug),
                    relic_price(median, used_offer, used_subtype, source, radiant_stats,
                        if used_subtype { intact_stats } else { radiant_stats }, item.ducats),
                );
            }
        } else {
            // Rank order is not stable. Cache each variant independently.
            let ranks: std::collections::BTreeSet<_> = item
                .statistics_today
                .iter()
                .chain(&item.statistics_yesterday)
                .chain(&item.statistics_live)
                .filter_map(|s| s.mod_rank)
                .collect();
            for rank in &ranks {
                let stats = item
                    .statistics_today
                    .iter()
                    .find(|s| s.mod_rank == Some(*rank));
                let yesterday = item
                    .statistics_yesterday
                    .iter()
                    .find(|s| s.mod_rank == Some(*rank));
                let live = item
                    .statistics_live
                    .iter()
                    .find(|s| s.mod_rank == Some(*rank));
                if let Some(price) = resolve_price(stats, yesterday, live, item.ducats) {
                    prices.insert(format!("{slug}_rank_{rank}"), price);
                }
            }
            let base_rank = if ranks.is_empty() { None } else { Some(0) };
            let stats_today = item
                .statistics_today
                .iter()
                .find(|s| s.mod_rank == base_rank);
            let stats_yesterday = item
                .statistics_yesterday
                .iter()
                .find(|s| s.mod_rank == base_rank);
            let offers = item
                .statistics_live
                .iter()
                .find(|s| s.mod_rank == base_rank);

            if let Some(price) = resolve_price(stats_today, stats_yesterday, offers, item.ducats) {
                prices.insert(slug.to_string(), price);
            }
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

    let prices = state.ocr_tradeable_prices.lock().ok();
    let prices_ref = prices.as_deref();
    let dictionary: &[OcrDictionaryEntry] = &dict;
    let start = Instant::now();
    let mapped = words
        .par_iter()
        .map(|word| match_single_word(word, dictionary, prices_ref, threshold))
        .collect::<Vec<_>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    info!(
        "dictionary matching: {:?} ({} words, {} entries)",
        start.elapsed(),
        words.len(),
        dictionary.len()
    );
    mapped
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

    let mut scratch = LevenshteinScratch::default();
    let mut best: Option<(&OcrDictionaryEntry, f64)> = None;
    for candidate in dictionary {
        // Bonus for tag overlap with OCR tokens.
        let tag_bonus = candidate
            .normalized_tags
            .iter()
            .filter(|tag| !tag.is_empty() && tokens.iter().any(|t| *t == tag.as_str()))
            .count() as f64
            * 0.02;
        let overlap = token_overlap_score(&normalized, &candidate.normalized_name);
        let max_len = normalized.len().max(candidate.normalized_name.len());
        let length_bound = 1.0
            - normalized.len().abs_diff(candidate.normalized_name.len()) as f64 / max_len as f64;
        let score_bound = (length_bound * 0.85 + overlap * 0.15 + tag_bonus).min(1.0);
        // Levenshtein distance cannot be smaller than the difference in lengths.
        if score_bound + 1e-12 < threshold.max(best.map_or(0.0, |(_, score)| score)) {
            continue;
        }
        let score = (similarity_score_with_overlap(
            &normalized,
            &candidate.normalized_name,
            overlap,
            &mut scratch,
        ) + tag_bonus)
            .min(1.0);
        // The previous max_by chose the last entry on equal scores.
        if best.is_none_or(|(_, best_score)| score >= best_score) {
            best = Some((candidate, score));
        }
    }

    let (candidate, score) = best?;
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
            mapped.wa_price = price.weighted_avg;
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
pub(super) fn similarity_score(left: &str, right: &str) -> f64 {
    let overlap = token_overlap_score(left, right);
    similarity_score_with_overlap(left, right, overlap, &mut LevenshteinScratch::default())
}

fn similarity_score_with_overlap(
    left: &str,
    right: &str,
    overlap: f64,
    scratch: &mut LevenshteinScratch,
) -> f64 {
    if left == right {
        return 1.0;
    }

    let max_len = left.len().max(right.len());
    if max_len == 0 {
        return 0.0;
    }

    let distance = scratch.distance(left.as_bytes(), right.as_bytes());
    let lev_score = 1.0 - distance as f64 / max_len as f64;
    (lev_score * 0.85 + overlap * 0.15).clamp(0.0, 1.0)
}

/// Fraction of tokens shared between two strings.
fn token_overlap_score(left: &str, right: &str) -> f64 {
    let left_tokens = left.split_whitespace();
    let left_count = left_tokens.clone().count();
    let right_count = right.split_whitespace().count();
    if left_count == 0 || right_count == 0 {
        return 0.0;
    }
    let shared = left_tokens
        .filter(|token| right.split_whitespace().any(|other| token == &other))
        .count();
    shared as f64 / left_count.max(right_count) as f64
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
        normalized_tags: Vec::new(),
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

/// Reuse the two DP rows across candidates for one OCR word.
#[derive(Default)]
struct LevenshteinScratch {
    prev: Vec<usize>,
    curr: Vec<usize>,
}

impl LevenshteinScratch {
    fn distance(&mut self, left: &[u8], right: &[u8]) -> usize {
        if left.is_empty() {
            return right.len();
        }
        if right.is_empty() {
            return left.len();
        }
        let row_len = right.len() + 1;
        self.prev.resize(row_len, 0);
        self.curr.resize(row_len, 0);
        for (index, value) in self.prev[..row_len].iter_mut().enumerate() {
            *value = index;
        }
        for (li, lb) in left.iter().enumerate() {
            self.curr[0] = li + 1;
            for (ri, rb) in right.iter().enumerate() {
                let cost = usize::from(lb != rb);
                self.curr[ri + 1] = (self.prev[ri + 1] + 1)
                    .min(self.curr[ri] + 1)
                    .min(self.prev[ri] + cost);
            }
            std::mem::swap(&mut self.prev, &mut self.curr);
        }
        self.prev[right.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimized_match_keeps_reference_ranking_and_threshold() {
        let mut entries = [
            "Gyre Prime",
            "Gyre Prime Chassis Blueprint",
            "Gyre Prime Systems Blueprint",
            "Gara Prime Chassis Blueprint",
            "Gyre Prime Neuroptics Blueprint",
            "Gyre Prime Set",
        ]
        .iter()
        .map(|name| build_custom_dictionary_entry(name).unwrap())
        .collect::<Vec<_>>();
        entries[1].tags = vec!["prime".into()];
        entries[1].normalized_tags = vec!["prime".into()];
        // Duplicate names exercise the original last-entry tie behavior.
        let mut duplicate = entries[1].clone();
        duplicate.slug = "last_equal_match".into();
        entries.push(duplicate);

        for text in [
            "Gyre Prime Chassis Blueprint",
            "Gyre Prirne Chassis Blueprint",
            "Gyre Prime Systems Blueprint",
            "Gara Prime Chassis Blueprint",
            "unrelated text",
        ] {
            let normalized = normalize_dictionary_text(text);
            let tokens = normalized.split_whitespace().collect::<Vec<_>>();
            let reference = entries
                .iter()
                .map(|entry| {
                    let tag_bonus = entry
                        .normalized_tags
                        .iter()
                        .filter(|tag| tokens.iter().any(|token| *token == tag.as_str()))
                        .count() as f64
                        * 0.02;
                    let score = (similarity_score(&normalized, &entry.normalized_name) + tag_bonus)
                        .min(1.0);
                    (entry, score)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            for threshold in [0.0, 0.62, 0.86, 1.0] {
                let expected = reference
                    .filter(|(_, score)| *score >= threshold)
                    .map(|(entry, score)| (entry.slug.as_str(), score));
                let word = OcrWord::new(text.into(), 0.0, 0.0, 0.0, 0.0);
                let actual = match_single_word(&word, &entries, None, threshold);
                assert_eq!(
                    actual.as_ref().and_then(|word| word.slug.as_deref()),
                    expected.map(|value| value.0)
                );
                if let (Some(actual), Some((_, expected_score))) = (actual, expected) {
                    assert!((actual.mapping_confidence.unwrap() - expected_score).abs() < 1e-12);
                }
            }
        }
    }
}
