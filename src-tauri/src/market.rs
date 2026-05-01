use serde_json::Value;
use tauri::State;
use crate::state::AppState;

async fn fetch_json(client: &reqwest::Client, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .header("Language", "en")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<Value>().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_market_item(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v2/item/{}", slug);
    fetch_json(&state.http_client, &url).await
}

#[tauri::command]
pub async fn get_market_orders(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v2/orders/item/{}", slug);
    fetch_json(&state.http_client, &url).await
}

#[tauri::command]
pub async fn get_market_statistics(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v1/items/{}/statistics", slug);
    fetch_json(&state.http_client, &url).await
}
