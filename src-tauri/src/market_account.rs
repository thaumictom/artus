//! Authenticated warframe.market operations. The legacy v1 sign-in supplies a JWT;
//! all order management uses v2. Passwords never enter backend persistence or logs.

use futures_util::{SinkExt, StreamExt};
use std::{sync::atomic::{AtomicBool, AtomicU64, Ordering}, time::{Duration, SystemTime, UNIX_EPOCH}};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Message};

use crate::{error::{AppError, AppResult}, state::AppState};

const API: &str = "https://api.warframe.market";
const USER_AGENT: &str = concat!("Artus/", env!("CARGO_PKG_VERSION"), " (+https://github.com/thaumictom/artus)");
const ACCOUNT_STORE_PATH: &str = "market-account.json";
const EXIT_INVISIBLE_KEY: &str = "invisible_on_exit";
static NEXT_TIMER_ID: AtomicU64 = AtomicU64::new(1);
static EXIT_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

pub struct MarketSession {
    token: String,
    pub ingame_name: String,
    pub slug: String,
    pub status: String,
    invisible_at: Option<u64>,
    timer_id: u64,
    socket: mpsc::Sender<StatusCommand>,
}

struct StatusCommand {
    status: String,
    reply: oneshot::Sender<AppResult<()>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    ingame_name: String,
    slug: String,
    status: String,
    invisible_at: Option<u64>,
}

fn view(session: &MarketSession) -> SessionView {
    SessionView { ingame_name: session.ingame_name.clone(), slug: session.slug.clone(), status: session.status.clone(), invisible_at: session.invisible_at }
}

fn token(state: &AppState) -> AppResult<String> {
    state.market_session.lock()?.as_ref()
        .map(|session| session.token.clone())
        .ok_or_else(|| AppError::msg("Log in to warframe.market first"))
}

async fn response_json(request: reqwest::RequestBuilder) -> AppResult<Value> {
    let response = request.header("Language", "en").header("Platform", "pc")
        .header("Crossplay", "true").header(reqwest::header::USER_AGENT, USER_AGENT)
        .timeout(std::time::Duration::from_secs(20)).send().await?;
    let status = response.status();
    let body: Value = response.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        let detail = body.pointer("/error/message").and_then(Value::as_str)
            .or_else(|| body.pointer("/error").and_then(Value::as_str))
            .unwrap_or("Request rejected by warframe.market");
        return Err(AppError::msg(format!("warframe.market ({status}): {detail}")));
    }
    Ok(body)
}

async fn authenticated(state: &AppState, method: reqwest::Method, path: &str, body: Option<Value>) -> AppResult<Value> {
    let jwt = token(state)?;
    let mut request = state.http_client.request(method, format!("{API}/v2/{path}"))
        .bearer_auth(jwt);
    if let Some(body) = body { request = request.json(&body); }
    response_json(request).await
}

async fn socket_actor(jwt: String, mut rx: mpsc::Receiver<StatusCommand>, ready: oneshot::Sender<AppResult<()>>) {
    let result = async {
        let mut request = "wss://warframe.market/socket-v2".into_client_request().map_err(AppError::msg)?;
        request.headers_mut().insert("User-Agent", USER_AGENT.parse().map_err(AppError::msg)?);
        request.headers_mut().insert("Sec-WebSocket-Protocol", "wfm".parse().map_err(AppError::msg)?);
        let (mut socket, _) = tokio_tungstenite::connect_async(request).await.map_err(AppError::msg)?;
        socket.send(Message::Text(json!({"route":"@wfm|cmd/auth/signIn","id":"artus-auth","payload":{"token":jwt,"deviceId":"artus-desktop"}}).to_string().into())).await.map_err(AppError::msg)?;
        let mut authenticated = false;
        let mut initial_status = false;
        while let Some(message) = socket.next().await {
            let message = message.map_err(AppError::msg)?;
            let Ok(value) = serde_json::from_str::<Value>(&message.to_string()) else { continue };
            match value["route"].as_str().unwrap_or_default() {
                "@wfm|cmd/auth/signIn:ok" => authenticated = true,
                "@wfm|cmd/auth/signIn:error" => return Err(AppError::msg("warframe.market WebSocket authentication failed")),
                "@wfm|event/status/set" if authenticated => { initial_status = true; break; }
                _ => {}
            }
        }
        if !authenticated || !initial_status { return Err(AppError::msg("warframe.market did not confirm account status")); }
        Ok(socket)
    }.await;
    let mut socket = match result {
        Ok(socket) => { let _ = ready.send(Ok(())); socket }
        Err(error) => { let _ = ready.send(Err(error)); return }
    };
    let mut pending: Option<oneshot::Sender<AppResult<()>>> = None;
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(30));
    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() { break; }
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(15)), if pending.is_some() => {
                if let Some(reply) = pending.take() { let _ = reply.send(Err(AppError::msg("Timed out waiting for market status confirmation"))); }
            }
            command = rx.recv(), if pending.is_none() => {
                let Some(command) = command else { break };
                let payload = json!({"route":"@wfm|cmd/status/set","id":"artus-status","payload":{"status":command.status,"activity":null}});
                if let Err(error) = socket.send(Message::Text(payload.to_string().into())).await {
                    let _ = command.reply.send(Err(AppError::msg(error)));
                    break;
                }
                pending = Some(command.reply);
            }
            message = socket.next() => {
                let Some(Ok(message)) = message else { break };
                let Ok(value) = serde_json::from_str::<Value>(&message.to_string()) else { continue };
                match value["route"].as_str().unwrap_or_default() {
                    "@wfm|cmd/status/set:ok" => { if let Some(reply) = pending.take() { let _ = reply.send(Ok(())); } }
                    "@wfm|cmd/status/set:error" => {
                        if let Some(reply) = pending.take() { let _ = reply.send(Err(AppError::msg("warframe.market rejected the status change"))); }
                    }
                    _ => {}
                }
            }
        }
    }
    if let Some(reply) = pending { let _ = reply.send(Err(AppError::msg("warframe.market connection closed"))); }
    let _ = socket.close(None).await;
}

#[tauri::command]
pub fn market_session(state: State<'_, AppState>) -> AppResult<Option<SessionView>> {
    Ok(state.market_session.lock()?.as_ref().filter(|session| !session.socket.is_closed()).map(view))
}

#[tauri::command]
pub async fn market_login(state: State<'_, AppState>, email: String, password: String) -> AppResult<SessionView> {
    if email.trim().is_empty() || password.is_empty() { return Err(AppError::msg("Email and password are required")); }
    let response = state.http_client.post(format!("{API}/v1/auth/signin"))
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .header("Language", "en").header("Platform", "pc")
        .header("Authorization", "JWT")
        .json(&json!({"email":email.trim(),"password":password,"auth_type":"header","device_id":"artus-desktop"}))
        .timeout(std::time::Duration::from_secs(20)).send().await?;
    if !response.status().is_success() { return Err(AppError::msg("Login failed. Check your email and password.")); }
    let jwt = response.headers().get(reqwest::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.split_once(' ').map(|(_, token)| token.to_owned()))
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::msg("warframe.market did not return a session token"))?;
    let me = response_json(state.http_client.get(format!("{API}/v2/me")).bearer_auth(&jwt)).await?;
    let ingame_name = me.pointer("/data/ingameName").and_then(Value::as_str)
        .unwrap_or("warframe.market account").to_owned();
    let slug = me.pointer("/data/slug").and_then(Value::as_str)
        .ok_or_else(|| AppError::msg("warframe.market did not return a profile slug"))?.to_owned();
    let (sender, receiver) = mpsc::channel(4);
    let (ready_sender, ready_receiver) = oneshot::channel();
    tokio::spawn(socket_actor(jwt.clone(), receiver, ready_sender));
    tokio::time::timeout(std::time::Duration::from_secs(15), ready_receiver).await
        .map_err(|_| AppError::msg("Timed out connecting to warframe.market status"))?.map_err(AppError::msg)??;
    let (reply, confirmation) = oneshot::channel();
    sender.send(StatusCommand { status: "invisible".into(), reply }).await.map_err(AppError::msg)?;
    tokio::time::timeout(std::time::Duration::from_secs(15), confirmation).await
        .map_err(|_| AppError::msg("Timed out setting invisible status"))?.map_err(AppError::msg)??;
    let mut guard = state.market_session.lock()?;
    *guard = Some(MarketSession { token: jwt, ingame_name, slug, status: "invisible".into(), invisible_at: None, timer_id: 0, socket: sender });
    Ok(view(guard.as_ref().unwrap()))
}

#[tauri::command]
pub fn market_logout(state: State<'_, AppState>) -> AppResult<()> {
    *state.market_session.lock()? = None;
    Ok(())
}

#[tauri::command]
pub async fn market_set_status(state: State<'_, AppState>, status: String) -> AppResult<SessionView> {
    set_status(&state, status).await
}

async fn set_status(state: &AppState, status: String) -> AppResult<SessionView> {
    if !matches!(status.as_str(), "invisible" | "online" | "ingame") { return Err(AppError::msg("Invalid market status")); }
    let _operation = state.market_status_operation.lock().await;
    set_status_locked(state, status).await
}

async fn set_status_locked(state: &AppState, status: String) -> AppResult<SessionView> {
    let sender = state.market_session.lock()?.as_ref()
        .ok_or_else(|| AppError::msg("Log in to warframe.market first"))?.socket.clone();
    let (reply, confirmation) = oneshot::channel();
    sender.send(StatusCommand { status: status.clone(), reply }).await.map_err(AppError::msg)?;
    tokio::time::timeout(std::time::Duration::from_secs(15), confirmation).await
        .map_err(|_| AppError::msg("Timed out changing market status"))?.map_err(AppError::msg)??;
    let mut guard = state.market_session.lock()?;
    let session = guard.as_mut().ok_or_else(|| AppError::msg("Logged out"))?;
    if !session.socket.same_channel(&sender) { return Err(AppError::msg("Market session changed")); }
    session.status = status;
    session.invisible_at = None;
    Ok(view(session))
}

#[tauri::command]
pub async fn market_schedule_invisible(app: AppHandle, state: State<'_, AppState>, minutes: u64) -> AppResult<SessionView> {
    if !matches!(minutes, 0 | 30 | 60 | 120 | 240) { return Err(AppError::msg("Invalid invisible timer")); }
    let _operation = state.market_status_operation.lock().await;
    let mut guard = state.market_session.lock()?;
    let session = guard.as_mut().ok_or_else(|| AppError::msg("Log in to warframe.market first"))?;
    if minutes > 0 && session.status == "invisible" { return Err(AppError::msg("Change your status before starting a timer")); }
    let timer_id = NEXT_TIMER_ID.fetch_add(1, Ordering::Relaxed);
    session.timer_id = timer_id;
    session.invisible_at = if minutes == 0 { None } else {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(AppError::msg)?.as_millis() as u64;
        Some(now + minutes * 60_000)
    };
    let result = view(session);
    drop(guard);
    if minutes > 0 {
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_secs(minutes * 60)).await;
            expire_invisible_timer(app, timer_id).await;
        });
    }
    Ok(result)
}

async fn expire_invisible_timer(app: AppHandle, timer_id: u64) {
    let state = app.state::<AppState>();
    let _operation = state.market_status_operation.lock().await;
    let active = state.market_session.lock().ok().and_then(|guard| guard.as_ref().map(|session| session.timer_id == timer_id && session.invisible_at.is_some())).unwrap_or(false);
    if !active { return; }
    match set_status_locked(&state, "invisible".into()).await {
        Ok(session) => { let _ = app.emit("market_session_changed", session); }
        Err(error) => {
            log::warn!("could not apply scheduled invisible status: {error}");
            if let Ok(mut guard) = state.market_session.lock() {
                if let Some(session) = guard.as_mut().filter(|session| session.timer_id == timer_id) {
                    session.invisible_at = None;
                    let _ = app.emit("market_session_changed", view(session));
                }
            }
            let _ = app.emit("market_invisible_timer_error", error.to_string());
        }
    }
}

fn invisible_on_close(app: &AppHandle) -> bool {
    app.store(ACCOUNT_STORE_PATH).ok()
        .and_then(|store| store.get(EXIT_INVISIBLE_KEY))
        .and_then(|value| value.as_bool())
        .unwrap_or(true)
}

pub async fn apply_close_status(app: &AppHandle) {
    if !invisible_on_close(app) { return; }
    let state = app.state::<AppState>();
    let should_change = state.market_session.lock().ok()
        .and_then(|guard| guard.as_ref().map(|session| session.status != "invisible" && !session.socket.is_closed()))
        .unwrap_or(false);
    if !should_change { return; }
    match tokio::time::timeout(Duration::from_secs(5), set_status(&state, "invisible".into())).await {
        Ok(Ok(session)) => { let _ = app.emit("market_session_changed", session); }
        Ok(Err(error)) => log::warn!("could not go invisible on close: {error}"),
        Err(_) => log::warn!("timed out going invisible on close"),
    }
}

pub fn hide_to_tray(app: AppHandle) {
    tauri::async_runtime::spawn(async move { apply_close_status(&app).await; });
}

pub fn exit_app(app: AppHandle, restart: bool) {
    if EXIT_IN_PROGRESS.swap(true, Ordering::SeqCst) { return; }
    tauri::async_runtime::spawn(async move {
        apply_close_status(&app).await;
        if restart { app.restart(); } else { app.exit(0); }
    });
}

#[tauri::command]
pub async fn market_top_orders(state: State<'_, AppState>, slug: String) -> AppResult<Value> {
    if !valid_slug(&slug) { return Err(AppError::msg("Invalid item slug")); }
    response_json(state.http_client.get(format!("{API}/v2/orders/item/{slug}/top"))).await
}

fn valid_slug(value: &str) -> bool { !value.is_empty() && value.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-') }
fn valid_id(value: &str) -> bool { !value.is_empty() && value.bytes().all(|c| c.is_ascii_alphanumeric()) }
fn valid_order(platinum: i64, quantity: i64) -> bool { (1..=900_000).contains(&platinum) && (1..=9_999).contains(&quantity) }

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingVariant {
    rank: Option<i64>,
    charges: Option<i64>,
    subtype: Option<String>,
    amber_stars: Option<i64>,
    cyan_stars: Option<i64>,
}

fn insert_level(body: &mut Value, item: &Value, field: &str, maximum: &str, selected: Option<i64>) -> AppResult<()> {
    match item[maximum].as_i64().filter(|maximum| *maximum > 0) {
        Some(maximum) => {
            let value = selected.ok_or_else(|| AppError::msg(format!("{field} is required for this item")))?;
            if !(0..=maximum).contains(&value) { return Err(AppError::msg(format!("Invalid {field}"))); }
            body[field] = json!(value);
        }
        None if selected.is_some() => return Err(AppError::msg(format!("{field} is not supported for this item"))),
        None => {}
    }
    Ok(())
}

#[tauri::command]
pub async fn market_my_orders(state: State<'_, AppState>) -> AppResult<Value> {
    authenticated(&state, reqwest::Method::GET, "orders/my", None).await
}

#[derive(Serialize)]
pub struct ListingItemDetails {
    name: String,
    slug: String,
}

#[tauri::command]
pub async fn market_item_details(state: State<'_, AppState>) -> AppResult<std::collections::HashMap<String, ListingItemDetails>> {
    let response = response_json(state.http_client.get(format!("{API}/v2/items"))).await?;
    let items = response["data"].as_array().ok_or_else(|| AppError::msg("Invalid item list"))?;
    Ok(items.iter().filter_map(|item| {
        Some((item["id"].as_str()?.to_owned(), ListingItemDetails {
            name: item.pointer("/i18n/en/name").and_then(Value::as_str).unwrap_or_else(|| item["slug"].as_str().unwrap_or("Item")).to_owned(),
            slug: item["slug"].as_str()?.to_owned(),
        }))
    }).collect())
}

#[tauri::command]
pub async fn market_create_listing(state: State<'_, AppState>, slug: String, platinum: i64, quantity: i64, variant: ListingVariant) -> AppResult<Value> {
    if !valid_slug(&slug) || !valid_order(platinum, quantity) { return Err(AppError::msg("Invalid listing details")); }
    let item = response_json(state.http_client.get(format!("{API}/v2/item/{slug}"))).await?;
    let data = &item["data"];
    let item_id = data["id"].as_str().ok_or_else(|| AppError::msg("Item ID unavailable"))?;
    let mut body = json!({"itemId":item_id,"type":"sell","platinum":platinum,"quantity":quantity,"visible":true});
    if data["bulkTradable"].as_bool() == Some(true) { body["perTrade"] = json!(1); }
    insert_level(&mut body, data, "rank", "maxRank", variant.rank)?;
    insert_level(&mut body, data, "charges", "maxCharges", variant.charges)?;
    insert_level(&mut body, data, "amberStars", "maxAmberStars", variant.amber_stars)?;
    insert_level(&mut body, data, "cyanStars", "maxCyanStars", variant.cyan_stars)?;
    if let Some(subtypes) = data["subtypes"].as_array().filter(|subtypes| !subtypes.is_empty()) {
        let subtype = variant.subtype.as_deref().ok_or_else(|| AppError::msg("Subtype is required for this item"))?;
        if !subtypes.iter().any(|candidate| candidate.as_str() == Some(subtype)) { return Err(AppError::msg("Invalid subtype")); }
        body["subtype"] = json!(subtype);
    } else if variant.subtype.is_some() {
        return Err(AppError::msg("Subtype is not supported for this item"));
    }
    authenticated(&state, reqwest::Method::POST, "order", Some(body)).await
}

#[tauri::command]
pub async fn market_update_listing(state: State<'_, AppState>, id: String, platinum: i64, quantity: i64) -> AppResult<Value> {
    if !valid_id(&id) || !valid_order(platinum, quantity) { return Err(AppError::msg("Invalid listing details")); }
    authenticated(&state, reqwest::Method::PATCH, &format!("order/{id}"), Some(json!({"platinum":platinum,"quantity":quantity}))).await
}

#[tauri::command]
pub async fn market_delete_listing(state: State<'_, AppState>, id: String) -> AppResult<Value> {
    if !valid_id(&id) { return Err(AppError::msg("Invalid listing ID")); }
    authenticated(&state, reqwest::Method::DELETE, &format!("order/{id}"), None).await
}
