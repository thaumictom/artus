<script lang="ts">
	import Icon from '@iconify/svelte';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';

	type SidebarSection = {
		id: string;
		label: string;
		icon: string;
	};

	let {
		sections,
		activeSection = $bindable(),
		isOpen = $bindable(false),
	}: {
		sections: SidebarSection[];
		activeSection: SidebarSection;
		isOpen?: boolean;
	} = $props();
</script>

<Sidebar.Root
	class="border-0!"
	collapsible="icon"
	onmouseenter={() => (isOpen = false)}
	onmouseleave={() => (isOpen = false)}
>
	<Sidebar.Content>
		<Sidebar.Group>
			<Sidebar.GroupContent>
				<Sidebar.Menu>
					{#each sections as section (section.id)}
						<Sidebar.MenuItem>
							<Sidebar.MenuButton
								isActive={activeSection?.id === section.id}
								onclick={() => (activeSection = section)}
							>
								<Icon icon={section.icon} class="text-2xl" aria-hidden="true"></Icon>
								<span>{section.label}</span>
							</Sidebar.MenuButton>
						</Sidebar.MenuItem>
					{/each}
				</Sidebar.Menu>
			</Sidebar.GroupContent>
		</Sidebar.Group>
	</Sidebar.Content>
</Sidebar.Root>
