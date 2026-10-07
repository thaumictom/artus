//! Rotation-aware wiki snapshots, persisted across app restarts.

use std::{collections::HashMap, path::Path, time::Duration};

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};

const DAY: i64 = 86_400_000;
const RETRY_INTERVAL: i64 = 5 * 60_000;

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiOfferings {
    items: Vec<Offering>,
    fetched_at: Option<i64>,
    attempted_at: Option<i64>,
    observed_at: Option<i64>,
    page_updated_at: Option<i64>,
    reported_batch: Option<String>,
    batches: HashMap<String, Vec<String>>,
    error: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Offering {
    name: String,
    element: Option<String>,
    bonus: Option<f64>,
}

fn rotation(source: &str, now: i64) -> (i64, &'static str) {
    // Keep these UTC rotation anchors aligned with src/lib/wiki-offerings.ts.
    let (year, month, day, days) = match source {
        "tenet" => (2015, 12, 3, 4),
        "coda" => (2025, 3, 18, 4),
        "acrithis" => (1970, 1, 5, 7),
        _ => unreachable!("validated wiki source"),
    };
    let seed = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp_millis();
    let cycle = (now - seed).div_euclid(days * DAY);
    (
        seed + cycle * days * DAY,
        if cycle.rem_euclid(2) == 0 { "A" } else { "B" },
    )
}

fn current_report(offerings: &WikiOfferings, source: &str, now: i64) -> bool {
    let (start, batch) = rotation(source, now);
    !offerings.items.is_empty()
        && offerings.fetched_at.is_some_and(|time| time >= start)
        && match source {
            "acrithis" => offerings
                .observed_at
                .is_some_and(|time| time >= start && time <= now),
            "coda" => offerings.reported_batch.as_deref() == Some(batch),
            _ => true,
        }
}

fn save_offerings(path: &Path, offerings: &WikiOfferings) -> AppResult<()> {
    // Replace only after the full snapshot is written, preserving the saved report on interruption.
    let temporary_path = path.with_extension("json.tmp");
    std::fs::write(&temporary_path, serde_json::to_vec(offerings)?)?;
    std::fs::rename(temporary_path, path)?;
    Ok(())
}

fn selector(value: &str) -> Selector {
    Selector::parse(value).expect("static wiki selector")
}

fn text(node: ElementRef<'_>) -> String {
    node.text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn utc_date(value: &str) -> Option<i64> {
    NaiveDate::parse_from_str(value.trim(), "%B %d, %Y")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|date| Utc.from_utc_datetime(&date).timestamp_millis())
}

fn parse_offerings(html: &str, source: &str) -> AppResult<WikiOfferings> {
    let document = Html::parse_document(html);
    let mut result = WikiOfferings::default();
    // Page edit time is useful context, but does not prove the transcluded reports are current.
    if let Some(footer) = document.select(&selector("#footer-info-lastmod")).next() {
        let footer = text(footer);
        if let Some(date) = footer.strip_prefix("This page was last edited on ") {
            result.page_updated_at =
                NaiveDateTime::parse_from_str(date.trim_end_matches('.'), "%d %B %Y, at %H:%M")
                    .ok()
                    .map(|date| Utc.from_utc_datetime(&date).timestamp_millis());
        }
    }

    if source == "acrithis" {
        for paragraph in document.select(&selector(".mw-parser-output > p")) {
            let paragraph = text(paragraph);
            let Some(report) = paragraph.strip_prefix("Currently reported (as of ") else {
                continue;
            };
            let Some((date, names)) = report.split_once("):") else {
                continue;
            };
            result.observed_at = utc_date(date);
            result.items = names
                .split(',')
                .map(|name| Offering {
                    name: name.trim().to_owned(),
                    element: None,
                    bonus: None,
                })
                .collect();
            if result.items.len() != 5 || result.items.iter().any(|item| item.name.is_empty()) {
                return Err(AppError::msg(
                    "The wiki's five reported Acrithis offerings could not be read.",
                ));
            }
            return Ok(result);
        }
        return Err(AppError::msg("The wiki's Acrithis report was not found."));
    }

    let prefix = if source == "tenet" { "Tenet " } else { "Coda " };
    let weapon_selector = selector("[data-param-source='Weapons'][data-param-name]");
    for table in document.select(&selector(".mw-parser-output table.wikitable")) {
        let headers: Vec<_> = table.select(&selector("th")).map(text).collect();
        if source == "coda" && headers.first().is_some_and(|header| header == "Batch") {
            for row in table.select(&selector("tr")) {
                let cells: Vec<_> = row.select(&selector("th, td")).collect();
                let Some(first) = cells.first() else { continue };
                let batch = text(*first).trim_matches(['>', '<', ' ']).to_owned();
                if batch != "A" && batch != "B" {
                    continue;
                }
                let names = cells
                    .iter()
                    .skip(1)
                    .flat_map(|cell| cell.select(&weapon_selector))
                    .filter_map(|item| item.value().attr("data-param-name").map(str::to_owned))
                    .collect();
                result.batches.insert(batch, names);
            }
            continue;
        }
        if headers.len() != 3
            || !headers[0].starts_with("Weapon")
            || headers[1] != "Element"
            || headers[2] != "Bonus %"
        {
            continue;
        }
        result.reported_batch = if headers[0].contains("Batch A") {
            Some("A".into())
        } else if headers[0].contains("Batch B") {
            Some("B".into())
        } else {
            None
        };
        for row in table.select(&selector("tr")) {
            let cells: Vec<_> = row.select(&selector("td")).collect();
            if cells.is_empty() {
                continue;
            }
            if cells.len() != 3 {
                return Err(AppError::msg("Unexpected wiki weapon table layout."));
            }
            let name = text(cells[0]);
            let element = text(cells[1]);
            let bonus_text = text(cells[2]);
            // Unreported values remain missing rather than being turned into zero bonuses.
            let bonus = bonus_text
                .trim_end_matches('%')
                .parse::<f64>()
                .ok()
                .filter(|value| value.is_finite() && (25.0..=60.0).contains(value));
            if !name.starts_with(prefix) && !(source == "coda" && name.starts_with("Dual Coda ")) {
                return Err(AppError::msg("Unexpected weapon in wiki offering table."));
            }
            let element = [
                "Impact",
                "Heat",
                "Cold",
                "Electricity",
                "Toxin",
                "Magnetic",
                "Radiation",
            ]
            .contains(&element.as_str())
            .then_some(element);
            result.items.push(Offering {
                name,
                element,
                bonus,
            });
        }
        if !result.items.is_empty() {
            return Ok(result);
        }
    }
    Err(AppError::msg(
        "The wiki's current weapon offering table was not found.",
    ))
}

#[tauri::command]
pub async fn get_wiki_offerings(app: AppHandle, source: String) -> AppResult<WikiOfferings> {
    fetch_offerings(&app, source, false).await
}

/// Maintenance explicitly refreshes all three reports, even within the retry interval.
pub async fn refresh_offerings(app: &AppHandle) -> AppResult<()> {
    let mut errors = Vec::new();
    for source in ["tenet", "coda", "acrithis"] {
        match fetch_offerings(app, source.into(), true).await {
            Ok(result) => {
                if let Some(error) = result.error {
                    errors.push(format!("{source}: {error}"));
                }
            }
            Err(error) => errors.push(format!("{source}: {error}")),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::msg(errors.join("; ")))
    }
}

async fn fetch_offerings(app: &AppHandle, source: String, force: bool) -> AppResult<WikiOfferings> {
    let path = match source.as_str() {
        "tenet" => "Tenet_Weapons",
        "coda" => "Coda_Weapons",
        "acrithis" => "Acrithis/Current_Offerings",
        _ => return Err(AppError::msg("Unknown wiki offering source.")),
    };
    let state = app.state::<AppState>();
    let mut cache = state.wiki_offerings.lock().await;
    let now = Utc::now().timestamp_millis();
    let directory = app.path().app_data_dir().map_err(AppError::msg)?;
    let cache_path = directory.join(format!("wiki-offerings-{source}.json"));
    // Restore reports and the latest attempt time before deciding whether to refresh.
    if !cache.contains_key(&source) {
        match std::fs::read(&cache_path) {
            Ok(bytes) => {
                cache.insert(source.clone(), serde_json::from_slice(&bytes)?);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    if !force {
        if let Some(saved) = cache.get(&source) {
            if current_report(saved, &source, now)
                || saved
                    .attempted_at
                    .is_some_and(|attempt| now - attempt < RETRY_INTERVAL)
            {
                return Ok(saved.clone());
            }
        }
    }

    std::fs::create_dir_all(directory)?;
    let fetched = async {
        let html = state
            .http_client
            .get(format!("https://wiki.warframe.com/w/{path}"))
            .timeout(Duration::from_secs(30))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        parse_offerings(&html, &source)
    }
    .await;
    let mut result = match fetched {
        Ok(mut result) => {
            result.fetched_at = Some(Utc::now().timestamp_millis());
            result.attempted_at = Some(now);
            result
        }
        Err(error) => {
            let mut result = cache.get(&source).cloned().unwrap_or_default();
            result.attempted_at = Some(now);
            result.error = Some(format!(
                "Could not read wiki offerings: {error}. Automatic retries are limited to once every five minutes."
            ));
            result
        }
    };
    // Failed refreshes retain the last successful report and persist the failure details as well.
    if let Err(error) = save_offerings(&cache_path, &result) {
        let message = format!("Could not save wiki offerings locally: {error}.");
        result.error = Some(match result.error.take() {
            Some(previous) => format!("{previous} {message}"),
            None => message,
        });
    }
    cache.insert(source, result.clone());
    Ok(result)
}
