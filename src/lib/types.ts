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
