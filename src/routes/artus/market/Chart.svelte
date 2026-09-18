<script lang="ts">
	import * as echarts from 'echarts';
	import { onMount } from 'svelte';

	let { data = [], hourly = false } = $props();

	let chartContainer: HTMLDivElement;
	let chartInstance = $state.raw<echarts.ECharts>();
	let reducedMotion = $state(false);

	onMount(() => {
		const chart = echarts.init(chartContainer);
		chartInstance = chart;
		const motionPreference = window.matchMedia('(prefers-reduced-motion: reduce)');
		const updateMotionPreference = () => { reducedMotion = motionPreference.matches; };
		updateMotionPreference();
		motionPreference.addEventListener('change', updateMotionPreference);
		const resizeObserver = new ResizeObserver(() => chart.resize());
		resizeObserver.observe(chartContainer);

		return () => {
			resizeObserver.disconnect();
			motionPreference.removeEventListener('change', updateMotionPreference);
			chart.dispose();
		};
	});

	$effect(() => {
		if (!chartInstance) return;

		const styles = getComputedStyle(document.documentElement);
		const getStyle = (varName: string, fallback: string = 'red') =>
			styles.getPropertyValue(varName).trim() || fallback;

		const bgColor = 'transparent';
		const tooltipBg = getStyle('--color-surface');
		const tooltipBorder = getStyle('--color-border');
		const textMain = getStyle('--color-foreground');
		const textMuted = getStyle('--color-muted-foreground');
		const axisLineColor = getStyle('--color-border');
		const splitLineColor = getStyle('--color-input'); // horizontal grid lines
		const shadowColor = getStyle('--color-foreground');

		const colorPlatinum = getStyle('--color-cyan-500');
		const colorMovingAvg = getStyle('--color-amber-500');
		const colorVolume = getStyle('--color-foreground');
		const opacityVolume = 0.1;
		const font = getStyle('--font-sans');

		chartInstance.dispatchAction({ type: 'hideTip' });
		chartInstance.setOption(
			{
				animation: !reducedMotion,
				animationDuration: 350,
				animationDurationUpdate: 450,
				animationEasingUpdate: 'cubicInOut',
				backgroundColor: bgColor,
				textStyle: {
					color: textMain,
					fontFamily: font, // Sets the global font for the chart
				},
				tooltip: {
					trigger: 'axis',
					axisPointer: {
						type: 'line',
						cursor: 'default',
						animation: true,
						animationDurationUpdate: 250,
						shadowStyle: { color: shadowColor, opacity: 0.5 },
					},
					backgroundColor: tooltipBg,
					borderColor: tooltipBorder,
					textStyle: {
						color: textMain,
						fontFamily: font, // Ensures tooltip inherits it perfectly
					},
					formatter: function (params: any) {
						const date = new Date(params[0].axisValue).toLocaleString('default', {
							month: 'short',
							day: 'numeric',
							hour: hourly ? 'numeric' : undefined,
							minute: hourly ? 'numeric' : undefined,
							hour12: false,
						});

						let html = `<div style="font-weight:bold;margin-bottom:4px;color:${textMain}">${date}</div>`;

						params.forEach((param: any) => {
							const val = param.value[1];

							if (val === null || val === undefined || isNaN(val)) {
								html += `
								<div style="display:flex;justify-content:space-between;align-items:center;gap:16px;">
									<span>${param.marker} <span style="color:${textMain}">${param.seriesName}</span></span>
									<span style="font-weight:bold;color:${textMuted}">N/A</span>
								</div>`;
								return;
							}

							const formattedVal =
								param.seriesName === 'Volume' ? Math.round(val).toString() : val.toFixed(1);

							html += `
							<div style="display:flex;justify-content:space-between;align-items:center;gap:16px;">
								<span>${param.marker} <span style="color:${textMain}">${param.seriesName}</span></span>
								<span style="font-weight:bold;color:${textMain}">${formattedVal}</span>
							</div>`;
						});
						return html;
					},
				},
				grid: {
					left: '5%',
					right: '5%',
					bottom: '10%',
					containLabel: true,
				},
				xAxis: {
					type: 'time',
					boundaryGap: false,
					axisLine: { lineStyle: { color: axisLineColor } },
					axisLabel: { color: textMuted },
					axisPointer: {
						type: 'shadow',
						label: {
							backgroundColor: tooltipBg,
							color: textMain,
							// formatter:
						},
					},
				},
				yAxis: [
					{
						type: 'value',
						name: 'Platinum',
						min: 0,
						nameTextStyle: { color: textMain },
						position: 'left',
						scale: true,
						axisLine: {
							show: true,
							lineStyle: { color: axisLineColor },
						},
						axisTick: {
							show: true,
							lineStyle: { color: axisLineColor },
						},
						splitLine: {
							show: true,
							lineStyle: { color: splitLineColor },
						},
						axisLabel: { color: textMuted },
						axisPointer: {
							label: {
								backgroundColor: tooltipBg,
								color: textMain,
								formatter: (params: any) => {
									if (params.value === null || params.value === undefined) return '';
									return params.value.toFixed(1);
								},
							},
						},
					},
					{
						type: 'value',
						name: 'Volume',
						// Match platinum tick positions while keeping an independent volume scale.
						alignTicks: true,
						nameTextStyle: { color: textMain },
						position: 'right',
						min: 0,
						axisLine: {
							show: true,
							lineStyle: { color: axisLineColor },
						},
						axisTick: {
							show: true,
							lineStyle: { color: axisLineColor },
						},
						splitLine: { show: false },
						axisLabel: { color: textMuted },
						axisPointer: {
							label: {
								backgroundColor: tooltipBg,
								color: textMain,
								formatter: (params: any) => {
									if (params.value === null || params.value === undefined) return '';
									return Math.round(params.value).toString();
								},
							},
						},
					},
				],
				series: [
					{
						id: 'median',
						name: 'Median',
						type: 'line',
						yAxisIndex: 0,
						// Map as an object with a 'name' property to prevent index-based sliding
						data: data.map((d: any) => ({
							name: String(d.datetime),
							value: [d.datetime, d.median],
						})),
						smooth: true,
						showSymbol: false,
						itemStyle: { color: colorPlatinum },
						emphasis: { disabled: true },
						silent: true,
						z: 2,
					},
					{
						id: 'moving-average',
						name: 'Moving Avg',
						type: 'line',
						yAxisIndex: 0,
						data: data.map((d: any) => ({
							name: String(d.datetime),
							value: [d.datetime, d.moving_avg],
						})),
						smooth: true,
						showSymbol: false,
						itemStyle: { color: colorMovingAvg },
						emphasis: { disabled: true },
						silent: true,
						z: 2,
					},
					{
						id: 'volume',
						name: 'Volume',
						type: 'bar',
						yAxisIndex: 1,
						data: data.map((d: any) => ({
							name: String(d.datetime),
							value: [d.datetime, d.volume],
						})),
						itemStyle: { color: colorVolume, opacity: opacityVolume },
						emphasis: { disabled: true },
						silent: true,
						z: 1,
					},
				],
			},
			// Merge by series ID so range changes animate existing geometry.
			{ notMerge: false },
		);
	});
</script>

<div bind:this={chartContainer} class="w-full h-125 cursor-default"></div>
