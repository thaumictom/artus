import type { Component } from 'svelte';

export type Sections = Record<string, { label: string; icon: string; component: Component }>;

export type OcrThemeOption = {
	name: string;
	rgb: [number, number, number];
};

export type OcrThemeSettingsPayload = {
	themes: OcrThemeOption[];
	selected_theme: string;
};

export type OcrDictionaryMappingSettingsPayload = {
	enabled: boolean;
	threshold: number;
	hard_disabled: boolean;
	min_threshold: number;
	max_threshold: number;
};

export type MarketStatEntry = {
	datetime: string;
	volume: number;
	min_price: number;
	max_price: number;
	open_price: number;
	closed_price: number;
	avg_price: number;
	wa_price: number;
	median: number;
	subtype?: string;
	donch_top?: number;
	donch_bot?: number;
	id: string;
	moving_avg?: number;
	mod_rank?: number;
};

export type SettingsPayload = {
	ocr_theme: OcrThemeSettingsPayload;
	overlay_duration_secs: number;
	overlay_toggle_mode: boolean;
	ocr_dictionary_mapping: OcrDictionaryMappingSettingsPayload;
	warframe_log_path: string;
	relic_reward_detection: boolean;
};

export type SettingsPatchPayload = {
	ocr_theme?: string;
	overlay_duration_secs?: number;
	overlay_toggle_mode?: boolean;
	ocr_dictionary_mapping_enabled?: boolean;
	ocr_dictionary_match_threshold?: number;
	warframe_log_path?: string;
	relic_reward_detection?: boolean;
};
