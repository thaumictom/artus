<script lang="ts">
	import Switch from '$lib/components/Switch.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import { config, updateSetting } from '$lib/settings.svelte';
</script>

<CommonSetting
	title="Show set prices"
	description="Show the Prime set price, total ducats and platinum per 100 ducats below a component."
	labelProps={{ for: 'show_set_prices' }}
>
	<Switch
		id="show_set_prices"
		bind:checked={config.show_set_prices}
		onCheckedChange={() => updateSetting('show_set_prices')}
	/>
</CommonSetting>

<CommonSetting
	title="Show max rank prices"
	description="Show the max rank price below the unranked price. Mods also require the setting below."
	labelProps={{ for: 'show_max_rank_prices' }}
>
	<Switch
		id="show_max_rank_prices"
		bind:checked={config.show_max_rank_prices}
		onCheckedChange={() => updateSetting('show_max_rank_prices')}
	/>
</CommonSetting>

<CommonSetting
	title="Show max rank prices for mods"
	description="Include max rank mod prices when max rank prices are enabled."
	labelProps={{ for: 'show_max_rank_mod_prices' }}
	disabled={!config.show_max_rank_prices}
>
	<Switch
		id="show_max_rank_mod_prices"
		bind:checked={config.show_max_rank_mod_prices}
		onCheckedChange={() => updateSetting('show_max_rank_mod_prices')}
		disabled={!config.show_max_rank_prices}
	/>
</CommonSetting>
