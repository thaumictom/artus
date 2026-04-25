<script lang="ts">
	import Select from '$lib/components/Select.svelte';

	const items = [
		{ value: 'vitruvian', label: 'Vitruvian' },
		{ value: 'stalker', label: 'Stalker' },
		{ value: 'baruuk', label: 'Baruuk' },
		{ value: 'corpus', label: 'Corpus' },
		{ value: 'fortuna', label: 'Fortuna' },
		{ value: 'grineer', label: 'Grineer' },
		{ value: 'lotus', label: 'Lotus' },
		{ value: 'nidus', label: 'Nidus' },
		{ value: 'orokin', label: 'Orokin' },
		{ value: 'tenno', label: 'Tenno' },
		{ value: 'high_contrast', label: 'High Contrast' },
		{ value: 'legacy', label: 'Legacy' },
		{ value: 'equinox', label: 'Equinox' },
		{ value: 'dark_lotus', label: 'Dark Lotus' },
		{ value: 'zepher', label: 'Zepher' },
	];

	let fruit = $state('vitruvian');
</script>

<p class="mb-1">In-Game Theme</p>
<Select type="single" {items} bind:value={fruit} />
