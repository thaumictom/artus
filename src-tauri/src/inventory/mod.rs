//! Authoritative inventory service and Tauri persistence/event adapters.
pub mod domain;

use crate::error::{AppError, AppResult};
use domain::{Inventory, ItemInput, QuantityChange};
use serde::Serialize;
use std::{collections::HashMap, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "inventory.json";

#[derive(Clone, Default, Serialize)]
pub struct InventorySnapshot {
    pub revision: u64,
    #[serde(flatten)]
    pub inventory: Inventory,
}

#[derive(Default)]
pub struct InventoryService {
    // Only inventory is locked. No network, OCR, or window operation holds this lock.
    state: Mutex<Option<InventorySnapshot>>,
}

impl InventoryService {
    fn load<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        current: &mut Option<InventorySnapshot>,
    ) -> AppResult<()> {
        if current.is_none() {
            let store = app.store(STORE_PATH).map_err(AppError::msg)?;
            let inventory = Inventory {
                items: store
                    .get("items")
                    .map(serde_json::from_value)
                    .transpose()?
                    .unwrap_or_default(),
                new_slugs: store
                    .get("newSlugs")
                    .map(serde_json::from_value)
                    .transpose()?
                    .unwrap_or_default(),
            };
            *current = Some(InventorySnapshot {
                revision: 0,
                inventory,
            });
        }
        Ok(())
    }

    pub fn snapshot<R: Runtime>(&self, app: &AppHandle<R>) -> AppResult<InventorySnapshot> {
        let mut current = self.state.lock()?;
        self.load(app, &mut current)?;
        Ok(current.as_ref().unwrap().clone())
    }

    pub fn mutate<R: Runtime, T>(
        &self,
        app: &AppHandle<R>,
        change: impl FnOnce(&mut Inventory) -> Result<T, String>,
    ) -> AppResult<(InventorySnapshot, T)> {
        let mut current = self.state.lock()?;
        self.load(app, &mut current)?;
        let previous = current.as_ref().unwrap();
        let mut next = previous.clone();
        let result = change(&mut next.inventory).map_err(AppError::msg)?;
        if next.inventory == previous.inventory {
            return Ok((previous.clone(), result));
        }
        next.revision = previous
            .revision
            .checked_add(1)
            .ok_or_else(|| AppError::msg("inventory revision overflow"))?;
        let store = app.store(STORE_PATH).map_err(AppError::msg)?;
        let old_items = store.get("items");
        let old_slugs = store.get("newSlugs");
        store.set("items", serde_json::to_value(&next.inventory.items)?);
        store.set("newSlugs", serde_json::to_value(&next.inventory.new_slugs)?);
        if let Err(error) = store.save() {
            // A failed save must not leave the plugin's in-memory store ahead of committed state.
            match old_items {
                Some(value) => store.set("items", value),
                None => {
                    store.delete("items");
                }
            }
            match old_slugs {
                Some(value) => store.set("newSlugs", value),
                None => {
                    store.delete("newSlugs");
                }
            }
            return Err(AppError::msg(error));
        }
        *current = Some(next.clone());
        drop(current);
        // Revisions let windows reject older events even if concurrent commits emit out of order.
        if let Err(error) = app.emit("inventory_changed", &next) {
            log::warn!("could not publish inventory snapshot: {error}");
        }
        Ok((next, result))
    }
}

#[tauri::command]
pub fn inventory_snapshot(
    app: AppHandle,
    service: State<'_, InventoryService>,
) -> AppResult<InventorySnapshot> {
    service.snapshot(&app)
}

#[derive(Serialize)]
pub struct QuantityResult {
    pub snapshot: InventorySnapshot,
    pub applied: HashMap<String, i64>,
}

#[tauri::command]
pub fn inventory_change_ocr_quantities(
    app: AppHandle,
    service: State<'_, InventoryService>,
    changes: Vec<QuantityChange>,
) -> AppResult<QuantityResult> {
    let (snapshot, applied) = service.mutate(&app, |inventory| inventory.apply_ocr(&changes))?;
    Ok(QuantityResult { snapshot, applied })
}
#[tauri::command]
pub fn inventory_add_item(
    app: AppHandle,
    service: State<'_, InventoryService>,
    word: ItemInput,
    quantity: i64,
) -> AppResult<InventorySnapshot> {
    service
        .mutate(&app, |inventory| inventory.add_manual(&word, quantity))
        .map(|(snapshot, ())| snapshot)
}
#[tauri::command]
pub fn inventory_change_row_quantity(
    app: AppHandle,
    service: State<'_, InventoryService>,
    slug: Option<String>,
    name: String,
    is_custom: bool,
    delta: i64,
) -> AppResult<InventorySnapshot> {
    service
        .mutate(&app, |inventory| {
            inventory.change_row(slug.as_deref(), &name, is_custom, delta)
        })
        .map(|(snapshot, ())| snapshot)
}
#[tauri::command]
pub fn inventory_change_market_quantity(
    app: AppHandle,
    service: State<'_, InventoryService>,
    slug: Option<String>,
    name: String,
    delta: i64,
) -> AppResult<InventorySnapshot> {
    service
        .mutate(&app, |inventory| {
            inventory.change_market(slug.as_deref(), &name, delta)
        })
        .map(|(snapshot, ())| snapshot)
}
#[tauri::command]
pub fn inventory_dismiss_new_items(
    app: AppHandle,
    service: State<'_, InventoryService>,
) -> AppResult<InventorySnapshot> {
    service
        .mutate(&app, |inventory| {
            inventory.new_slugs.clear();
            Ok(())
        })
        .map(|(snapshot, ())| snapshot)
}
#[tauri::command]
pub fn inventory_reset(
    app: AppHandle,
    service: State<'_, InventoryService>,
) -> AppResult<InventorySnapshot> {
    service
        .mutate(&app, |inventory| {
            inventory.items.clear();
            inventory.new_slugs.clear();
            Ok(())
        })
        .map(|(snapshot, ())| snapshot)
}

pub fn add_relic_reward<R: Runtime>(
    app: &AppHandle<R>,
    word: &crate::ocr::OcrWord,
) -> AppResult<()> {
    let input = ItemInput {
        text: word.text.clone(),
        slug: word.slug.clone(),
        is_custom: word.is_custom,
        ducats: word.ducats,
        market_median: word.market_median,
        market_median_from_current_offers: word.market_median_from_current_offers,
    };
    app.state::<InventoryService>()
        .mutate(app, |inventory| inventory.add_relic(&input))
        .map(|_| ())
}
