import { invoke } from '@tauri-apps/api/core';
import { load } from '@tauri-apps/plugin-store';

import type {
	OcrDictionaryMappingSettingsPayload,
	OcrThemeSettingsPayload,
	OcrThemeOption,
} from '$lib/types';

// Utility to get the store
async function getStore() {
	return await load('settings.json', { autoSave: false } as any);
}

export async function getOcrThemeSettings(): Promise<OcrThemeSettingsPayload> {
	const themes = await invoke<OcrThemeOption[]>('get_ocr_themes');
	const store = await getStore();
	let selected_theme = await store.get<string>('ocr_theme');
	if (!selected_theme) selected_theme = 'EQUINOX';
	return { themes, selected_theme };
}

export async function setOcrTheme(theme: string): Promise<void> {
	const store = await getStore();
	await store.set('ocr_theme', theme);
	await store.save();
}

export async function getOverlayDurationSecs(): Promise<number> {
	const store = await getStore();
	const val = await store.get<number>('overlay_duration_secs');
	return val ?? 10;
}

export async function setOverlayDurationSecs(seconds: number): Promise<number> {
	const store = await getStore();
	await store.set('overlay_duration_secs', seconds);
	await store.save();
	return seconds;
}

export async function getOverlayToggleMode(): Promise<boolean> {
	const store = await getStore();
	const val = await store.get<boolean>('overlay_toggle_mode');
	return val ?? false;
}

export async function setOverlayToggleMode(enabled: boolean): Promise<boolean> {
	const store = await getStore();
	await store.set('overlay_toggle_mode', enabled);
	await store.save();
	return enabled;
}

export async function getOcrDictionaryMappingSettings(): Promise<OcrDictionaryMappingSettingsPayload> {
	const store = await getStore();
	const enabled = await store.get<boolean>('ocr_dictionary_mapping_enabled') ?? true;
	const threshold = await store.get<number>('ocr_dictionary_match_threshold') ?? 0.62;
	return {
		enabled,
		threshold,
		hard_disabled: false,
		min_threshold: 0.0,
		max_threshold: 1.0,
	};
}

export async function setOcrDictionaryMappingEnabled(enabled: boolean): Promise<boolean> {
	const store = await getStore();
	await store.set('ocr_dictionary_mapping_enabled', enabled);
	await store.save();
	return enabled;
}

export async function setOcrDictionaryMatchThreshold(threshold: number): Promise<number> {
	const store = await getStore();
	await store.set('ocr_dictionary_match_threshold', threshold);
	await store.save();
	return threshold;
}

export async function getWarframeLogPath(): Promise<string> {
	return invoke<string>('get_warframe_log_path');
}

export async function setWarframeLogPath(path: string): Promise<string> {
	return invoke<string>('set_warframe_log_path', { path });
}

export async function getRelicRewardDetection(): Promise<boolean> {
	const store = await getStore();
	const val = await store.get<boolean>('relic_reward_detection');
	return val ?? false;
}

export async function setRelicRewardDetection(enabled: boolean): Promise<boolean> {
	const store = await getStore();
	await store.set('relic_reward_detection', enabled);
	await store.save();
	return enabled;
}

export async function getShowOcrBoundingBoxes(): Promise<boolean> {
	const store = await getStore();
	const val = await store.get<boolean>('show_ocr_bounding_boxes');
	return val ?? false;
}

export async function setShowOcrBoundingBoxes(enabled: boolean): Promise<boolean> {
	const store = await getStore();
	await store.set('show_ocr_bounding_boxes', enabled);
	await store.save();
	return enabled;
}
