<script lang="ts">
	let {
		label,
		state,
		nextState,
		progress,
	}: {
		label: string;
		state: string;
		nextState: string;
		progress: number;
	} = $props();

	const warmStates = new Set(['day', 'fass', 'warm', 'joy', 'corpus']);
	const dangerStates = new Set(['anger', 'grineer']);
	let id = $derived(label.toLowerCase().replaceAll(/[^a-z0-9]/g, '-'));
	let fadeId = $derived(`cycle-next-${id}`);
	let guideId = $derived(`cycle-guide-${id}`);

	function stateColor(value: string) {
		if (dangerStates.has(value)) return 'var(--color-danger)';
		if (warmStates.has(value)) return 'var(--color-warn)';
		return 'var(--color-accent)';
	}

	function sinePath(startX: number, endX: number, startAngle: number, endAngle: number) {
		const points = 24;
		return Array.from({ length: points + 1 }, (_, index) => {
			const ratio = index / points;
			const x = startX + (endX - startX) * ratio;
			const angle = startAngle + (endAngle - startAngle) * ratio;
			const y = 21 - 12 * Math.sin(angle);
			return `${index === 0 ? 'M' : 'L'}${x.toFixed(2)} ${y.toFixed(2)}`;
		}).join(' ');
	}

	const currentWave = sinePath(6, 70, 0, Math.PI);
	// Keep the same horizontal scale across the boundary so both halves form one sine wave.
	const nextWave = sinePath(70, 134, Math.PI, Math.PI * 2);
	let currentColor = $derived(stateColor(state));
	let x = $derived(6 + 64 * progress);
	let y = $derived(21 - 12 * Math.sin(Math.PI * progress));
	let guideOffset = $derived(`${((y - 2) / 38) * 100}%`);
</script>

<svg
	class="block mt-1 w-full h-12 overflow-hidden"
	viewBox="0 0 100 42"
	role="img"
	aria-label={`${label} ${state} phase, ${Math.round(progress * 100)}% complete; ${nextState} follows`}
>
	<defs>
		<linearGradient id={fadeId} x1="70" y1="0" x2="98" y2="0" gradientUnits="userSpaceOnUse">
			<stop offset="0" stop-color="var(--color-foreground)" stop-opacity="0.42" />
			<stop offset="1" stop-color="var(--color-foreground)" stop-opacity="0" />
		</linearGradient>
		<linearGradient id={guideId} x1="0" y1="2" x2="0" y2="40" gradientUnits="userSpaceOnUse">
			<stop offset="0" stop-color="var(--color-foreground)" stop-opacity="0" />
			<stop offset={guideOffset} stop-color="var(--color-foreground)" stop-opacity="0.8" />
			<stop offset="1" stop-color="var(--color-foreground)" stop-opacity="0" />
		</linearGradient>
	</defs>

	<path d="M2 21 H98" fill="none" stroke="var(--color-foreground)" stroke-width="0.6" opacity="0.12" />

	<path
		d={currentWave}
		fill="none"
		stroke="var(--color-foreground)"
		stroke-width="1.6"
		stroke-linecap="round"
		stroke-linejoin="round"
		opacity="0.42"
	/>
	<path
		d={currentWave}
		fill="none"
		stroke="var(--color-foreground)"
		stroke-width="2"
		stroke-linecap="round"
		stroke-linejoin="round"
		pathLength="1"
		stroke-dasharray={`${progress} 1`}
	/>
	<path
		d={nextWave}
		fill="none"
		stroke={`url(#${fadeId})`}
		stroke-width="1.6"
		stroke-linecap="round"
		stroke-linejoin="round"
	/>

	<line
		x1={x}
		x2={x}
		y1="2"
		y2="40"
		stroke={`url(#${guideId})`}
		stroke-width="1"
	/>
	<circle cx={x} cy={y} r="5" fill={currentColor} opacity="0.16" />
	<circle
		cx={x}
		cy={y}
		r="3"
		fill="var(--color-background)"
		stroke="var(--color-foreground)"
		stroke-width="1"
	/>
</svg>
