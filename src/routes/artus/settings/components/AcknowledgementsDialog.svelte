<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { OverlayScrollbarsComponent } from 'overlayscrollbars-svelte';
	import Button from '$lib/components/Button.svelte';
	import Dialog from '$lib/components/Dialog.svelte';
	import browseLicense from '$lib/data/BROWSE-WF-LICENSE.txt?raw';

	let linkError = $state(false);

	async function openLink(url: string) {
		try {
			await openUrl(url);
			linkError = false;
		} catch {
			linkError = true;
		}
	}
</script>

{#snippet creditLink(label: string, url: string)}
	<a
		href={url}
		class="text-accent hover:underline"
		onclick={(event) => {
			event.preventDefault();
			void openLink(url);
		}}
	>
		{label}
	</a>
{/snippet}

{#snippet trigger()}<Button>Acknowledgements</Button>{/snippet}
{#snippet title()}Acknowledgements{/snippet}
{#snippet description()}
	{#if linkError}<span role="status" class="text-danger">Could not open the link.</span>{/if}
{/snippet}
{#snippet dialogClose()}<Button>Close</Button>{/snippet}

<Dialog
	{trigger}
	{title}
	{description}
	{dialogClose}
	contentProps={{ class: 'grid-rows-[auto_minmax(0,1fr)_auto]' }}
>
	<OverlayScrollbarsComponent
		defer
		class="flex-1 mr-1.75 min-w-0 min-h-0"
		options={{ scrollbars: { theme: 'os-theme-light', autoHide: 'move' } }}
	>
		<div class="flex flex-col gap-6 pr-4.25 pb-1 pl-6 text-sm">
			<section class="flex flex-col gap-2" aria-labelledby="acknowledgements-data">
				<h3 id="acknowledgements-data" class="font-semibold text-base">Data and services</h3>
				<ul class="flex flex-col gap-2 pl-5 text-muted-foreground list-disc">
					<li>
						{@render creditLink('WFInfo', 'https://github.com/WFCD/WFInfo')} has been a major inspiration
						for this project. Licensed under
						{@render creditLink(
							'Apache-2.0 license',
							'https://github.com/WFCD/WFinfo/blob/master/LICENSE.txt',
						)}.
					</li>
					<li>
						{@render creditLink('warframe.market', 'https://warframe.market/')} supplies trading listings,
						prices, and market history.
					</li>
					<li>
						{@render creditLink('WARFRAME Wiki contributors', 'https://wiki.warframe.com/')} supplies
						rotational data. Licensed under
						{@render creditLink(
							'CC BY-NC-SA 3.0',
							'https://creativecommons.org/licenses/by-nc-sa/3.0/',
						)}.
					</li>
					<li>
						{@render creditLink('warframe-items', 'https://github.com/WFCD/warframe-items')} supplies
						item metadata. Licensed under
						{@render creditLink(
							'MIT license',
							'https://github.com/WFCD/warframe-items/blob/master/LICENSE',
						)}.
					</li>
					<li>
						{@render creditLink(
							'world-state-data',
							'https://github.com/WFCD/warframe-worldstate-data',
						)} supplies Warframe world-state data. Licensed under
						{@render creditLink(
							'MIT license',
							'https://github.com/WFCD/warframe-worldstate-data/blob/master/LICENSE',
						)}.
					</li>
					<li>
						{@render creditLink(
							'world-state-parser',
							'https://github.com/WFCD/warframe-worldstate-parser',
						)} supplies parsing Warframe world-state data. Licensed under
						{@render creditLink(
							'MIT license',
							'https://github.com/WFCD/warframe-worldstate-parser/blob/master/LICENSE',
						)}.
					</li>
					<li>
						{@render creditLink('browse.wf', 'https://browse.wf/')} supplies additional world-state and
						Arbitration data. Licensed under {@render creditLink(
							'MIT license',
							'https://github.com/calamity-inc/browse.wf/blob/senpai/LICENSE',
						)}.
					</li>
					<li>Additional Arbitration tier ratings are provided by Arbitration Goons.</li>
				</ul>
			</section>

			<section class="flex flex-col gap-2" aria-labelledby="acknowledgements-software">
				<h3 id="acknowledgements-software" class="font-semibold text-base">
					Open-source foundations
				</h3>
				<ul class="flex flex-col gap-2 pl-5 text-muted-foreground list-disc">
					<li>
						{@render creditLink('Tauri', 'https://tauri.app/')} is licensed under
						{@render creditLink('MIT', 'https://github.com/tauri-apps/tauri/blob/dev/LICENSE-MIT')}
						or {@render creditLink(
							'Apache-2.0',
							'https://github.com/tauri-apps/tauri/blob/dev/LICENSE-APACHE-2.0',
						)} where applicable.
					</li>
					<li>
						{@render creditLink('Svelte', 'https://svelte.dev/')} is licensed under
						{@render creditLink(
							'MIT license',
							'https://github.com/sveltejs/svelte/blob/main/LICENSE.md',
						)}.
					</li>
					<li>
						{@render creditLink('Tailwind CSS', 'https://tailwindcss.com/')} is licensed under
						{@render creditLink(
							'MIT license',
							'https://github.com/tailwindlabs/tailwindcss/blob/main/LICENSE',
						)}.
					</li>
					<li>
						{@render creditLink('Tesseract OCR', 'https://github.com/tesseract-ocr/tesseract')} and the
						English language data are licensed under
						{@render creditLink(
							'Apache-2.0',
							'https://github.com/tesseract-ocr/tesseract/blob/main/LICENSE',
						)}.
					</li>
					<li>
						{@render creditLink('Iconify', 'https://iconify.design/')} is licensed under {@render creditLink(
							'MIT license',
							'https://github.com/iconify/iconify/blob/main/license.txt',
						)}.
					</li>
					<li>
						{@render creditLink('Archivo', 'https://fonts.google.com/specimen/Archivo')} is licensed
						under {@render creditLink(
							'SIL Open Font License 1.1',
							'https://github.com/Omnibus-Type/Archivo/blob/master/OFL.txt',
						)}.
					</li>
				</ul>
			</section>

			<section class="flex flex-col gap-2" aria-labelledby="acknowledgements-artus">
				<h3 id="acknowledgements-artus" class="font-semibold text-base">Artus</h3>
				<p>
					Artus is not affiliated with Digital Extremes, the developer of {@render creditLink(
						'Warframe',
						'https://www.warframe.com/',
					)}.
				</p>
				<p>
					It is licensed under the {@render creditLink(
						'GPL-3.0 license',
						'https://github.com/thaumictom/artus/blob/main/LICENSE',
					)}. The source code is available on {@render creditLink(
						'GitHub',
						'https://github.com/thaumictom/artus',
					)}.
				</p>
			</section>
		</div>
	</OverlayScrollbarsComponent>
</Dialog>
