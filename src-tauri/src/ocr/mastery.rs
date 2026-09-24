//! Fuzzy OCR matching for mastered gear cards.

use std::fs::File;
use std::io::BufReader;

use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};

use super::dictionary::{normalize_dictionary_text, similarity_score};
use super::OcrWord;
use crate::error::{AppError, AppResult};
use crate::market::catalog_path;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct MasteryDictionaryEntry {
    pub key: String,
    pub name: String,
    pub normalized_name: String,
}

/// Build the same names shown on mastered cards, including the MAX RANK line.
pub fn load_mastery_dictionary(app: &AppHandle) -> AppResult<usize> {
    let catalog: Value = serde_json::from_reader(BufReader::new(File::open(catalog_path(app)?)?))?;
    let dictionary = build_mastery_dictionary(&catalog)?;
    let count = dictionary.len();
    *app.state::<AppState>()
        .mastery_dictionary
        .lock()
        .map_err(|_| AppError::msg("mastery dictionary lock poisoned"))? = dictionary;
    Ok(count)
}

fn build_mastery_dictionary(catalog: &Value) -> AppResult<Vec<MasteryDictionaryEntry>> {
    let items = catalog
        .as_object()
        .ok_or_else(|| AppError::msg("invalid cached mastery catalog"))?;
    let mut dictionary = Vec::new();
    for (key, item) in items {
        if item.get("masterable").and_then(Value::as_bool) != Some(true) {
            continue;
        }
        let Some(name) = item.get("name").and_then(Value::as_str) else {
            continue;
        };
        push_entry(&mut dictionary, key, name);
        if let Some(components) = item.get("components").and_then(Value::as_array) {
            for component_key in components.iter().filter_map(Value::as_str) {
                let Some(component_name) = items
                    .get(component_key)
                    .and_then(|component| component.get("name"))
                    .and_then(Value::as_str)
                else {
                    continue;
                };
                push_entry(
                    &mut dictionary,
                    component_key,
                    &format!("{name} {component_name}"),
                );
            }
        }
    }
    Ok(dictionary)
}

fn push_entry(dictionary: &mut Vec<MasteryDictionaryEntry>, key: &str, name: &str) {
    let name = format!("{name} MAX RANK");
    let normalized_name = normalize_dictionary_text(&name);
    if !normalized_name.is_empty() {
        dictionary.push(MasteryDictionaryEntry {
            key: key.to_owned(),
            name,
            normalized_name,
        });
    }
}

pub fn map_mastery_words_to_dictionary<R: Runtime>(
    app: &AppHandle<R>,
    words: &[OcrWord],
    threshold: f64,
) -> Vec<OcrWord> {
    let state = app.state::<AppState>();
    let dictionary = match state.mastery_dictionary.lock() {
        Ok(dictionary) => dictionary,
        Err(_) => return words.to_vec(),
    };
    if dictionary.is_empty() {
        return words.to_vec();
    }
    words
        .iter()
        .filter_map(|word| match_mastery_word(word, &dictionary, threshold))
        .collect()
}

fn match_mastery_word(
    word: &OcrWord,
    dictionary: &[MasteryDictionaryEntry],
    threshold: f64,
) -> Option<OcrWord> {
    let normalized = normalize_dictionary_text(&word.text);
    if normalized.is_empty() {
        return None;
    }
    let mut best: Option<&MasteryDictionaryEntry> = None;
    let mut best_score = 0.0;
    let mut ambiguous = false;
    for candidate in dictionary {
        let score = similarity_score(&normalized, &candidate.normalized_name);
        if score > best_score + f64::EPSILON {
            best = Some(candidate);
            best_score = score;
            ambiguous = false;
        } else if (score - best_score).abs() <= f64::EPSILON
            && best.is_some_and(|entry| entry.key != candidate.key)
        {
            ambiguous = true;
        }
    }
    if best_score < threshold || ambiguous {
        return None;
    }
    let candidate = best?;
    let mut mapped = word.clone();
    mapped.text = candidate.name.clone();
    mapped.mastery_key = Some(candidate.key.clone());
    mapped.mapping_confidence = Some(best_score);
    Some(mapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_misread_roman_one() {
        let dictionary = vec![MasteryDictionaryEntry {
            key: "/Lotus/BratonI".into(),
            name: "Braton I MAX RANK".into(),
            normalized_name: normalize_dictionary_text("Braton I MAX RANK"),
        }];
        let word = OcrWord::new("Braton ] MAX RANK".into(), 0.0, 0.0, 0.0, 0.0);
        assert_eq!(
            match_mastery_word(&word, &dictionary, 0.86).and_then(|matched| matched.mastery_key),
            Some("/Lotus/BratonI".into()),
        );
    }

    #[test]
    fn builds_parent_and_component_names_from_catalog() {
        let catalog = serde_json::json!({
            "/Lotus/Weapon": {
                "name": "Example Prime",
                "masterable": true,
                "components": ["/Lotus/WeaponBarrel"]
            },
            "/Lotus/WeaponBarrel": { "name": "Barrel" }
        });
        let dictionary = build_mastery_dictionary(&catalog).unwrap();
        assert!(dictionary
            .iter()
            .any(|entry| entry.key == "/Lotus/Weapon" && entry.name == "Example Prime MAX RANK"));
        assert!(dictionary
            .iter()
            .any(|entry| entry.key == "/Lotus/WeaponBarrel"
                && entry.name == "Example Prime Barrel MAX RANK"));
    }

    #[test]
    fn duplicate_names_do_not_mark_an_arbitrary_item() {
        let dictionary = ["first", "second"]
            .into_iter()
            .map(|key| MasteryDictionaryEntry {
                key: key.into(),
                name: "Shared MAX RANK".into(),
                normalized_name: normalize_dictionary_text("Shared MAX RANK"),
            })
            .collect::<Vec<_>>();
        let word = OcrWord::new("Shared MAX RANK".into(), 0.0, 0.0, 0.0, 0.0);
        assert!(match_mastery_word(&word, &dictionary, 0.86).is_none());
    }
}
