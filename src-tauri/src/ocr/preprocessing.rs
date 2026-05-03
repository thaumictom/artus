//! Image preprocessing for OCR: binary color filtering and morphological ops.

use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use rayon::prelude::*;
use std::io::Cursor;

use super::{BINARY_FILTER_SPILL_THRESHOLD, ENABLE_MORPHOLOGY};
use crate::error::AppResult;

pub const MOD_COLOR_GOLD: [u8; 3] = [253, 235, 189];
pub const MOD_COLOR_SILVER: [u8; 3] = [228, 228, 228];
pub const MOD_COLOR_BRONZE: [u8; 3] = [221, 160, 133];
pub const MOD_COLOR_ARCHON: [u8; 3] = [190, 169, 102];
pub const MOD_COLOR_SPECIAL: [u8; 3] = [255, 255, 255];

/// Produces a binary (black/white) image where pixels matching any of the `target_rgbs`
/// become black (foreground) and everything else becomes white (background).
pub fn binary_target_filter(source: &image::RgbaImage, target_rgbs: &[[u8; 3]]) -> GrayImage {
    let width = source.width();
    let height = source.height();
    let raw = source.as_raw();

    let output: Vec<u8> = raw
        .par_chunks_exact(4)
        .map(|pixel| {
            let mut matched = false;
            for target_rgb in target_rgbs {
                if matches_target_color(
                    pixel[0],
                    pixel[1],
                    pixel[2],
                    target_rgb[0],
                    target_rgb[1],
                    target_rgb[2],
                ) {
                    matched = true;
                    break;
                }
            }
            if matched {
                0
            } else {
                255
            }
        })
        .collect();

    GrayImage::from_raw(width, height, output).expect("invalid binary filter output dimensions")
}

/// Applies erosion followed by dilation to remove noise.
/// Currently gated behind [`ENABLE_MORPHOLOGY`].
pub fn apply_morphology(source: &mut GrayImage) {
    if !ENABLE_MORPHOLOGY {
        return;
    }
    erode_mut(source, Norm::L1, 1);
    dilate_mut(source, Norm::L1, 1);
}

/// Returns `true` if each channel is within [`BINARY_FILTER_SPILL_THRESHOLD`]
/// of the target value.
pub fn matches_target_color(r: u8, g: u8, b: u8, tr: u8, tg: u8, tb: u8) -> bool {
    r.abs_diff(tr) <= BINARY_FILTER_SPILL_THRESHOLD
        && g.abs_diff(tg) <= BINARY_FILTER_SPILL_THRESHOLD
        && b.abs_diff(tb) <= BINARY_FILTER_SPILL_THRESHOLD
}

/// Identifies the mod type based on the most prevalent mod color in the word's bounding box.
pub fn identify_mod_type(
    image: &image::RgbaImage,
    word: &crate::ocr::OcrWord,
) -> Option<crate::ocr::ModType> {
    let x_start = word.x.max(0.0) as u32;
    let y_start = word.y.max(0.0) as u32;
    let x_end = (word.x + word.width).min(image.width() as f64) as u32;
    let y_end = (word.y + word.height).min(image.height() as f64) as u32;

    let mut gold = 0;
    let mut silver = 0;
    let mut bronze = 0;
    let mut archon = 0;
    let mut special = 0;

    for y in y_start..y_end {
        for x in x_start..x_end {
            let pixel = image.get_pixel(x, y);
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];

            if matches_target_color(
                r,
                g,
                b,
                MOD_COLOR_GOLD[0],
                MOD_COLOR_GOLD[1],
                MOD_COLOR_GOLD[2],
            ) {
                gold += 1;
            } else if matches_target_color(
                r,
                g,
                b,
                MOD_COLOR_SILVER[0],
                MOD_COLOR_SILVER[1],
                MOD_COLOR_SILVER[2],
            ) {
                silver += 1;
            } else if matches_target_color(
                r,
                g,
                b,
                MOD_COLOR_BRONZE[0],
                MOD_COLOR_BRONZE[1],
                MOD_COLOR_BRONZE[2],
            ) {
                bronze += 1;
            } else if matches_target_color(
                r,
                g,
                b,
                MOD_COLOR_ARCHON[0],
                MOD_COLOR_ARCHON[1],
                MOD_COLOR_ARCHON[2],
            ) {
                archon += 1;
            } else if matches_target_color(
                r,
                g,
                b,
                MOD_COLOR_SPECIAL[0],
                MOD_COLOR_SPECIAL[1],
                MOD_COLOR_SPECIAL[2],
            ) {
                special += 1;
            }
        }
    }

    let max = gold.max(silver).max(bronze).max(archon).max(special);
    if max == 0 {
        return None;
    }

    if max == gold {
        Some(crate::ocr::ModType::Gold)
    } else if max == silver {
        Some(crate::ocr::ModType::Silver)
    } else if max == bronze {
        Some(crate::ocr::ModType::Bronze)
    } else if max == archon {
        Some(crate::ocr::ModType::Archon)
    } else {
        Some(crate::ocr::ModType::Special)
    }
}

/// Encodes a grayscale image as PNG bytes (used for the debug image overlay).
pub fn gray_to_png_bytes(gray: &GrayImage) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            crate::error::AppError::msg(format!("failed to encode debug image: {err}"))
        })?;
    Ok(bytes)
}
