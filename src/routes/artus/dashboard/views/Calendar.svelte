<script lang="ts">
	import Icon from '@iconify/svelte';
	import type { DayEvent } from 'warframe-worldstate-parser';
	import Tooltip from '$lib/components/Tooltip.svelte';
	import type { DashboardViewProps } from './view-types';

	const WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
	const MONTH_FORMATTER = new Intl.DateTimeFormat(undefined, {
		month: 'long',
		timeZone: 'UTC',
	});
	const DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
		weekday: 'long',
		month: 'long',
		day: 'numeric',
		year: 'numeric',
		timeZone: 'UTC',
	});

	type CalendarDay = {
		date: string;
		events: DayEvent[];
	};
	type CalendarCell = {
		day: number;
		date: Date;
		events: DayEvent[];
	};
	type CalendarMonth = {
		key: string;
		label: string;
		cells: (CalendarCell | null)[];
	};

	let { world }: DashboardViewProps = $props();
	let calendar = $derived(world.calendar);
	let months = $derived(buildMonths(calendar?.days ?? [], calendar?.season));

	function seasonStartMonth(season?: string) {
		const value = season?.toLowerCase() ?? '';
		if (value.includes('spring')) return 3;
		if (value.includes('summer')) return 6;
		if (value.includes('fall') || value.includes('autumn')) return 9;
		return 0;
	}

	function dateKey(date: Date) {
		return date.toISOString().slice(0, 10);
	}

	function buildMonths(days: CalendarDay[], season?: string): CalendarMonth[] {
		const eventsByDate = new Map<string, DayEvent[]>();
		const validDates: Date[] = [];

		for (const day of days) {
			const date = new Date(day.date);
			if (!Number.isFinite(date.getTime())) continue;
			validDates.push(date);
			const key = dateKey(date);
			eventsByDate.set(key, [...(eventsByDate.get(key) ?? []), ...day.events]);
		}

		validDates.sort((a, b) => a.getTime() - b.getTime());
		const startMonth = validDates[0]?.getUTCMonth() ?? seasonStartMonth(season);

		return Array.from({ length: 3 }, (_, offset) => {
			const month = startMonth + offset;
			const firstDate = new Date(Date.UTC(1999, month, 1));
			const daysInMonth = new Date(Date.UTC(1999, month + 1, 0)).getUTCDate();
			const cells: (CalendarCell | null)[] = Array(firstDate.getUTCDay()).fill(null);

			for (let day = 1; day <= daysInMonth; day += 1) {
				const date = new Date(Date.UTC(1999, month, day));
				cells.push({ day, date, events: eventsByDate.get(dateKey(date)) ?? [] });
			}
			while (cells.length % 7 !== 0) cells.push(null);

			return {
				key: dateKey(firstDate),
				label: MONTH_FORMATTER.format(firstDate),
				cells,
			};
		});
	}

	function eventTitle(event: DayEvent) {
		return (
			event.challenge?.title ||
			event.upgrade?.title ||
			event.reward ||
			event.dialogueName ||
			event.type
		);
	}

	function eventDescription(event: DayEvent) {
		return event.challenge?.description || event.upgrade?.description || event.dialogueConvo;
	}

	function eventAppearance(type: string) {
		switch (type.toLowerCase()) {
			case 'to do':
				return { icon: 'material-symbols:checklist-rounded', color: 'text-[#88c0d0]' };
			case 'override':
				return { icon: 'material-symbols:tune-rounded', color: 'text-[#b48ead]' };
			case 'big prize!':
				return {
					icon: 'material-symbols:featured-seasonal-and-gifts-rounded',
					color: 'text-[#ebcb8b]',
				};
			case 'birthday':
				return { icon: 'material-symbols:cake-rounded', color: 'text-[#d08770]' };
			default:
				return { icon: 'material-symbols:event-rounded', color: 'text-muted-foreground' };
		}
	}
</script>

<section aria-labelledby="calendar-heading" class="flex flex-col gap-5">
	<header class="flex justify-between items-end gap-4">
		<div>
			<h2 id="calendar-heading" class="font-expanded font-medium text-lg">1999 Calendar</h2>
			{#if calendar}
				<p class="mt-1 text-muted-foreground text-sm">
					{calendar.season} · Loop {calendar.yearIteration}
				</p>
			{/if}
		</div>
		<p class="text-muted-foreground text-xs">Hover an event for details</p>
	</header>

	<div class="flex flex-col gap-6">
		{#each months as month (month.key)}
			<article class="bg-surface border border-surface overflow-hidden">
				<h3 class="bg-background px-4 py-3 border-surface border-b font-expanded font-medium">
					{month.label} 1999
				</h3>
				<div class="gap-px grid grid-cols-7 bg-surface border-surface border-b">
					{#each WEEKDAYS as weekday}
						<div class="bg-background/80 px-2 py-1.5 text-muted-foreground text-xs text-center">
							{weekday}
						</div>
					{/each}
				</div>
				<div class="gap-px grid grid-cols-7 bg-surface">
					{#each month.cells as cell}
						{#if cell}
							<div class="bg-background p-2 min-h-24 min-w-0">
								<time
									class="block mb-1.5 text-muted-foreground text-xs tabular-nums"
									datetime={dateKey(cell.date)}
								>
									{cell.day}
								</time>
								<div class="flex flex-col gap-1">
									{#each cell.events as event, eventIndex (`${event.type}-${eventIndex}`)}
										{@const appearance = eventAppearance(event.type)}
										<Tooltip
											class="flex items-center gap-1 hover:bg-surface px-1 py-0.5 w-full min-w-0 text-left"
										>
											{#snippet children()}
												<Icon
													icon={appearance.icon}
													class={`size-3.5 shrink-0 ${appearance.color}`}
												/>
												<span class="text-xs truncate">{eventTitle(event)}</span>
											{/snippet}
											{#snippet content()}
												<div class="flex items-start gap-2">
													<Icon
														icon={appearance.icon}
														class={`mt-0.5 size-4 shrink-0 ${appearance.color}`}
													/>
													<div class="min-w-0">
														<p class="text-muted-foreground text-xs">{event.type}</p>
														<p class="font-medium">{eventTitle(event)}</p>
														{#if eventDescription(event)}
															<p class="mt-1 text-muted-foreground">{eventDescription(event)}</p>
														{/if}
														<p class="mt-2 text-muted-foreground text-xs">
															{DATE_FORMATTER.format(cell.date)}
														</p>
													</div>
												</div>
											{/snippet}
										</Tooltip>
									{/each}
								</div>
							</div>
						{:else}
							<div aria-hidden="true" class="bg-background/35 min-h-24"></div>
						{/if}
					{/each}
				</div>
			</article>
		{/each}
	</div>
</section>
