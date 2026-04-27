<script lang="ts">
	import { onMount } from 'svelte';
	import { Label } from 'bits-ui';

	import Switch from '$lib/components/Switch.svelte';
	import {
		getRelicRewardDetection,
		getWarframeLogPath,
		setRelicRewardDetection
	} from '../settings-api';

	let isLoading = $state(true);
	let relicRewardDetection = $state(false);
	let logPath = $state('');

	async function load() {
		isLoading = true;
		try {
			relicRewardDetection = await getRelicRewardDetection();
			logPath = await getWarframeLogPath();
		} catch (error) {
			console.error('Failed to load relic reward detection settings:', error);
		} finally {
			isLoading = false;
		}
	}

	async function save(enabled: boolean) {
		try {
			relicRewardDetection = await setRelicRewardDetection(enabled);
		} catch (error) {
			console.error('Failed to save relic reward detection setting:', error);
		}
	}

	onMount(() => {
		void load();

		const interval = setInterval(async () => {
			try {
				const currentPath = await getWarframeLogPath();
				if (currentPath !== logPath) {
					logPath = currentPath;
				}
			} catch {
				// Ignore errors in background refresh
			}
		}, 1000);

		return () => clearInterval(interval);
	});
</script>

<div class="flex justify-between items-center gap-1">
	<div class="flex-1 transition-opacity" class:opacity-50={!logPath}>
		<Label.Root for="relic-reward-detection-toggle">
			<p>Automatic relic reward detection</p>
			<p class="text-muted-foreground text-xs">
				Automatically shows the overlay when the relic reward screen appears. Requires EE.log path to
				be set.
			</p>
		</Label.Root>
	</div>
	<Switch
		id="relic-reward-detection-toggle"
		disabled={isLoading || !logPath}
		onCheckedChange={(enabled) => void save(enabled)}
		checked={relicRewardDetection}
	/>
</div>
