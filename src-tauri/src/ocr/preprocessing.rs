//! Image preprocessing for OCR: binary color filtering and morphological ops.

use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use std::io::Cursor;

use crate::error::AppResult;
use super::{BINARY_FILTER_SPILL_THRESHOLD, ENABLE_MORPHOLOGY};

/// Produces a binary (black/white) image where pixels matching `target_rgb`
/// become black (foreground) and everything else becomes white (background).
pub fn binary_target_filter(source: &image::RgbaImage, target_rgb: [u8; 3]) -> GrayImage {
    let width = source.width();
    let height = source.height();
    let raw = source.as_raw();
    let mut output = vec![255u8; (width * height) as usize];

    for (i, pixel) in raw.chunks_exact(4).enumerate() {
        if matches_target_color(
            pixel[0], pixel[1], pixel[2],
            target_rgb[0], target_rgb[1], target_rgb[2],
        ) {
            output[i] = 0;
        }
    }

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
fn matches_target_color(r: u8, g: u8, b: u8, tr: u8, tg: u8, tb: u8) -> bool {
    r.abs_diff(tr) <= BINARY_FILTER_SPILL_THRESHOLD
        && g.abs_diff(tg) <= BINARY_FILTER_SPILL_THRESHOLD
        && b.abs_diff(tb) <= BINARY_FILTER_SPILL_THRESHOLD
}

/// Encodes a grayscale image as PNG bytes (used for the debug image overlay).
pub fn gray_to_png_bytes(gray: &GrayImage) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| crate::error::AppError::msg(format!("failed to encode debug image: {err}")))?;
    Ok(bytes)
}
