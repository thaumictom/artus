use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Deserialize;
use tauri::{AppHandle, Runtime};

use crate::dictionary;

const EMBEDDED_MARKET_STATISTICS_JSON: &str =
    include_str!("../data/warframe_market_statistics.json");

#[derive(Debug, Deserialize)]
struct MarketStatisticsRoot {
    #[serde(default)]
    item_statistics: Vec<MarketStatisticsItem>,
}

#[derive(Debug, Deserialize)]
struct MarketStatisticsItem {
    item: String,
    #[serde(default)]
    statistics_today: MarketMedianContainer,
    #[serde(default)]
    current_offers: MarketMedianContainer,
}

#[derive(Debug, Default, Deserialize)]
struct MarketMedianContainer {
    median: Option<f64>,
}

static MARKET_PRICES: OnceLock<Result<HashMap<String, String>, String>> = OnceLock::new();
static DICTIONARY_NAME_TO_SLUG: OnceLock<Result<HashMap<String, String>, String>> = OnceLock::new();

pub fn lookup_price_for_name<R: Runtime>(app: &AppHandle<R>, name: &str) -> Option<String> {
    let prices = match load_market_prices() {
        Ok(prices) => prices,
        Err(err) => {
            eprintln!("[market_prices] failed to load market prices: {err}");
            return None;
        }
    };

    let name_to_slug = match load_name_to_slug_map(app) {
        Ok(name_to_slug) => name_to_slug,
        Err(err) => {
            eprintln!("[market_prices] failed to load dictionary cache: {err}");
            return None;
        }
    };

    let normalized_name = normalize_lookup_name(name);
    let slug = name_to_slug.get(&normalized_name)?;
    prices.get(slug).cloned()
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

fn load_market_prices() -> Result<&'static HashMap<String, String>, String> {
    let prices = MARKET_PRICES.get_or_init(|| {
        let parsed: MarketStatisticsRoot = serde_json::from_str(EMBEDDED_MARKET_STATISTICS_JSON)
            .map_err(|err| format!("failed to parse embedded market statistics JSON: {err}"))?;

        let mut prices = HashMap::new();
        for entry in parsed.item_statistics {
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
        }

        Ok(prices)
    });

    match prices {
        Ok(value) => Ok(value),
        Err(err) => Err(err.clone()),
    }
}

fn normalize_lookup_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn format_market_median(median: f64) -> String {
    if median.fract() == 0.0 {
        format!("{}", median as i64)
    } else {
        format!("{median:.1}")
    }
}
