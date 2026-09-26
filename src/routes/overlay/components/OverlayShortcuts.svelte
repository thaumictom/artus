<script lang="ts">
	import Icon from '@iconify/svelte';
	import Keybind from '$lib/components/Keybind.svelte';
	import { config } from '$lib/settings.svelte';
</script>

	<aside
		aria-label="Overlay keyboard shortcuts"
		class="right-4 bottom-4 absolute bg-background/95 shadow-lg px-3 py-2 border border-border-secondary text-foreground text-xs whitespace-nowrap"
	>
		<div class="flex items-center gap-3">
			<div class="flex items-center gap-1.5">
				<Icon
					icon="material-symbols:autorenew-rounded"
					class="size-4 text-accent"
					aria-hidden="true"
				/>
				<span class="text-muted-foreground">Cycle</span>
				<Keybind value={config.hotkeys.cycle ?? ''} />
				<span>next</span>
				<Keybind value={config.hotkeys.cycle_back ?? ''} />
				<span>back</span>
			</div>

			<div class="flex items-center gap-1 pl-3 border-border-secondary border-l">
				<Icon
					icon="material-symbols:open-with-rounded"
					class="size-4 text-accent"
					aria-hidden="true"
				/>
				<span class="mr-0.5 text-muted-foreground">Navigate</span>
				<Keybind value={config.hotkeys.navigate_up ?? ''} />
				<Keybind value={config.hotkeys.navigate_left ?? ''} />
				<Keybind value={config.hotkeys.navigate_down ?? ''} />
				<Keybind value={config.hotkeys.navigate_right ?? ''} />
			</div>

			<div class="flex items-center gap-1.5 pl-3 border-border-secondary border-l">
				<Icon
					icon="material-symbols:inventory-2-outline-rounded"
					class="size-4 text-accent"
					aria-hidden="true"
				/>
				<span class="text-muted-foreground">Inventory</span>
				<Keybind value={config.hotkeys.inventory_decrement ?? ''} />
				<span>−1</span>
				<Keybind value={config.hotkeys.inventory_increment ?? ''} />
				<span>+1</span>
			</div>

			<div class="flex items-center gap-1.5 pl-3 border-border-secondary border-l">
				<Icon
					icon="material-symbols:close-rounded"
					class="size-4 text-accent"
					aria-hidden="true"
				/>
				<span class="text-muted-foreground">Dismiss</span>
				<Keybind value="Esc" />
			</div>
		</div>
	</aside>
