//! Inventory mutation rules, independent of Tauri and persistence.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

const MAX_QUANTITY: i64 = 9_007_199_254_740_991;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    // Retain unknown item metadata when reading and writing existing records.
    pub items: Vec<Value>,
    pub new_slugs: Vec<String>,
}

#[derive(Clone, Deserialize)]
pub struct ItemInput {
    pub text: String,
    pub slug: Option<String>,
    pub is_custom: Option<bool>,
    pub ducats: Option<u64>,
    pub market_median: Option<f64>,
    pub market_median_from_current_offers: Option<bool>,
}

#[derive(Deserialize)]
pub struct QuantityChange {
    pub word: ItemInput,
    pub delta: Option<i64>,
    pub quantity: Option<i64>,
}

fn name_key(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn item_slug(item: &Value) -> Option<&str> {
    item.get("slug")?.as_str().filter(|slug| !slug.is_empty())
}
fn item_name(item: &Value) -> &str {
    item.get("name").and_then(Value::as_str).unwrap_or_default()
}
fn custom(item: &Value) -> bool {
    item.get("isCustom").and_then(Value::as_bool) == Some(true)
}
fn market_slug(item: &Value) -> Option<&str> {
    if custom(item) {
        return None;
    }
    let slug = item_slug(item)?;
    Some(if item_name(item).contains("Relic") {
        slug.strip_suffix("_intact")
            .or_else(|| slug.strip_suffix("_radiant"))
            .unwrap_or(slug)
    } else {
        slug
    })
}
fn quantity(item: &Value) -> Result<i64, String> {
    item.get("quantity")
        .and_then(Value::as_i64)
        .filter(|value| (0..=MAX_QUANTITY).contains(value))
        .ok_or_else(|| "existing inventory quantity is invalid".into())
}
fn next_quantity(previous: i64, delta: i64) -> Result<i64, String> {
    previous
        .checked_add(delta)
        .filter(|next| *next <= MAX_QUANTITY)
        .ok_or_else(|| "inventory quantity overflow".into())
}
fn valid_delta(delta: i64) -> bool {
    delta != 0 && delta.unsigned_abs() <= MAX_QUANTITY as u64
}
fn fill_metadata(item: &mut Value, word: &ItemInput) {
    let object = item.as_object_mut().expect("validated inventory item");
    if object.get("slug").is_none_or(Value::is_null) {
        object.insert("slug".into(), json!(word.slug));
    }
    if let Some(ducats) = word.ducats {
        if object.get("ducats").is_none_or(Value::is_null) {
            object.insert("ducats".into(), json!(ducats));
        }
    }
}

impl Inventory {
    pub fn apply_ocr(
        &mut self,
        changes: &[QuantityChange],
    ) -> Result<HashMap<String, i64>, String> {
        let mut applied = HashMap::new();
        for change in changes {
            let word = &change.word;
            let Some(slug) = word.slug.as_deref().filter(|slug| !slug.is_empty()) else {
                continue;
            };
            if word.is_custom == Some(true) {
                continue;
            }
            let delta = change.delta.filter(|delta| valid_delta(*delta));
            let set = change
                .quantity
                .filter(|quantity| (1..=MAX_QUANTITY).contains(quantity));
            if delta.is_none() && set.is_none() {
                continue;
            }
            let index = self.items.iter().position(|item| match item_slug(item) {
                Some(saved) => saved == slug,
                None => name_key(item_name(item)) == name_key(&word.text),
            });
            let previous = index
                .map(|index| quantity(&self.items[index]))
                .transpose()?
                .unwrap_or(0);
            let next = match set {
                Some(value) => value,
                None => next_quantity(previous, delta.unwrap())?.max(0),
            };
            if next == previous {
                continue;
            }
            if let Some(index) = index {
                if next == 0 {
                    self.items.remove(index);
                    self.new_slugs.retain(|saved| saved != slug);
                } else {
                    self.items[index]["quantity"] = json!(next);
                    fill_metadata(&mut self.items[index], word);
                }
            } else {
                let mut item = json!({ "name": word.text, "slug": slug, "quantity": next });
                if let Some(custom) = word.is_custom {
                    item["isCustom"] = json!(custom);
                }
                fill_metadata(&mut item, word);
                self.items.push(item);
                if !self.new_slugs.iter().any(|saved| saved == slug) {
                    self.new_slugs.push(slug.into());
                }
            }
            *applied.entry(slug.to_owned()).or_insert(0) += next - previous;
        }
        Ok(applied)
    }

    pub fn add_manual(&mut self, word: &ItemInput, amount: i64) -> Result<(), String> {
        if !(1..=MAX_QUANTITY).contains(&amount) || word.is_custom == Some(true) {
            return Err("invalid inventory addition".into());
        }
        let slug = word
            .slug
            .as_deref()
            .filter(|slug| !slug.is_empty())
            .ok_or("item has no market slug")?;
        if let Some(item) = self.items.iter_mut().find(|item| {
            !custom(item)
                && (item_slug(item) == Some(slug)
                    || item_slug(item).is_none()
                        && name_key(item_name(item)) == name_key(&word.text))
        }) {
            item["quantity"] = json!(next_quantity(quantity(item)?, amount)?);
            fill_metadata(item, word);
        } else {
            let mut item = json!({ "name": word.text, "slug": slug, "quantity": amount });
            fill_metadata(&mut item, word);
            self.items.push(item);
        }
        // Manual additions historically do not introduce a new-item dot.
        Ok(())
    }

    pub fn change_row(
        &mut self,
        slug: Option<&str>,
        name: &str,
        is_custom: bool,
        delta: i64,
    ) -> Result<(), String> {
        if !valid_delta(delta) {
            return Err("invalid inventory quantity change".into());
        }
        let index = self
            .items
            .iter()
            .position(|item| {
                custom(item) == is_custom
                    && match slug {
                        Some(slug) => item_slug(item) == Some(slug),
                        None => item_name(item) == name,
                    }
            })
            .ok_or("inventory item no longer exists")?;
        let next = next_quantity(quantity(&self.items[index])?, delta)?;
        if next <= 0 {
            let removed = self.items.remove(index);
            if let Some(slug) = item_slug(&removed) {
                self.new_slugs.retain(|saved| saved != slug);
            }
        } else {
            self.items[index]["quantity"] = json!(next);
        }
        Ok(())
    }

    pub fn change_market(
        &mut self,
        slug: Option<&str>,
        name: &str,
        delta: i64,
    ) -> Result<(), String> {
        if !valid_delta(delta) {
            return Err("invalid inventory quantity change".into());
        }
        let index = self.items.iter().position(|item| {
            !custom(item)
                && (slug.is_some() && market_slug(item) == slug
                    || item_slug(item).is_none() && name_key(item_name(item)) == name_key(name))
                && (delta > 0 || quantity(item).is_ok_and(|quantity| quantity > 0))
        });
        if let Some(index) = index {
            let next = next_quantity(quantity(&self.items[index])?, delta)?;
            if next < 0 {
                return Err(format!("No {name} remains in inventory"));
            }
            if next == 0 {
                let removed = self.items.remove(index);
                if !self.items.iter().any(|item| {
                    market_slug(item) == slug && quantity(item).is_ok_and(|quantity| quantity > 0)
                }) {
                    if let Some(saved) = item_slug(&removed) {
                        self.new_slugs.retain(|slug| slug != saved);
                    }
                }
            } else {
                self.items[index]["quantity"] = json!(next);
            }
        } else {
            if delta < 0 {
                return Err(format!("No {name} remains in inventory"));
            }
            let slug = slug
                .ok_or_else(|| format!("Cannot add {name} to inventory without a market item"))?;
            self.items
                .push(json!({ "name": name, "slug": slug, "quantity": delta }));
            if !self.new_slugs.iter().any(|saved| saved == slug) {
                self.new_slugs.push(slug.into());
            }
        }
        Ok(())
    }

    pub fn add_relic(&mut self, word: &ItemInput) -> Result<(), String> {
        if word
            .slug
            .as_deref()
            .filter(|slug| !slug.is_empty())
            .is_none()
            || word.is_custom == Some(true)
        {
            return Err("selected reward cannot be added to inventory".into());
        }
        self.apply_ocr(&[QuantityChange {
            word: word.clone(),
            delta: Some(1),
            quantity: None,
        }])?;
        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item_slug(item) == word.slug.as_deref())
        {
            if let Some(price) = word.market_median.filter(|price| price.is_finite()) {
                item["marketMedian"] = json!(price);
                if let Some(fallback) = word.market_median_from_current_offers {
                    item["marketMedianUsesOfferFallback"] = json!(fallback);
                }
            }
        }
        Ok(())
    }
}
