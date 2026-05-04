<script lang="ts">
	import { config, updateSetting } from '$lib/settings.svelte';
	import Slider from '$lib/components/Slider.svelte';
	import { Label } from 'bits-ui';
	import Collapsible from '$lib/components/Collapsible.svelte';
	import CommonSetting from '$lib/components/ui/CommonSetting.svelte';
	import Icon from '@iconify/svelte';
	import { fade, slide } from 'svelte/transition';
</script>

<Collapsible>
	{#snippet button(open)}
		<div class="group">
			<CommonSetting
				title="Overlay price thresholds"
				description="Set when the platinum per 100 ducats price label should turn yellow (salvage) or blue (sell) for each ducat tier"
			>
				<div class="shrink border *:size-6 *:p-1 group-hover:bg-surface transition relative">
					<div
						class={{
							'opacity-0 rotate-180': !open,
							'duration-300 transition': true,
						}}
					>
						<Icon icon="material-symbols:collapse-all-rounded" class="size-full" />
					</div>
					<div
						class={{
							'opacity-0 -rotate-180': open,
							'absolute inset-0 duration-300 transition': true,
						}}
					>
						<Icon icon="material-symbols:expand-all-rounded" class="size-full" />
					</div>
				</div>
			</CommonSetting>
		</div>
	{/snippet}
	{#snippet content(open)}
		{#if open}
			<div transition:slide class="pl-4 border-l-4 py-2 flex flex-col gap-4 mt-4">
				{#each [100, 65, 45, 25, 15] as tier}
					{@const key = `threshold_${tier}` as keyof typeof config}
					<div class="flex flex-col">
						<p>{tier} Ducat Item</p>
						<Slider
							min={1}
							max={25}
							step={1}
							type="multiple"
							onValueCommit={() => updateSetting(key)}
							bind:value={config[key] as any}
						>
							{#snippet thumbLabel({ value })}
								{value}p
							{/snippet}
						</Slider>
					</div>
				{/each}
			</div>
		{/if}
	{/snippet}
</Collapsible>
