<script lang="ts">
	import DataTable from '$lib/components/ui/data-table.svelte';
	import { columns, type MasterableItem } from './columns';

	import masterableItems from '$lib/data/masterable-items.json';

	const data: MasterableItem[] = masterableItems;
</script>

<DataTable {data} {columns} />
