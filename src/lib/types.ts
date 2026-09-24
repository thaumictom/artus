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

export interface SettingsPayload {
	ocr_theme: OcrThemeSettingsPayload;
	overlay_duration_secs: number;
	overlay_toggle_mode: boolean;
	ocr_dictionary_mapping: OcrDictionaryMappingSettingsPayload;
	relic_reward_detection: boolean;
	visual_relic_reward_detection: boolean;
	relic_reward_sound: boolean;
	show_ocr_bounding_boxes: boolean;
}

export interface SettingsPatchPayload {
	ocr_theme?: string;
	overlay_duration_secs?: number;
	overlay_toggle_mode?: boolean;
	ocr_dictionary_mapping_enabled?: boolean;
	ocr_dictionary_match_threshold?: number;
	ocr_mastery_dictionary_match_threshold?: number;
	relic_reward_detection?: boolean;
	visual_relic_reward_detection?: boolean;
	relic_reward_sound?: boolean;
	show_ocr_bounding_boxes?: boolean;
}
