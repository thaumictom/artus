<script lang="ts">
	import Icon from '@iconify/svelte';
	import { Combobox, type WithoutChildrenOrChild, mergeProps } from 'bits-ui';
	import * as fuzzball from 'fuzzball';

	type Props = Combobox.RootProps & {
		inputProps?: WithoutChildrenOrChild<Combobox.InputProps>;
		contentProps?: WithoutChildrenOrChild<Combobox.ContentProps>;
		minSearchLength?: number;
	};

	let {
		items = [],
		value = $bindable(),
		open = $bindable(false),
		inputProps,
		contentProps,
		minSearchLength = 3,
		type,
		...restProps
	}: Props = $props();

	let searchValue = $state('');

	type HighlightSegment = {
		text: string;
		matched: boolean;
	};

	function isOrderedSubsequence(query: string, text: string): boolean {
		let queryIndex = 0;
		for (const char of text) {
			if (char === query[queryIndex]) {
				queryIndex += 1;
				if (queryIndex === query.length) {
					return true;
				}
			}
		}

		return query.length === 0;
	}

	function getHighlightSegments(label: string, query: string): HighlightSegment[] {
		const normalizedQuery = query.trim().toLowerCase();
		if (!normalizedQuery) {
			return [{ text: label, matched: false }];
		}

		const normalizedLabel = label.toLowerCase();
		const matchedIndices = new Set<number>();
		let queryIndex = 0;

		for (let i = 0; i < normalizedLabel.length; i += 1) {
			if (normalizedLabel[i] === normalizedQuery[queryIndex]) {
				matchedIndices.add(i);
				queryIndex += 1;
				if (queryIndex === normalizedQuery.length) {
					break;
				}
			}
		}

		if (queryIndex !== normalizedQuery.length) {
			return [{ text: label, matched: false }];
		}

		const segments: HighlightSegment[] = [];
		let buffer = '';
		let currentMatchState: boolean | null = null;

		for (let i = 0; i < label.length; i += 1) {
			const isMatch = matchedIndices.has(i);
			if (currentMatchState === null) {
				currentMatchState = isMatch;
				buffer = label[i];
				continue;
			}

			if (isMatch === currentMatchState) {
				buffer += label[i];
				continue;
			}

			segments.push({ text: buffer, matched: currentMatchState });
			buffer = label[i];
			currentMatchState = isMatch;
		}

		if (buffer && currentMatchState !== null) {
			segments.push({ text: buffer, matched: currentMatchState });
		}

		return segments;
	}

	const filteredItems = $derived.by(() => {
		const normalizedQuery = searchValue.trim().toLowerCase();
		if (normalizedQuery.length < minSearchLength) return [];

		return items
			.map((item) => {
				const normalizedLabel = item.label.toLowerCase();
				return {
					item,
					startsWithMatch: normalizedLabel.startsWith(normalizedQuery),
					exactMatch: normalizedLabel.includes(normalizedQuery),
					isSubsequenceMatch: isOrderedSubsequence(normalizedQuery, normalizedLabel),
					score: fuzzball.WRatio(item.label, normalizedQuery),
				};
			})
			.filter(({ isSubsequenceMatch }) => isSubsequenceMatch)
			.sort(
				(left, right) =>
					Number(right.startsWithMatch) - Number(left.startsWithMatch) ||
					Number(right.exactMatch) - Number(left.exactMatch) ||
					right.score - left.score ||
					left.item.label.localeCompare(right.item.label),
			)
			.map(({ item }) => item);
	});

	function handleInput(e: Event & { currentTarget: HTMLInputElement }) {
		searchValue = e.currentTarget.value;
	}

	function handleOpenChange(newOpen: boolean) {
		if (!newOpen) searchValue = '';
	}

	const mergedRootProps = $derived(mergeProps(restProps, { onOpenChange: handleOpenChange }));
	const mergedInputProps = $derived(mergeProps(inputProps, { oninput: handleInput }));
</script>

<Combobox.Root {type} {items} bind:value={value as never} bind:open {...mergedRootProps}>
	<div class="relative">
		<Combobox.Input {...mergedInputProps} class="p-2 border w-full" />
		<Combobox.Trigger class="top-1/2 absolute -translate-y-1/2 touch-none cursor-pointer end-3">
			<Icon icon="material-symbols:unfold-more-rounded" class="size-5" />
		</Combobox.Trigger>
	</div>
	<Combobox.Portal>
		<Combobox.Content
			{...contentProps}
			class="bg-surface/50 backdrop-blur border w-[var(--bits-combobox-anchor-width)] min-w-[var(--bits-combobox-anchor-width)] max-h-56 overflow-hidden"
			sideOffset={4}
		>
			<Combobox.Viewport>
				{#each filteredItems as item, i (i + item.value)}
					<Combobox.Item
						{...item}
						class="flex justify-between items-center hover:bg-elevated p-2 cursor-pointer"
					>
						{#snippet children({ selected })}
							<span class="flex-1 min-w-0 truncate">
								{#each getHighlightSegments(item.label, searchValue) as segment, segmentIndex (segmentIndex)}
									<span class={segment.matched ? 'text-accent' : ''}>
										{segment.text}
									</span>
								{/each}
							</span>
							<span class="shrink-0">{selected ? '✅' : ''}</span>
						{/snippet}
					</Combobox.Item>
				{:else}
					<div class="p-2 text-muted-foreground">
						{#if searchValue.trim().length < minSearchLength}
							Type {minSearchLength} letters to search
						{:else}
							No results found
						{/if}
					</div>
				{/each}
			</Combobox.Viewport>
		</Combobox.Content>
	</Combobox.Portal>
</Combobox.Root>
