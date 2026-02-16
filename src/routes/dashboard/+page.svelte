<script lang="ts">
	import { onMount } from 'svelte';
	import { listen } from '@tauri-apps/api/event';

	interface BinaryImageData {
		width: number;
		height: number;
		data: number[];
	}

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;

	onMount(() => {
		ctx = canvas.getContext('2d');

		const unlistenBinaryImage = listen<BinaryImageData>('binary-image', async ({ payload }) => {
			console.log('Received binary image data:', payload);

			const rgba = new Uint8ClampedArray(payload.width * payload.height * 4);

			let src = 0;
			let dst = 0;

			while (src < payload.data.length) {
				rgba[dst++] = payload.data[src++];
				rgba[dst++] = payload.data[src++];
				rgba[dst++] = payload.data[src++];
				rgba[dst++] = 255;
			}

			const imageData = new ImageData(rgba, payload.width, payload.height);

			if (!ctx) return;

			canvas.width = imageData.width;
			canvas.height = imageData.height;
			ctx.putImageData(imageData, 0, 0);
		});

		return () => {
			unlistenBinaryImage.then((f) => f());
		};
	});
</script>

<div>
	Binary image:
	<canvas bind:this={canvas} class="w-full"></canvas>
</div>
