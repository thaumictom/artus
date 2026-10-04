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
	const expiryFormat = new Intl.DateTimeFormat(undefined, {
		weekday: 'short',
		hour: 'numeric',
		minute: '2-digit',
	});
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
		const clock = setInterval(() => {
			now = Date.now();
		}, 1000);
		void listen<MarketSession>('market_session_changed', ({ payload }) => {
			marketAccount.session = payload;
		})
			.then((unlisten) => {
				if (disposed) unlisten();
				else unlistenSession = unlisten;
			})
			.catch((cause) => console.error('Could not listen for market status:', cause));
		void listen<string>('market_invisible_timer_error', ({ payload }) => {
			error = payload;
		})
			.then((unlisten) => {
				if (disposed) unlisten();
				else unlistenError = unlisten;
			})
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
		online: 'bg-emerald-500',
		ingame: 'bg-purple-400',
	};
	const statusOptions: MarketStatus[] = ['invisible', 'online', 'ingame'];

	function showLogin() {
		error = null;
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
			<Button class="flex items-center gap-1 hover:bg-elevated text-sm" size="small">
				<Icon icon="material-symbols:account-circle-outline-rounded" class="size-5" />
				<span
					class={`rounded-full size-2 ${dotColor[marketAccount.session?.status ?? 'invisible']} ${marketAccount.session?.status === 'ingame' ? 'animate-pulse' : ''}`}
				></span>
				<span>{statusLabel[marketAccount.session?.status ?? 'invisible']}</span>
			</Button>
		{/snippet}
		<div class="flex flex-col text-base">
			<Button
				variant="ghost"
				href={marketProfileUrl(marketAccount.session)}
				target="_blank"
				rel="noopener noreferrer"
				onclick={() => (profileOpen = false)}
				class="flex justify-between items-center"
			>
				<span class="truncate">{marketAccount.session.ingameName}</span>
				<Icon icon="material-symbols:arrow-outward-rounded" class="size-4 shrink-0" />
			</Button>
			<div class="my-1 bg-border-secondary h-px"></div>
			<p class="px-3 py-1 text-muted-foreground text-sm uppercase tracking-widest">Status</p>
			{#each statusOptions as status}
				<Button
					variant="ghost"
					type="button"
					disabled={busy}
					onclick={() => changeStatus(status)}
					class="flex justify-between items-center"
				>
					<div class="flex items-center gap-3 text-trim">
						<div class={`size-2 rounded-full ${dotColor[status]}`}></div>
						<span class="flex-1">{statusLabel[status]}</span>
					</div>
					{#if marketAccount.session.status === status}<Icon
							icon="material-symbols:check-rounded"
							class="size-4 text-accent"
						/>{/if}
				</Button>
			{/each}
			<div class="my-1 bg-border-secondary h-px"></div>
			<Collapsible bind:open={automationOpen} triggerClass="w-full">
				{#snippet button(open)}
					<Button
						variant="ghost"
						type="button"
						class="flex justify-between items-center pl-1 w-full"
					>
						<div class="flex items-center gap-1 text-trim">
							{#if marketAccount.session?.invisibleAt != null}
								<Icon
									icon="material-symbols:timer-rounded"
									class="mx-1 size-4 animate-pulse shrink-0"
								/>
							{:else}
								<Icon icon="material-symbols:timer-outline-rounded" class="mx-1 size-4 shrink-0" />
							{/if}
							<span class="flex-1">Status timer</span>
							{#if marketAccount.session?.invisibleAt != null}
								<span class="font-medium text-muted-foreground text-sm">(active)</span>
							{/if}
						</div>

						<Icon
							icon="material-symbols:keyboard-arrow-down-rounded"
							class={`size-4 text-muted-foreground transition-transform ${open ? 'rotate-180' : ''}`}
						/>
					</Button>
				{/snippet}
				{#snippet content(open)}
					{@const deadline = marketAccount.session?.invisibleAt}
					{#if open}
						<div transition:slide={{ duration: 180 }}>
							<div class="flex flex-col gap-1 px-3 py-2">
								<p class="text-muted-foreground text-sm uppercase tracking-widest">
									Go invisible after
								</p>
								<div class="gap-1 grid grid-cols-2">
									{#each invisibleDelays as delay}
										<Button
											type="button"
											disabled={busy || marketAccount.session?.status === 'invisible'}
											onclick={() => scheduleInvisible(delay.minutes)}
											class="py-1 text-sm"
										>
											{delay.label}
										</Button>
									{/each}
								</div>
								{#if deadline != null}
									<div class="text-sm">
										<div class="flex justify-between items-center gap-2">
											<span class="text-muted-foreground">
												going invisible in <span class="tabular-nums text-foreground">
													{remaining}
												</span>
											</span>
											<Button
												variant="link"
												size="none"
												type="button"
												disabled={busy}
												onclick={() => scheduleInvisible(0)}
												class="disabled:opacity-50 text-accent hover:underline cursor-pointer"
											>
												cancel
											</Button>
										</div>
									</div>
								{/if}
							</div>
							<div class="my-1 bg-border-secondary h-px"></div>
							<div class="flex items-center gap-2 px-2 py-2 text-base">
								<Checkbox
									id="market-invisible-on-exit"
									class="size-4"
									checked={marketAccount.invisibleOnExit}
									disabled={savingPreference}
									onCheckedChange={(checked) => changeInvisibleOnExit(checked === true)}
								/>
								<label for="market-invisible-on-exit" class="text-trim cursor-pointer">
									Go invisible when Artus exits
								</label>
							</div>
						</div>
					{/if}
				{/snippet}
			</Collapsible>
			<div class="my-1 bg-border-secondary h-px"></div>
			<Button
				variant="ghost"
				type="button"
				class="flex items-center gap-1 pl-1 w-full text-trim"
				disabled={busy}
				onclick={logout}
			>
				<Icon icon="lucide:log-out" class="mx-1 size-4" />
				<span>Sign out</span>
			</Button>

			{#if error}
				<div class="my-1 bg-border-secondary h-px"></div>
				<p role="alert" class="px-3 py-1 font-bold text-danger text-sm">
					error: {error}
				</p>
			{/if}
		</div>
	</ActionPopover>
{:else}
	<Button
		type="button"
		onclick={showLogin}
		class="flex items-center gap-1 hover:bg-elevated"
		size="small"
		aria-label="warframe.market profile, logged out"
	>
		<Icon icon="material-symbols:account-circle-outline-rounded" class="size-4" />
		<span>Sign in</span>
	</Button>
{/if}

{#snippet title()}Log in to warframe.market{/snippet}
{#snippet description()}Sign in with your warframe.market email and password.{/snippet}
<Dialog bind:open={loginOpen} {title} {description} contentProps={{ class: 'h-auto' }}>
	<div class="flex flex-col gap-4 px-6">
		<label class="flex flex-col gap-1 text-base">
			Email
			<input
				type="email"
				autocomplete="username"
				bind:value={email}
				class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground"
			/>
		</label>
		<label class="flex flex-col gap-1 text-base">
			Password
			<input
				type="password"
				autocomplete="current-password"
				bind:value={password}
				class="bg-background p-2 border border-border-secondary focus-visible:border-accent outline-none text-foreground"
			/>
		</label>
		<div class="flex items-center gap-2 text-base">
			<Checkbox id="remember-market-login" bind:checked={remember} />
			<label for="remember-market-login" class="cursor-pointer">Remember me</label>
		</div>
		{#if remember}<p class="text-muted-foreground text-sm">
				Artus stores only your authorization token in local app data without encryption. Anyone with
				the token can access your account until it expires or is revoked.
			</p>{/if}
		<Button
			variant="primary"
			class="self-start"
			disabled={busy || !email.trim() || !password}
			onclick={login}
		>
			{busy ? 'Signing in...' : 'Log in'}
		</Button>
		{#if error}<p role="alert" class="text-danger text-base">{error}</p>{/if}
	</div>
</Dialog>
