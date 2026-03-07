<script lang="ts">
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { fade } from 'svelte/transition';

	type OcrWord = {
		text: string;
		x: number;
		y: number;
		width: number;
		height: number;
	};

	let words: OcrWord[] = [];

	onMount(() => {
		let unlisten: (() => void) | undefined;

		listen<{ words: OcrWord[] }>('ocr_result', (event) => {
			words = event.payload?.words ?? [];
		}).then((cleanup) => {
			unlisten = cleanup;
		});

		return () => {
			unlisten?.();
		};
	});
</script>

<main class="relative w-screen h-screen pointer-events-none">
	{#each words as word (`${word.text}-${word.x}-${word.y}-${word.width}-${word.height}`)}
		<div
			in:fade={{ duration: 200 }}
			class="absolute bg-black/70 p-1 border rounded-sm font-semibold text-teal-400 text-sm whitespace-pre-line"
			style={`left:${word.x}px;top:${word.y}px;`}
		>
			{word.text}
		</div>
	{/each}
</main>
