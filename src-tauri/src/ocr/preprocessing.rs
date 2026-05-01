use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use std::io::Cursor;

use super::{BINARY_FILTER_SPILL_THRESHOLD, ENABLE_MORPHOLOGY};

pub fn binary_target_filter(source: &image::RgbaImage, target_rgb: [u8; 3]) -> GrayImage {
    let width = source.width();
    let height = source.height();
    let raw = source.as_raw();
    let mut output = vec![255u8; (width * height) as usize];
    
    for (i, pixel) in raw.chunks_exact(4).enumerate() {
        if matches_target_color(pixel[0], pixel[1], pixel[2], target_rgb[0], target_rgb[1], target_rgb[2]) {
            output[i] = 0;
        }
    }
    
    GrayImage::from_raw(width, height, output).expect("invalid binary filter output dimensions")
}

pub fn apply_morphology(source: &mut GrayImage) {
    if !ENABLE_MORPHOLOGY {
        return;
    }

    // Experimental
    erode_mut(source, Norm::L1, 1);
    dilate_mut(source, Norm::L1, 1);
}

pub fn matches_target_color(r: u8, g: u8, b: u8, target_r: u8, target_g: u8, target_b: u8) -> bool {
    r.abs_diff(target_r) <= BINARY_FILTER_SPILL_THRESHOLD
        && g.abs_diff(target_g) <= BINARY_FILTER_SPILL_THRESHOLD
        && b.abs_diff(target_b) <= BINARY_FILTER_SPILL_THRESHOLD
}

pub fn gray_to_png_bytes(gray: &GrayImage) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| format!("failed to encode debug image: {err}"))?;
    Ok(bytes)
}
