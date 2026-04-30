<script lang="ts">
	import { columns, type MasterableItem } from "./columns";

	import masterableItems from "$lib/data/masterable-items.json";
	// import Select from "$lib/components/Select.svelte";
	import { Select } from "bits-ui";
	import CustomSelect from "$lib/components/Select.svelte";

	const data: MasterableItem[] = masterableItems;

	const categories = [
		{ label: "Weapons", value: "weapons" },
		{ label: "Warframes", value: "warframes" },
		{ label: "Companions", value: "companions" },
		{ label: "Archwings", value: "archwings" },
		{ label: "K-Drives", value: "k-drives" },
		{ label: "Amps", value: "amps" },
		{ label: "Zaws", value: "zaws" },
		{ label: "Kitguns", value: "kitguns" },
		{ label: "Sentinels", value: "sentinels" },
		{ label: "Other", value: "other" },
	];

	let currentValue = $state(categories[0].value);
</script>

<div>Work in progress</div>

<Select.Root type="single" items={categories} allowDeselect={true}>
	<Select.Trigger
		class="h-input rounded-9px border-border-input bg-background data-placeholder:text-foreground-alt/50 inline-flex w-[296px] touch-none select-none items-center border px-[11px] text-sm transition-colors"
		aria-label="Select a category"
	>
		<Select.Value placeholder="Select a category" />
	</Select.Trigger>
	<Select.Portal>
		<Select.Content
			class="focus-override border-muted bg-background shadow-popover data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 outline-hidden z-50 h-96 max-h-[var(--bits-select-content-available-height)] w-[var(--bits-select-anchor-width)] min-w-[var(--bits-select-anchor-width)] select-none rounded-xl border px-1 py-3 data-[side=bottom]:translate-y-1 data-[side=left]:-translate-x-1 data-[side=right]:translate-x-1 data-[side=top]:-translate-y-1"
			sideOffset={10}
		>
			<Select.Viewport class="p-1">
				{#each categories as category, i (i + category.value)}
					<Select.Item
						class="rounded-button data-highlighted:bg-muted outline-hidden data-disabled:opacity-50 flex h-10 w-full select-none items-center py-3 pl-5 pr-1.5 text-sm capitalize"
						value={category.value}
						label={category.label}
						disabled={category.disabled}
					>
						{#snippet children({ selected })}
							{category.label}
							{#if selected}
								<div class="ml-auto">Check</div>
							{/if}
						{/snippet}
					</Select.Item>
				{/each}
			</Select.Viewport>
		</Select.Content>
	</Select.Portal>
</Select.Root>

<CustomSelect type="single" items={categories}></CustomSelect>
<!-- <DataTable {data} {columns} /> -->
