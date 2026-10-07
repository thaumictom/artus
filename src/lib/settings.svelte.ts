// src/lib/settings.svelte.ts
import { LazyStore } from '@tauri-apps/plugin-store';
import { isCustomTheme, type CustomTheme } from '$lib/custom-theme';
import { isAppTheme, isThemeMode, isAccentColor, isAccentShade, type AppTheme, type ThemeMode, type AccentColor, type AccentShade } from '$lib/app-themes';
import { isQuicklistStrategy, type QuicklistStrategy } from '$lib/quicklist';

const store = new LazyStore('settings.json');
let settingsLoadPromise: Promise<void> | null = null;

export type FissureNotificationCategory = 'normal' | 'steelPath' | 'voidStorm';

export type NotificationRules = {
	fissures: {
		enabled: boolean;
		eras: string[];
		missionTypes: string[];
		categories: FissureNotificationCategory[];
	};
	alerts: boolean;
	invasions: boolean;
	invasionExcludeCommonRewards: boolean;
	dailyDeals: boolean;
	baro: boolean;
};

export const defaultNotificationRules = (): NotificationRules => ({
	fissures: {
		enabled: false,
		eras: [],
		missionTypes: [],
		categories: [],
	},
	alerts: false,
	invasions: false,
	invasionExcludeCommonRewards: true,
	dailyDeals: false,
	baro: false,
});

type Config = {
	app_theme_custom_enabled: boolean;
	app_theme_custom_colors: CustomTheme | null;
	app_theme: AppTheme;
	app_theme_mode: ThemeMode;
	app_theme_accent: AccentColor;
	app_theme_accent_shade: AccentShade;
	overlay_opposite_theme: boolean;
	full_width_content: boolean;
	sidebar_open: boolean;
	hotkeys: {
		[action: string]: string;
	};

	relic_reward_detection: boolean;
	relic_reward_auto_add: boolean;
	visual_relic_reward_detection: boolean;
	relic_reward_sound: boolean;
	dashboard_view_favorites: string[];
	dashboard_navigation_width: number;
	show_unused_dashboard_views: boolean;
	show_debug_settings: boolean;
	desktop_notifications_enabled: boolean;
	hide_to_tray_on_close: boolean;
	hide_donate_button: boolean;
	notification_sound: boolean;
	notification_rules: NotificationRules;

	ocr_theme: string;
	overlay_toggle_mode: boolean;
	overlay_duration_secs: number;
	show_set_prices: boolean;
	show_max_rank_prices: boolean;
	show_max_rank_mod_prices: boolean;
	quicklist_hide_first: boolean;
	quicklist_price_strategy: QuicklistStrategy;

	show_ocr_bounding_boxes: boolean;
	ocr_checkmark_match_threshold: number;
	ocr_dictionary_mapping_enabled: boolean;
	ocr_dictionary_match_threshold: number;
	ocr_mastery_dictionary_match_threshold: number;
	capture_mods: boolean;
	hide_overlay_on_focus_loss: boolean;
	cleanup_on_focus_loss: boolean;
	use_window_ownership: boolean;

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

// Fresh settings use dark Mist with Emerald 500; valid saved values load unchanged.
export const config = $state({
	app_theme_custom_enabled: false as boolean,
	app_theme_custom_colors: null as CustomTheme | null,
	app_theme: 'mist' as AppTheme,
	app_theme_mode: 'dark' as ThemeMode,
	app_theme_accent: 'emerald' as AccentColor,
	app_theme_accent_shade: '500' as AccentShade,
	overlay_opposite_theme: false as boolean,
	full_width_content: false as boolean,
	sidebar_open: false as boolean,
	hotkeys: {
		screenshot: 'control+Home',
		screenshot_add_mastery: 'alt+control+Home',
		cycle: 'tab',
		cycle_back: 'shift+tab',
		navigate_up: 'w',
		navigate_left: 'a',
		navigate_down: 's',
		navigate_right: 'd',
		inventory_decrement: 'q',
		inventory_increment: 'e',
		create_sell_listing: 'r',
		quicklist: 'f',
		listing_confirm: 'enter',
	},

	// Warframe settings
	ocr_theme: 'EQUINOX',
	relic_reward_detection: false as boolean,
	relic_reward_auto_add: true as boolean,
	visual_relic_reward_detection: false as boolean,
	relic_reward_sound: false as boolean,
	dashboard_view_favorites: ['WorldCycles', 'fissures', 'Alerts'] as string[],
	dashboard_navigation_width: 240,
	desktop_notifications_enabled: true as boolean,
	hide_to_tray_on_close: true as boolean,
	hide_donate_button: false as boolean,
	notification_sound: false as boolean,
	notification_rules: defaultNotificationRules(),

	// Overlay settings
	hide_overlay_on_focus_loss: true,
	cleanup_on_focus_loss: false,
	use_window_ownership: false as boolean,
	overlay_toggle_mode: true,
	overlay_duration_secs: 25,
	show_set_prices: true as boolean,
	show_max_rank_prices: true as boolean,
	show_max_rank_mod_prices: false as boolean,
	quicklist_hide_first: false as boolean,
	quicklist_price_strategy: 'undercut_at_or_above_median' as QuicklistStrategy,
	capture_mods: false,

	// Debug settings
	show_debug_settings: false as boolean,
	show_unused_dashboard_views: false,
	show_ocr_bounding_boxes: false,
	ocr_checkmark_match_threshold: 0.8,

	ocr_max_x_gap_multiplier: 1.0,
	ocr_max_y_gap_multiplier: 2.0,
	ocr_vertical_column_tolerance: 2.5,
	ocr_row_bucket_y_tolerance: 0.6,

	ocr_dictionary_mapping_enabled: true,
	ocr_dictionary_match_threshold: 0.62,
	ocr_mastery_dictionary_match_threshold: 0.86,

	// Ducat price/salvage ratio settings
	threshold_100: [10, 15],
	threshold_65: [8, 12],
	threshold_45: [6, 12],
	threshold_25: [5, 10],
	threshold_15: [5, 8],
}) satisfies Config;

// 2. Export the initialization logic
export function loadSettings() {
	if (settingsLoadPromise) return settingsLoadPromise;
	settingsLoadPromise = (async () => {
		const savedEntries = await store.entries();
		for (const [key, val] of savedEntries) {
			if (key in config) {
				if (key === 'app_theme_custom_enabled' && typeof val !== 'boolean') continue;
				if (key === 'app_theme_custom_colors' && val !== null && !isCustomTheme(val)) continue;
				// Obsolete theme names are ignored rather than mapped to a current theme.
				if (key === 'app_theme' && !isAppTheme(val)) continue;
				if (key === 'app_theme_mode' && !isThemeMode(val)) continue;
				if (key === 'app_theme_accent' && !isAccentColor(val)) continue;
				if (key === 'app_theme_accent_shade' && !isAccentShade(val)) continue;
				if (key === 'overlay_opposite_theme' && typeof val !== 'boolean') continue;
				if (key === 'quicklist_price_strategy' && !isQuicklistStrategy(val)) continue;
				// @ts-ignore
				config[key] = val;
			}
		}
		config.hotkeys = {
			screenshot: config.hotkeys?.screenshot ?? 'control+Home',
			screenshot_add_mastery: config.hotkeys?.screenshot_add_mastery ?? 'alt+control+Home',
			cycle: config.hotkeys?.cycle ?? 'tab',
			cycle_back: config.hotkeys?.cycle_back ?? 'shift+tab',
			navigate_up: config.hotkeys?.navigate_up ?? 'w',
			navigate_left: config.hotkeys?.navigate_left ?? 'a',
			navigate_down: config.hotkeys?.navigate_down ?? 's',
			navigate_right: config.hotkeys?.navigate_right ?? 'd',
			inventory_decrement: config.hotkeys?.inventory_decrement ?? 'q',
			inventory_increment: config.hotkeys?.inventory_increment ?? 'e',
			create_sell_listing: config.hotkeys?.create_sell_listing ?? 'r',
			quicklist: config.hotkeys?.quicklist ?? 'f',
			listing_confirm: config.hotkeys?.listing_confirm ?? 'enter',
		};

	})();
	return settingsLoadPromise;
}

export function watchAppTheme() {
	return store.onChange<unknown>((key, value) => {
		if (key === 'app_theme_custom_enabled' && typeof value === 'boolean') config.app_theme_custom_enabled = value;
		if (key === 'app_theme_custom_colors' && (value === null || isCustomTheme(value))) config.app_theme_custom_colors = value;
		if (key === 'app_theme' && isAppTheme(value)) config.app_theme = value;
		if (key === 'app_theme_mode' && isThemeMode(value)) config.app_theme_mode = value;
		if (key === 'app_theme_accent' && isAccentColor(value)) config.app_theme_accent = value;
		if (key === 'app_theme_accent_shade' && isAccentShade(value)) config.app_theme_accent_shade = value;
		if (key === 'overlay_opposite_theme' && typeof value === 'boolean') config.overlay_opposite_theme = value;
	});
}

// Overlay windows have their own state; keep these display toggles in sync.
export function watchOverlayPriceSettings() {
	return store.onChange<unknown>((key, value) => {
		if (key === 'show_set_prices' || key === 'show_max_rank_prices') {
			config[key] = typeof value === 'boolean' ? value : true;
		} else if (key === 'show_max_rank_mod_prices') {
			config[key] = typeof value === 'boolean' ? value : false;
		} else if (key === 'hotkeys' && value && typeof value === 'object') {
			config.hotkeys = { ...config.hotkeys, ...(value as typeof config.hotkeys) };
		} else if (key === 'quicklist_hide_first' && typeof value === 'boolean') {
			config.quicklist_hide_first = value;
		} else if (key === 'quicklist_price_strategy' && isQuicklistStrategy(value)) {
			config.quicklist_price_strategy = value;
		}
	});
}

// 3. Export the update logic
export async function updateSetting(key: keyof typeof config) {
	console.log(`Updating setting ${key} to`, config[key]);
	await store.set(key, config[key]);
	await store.save();
}

type RelicDetectionSetting = 'relic_reward_detection' | 'visual_relic_reward_detection';

// DBWIN and visual polling are alternative automatic detectors. Persist both
// values together so a restart can never leave both enabled through the UI.
export async function updateRelicDetectionSetting(
	setting: RelicDetectionSetting,
	enabled: boolean
) {
	const otherSetting: RelicDetectionSetting =
		setting === 'relic_reward_detection'
			? 'visual_relic_reward_detection'
			: 'relic_reward_detection';

	config[setting] = enabled;
	if (enabled) {
		config[otherSetting] = false;
	}

	// Disable the previous detector before enabling the replacement so the
	// backend never observes an overlap while these writes are in flight.
	if (enabled) {
		await store.set(otherSetting, false);
	}
	await store.set(setting, config[setting]);
	if (!enabled) {
		await store.set(otherSetting, config[otherSetting]);
	}
	await store.save();
}
