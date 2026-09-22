//! Native Warframe Market WebSocket bridge.
//!
//! The socket cannot reliably be opened from a Tauri WebView because the
//! marketplace rejects its browser Origin during the HTTP upgrade. A native
//! connection also lets Artus provide the descriptive User-Agent requested by
//! Warframe Market's API rules.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    http::{
        header::{SEC_WEBSOCKET_PROTOCOL, USER_AGENT},
        HeaderValue,
    },
    Message,
};

const SOCKET_URL: &str = "wss://ws.warframe.market/socket";
const SUBSCRIBE_ROUTE: &str = "@wfm|cmd/subscribe/newOrders";
const ORDER_EVENT_ROUTE: &str = "@wfm|event/subscriptions/newOrder";
const RECONNECT_MAX_SECONDS: u64 = 60;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);

#[tauri::command]
pub fn start_market_notification_socket(app: AppHandle) {
    if ACTIVE.swap(true, Ordering::AcqRel) {
        return;
    }

    let generation = GENERATION.fetch_add(1, Ordering::AcqRel) + 1;
    tauri::async_runtime::spawn(run_socket(app, generation));
}

#[tauri::command]
pub fn stop_market_notification_socket(app: AppHandle) {
    ACTIVE.store(false, Ordering::Release);
    GENERATION.fetch_add(1, Ordering::AcqRel);
    emit_state(&app, "disconnected", None);
}

async fn run_socket(app: AppHandle, generation: u64) {
    let mut reconnect_seconds = 2;

    while is_current(generation) {
        emit_state(&app, "connecting", None);

        match connect().await {
            Ok(mut socket) => {
                reconnect_seconds = 2;
                emit_state(&app, "connected", None);

                let subscription = json!({
                    "route": SUBSCRIBE_ROUTE,
                    "id": format!("artus-{generation}"),
                    "payload": { "platform": "pc", "crossplay": true }
                });

                if let Err(error) = socket
                    .send(Message::Text(subscription.to_string().into()))
                    .await
                {
                    emit_state(&app, "disconnected", Some(error.to_string()));
                } else {
                    loop {
                        if !is_current(generation) {
                            let _ = socket.close(None).await;
                            return;
                        }

                        match tokio::time::timeout(Duration::from_secs(1), socket.next()).await {
                            Ok(Some(Ok(Message::Text(text)))) => {
                                handle_message(&app, text.as_str())
                            }
                            Ok(Some(Ok(Message::Ping(payload)))) => {
                                if let Err(error) = socket.send(Message::Pong(payload)).await {
                                    emit_state(&app, "disconnected", Some(error.to_string()));
                                    break;
                                }
                            }
                            Ok(Some(Ok(Message::Close(_)))) | Ok(None) => {
                                emit_state(
                                    &app,
                                    "disconnected",
                                    Some("Warframe Market closed the live connection.".into()),
                                );
                                break;
                            }
                            Ok(Some(Err(error))) => {
                                emit_state(&app, "disconnected", Some(error.to_string()));
                                break;
                            }
                            Ok(Some(Ok(_))) | Err(_) => {}
                        }
                    }
                }
            }
            Err(error) => emit_state(&app, "disconnected", Some(error)),
        }

        if !is_current(generation) {
            return;
        }
        tokio::time::sleep(Duration::from_secs(reconnect_seconds)).await;
        reconnect_seconds = (reconnect_seconds * 2).min(RECONNECT_MAX_SECONDS);
    }
}

async fn connect() -> Result<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    String,
> {
    let mut request = SOCKET_URL
        .into_client_request()
        .map_err(|error| error.to_string())?;
    request
        .headers_mut()
        .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static("wfm"));
    request.headers_mut().insert(
        USER_AGENT,
        HeaderValue::from_static(concat!("Artus/", env!("CARGO_PKG_VERSION"))),
    );

    connect_async(request)
        .await
        .map(|(socket, _)| socket)
        .map_err(|error| error.to_string())
}

fn handle_message(app: &AppHandle, text: &str) {
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        return;
    };

    if message.get("route").and_then(Value::as_str) == Some(ORDER_EVENT_ROUTE) {
        if let Some(payload) = message.get("payload") {
            let _ = app.emit("market_order", payload.clone());
        }
    } else if message
        .get("route")
        .and_then(Value::as_str)
        .is_some_and(|route| route.ends_with(":error"))
    {
        emit_state(
            app,
            "disconnected",
            Some(
                message
                    .get("payload")
                    .map(Value::to_string)
                    .unwrap_or_else(|| "Warframe Market rejected the subscription.".into()),
            ),
        );
    }
}

fn is_current(generation: u64) -> bool {
    ACTIVE.load(Ordering::Acquire) && GENERATION.load(Ordering::Acquire) == generation
}

fn emit_state(app: &AppHandle, state: &str, error: Option<String>) {
    let _ = app.emit(
        "market_socket_state",
        json!({ "state": state, "error": error }),
    );
}
