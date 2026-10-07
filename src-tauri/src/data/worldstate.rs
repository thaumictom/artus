//! Direct world-state access outside the webview's CORS restrictions.

use serde_json::Value;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[tauri::command]
pub async fn get_world_state(state: State<'_, AppState>) -> AppResult<Value> {
    let world: Value = state
        .http_client
        .get("https://api.warframe.com/cdn/worldState.php")
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    if !world.get("Time").is_some_and(Value::is_number) {
        return Err(AppError::msg("Invalid world-state response"));
    }
    Ok(world)
}
