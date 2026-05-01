import { invoke } from '@tauri-apps/api/core';

import type {
	OcrDictionaryMappingSettingsPayload,
	OcrThemeSettingsPayload,
	SettingsPatchPayload,
	SettingsPayload,
} from '$lib/types';

let cachedSettings: SettingsPayload | null = null;
let inFlightSettingsRequest: Promise<SettingsPayload> | null = null;

export async function getSettings(force = false): Promise<SettingsPayload> {
	if (!force && cachedSettings) {
		return cachedSettings;
	}

	if (!force && inFlightSettingsRequest) {
		return inFlightSettingsRequest;
	}

	inFlightSettingsRequest = invoke<SettingsPayload>('get_settings')
		.then((settings) => {
			cachedSettings = settings;
			return settings;
		})
		.finally(() => {
			inFlightSettingsRequest = null;
		});

	return inFlightSettingsRequest;
}

export async function patchSettings(patch: SettingsPatchPayload): Promise<SettingsPayload> {
	const settings = await invoke<SettingsPayload>('patch_settings', { patch });
	cachedSettings = settings;
	return settings;
}

export function getOcrThemeSettings() {
	return getSettings().then((settings) => settings.ocr_theme);
}

export function setOcrTheme(theme: string) {
	return patchSettings({ ocr_theme: theme }).then(() => undefined);
}

export function getOverlayDurationSecs() {
	return getSettings().then((settings) => settings.overlay_duration_secs);
}

export function setOverlayDurationSecs(seconds: number) {
	return patchSettings({ overlay_duration_secs: seconds }).then(
		(settings) => settings.overlay_duration_secs,
	);
}

export function getOverlayToggleMode() {
	return getSettings().then((settings) => settings.overlay_toggle_mode);
}

export function setOverlayToggleMode(enabled: boolean) {
	return patchSettings({ overlay_toggle_mode: enabled }).then(
		(settings) => settings.overlay_toggle_mode,
	);
}

export function getOcrDictionaryMappingSettings() {
	return getSettings().then((settings) => settings.ocr_dictionary_mapping);
}

export function setOcrDictionaryMappingEnabled(enabled: boolean) {
	return patchSettings({ ocr_dictionary_mapping_enabled: enabled }).then(
		(settings) => settings.ocr_dictionary_mapping.enabled,
	);
}

export function setOcrDictionaryMatchThreshold(threshold: number) {
	return patchSettings({ ocr_dictionary_match_threshold: threshold }).then(
		(settings) => settings.ocr_dictionary_mapping.threshold,
	);
}

export function getWarframeLogPath() {
	return getSettings().then((settings) => settings.warframe_log_path);
}

export function setWarframeLogPath(path: string) {
	return patchSettings({ warframe_log_path: path }).then((settings) => settings.warframe_log_path);
}

export function getRelicRewardDetection() {
	return getSettings().then((settings) => settings.relic_reward_detection);
}

export function setRelicRewardDetection(enabled: boolean) {
	return patchSettings({ relic_reward_detection: enabled }).then(
		(settings) => settings.relic_reward_detection,
	);
}

export function getShowOcrBoundingBoxes() {
	return getSettings().then((settings) => settings.show_ocr_bounding_boxes);
}

export function setShowOcrBoundingBoxes(enabled: boolean) {
	return patchSettings({ show_ocr_bounding_boxes: enabled }).then(
		(settings) => settings.show_ocr_bounding_boxes,
	);
}
