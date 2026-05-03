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
};

// 1. Define the reactive state globally
export const config = $state({
	hotkeys: {
		screenshot: 'control+Home',
		screenshot_add_inventory: 'shift+control+Home',
	},
	show_ocr_bounding_boxes: false,
	overlay_duration_secs: 25,
	ocr_dictionary_mapping_enabled: true,
	ocr_theme: 'EQUINOX',
	ocr_dictionary_match_threshold: 0.62,
	relic_reward_detection: false,
	warframe_log_path: '%LocalAppData%\\Warframe\\EE.log',
	overlay_toggle_mode: true,
	capture_mods: false,
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
