use serde_json::Value;
use tauri::State;
use crate::state::AppState;

#[tauri::command]
pub async fn get_market_item(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v2/item/{}", slug);
    let response = state.http_client
        .get(url)
        .header("Language", "en")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<Value>().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_market_orders(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v2/orders/item/{}", slug);
    let response = state.http_client
        .get(url)
        .header("Language", "en")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<Value>().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_market_statistics(state: State<'_, AppState>, slug: String) -> Result<Value, String> {
    let url = format!("https://api.warframe.market/v1/items/{}/statistics", slug);
    let response = state.http_client
        .get(url)
        .header("Language", "en")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<Value>().await.map_err(|e| e.to_string())
}
