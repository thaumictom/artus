<script lang="ts">
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import { mastery, dismissMasteryDots, setOtherMasteryXp } from '$lib/mastery.svelte';
	import { masteryRankProgress, masteryXpFor } from '$lib/mastery-xp';

	let dialogOpen = $state(false);
	let draftXp = $state('');
	let inputError = $state('');
	const checked = $derived(new Set(mastery.checked));
	const checkedCount = $derived(mastery.items.filter((item) => checked.has(item.key)).length);
	const gearXp = $derived(
		mastery.items.reduce(
			(total, item) => total + (checked.has(item.key) ? masteryXpFor(item) : 0),
			0,
		),
	);
	const rankProgress = $derived(masteryRankProgress(gearXp + mastery.otherXp));

	function openOtherXp() {
		draftXp = String(mastery.otherXp);
		inputError = '';
		dialogOpen = true;
	}

	function saveOtherXp(event: SubmitEvent) {
		event.preventDefault();
		const value = Number(draftXp);
		if (draftXp.trim() === '' || !Number.isSafeInteger(value) || value < 0) {
			inputError = 'Enter a whole number of XP greater than or equal to 0.';
			return;
		}
		setOtherMasteryXp(value);
		dialogOpen = false;
	}
</script>

<div class="p-4 border w-full text-sm">
	<div class="flex justify-between items-baseline gap-3">
		<span class="font-semibold">{rankProgress.label} · tracked mastery</span>
		<span class="text-muted-foreground text-xs">
			{checkedCount} / {mastery.items.length} checked
		</span>
	</div>
	<div
		class="bg-muted mt-2 rounded-full h-2 overflow-hidden"
		role="progressbar"
		aria-label="Mastery XP toward next rank"
		aria-valuenow={rankProgress.current}
		aria-valuemin="0"
		aria-valuemax={rankProgress.required}
	>
		<div
			class="bg-accent rounded-full h-full transition-[width]"
			style:width={`${rankProgress.percent}%`}
		></div>
	</div>
	<div class="flex justify-between gap-3 mt-1.5 text-muted-foreground text-xs">
		<span>
			{rankProgress.current.toLocaleString()} / {rankProgress.required.toLocaleString()} XP to next rank
		</span>
		<span>{rankProgress.xp.toLocaleString()} total XP</span>
	</div>
	<p class="mt-1 text-muted-foreground text-xs">
		{gearXp.toLocaleString()} gear XP + {mastery.otherXp.toLocaleString()} other XP
	</p>
	<div class="flex flex-wrap gap-2 mt-3">
		<Button onclick={openOtherXp} class="text-xs">Add other mastery</Button>
		{#if mastery.automatic.length > 0}
			<Button onclick={dismissMasteryDots} class="text-xs">
				Dismiss {mastery.automatic.length} new dots
			</Button>
		{/if}
	</div>
</div>

{#snippet title()}Other mastery XP{/snippet}
{#snippet description()}
	Add Star Chart and Intrinsics mastery XP here. You can find these values in your in-game profile.
{/snippet}

<Dialog bind:open={dialogOpen} {title} {description} contentProps={{ class: 'h-auto' }}>
	<form onsubmit={saveOtherXp} class="px-6 pt-3">
		<label for="other-mastery-xp" class="block mb-1.5 font-semibold text-sm">
			Other mastery XP
		</label>
		<input
			id="other-mastery-xp"
			type="number"
			min="0"
			step="1"
			required
			value={draftXp}
			oninput={(event) => {
				draftXp = event.currentTarget.value;
				inputError = '';
			}}
			class="bg-background p-2 border border-border focus-visible:border-accent outline-none w-full text-foreground"
		/>
		<p class="mt-2 text-muted-foreground text-xs">
			This amount is added to checked gear XP in the rank estimate.
		</p>
		{#if inputError}<p role="alert" class="mt-2 text-danger text-xs">{inputError}</p>{/if}
		<div class="flex justify-end gap-2 mt-5">
			<Button onclick={() => (dialogOpen = false)}>Cancel</Button>
			<Button type="submit" variant="primary">Save XP</Button>
		</div>
	</form>
</Dialog>
