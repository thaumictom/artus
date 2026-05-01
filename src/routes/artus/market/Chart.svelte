<script lang="ts">
	import * as echarts from 'echarts';

	let { data = [], hourly = false } = $props();

	let chartContainer: HTMLDivElement;
	let chartInstance: echarts.ECharts;

	$effect(() => {
		if (!chartContainer) return;

		chartInstance = echarts.init(chartContainer);
		const resizeObserver = new ResizeObserver(() => chartInstance.resize());
		resizeObserver.observe(chartContainer);

		return () => {
			resizeObserver.disconnect();
			chartInstance.dispose();
		};
	});

	$effect(() => {
		if (!chartInstance || data.length === 0) return;

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

		const colorPlatinum = getStyle('--color-blue-500');
		const colorMovingAvg = getStyle('--color-red-500');
		const colorVolume = getStyle('--color-foreground');
		const opacityVolume = 0.1;
		const font = getStyle('--font-sans');

		chartInstance.setOption(
			{
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
						name: 'Platinum',
						type: 'line',
						yAxisIndex: 0,
						data: data.map((d: any) => [d.datetime, d.median]),
						smooth: true,
						animation: false,
						showSymbol: false,
						itemStyle: { color: colorPlatinum },
						emphasis: { disabled: true },
						silent: true,
						z: 2,
					},
					{
						name: 'Moving Avg',
						type: 'line',
						yAxisIndex: 0,
						data: data.map((d: any) => [d.datetime, d.moving_avg]),
						smooth: true,
						animation: false,
						showSymbol: false,
						itemStyle: { color: colorMovingAvg },
						emphasis: { disabled: true },
						silent: true,
						z: 2,
					},
					{
						name: 'Volume',
						type: 'bar',
						yAxisIndex: 1,
						data: data.map((d: any) => [d.datetime, d.volume]),
						itemStyle: { color: colorVolume, opacity: opacityVolume },
						emphasis: { disabled: true },
						silent: true,
						z: 1,
					},
				],
			},
			true,
		);
	});
</script>

<div bind:this={chartContainer} class="w-full h-125 cursor-default"></div>
