<script lang="ts">
	import Icon from '@iconify/svelte';
	import { listen } from '@tauri-apps/api/event';
	import { onMount } from 'svelte';
	import { slide } from 'svelte/transition';
	import Dialog from '$lib/components/Dialog.svelte';
	import Button from '$lib/components/Button.svelte';
	import Checkbox from '$lib/components/Checkbox.svelte';
	import Collapsible from '$lib/components/Collapsible.svelte';
	import ActionPopover from '$lib/components/ActionPopover.svelte';
	import {
		loginMarket,
		logoutMarket,
		marketAccount,
		marketProfileUrl,
		rememberedMarketLogin,
		scheduleMarketInvisible,
		setMarketInvisibleOnExit,
		setMarketStatus,
		type MarketSession,
		type MarketStatus,
	} from '$lib/market-account.svelte';

	let loginOpen = $state(false);
	let profileOpen = $state(false);
	let automationOpen = $state(false);
	let email = $state('');
	let password = $state('');
	let remember = $state(false);
	let busy = $state(false);
	let savingPreference = $state(false);
	let error = $state<string | null>(null);
	let now = $state(Date.now());
	const remaining = $derived.by(() => {
		const deadline = marketAccount.session?.invisibleAt;
		if (deadline == null) return null;
		const seconds = Math.max(0, Math.ceil((deadline - now) / 1000));
		const hours = Math.floor(seconds / 3600);
		const minutes = Math.floor((seconds % 3600) / 60);
		const rest = String(seconds % 60).padStart(2, '0');
		return hours ? `${hours}:${String(minutes).padStart(2, '0')}:${rest}` : `${minutes}:${rest}`;
	});
	const expiryFormat = new Intl.DateTimeFormat(undefined, { weekday: 'short', hour: 'numeric', minute: '2-digit' });
	const invisibleDelays = [
		{ minutes: 30, label: '30 min' },
		{ minutes: 60, label: '60 min' },
		{ minutes: 120, label: '2 hours' },
		{ minutes: 240, label: '4 hours' },
	] as const;

	onMount(() => {
		let disposed = false;
		let unlistenSession: (() => void) | undefined;
		let unlistenError: (() => void) | undefined;
		const clock = setInterval(() => { now = Date.now(); }, 1000);
		void listen<MarketSession>('market_session_changed', ({ payload }) => {
			marketAccount.session = payload;
		}).then((unlisten) => { if (disposed) unlisten(); else unlistenSession = unlisten; })
			.catch((cause) => console.error('Could not listen for market status:', cause));
		void listen<string>('market_invisible_timer_error', ({ payload }) => { error = payload; })
			.then((unlisten) => { if (disposed) unlisten(); else unlistenError = unlisten; })
			.catch((cause) => console.error('Could not listen for invisible timer errors:', cause));
		return () => {
			disposed = true;
			clearInterval(clock);
			unlistenSession?.();
			unlistenError?.();
		};
	});
	const statusLabel: Record<MarketStatus, string> = {
		invisible: 'Invisible',
		online: 'Online',
		ingame: 'In game',
	};
	const dotColor: Record<MarketStatus, string> = {
		invisible: 'bg-muted-foreground',
		online: 'bg-amber-400',
		ingame: 'bg-green-500 animate-pulse',
	};
	const statusOptions: MarketStatus[] = ['invisible', 'online', 'ingame'];

	async function showLogin() {
		error = null;
		try {
			const saved = await rememberedMarketLogin();
			if (saved) {
				email = saved.email;
				password = saved.password;
				remember = true;
			}
		} catch (cause) {
			console.error('Could not load remembered market login:', cause);
		}
		loginOpen = true;
	}

	async function login() {
		busy = true;
		error = null;
		try {
			await loginMarket(email, password, remember);
			password = '';
			loginOpen = false;
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
	async function changeStatus(status: MarketStatus) {
		busy = true;
		error = null;
		try {
			await setMarketStatus(status);
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
	async function scheduleInvisible(minutes: 0 | 30 | 60 | 120 | 240) {
		busy = true;
		error = null;
		try {
			await scheduleMarketInvisible(minutes);
			now = Date.now();
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
	async function changeInvisibleOnExit(enabled: boolean) {
		savingPreference = true;
		error = null;
		try {
			await setMarketInvisibleOnExit(enabled);
		} catch (cause) {
			error = String(cause);
		} finally {
			savingPreference = false;
		}
	}
	async function logout() {
		busy = true;
		error = null;
		try {
			await logoutMarket();
			profileOpen = false;
		} catch (cause) {
			error = String(cause);
		} finally {
			busy = false;
		}
	}
</script>

{#if marketAccount.session}
	<ActionPopover
		bind:open={profileOpen}
		contentClass="w-64 max-h-[calc(100vh-1rem)] overflow-y-auto"
		triggerAriaLabel={`warframe.market profile, ${statusLabel[marketAccount.session.status]}`}
	>
		{#snippet trigger()}
			<span class="flex items-center gap-1.5 hover:bg-elevated px-2 py-1 border text-sm">
				<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
				<span
					class={`rounded-full size-2  ${dotColor[marketAccount.session?.status ?? 'invisible']}`}
				></span>
				<span>{statusLabel[marketAccount.session?.status ?? 'invisible']}</span>
			</span>
		{/snippet}
		<a
			href={marketProfileUrl(marketAccount.session)}
			target="_blank"
			rel="noopener noreferrer"
			onclick={() => (profileOpen = false)}
			class="flex justify-between items-center gap-2 hover:bg-elevated p-2 font-semibold text-sm"
		>
			<span class="truncate">{marketAccount.session.ingameName}</span>
			<Icon icon="lucide:external-link" class="size-3.5 shrink-0" />
		</a>
		<div class="my-1 border-border-secondary border-t"></div>
		<p class="px-2 py-1 text-muted-foreground text-xs uppercase tracking-wider">Activity</p>
		{#each statusOptions as status}
			<button
				type="button"
				disabled={busy}
				onclick={() => changeStatus(status)}
				class="flex items-center gap-2 hover:bg-elevated disabled:opacity-50 px-2 py-1.5 w-full text-sm text-left cursor-pointer"
			>
				<span class={`rounded-full mx-1 size-2 ${dotColor[status]}`}></span>
				<span class="flex-1">{statusLabel[status]}</span>
				{#if marketAccount.session.status === status}<Icon
						icon="lucide:check"
						class="size-4 text-accent"
					/>{/if}
			</button>
		{/each}
		<div class="my-1 border-border-secondary border-t"></div>
		<Collapsible bind:open={automationOpen} triggerClass="w-full text-left cursor-pointer hover:bg-elevated px-2 py-2">
			{#snippet button(open)}
				<span class="flex items-center gap-2 w-full text-sm">
					<Icon icon="lucide:timer" class="size-4 shrink-0" />
					<span class="flex-1">Automatic invisibility</span>
					{#if marketAccount.session?.invisibleAt != null}<span class="text-accent text-[10px]">Timer active</span>{/if}
					<Icon icon="lucide:chevron-down" class={`size-4 text-muted-foreground transition-transform ${open ? 'rotate-180' : ''}`} />
				</span>
			{/snippet}
			{#snippet content(open)}
				{@const deadline = marketAccount.session?.invisibleAt}
				{#if open}
					<div transition:slide={{ duration: 180 }} class="pb-1">
						{#if deadline != null}
							<div class="px-2 py-1.5 text-xs">
								<div class="flex items-center justify-between gap-2">
									<span class="text-muted-foreground">Invisible in <span class="text-foreground tabular-nums">{remaining}</span></span>
									<button type="button" disabled={busy} onclick={() => scheduleInvisible(0)} class="text-accent hover:underline disabled:opacity-50 cursor-pointer">Cancel</button>
								</div>
								<time datetime={new Date(deadline).toISOString()} class="text-muted-foreground">At {expiryFormat.format(deadline)}</time>
							</div>
						{/if}
						<p class="px-2 py-1 text-muted-foreground text-xs uppercase tracking-wider">Go invisible after</p>
						<div class="grid grid-cols-2 gap-1 px-2 pb-2">
							{#each invisibleDelays as delay}
								<button type="button" disabled={busy || marketAccount.session?.status === 'invisible'} onclick={() => scheduleInvisible(delay.minutes)} class="hover:bg-elevated disabled:opacity-40 px-2 py-1.5 border border-border-secondary text-xs cursor-pointer disabled:cursor-not-allowed">{delay.label}</button>
							{/each}
						</div>
						<div class="my-1 border-border-secondary border-t"></div>
						<div class="flex items-start gap-2 px-2 py-2 text-xs">
							<Checkbox id="market-invisible-on-exit" checked={marketAccount.invisibleOnExit} disabled={savingPreference} onCheckedChange={(checked) => changeInvisibleOnExit(checked === true)} />
							<label for="market-invisible-on-exit" class="cursor-pointer">Always go invisible after closing the app</label>
						</div>
					</div>
				{/if}
			{/snippet}
		</Collapsible>
		<div class="my-1 border-border-secondary border-t"></div>
		<button
			type="button"
			disabled={busy}
			onclick={logout}
			class="flex items-center gap-2 hover:bg-elevated disabled:opacity-50 px-2 py-1.5 w-full text-sm text-left cursor-pointer"
		>
			<Icon icon="lucide:log-out" class="size-4" /> Log out
		</button>
		{#if error}<p role="alert" class="px-2 py-1 text-danger text-xs">{error}</p>{/if}
	</ActionPopover>
{:else}
	<button
		type="button"
		onclick={showLogin}
		class="flex items-center gap-1.5 hover:bg-elevated px-2 py-1 border text-sm cursor-pointer"
		aria-label="warframe.market profile, logged out"
	>
		<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
		<!-- <span class="bg-muted-foreground/50 rounded-full size-2"></span> -->
		<span>Sign in</span>
	</button>
{/if}

{#snippet title()}Log in to warframe.market{/snippet}
{#snippet description()}Sign in with your warframe.market email and password.{/snippet}
<Dialog bind:open={loginOpen} {title} {description} contentProps={{ class: 'h-auto' }}>
	<div class="flex flex-col gap-4 px-6">
		<label class="flex flex-col gap-1 text-sm">
			Email
			<input
				type="email"
				autocomplete="username"
				bind:value={email}
				class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground"
			/>
		</label>
		<label class="flex flex-col gap-1 text-sm">
			Password
			<input
				type="password"
				autocomplete="current-password"
				bind:value={password}
				class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground"
			/>
		</label>
		<div class="flex items-center gap-2 text-sm">
			<Checkbox id="remember-market-login" bind:checked={remember} />
			<label for="remember-market-login" class="cursor-pointer">Remember me</label>
		</div>
		{#if remember}<p class="text-muted-foreground text-xs">
				Your email and password will be stored in Artus's local app data without encryption. Use
				this only on a device you trust.
			</p>{/if}
		<Button
			variant="primary"
			class="self-start"
			disabled={busy || !email.trim() || !password}
			onclick={login}
		>
			{busy ? 'Signing in...' : 'Log in'}
		</Button>
		{#if error}<p role="alert" class="text-danger text-sm">{error}</p>{/if}
	</div>
</Dialog>
