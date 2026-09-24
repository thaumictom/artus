<script lang="ts">
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import { loginMarket, logoutMarket, marketAccount, rememberedMarketLogin, setMarketStatus, type MarketStatus } from '$lib/market-account.svelte';

	let open = $state(false);
	let email = $state('');
	let password = $state('');
	let remember = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
	const statusLabel: Record<MarketStatus, string> = { invisible: 'Invisible', online: 'Online', ingame: 'In game' };
	const dotColor: Record<MarketStatus, string> = { invisible: 'bg-muted-foreground', online: 'bg-amber-400', ingame: 'bg-green-500' };

	async function showProfile() {
		error = null;
		if (!marketAccount.session) {
			try {
				const saved = await rememberedMarketLogin();
				if (saved) { email = saved.email; password = saved.password; remember = true; }
			} catch (cause) { console.error('Could not load remembered market login:', cause); }
		}
		open = true;
	}

	async function login() {
		busy = true; error = null;
		try { await loginMarket(email, password, remember); password = ''; }
		catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
	async function changeStatus(status: MarketStatus) {
		busy = true; error = null;
		try { await setMarketStatus(status); }
		catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
	async function logout() {
		busy = true; error = null;
		try { await logoutMarket(); open = false; }
		catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
</script>

<button type="button" onclick={showProfile} class="flex items-center gap-1.5 hover:bg-elevated px-2 py-1 text-sm cursor-pointer" aria-label={`warframe.market profile, ${marketAccount.session ? statusLabel[marketAccount.session.status] : 'logged out'}`}>
	<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
	<span class={`rounded-full size-2 ${marketAccount.session ? dotColor[marketAccount.session.status] : 'bg-muted-foreground/50'}`}></span>
	<span>{marketAccount.session ? statusLabel[marketAccount.session.status] : 'Logged out'}</span>
</button>

{#snippet title()}warframe.market account{/snippet}
{#snippet description()}{marketAccount.session ? marketAccount.session.ingameName : 'Sign in with your warframe.market email and password.'}{/snippet}
<Dialog bind:open {title} {description} contentProps={{ class: 'h-auto' }}>
	<div class="flex flex-col gap-4 px-6">
		{#if marketAccount.session}
			<div>
				<div class="mb-2 font-semibold text-muted-foreground text-xs uppercase tracking-wider">Status</div>
				<div class="flex flex-wrap gap-2">
					{#each ['invisible', 'online', 'ingame'] as status}
						<Button variant={marketAccount.session.status === status ? 'primary' : 'default'} disabled={busy} onclick={() => changeStatus(status as MarketStatus)}>{statusLabel[status as MarketStatus]}</Button>
					{/each}
				</div>
			</div>
			<Button class="self-start" disabled={busy} onclick={logout}>Log out</Button>
		{:else}
			<label class="flex flex-col gap-1 text-sm">Email
				<input type="email" autocomplete="username" bind:value={email} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" />
			</label>
			<label class="flex flex-col gap-1 text-sm">Password
				<input type="password" autocomplete="current-password" bind:value={password} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" />
			</label>
			<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={remember} /> Remember me</label>
			<p class="text-muted-foreground text-xs">Remember me stores your email and password in Artus's local app data without encryption. Use it only on a device you trust.</p>
			<Button variant="primary" class="self-start" disabled={busy || !email.trim() || !password} onclick={login}>{busy ? 'Signing in...' : 'Log in'}</Button>
		{/if}
		{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>
