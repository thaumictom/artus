import { invoke } from '@tauri-apps/api/core';

import type { OcrDictionaryMappingSettingsPayload, OcrThemeSettingsPayload } from '$lib/types';

export const MIN_OVERLAY_DURATION_SECS = 1;
export const MAX_OVERLAY_DURATION_SECS = 60;

export function getOcrThemeSettings() {
	return invoke<OcrThemeSettingsPayload>('get_ocr_theme_settings');
}

export function setOcrTheme(theme: string) {
	return invoke<void>('set_ocr_theme', { theme });
}

export function getOverlayDurationSecs() {
	return invoke<number>('get_overlay_duration_secs');
}

export function setOverlayDurationSecs(seconds: number) {
	return invoke<number>('set_overlay_duration_secs', { seconds });
}

export function getOverlayToggleMode() {
	return invoke<boolean>('get_overlay_toggle_mode');
}

export function setOverlayToggleMode(enabled: boolean) {
	return invoke<boolean>('set_overlay_toggle_mode', { enabled });
}

export function getOcrDictionaryMappingSettings() {
	return invoke<OcrDictionaryMappingSettingsPayload>('get_ocr_dictionary_mapping_settings');
}

export function setOcrDictionaryMappingEnabled(enabled: boolean) {
	return invoke<boolean>('set_ocr_dictionary_mapping_enabled', { enabled });
}

export function setOcrDictionaryMatchThreshold(threshold: number) {
	return invoke<number>('set_ocr_dictionary_match_threshold', { threshold });
}
