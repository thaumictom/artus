<script lang="ts">
	import { Label } from 'bits-ui';
	import { config, updateSetting } from '$lib/settings.svelte';

	import Switch from '$lib/components/Switch.svelte';
</script>

<div class="flex justify-between items-center gap-1">
	<div class="flex-1 transition-opacity" class:opacity-50={!config.warframe_log_path}>
		<Label.Root for="relic-reward-detection-toggle">
			<p>Automatic relic reward detection</p>
			<p class="text-muted-foreground text-xs">
				Automatically shows the overlay when the relic reward screen appears. Requires EE.log path
				to be set.
			</p>
		</Label.Root>
	</div>
	<Switch
		id="relic-reward-detection-toggle"
		disabled={!config.warframe_log_path}
		onCheckedChange={() => updateSetting('relic_reward_detection')}
		bind:checked={config.relic_reward_detection}
	/>
</div>
