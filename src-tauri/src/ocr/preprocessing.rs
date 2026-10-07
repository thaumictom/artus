//! Image preprocessing for OCR: binary color filtering and morphological ops.

use image::{DynamicImage, GrayImage, ImageFormat};
use imageproc::distance_transform::Norm;
use imageproc::morphology::{dilate_mut, erode_mut};
use std::io::Cursor;
use std::sync::OnceLock;

use super::{BINARY_FILTER_SPILL_THRESHOLD, ENABLE_MORPHOLOGY};
use crate::error::AppResult;

pub const MOD_COLOR_GOLD: [u8; 3] = [253, 235, 189];
pub const MOD_COLOR_SILVER: [u8; 3] = [228, 228, 228];
pub const MOD_COLOR_BRONZE: [u8; 3] = [221, 160, 133];
pub const MOD_COLOR_ARCHON: [u8; 3] = [190, 169, 102];
pub const MOD_COLOR_SPECIAL: [u8; 3] = [255, 255, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckmarkMatch {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Produces a binary (black/white) image where pixels matching any of the `target_rgbs`
/// become black (foreground) and everything else becomes white (background).
pub fn binary_target_filter(source: &image::RgbaImage, target_rgbs: &[[u8; 3]]) -> GrayImage {
    binary_target_filter_with_tolerance(source, target_rgbs, BINARY_FILTER_SPILL_THRESHOLD)
}

/// Binary filter with a caller-specific per-channel tolerance for noisy UI colors.
pub fn binary_target_filter_with_tolerance(
    source: &image::RgbaImage,
    target_rgbs: &[[u8; 3]],
    tolerance: u8,
) -> GrayImage {
    let width = source.width();
    let height = source.height();
    let raw = source.as_raw();
    let mut output = vec![255u8; (width * height) as usize];

    for (i, pixel) in raw.chunks_exact(4).enumerate() {
        let mut matched = false;
        for target_rgb in target_rgbs {
            if pixel[0].abs_diff(target_rgb[0]) <= tolerance
                && pixel[1].abs_diff(target_rgb[1]) <= tolerance
                && pixel[2].abs_diff(target_rgb[2]) <= tolerance
            {
                matched = true;
                break;
            }
        }
        if matched {
            output[i] = 0;
        }
    }

    GrayImage::from_raw(width, height, output).expect("invalid binary filter output dimensions")
}

struct CheckmarkTemplate {
    width: u32,
    height: u32,
    foreground: Vec<(u32, u32)>,
    icon_size: u32,
    offset_x: u32,
    offset_y: u32,
}

/// Matches only the inner check, then erases the full icon before OCR.
/// Confidence is the lower of foreground and background agreement, so blank
/// space or a solid dark patch cannot dominate the score.
pub fn remove_checkmarks(image: &mut GrayImage, threshold: f64) -> Vec<CheckmarkMatch> {
    static TEMPLATES: OnceLock<Vec<CheckmarkTemplate>> = OnceLock::new();
    let templates = TEMPLATES.get_or_init(|| {
        let template = image::load_from_memory(include_bytes!("../checkmark_template_3.png"))
            .expect("invalid checkmark template")
            .into_rgba8();
        // Transparent pixels in this cutout are background, not black pixels.
        let binary = GrayImage::from_fn(template.width(), template.height(), |x, y| {
            let pixel = template.get_pixel(x, y);
            image::Luma([if pixel[3] > 0 && pixel[0] < 128 {
                0
            } else {
                255
            }])
        });
        // Filtering can thin the check itself as well as remove its ring.
        // Match both stroke widths so a thin, large check does not win as a
        // smaller match covering only part of its longer diagonal.
        let mut thin = binary.clone();
        dilate_mut(&mut thin, Norm::L1, 1);
        // Cover 60–200% of the original UI size, including both supplied samples.
        [binary, thin]
            .into_iter()
            .flat_map(|binary| {
                (6..=20).map(move |step| {
                    let scale = step as f64 / 10.0;
                    let width = (binary.width() as f64 * scale).round() as u32;
                    let height = (binary.height() as f64 * scale).round() as u32;
                    let scaled = image::imageops::resize(
                        &binary,
                        width,
                        height,
                        image::imageops::FilterType::Nearest,
                    );
                    let foreground = scaled
                        .enumerate_pixels()
                        .filter(|(_, _, pixel)| pixel[0] == 0)
                        .map(|(x, y, _)| (x, y))
                        .collect();
                    // The 20×20 cutout starts at (7, 4) inside the original 31×31 icon.
                    // Keep full icon geometry for erasure and the adjacent quantity crop.
                    CheckmarkTemplate {
                        width,
                        height,
                        foreground,
                        icon_size: (31.0 * scale).round() as u32,
                        offset_x: (7.0 * scale).round() as u32,
                        offset_y: (4.0 * scale).round() as u32,
                    }
                })
            })
            .collect()
    });
    let (width, height) = image.dimensions();
    let threshold = threshold.clamp(0.5, 1.0);
    let stride = width as usize + 1;
    let raw = image.as_raw();
    // Summed foreground counts reject impossible matches in constant time.
    let mut counts = vec![0u32; stride * (height as usize + 1)];
    for y in 0..height as usize {
        let mut row_sum = 0;
        for x in 0..width as usize {
            row_sum += u32::from(raw[y * width as usize + x] == 0);
            counts[(y + 1) * stride + x + 1] = counts[y * stride + x + 1] + row_sum;
        }
    }

    let mut candidates = Vec::new();
    for template in templates {
        let (tw, th) = (template.width, template.height);
        if width < tw || height < th {
            continue;
        }
        let black_total = template.foreground.len();
        let white_total = (tw * th) as usize - black_total;
        let required_black = (black_total as f64 * threshold).ceil() as usize;
        let allowed_white_errors = (white_total as f64 * (1.0 - threshold)).floor() as usize;
        for y in 0..=height - th {
            for x in 0..=width - tw {
                let left = x as usize;
                let right = (x + tw) as usize;
                let top = y as usize * stride;
                let bottom = (y + th) as usize * stride;
                let patch_black = (counts[bottom + right]
                    - counts[top + right]
                    - (counts[bottom + left] - counts[top + left]))
                    as usize;
                if patch_black < required_black || patch_black > black_total + allowed_white_errors
                {
                    continue;
                }
                let mut black = 0;
                let mut misses = 0;
                for &(tx, ty) in &template.foreground {
                    if raw[(y + ty) as usize * width as usize + (x + tx) as usize] == 0 {
                        black += 1;
                    } else {
                        misses += 1;
                        if misses > black_total - required_black {
                            break;
                        }
                    }
                }
                if black < required_black || patch_black - black > allowed_white_errors {
                    continue;
                }
                let confidence = (black as f64 / black_total as f64)
                    .min(1.0 - (patch_black - black) as f64 / white_total as f64);
                let icon_x = x.saturating_sub(template.offset_x);
                let icon_y = y.saturating_sub(template.offset_y);
                candidates.push((
                    confidence,
                    CheckmarkMatch {
                        x: icon_x,
                        y: icon_y,
                        width: (x + template.icon_size - template.offset_x).min(width) - icon_x,
                        height: (y + template.icon_size - template.offset_y).min(height) - icon_y,
                    },
                ));
            }
        }
    }
    // Select the strongest match across positions and scales before modifying
    // the image, so one icon cannot be detected repeatedly at different sizes.
    candidates.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
    let mut matches: Vec<CheckmarkMatch> = Vec::new();
    for (_, mark) in candidates {
        if matches.iter().any(|other| {
            mark.x < other.x + other.width
                && other.x < mark.x + mark.width
                && mark.y < other.y + other.height
                && other.y < mark.y + mark.height
        }) {
            continue;
        }
        matches.push(mark);
        // Clear a two-pixel margin too, so ring remnants cannot become OCR digits.
        let left = mark.x.saturating_sub(2);
        let top = mark.y.saturating_sub(2);
        let right = (mark.x + mark.width).saturating_add(2).min(width);
        let bottom = (mark.y + mark.height).saturating_add(2).min(height);
        for y in top..bottom {
            let start = y as usize * width as usize + left as usize;
            image.as_mut()[start..start + (right - left) as usize].fill(255);
        }
    }
    matches.sort_unstable_by_key(|mark| (mark.y, mark.x));
    matches
}

#[cfg(test)]
mod checkmark_tests {
    use super::*;

    #[test]
    fn erases_checkmark_without_erasing_other_pixels() {
        let template = image::load_from_memory(include_bytes!("../checkmark_template.png"))
            .unwrap()
            .into_luma8();
        let mut image = GrayImage::from_pixel(60, 40, image::Luma([255]));
        for (x, y, pixel) in template.enumerate_pixels() {
            if pixel[0] == 0 {
                image.put_pixel(x + 5, y + 5, image::Luma([0]));
            }
        }
        image.put_pixel(50, 20, image::Luma([0]));

        let matches = remove_checkmarks(&mut image, 0.8);

        assert_eq!(
            matches,
            vec![CheckmarkMatch {
                x: 5,
                y: 5,
                width: 31,
                height: 31
            }]
        );
        assert!(image
            .enumerate_pixels()
            .all(|(x, y, pixel)| (x, y) == (50, 20) || pixel[0] == 255));
    }
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

/// Encodes a grayscale image as PNG bytes (used for the developer image overlay).
pub fn gray_to_png_bytes(gray: &GrayImage) -> AppResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    DynamicImage::ImageLuma8(gray.clone())
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|err| {
            crate::error::AppError::msg(format!("failed to encode developer image: {err}"))
        })?;
    Ok(bytes)
}
