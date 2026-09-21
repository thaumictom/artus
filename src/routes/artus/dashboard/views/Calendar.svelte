<script lang="ts">
	import ViewPanel from './ViewPanel.svelte';
	import { type DashboardViewProps, type ViewRow } from './view-types';
	let { world, now }: DashboardViewProps = $props();
	let rows: ViewRow[] = $derived((world.calendar?.days ?? []).flatMap((day) => day.events.map((event) => ({
		title: event.challenge?.title || event.upgrade?.title || event.reward || event.dialogueName || event.type,
		description: day.date,
		details: [event.challenge?.description, event.upgrade?.description, event.dialogueConvo].filter((value): value is string => Boolean(value)),
	}))));
</script>

<ViewPanel title="1999 Calendar" {rows} {now} summary={world.calendar?.season} />

