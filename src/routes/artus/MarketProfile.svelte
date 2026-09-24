<script lang="ts">
	import Icon from '@iconify/svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';
	import { loginMarket, logoutMarket, marketAccount, marketProfileUrl, rememberedMarketLogin, setMarketStatus, type MarketStatus } from '$lib/market-account.svelte';

	let loginOpen = $state(false);
	let profileOpen = $state(false);
	let email = $state('');
	let password = $state('');
	let remember = $state(false);
	let busy = $state(false);
	let error = $state<string | null>(null);
	const statusLabel: Record<MarketStatus, string> = { invisible: 'Invisible', online: 'Online', ingame: 'In game' };
	const dotColor: Record<MarketStatus, string> = { invisible: 'bg-muted-foreground', online: 'bg-amber-400', ingame: 'bg-green-500' };
	const statusOptions: MarketStatus[] = ['invisible', 'online', 'ingame'];

	async function showLogin() {
		error = null;
		try {
			const saved = await rememberedMarketLogin();
			if (saved) { email = saved.email; password = saved.password; remember = true; }
		} catch (cause) { console.error('Could not load remembered market login:', cause); }
		loginOpen = true;
	}

	async function login() {
		busy = true; error = null;
		try { await loginMarket(email, password, remember); password = ''; loginOpen = false; }
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
		try { await logoutMarket(); profileOpen = false; }
		catch (cause) { error = String(cause); }
		finally { busy = false; }
	}
</script>

{#if marketAccount.session}
	<ActionPopover bind:open={profileOpen} contentClass="w-56" triggerAriaLabel={`warframe.market profile, ${statusLabel[marketAccount.session.status]}`}>
		{#snippet trigger()}
			<span class="flex items-center gap-1.5 hover:bg-elevated px-2 py-1 text-sm">
				<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
				<span class={`rounded-full size-2 ${dotColor[marketAccount.session?.status ?? 'invisible']}`}></span>
				<span>{statusLabel[marketAccount.session?.status ?? 'invisible']}</span>
			</span>
		{/snippet}
		<a href={marketProfileUrl(marketAccount.session)} target="_blank" rel="noopener noreferrer" onclick={() => (profileOpen = false)} class="flex items-center justify-between gap-2 hover:bg-elevated p-2 font-semibold text-sm">
			<span class="truncate">{marketAccount.session.ingameName}</span><Icon icon="lucide:external-link" class="size-3.5 shrink-0" />
		</a>
		<div class="my-1 border-t border-border-secondary"></div>
		<p class="px-2 py-1 text-muted-foreground text-xs uppercase tracking-wider">Activity</p>
		{#each statusOptions as status}
			<button type="button" disabled={busy} onclick={() => changeStatus(status)} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-50 px-2 py-1.5 w-full text-sm text-left cursor-pointer">
				<span class={`rounded-full size-2 ${dotColor[status]}`}></span><span class="flex-1">{statusLabel[status]}</span>
				{#if marketAccount.session.status === status}<Icon icon="lucide:check" class="size-4 text-accent" />{/if}
			</button>
		{/each}
		<div class="my-1 border-t border-border-secondary"></div>
		<button type="button" disabled={busy} onclick={logout} class="flex items-center gap-2 hover:bg-elevated disabled:opacity-50 px-2 py-1.5 w-full text-sm text-left cursor-pointer"><Icon icon="lucide:log-out" class="size-4" /> Log out</button>
		{#if error}<p role="alert" class="px-2 py-1 text-danger text-xs">{error}</p>{/if}
	</ActionPopover>
{:else}
	<button type="button" onclick={showLogin} class="flex items-center gap-1.5 hover:bg-elevated px-2 py-1 text-sm cursor-pointer" aria-label="warframe.market profile, logged out">
		<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
		<span class="bg-muted-foreground/50 rounded-full size-2"></span>
		<span>Logged out</span>
	</button>
{/if}

{#snippet title()}Log in to warframe.market{/snippet}
{#snippet description()}Sign in with your warframe.market email and password.{/snippet}
<Dialog bind:open={loginOpen} {title} {description} contentProps={{ class: 'h-auto' }}>
	<div class="flex flex-col gap-4 px-6">
		<label class="flex flex-col gap-1 text-sm">Email
			<input type="email" autocomplete="username" bind:value={email} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" />
		</label>
		<label class="flex flex-col gap-1 text-sm">Password
			<input type="password" autocomplete="current-password" bind:value={password} class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground" />
		</label>
		<div class="flex items-center gap-2 text-sm"><Checkbox id="remember-market-login" bind:checked={remember} /><label for="remember-market-login" class="cursor-pointer">Remember me</label></div>
		{#if remember}<p class="text-muted-foreground text-xs">Your email and password will be stored in Artus's local app data without encryption. Use this only on a device you trust.</p>{/if}
		<Button variant="primary" class="self-start" disabled={busy || !email.trim() || !password} onclick={login}>{busy ? 'Signing in...' : 'Log in'}</Button>
		{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>
