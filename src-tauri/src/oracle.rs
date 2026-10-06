//! Shared, session-only Oracle rotations and the metadata needed to display them.

use std::{collections::HashMap, time::Duration};

use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{error::{AppError, AppResult}, state::AppState};

const RETRY: i64 = 5 * 60_000;
const METADATA_LIFETIME: i64 = 24 * 60 * 60_000;
const FACTIONS: [&str; 3] = ["ZarimanSyndicate", "EntratiLabSyndicate", "HexSyndicate"];

#[derive(Default)]
struct CachedDocument {
    data: Option<Value>,
    attempted_at: Option<i64>,
    expires_at: i64,
    error: Option<String>,
}

#[derive(Default)]
pub struct OracleCache {
    documents: HashMap<&'static str, CachedDocument>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OracleBounties {
    expiry: Option<i64>,
    bounties: HashMap<String, Vec<Bounty>>,
    error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Bounty {
    node: String,
    mission_type: String,
    challenge: String,
    objective: String,
    ally: Option<String>,
    faction: Option<String>,
}

fn nonempty(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|value| !value.trim().is_empty())
}

fn valid_rotation(data: &Value, now: i64) -> bool {
    data["expiry"].as_i64().is_some_and(|expiry| expiry > now)
        && matches!(data["zarimanFaction"].as_str(), Some("FC_CORPUS" | "FC_GRINEER"))
        && FACTIONS.iter().all(|faction| {
            data["bounties"][faction].as_array().is_some_and(|entries| {
                !entries.is_empty() && entries.iter().all(|entry| {
                    nonempty(entry.get("node")) && nonempty(entry.get("challenge"))
                        && entry.get("ally").is_none_or(|ally| nonempty(Some(ally)))
                })
            })
        })
}

async fn document(
    client: &reqwest::Client,
    cache: &mut OracleCache,
    source: &'static str,
    force: bool,
) -> AppResult<Value> {
    let saved = cache.documents.entry(source).or_default();
    let now = Utc::now().timestamp_millis();
    if !force {
        if saved.expires_at > now {
            if let Some(data) = &saved.data { return Ok(data.clone()); }
        }
        if saved.attempted_at.is_some_and(|attempt| now - attempt < RETRY) {
            return Err(AppError::msg(saved.error.clone().unwrap_or_else(||
                "Oracle data is awaiting the next retry.".into())));
        }
    }
    saved.attempted_at = Some(now);
    let url = if source == "rotation" {
        "https://oracle.browse.wf/bounty-cycle".to_owned()
    } else {
        format!("https://browse.wf/warframe-public-export-plus/{source}.json")
    };
    let fetched: AppResult<Value> = async {
        let data: Value = client.get(url).timeout(Duration::from_secs(20))
            .send().await?.error_for_status()?.json().await?;
        let valid = if source == "rotation" {
            valid_rotation(&data, Utc::now().timestamp_millis())
        } else {
            data.as_object().is_some_and(|entries| !entries.is_empty())
        };
        if !valid { return Err(AppError::msg(format!("Empty, expired, or invalid {source} data."))); }
        Ok(data)
    }.await;
    match fetched {
        Ok(data) => {
            saved.expires_at = if source == "rotation" {
                data["expiry"].as_i64().expect("validated rotation expiry")
            } else { Utc::now().timestamp_millis() + METADATA_LIFETIME };
            saved.data = Some(data.clone());
            saved.error = None;
            Ok(data)
        }
        Err(error) => {
            saved.error = Some(format!("Could not load {source}: {error}. Automatic retries are limited to once every five minutes."));
            // A failed manual refresh may keep serving a still-current snapshot.
            if saved.expires_at > Utc::now().timestamp_millis() {
                if let Some(data) = &saved.data { return Ok(data.clone()); }
            }
            Err(AppError::msg(saved.error.clone().unwrap()))
        }
    }
}

fn translated(dict: &Value, record: &Value, field: &str) -> AppResult<String> {
    record[field].as_str().and_then(|tag| dict[tag].as_str())
        .filter(|text| !text.trim().is_empty()).map(str::to_owned)
        .ok_or_else(|| AppError::msg(format!("Missing bounty {field} metadata.")))
}

fn display_bounties(rotation: &Value, regions: &Value, challenges: &Value, dict: &Value)
    -> AppResult<HashMap<String, Vec<Bounty>>> {
    let mut bounties = HashMap::new();
    for faction in FACTIONS {
        let mut entries = Vec::new();
        for entry in rotation["bounties"][faction].as_array().expect("validated bounties") {
            let region = &regions[entry["node"].as_str().unwrap()];
            let challenge = &challenges[entry["challenge"].as_str().unwrap()];
            let ally = match entry["ally"].as_str().and_then(|path| path.rsplit('/').next()) {
                Some("AmirAllyAgent") => Some("Amir"),
                Some("AoiAllyAgent") => Some("Aoi"),
                Some("ArthurAllyAgent") => Some("Arthur"),
                Some("EleanorAllyAgent") => Some("Eleanor"),
                Some("LettieAllyAgent") => Some("Lettie"),
                Some("QuincyAllyAgent") => Some("Quincy"),
                Some(_) => return Err(AppError::msg("Unknown Hex bounty ally.")),
                None => None,
            };
            let objective = translated(dict, challenge, "description")?
                .lines().last().unwrap_or_default().replace("|COUNT|", &challenge["requiredCount"].to_string());
            entries.push(Bounty {
                node: format!("{} ({})", translated(dict, region, "name")?, translated(dict, region, "systemName")?),
                mission_type: translated(dict, region, "missionName")?.to_lowercase(),
                challenge: translated(dict, challenge, "name")?,
                objective,
                ally: ally.map(str::to_owned),
                faction: if faction == "ZarimanSyndicate" {
                    Some(if rotation["zarimanFaction"] == "FC_CORPUS" { "Corpus" } else { "Grineer" }.into())
                } else { None },
            });
        }
        bounties.insert(faction.to_owned(), entries);
    }
    Ok(bounties)
}

#[tauri::command]
pub async fn get_oracle_bounties(app: AppHandle) -> AppResult<OracleBounties> {
    fetch_bounties(&app, false).await
}

pub async fn refresh_bounties(app: &AppHandle) -> AppResult<()> {
    let result = fetch_bounties(app, true).await?;
    if let Some(error) = result.error { Err(AppError::msg(error)) } else { Ok(()) }
}

async fn fetch_bounties(app: &AppHandle, force: bool) -> AppResult<OracleBounties> {
    let state = app.state::<AppState>();
    // Serialize requests across windows, including Maintenance, to coalesce refreshes.
    let mut cache = state.oracle_bounties.lock().await;
    let result: AppResult<(i64, HashMap<String, Vec<Bounty>>)> = async {
        let rotation = document(&state.http_client, &mut cache, "rotation", force).await?;
        let regions = document(&state.http_client, &mut cache, "ExportRegions", force).await?;
        let challenges = document(&state.http_client, &mut cache, "ExportChallenges", force).await?;
        let dict = document(&state.http_client, &mut cache, "dict.en", force).await?;
        match display_bounties(&rotation, &regions, &challenges, &dict) {
            Ok(bounties) => Ok((rotation["expiry"].as_i64().unwrap(), bounties)),
            Err(error) => {
                // Metadata can be structurally valid yet omit a new node or challenge.
                for source in ["ExportRegions", "ExportChallenges", "dict.en"] {
                    if let Some(saved) = cache.documents.get_mut(source) {
                        saved.expires_at = 0;
                        saved.attempted_at = Some(Utc::now().timestamp_millis());
                        saved.error = Some(error.to_string());
                    }
                }
                Err(error)
            }
        }
    }.await;
    match result {
        Ok((expiry, bounties)) => Ok(OracleBounties {
            expiry: Some(expiry), bounties,
            error: cache.documents.values().filter_map(|doc| doc.error.clone()).next(),
        }),
        Err(error) => Ok(OracleBounties { expiry: None, bounties: HashMap::new(), error: Some(error.to_string()) }),
    }
}
