<script lang="ts">
	import { config, updateSetting } from '$lib/settings.svelte';
	import Slider from '$lib/components/Slider.svelte';
</script>

<div class="flex flex-col gap-8">
	<div>
		<h3 class="text-lg font-medium">Word Grouping Parameters</h3>
		<p class="text-muted-foreground text-sm mb-4">
			Advanced settings for the OCR engine's word block grouping algorithm.
		</p>

		<div class="flex flex-col gap-6">
			<!-- Horizontal Word Gap Factor -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Horizontal Word Gap Factor</p>
					<p class="text-muted-foreground text-xs">
						Maximum gap between words on the same line to merge them (multiplier of line height).
					</p>
				</div>
				<Slider
					min={0}
					max={3}
					step={0.05}
					type="single"
					onValueCommit={() => updateSetting('horizontal_word_gap_factor')}
					bind:value={config.horizontal_word_gap_factor}
				>
					{#snippet thumbLabel({ value })}
						{(typeof value === 'number' ? value : config.horizontal_word_gap_factor).toFixed(2)}
					{/snippet}
				</Slider>
			</div>

			<!-- Same Line Vertical Factor -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Same Line Vertical Factor</p>
					<p class="text-muted-foreground text-xs">
						Vertical tolerance for assigning words to the same line.
					</p>
				</div>
				<Slider
					min={0}
					max={3}
					step={0.05}
					type="single"
					onValueCommit={() => updateSetting('same_line_vertical_factor')}
					bind:value={config.same_line_vertical_factor}
				>
					{#snippet thumbLabel({ value })}
						{(typeof value === 'number' ? value : config.same_line_vertical_factor).toFixed(2)}
					{/snippet}
				</Slider>
			</div>

			<!-- Merge Line Vertical Factor -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Merge Line Vertical Factor</p>
					<p class="text-muted-foreground text-xs">
						Vertical tolerance for merging adjacent lines into a block.
					</p>
				</div>
				<Slider
					min={0}
					max={5}
					step={0.1}
					type="single"
					onValueCommit={() => updateSetting('merge_line_vertical_factor')}
					bind:value={config.merge_line_vertical_factor}
				>
					{#snippet thumbLabel({ value })}
						{(typeof value === 'number' ? value : config.merge_line_vertical_factor).toFixed(2)}
					{/snippet}
				</Slider>
			</div>

			<!-- Max Merged Lines -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Max Merged Lines</p>
					<p class="text-muted-foreground text-xs">
						Maximum number of lines that can be merged into a single block.
					</p>
				</div>
				<Slider
					min={1}
					max={10}
					step={1}
					type="single"
					onValueCommit={() => updateSetting('max_merged_lines')}
					bind:value={config.max_merged_lines}
				>
					{#snippet thumbLabel({ value })}
						{typeof value === 'number' ? value : config.max_merged_lines}
					{/snippet}
				</Slider>
			</div>

			<!-- Center Aligned Merge Factor -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Center Aligned Merge Factor</p>
					<p class="text-muted-foreground text-xs">
						Tolerance for center alignment when merging multi-line blocks.
					</p>
				</div>
				<Slider
					min={0}
					max={10}
					step={0.5}
					type="single"
					onValueCommit={() => updateSetting('center_aligned_merge_factor')}
					bind:value={config.center_aligned_merge_factor}
				>
					{#snippet thumbLabel({ value })}
						{(typeof value === 'number' ? value : config.center_aligned_merge_factor).toFixed(2)}
					{/snippet}
				</Slider>
			</div>

			<!-- Center Aligned Horizontal Gap Factor -->
			<div class="flex flex-col gap-3">
				<div>
					<p class="text-sm font-medium">Center Aligned Horizontal Gap Factor</p>
					<p class="text-muted-foreground text-xs">
						Horizontal gap tolerance for center-aligned multi-line blocks.
					</p>
				</div>
				<Slider
					min={0}
					max={10}
					step={0.5}
					type="single"
					onValueCommit={() => updateSetting('center_aligned_horizontal_gap_factor')}
					bind:value={config.center_aligned_horizontal_gap_factor}
				>
					{#snippet thumbLabel({ value })}
						{(typeof value === 'number'
							? value
							: config.center_aligned_horizontal_gap_factor
						).toFixed(2)}
					{/snippet}
				</Slider>
			</div>
		</div>
	</div>
</div>
