// src/lib/settings.svelte.ts
import { LazyStore } from '@tauri-apps/plugin-store';

const store = new LazyStore('settings.json');

type Config = {
	hotkeys: {
		[action: string]: string;
	};

	warframe_log_path: string;
	relic_reward_detection: boolean;

	ocr_theme: string;
	overlay_toggle_mode: boolean;
	overlay_duration_secs: number;

	show_ocr_bounding_boxes: boolean;
	ocr_dictionary_mapping_enabled: boolean;
	ocr_dictionary_match_threshold: number;
	capture_mods: boolean;
	hide_overlay_on_focus_loss: boolean;

	ocr_max_x_gap_multiplier: number;
	ocr_max_y_gap_multiplier: number;
	ocr_vertical_column_tolerance: number;
	ocr_row_bucket_y_tolerance: number;

	threshold_100: [number, number];
	threshold_65: [number, number];
	threshold_45: [number, number];
	threshold_25: [number, number];
	threshold_15: [number, number];
};

// 1. Define the reactive state globally
export const config = $state({
	hotkeys: {
		screenshot: 'control+Home',
		screenshot_add_inventory: 'shift+control+Home',
	},

	// Warframe settings
	ocr_theme: 'EQUINOX',
	warframe_log_path: '%LocalAppData%\\Warframe\\EE.log',
	relic_reward_detection: false,

	// Overlay settings
	hide_overlay_on_focus_loss: true,
	overlay_toggle_mode: true,
	overlay_duration_secs: 25,
	capture_mods: false,

	// Debug settings
	show_ocr_bounding_boxes: false,

	ocr_max_x_gap_multiplier: 1.0,
	ocr_max_y_gap_multiplier: 2.0,
	ocr_vertical_column_tolerance: 2.5,
	ocr_row_bucket_y_tolerance: 0.6,

	ocr_dictionary_mapping_enabled: true,
	ocr_dictionary_match_threshold: 0.62,

	// Ducat price/salvage ratio settings
	threshold_100: [10, 15],
	threshold_65: [8, 12],
	threshold_45: [6, 12],
	threshold_25: [5, 10],
	threshold_15: [5, 8],
}) satisfies Config;

// 2. Export the initialization logic
export async function loadSettings() {
	const savedEntries = await store.entries();
	for (const [key, val] of savedEntries) {
		if (key in config) {
			// @ts-ignore
			config[key] = val;
		}
	}
}

// 3. Export the update logic
export async function updateSetting(key: keyof typeof config) {
	console.log(`Updating setting ${key} to`, config[key]);
	await store.set(key, config[key]);
	await store.save();
}
