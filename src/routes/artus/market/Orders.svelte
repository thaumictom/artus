<script lang="ts">
    import { untrack } from "svelte";
    import { Slider } from "bits-ui"; // Adjust based on your setup
    import type { z } from "zod";
    import type { GetOrdersResponseSchema } from "$lib/schemas"; // Replace with your path

    // 1. Adjusted prop type to target the 'data' array directly
    let {
        orderData = [],
    }: { orderData: z.infer<typeof GetOrdersResponseSchema>["data"] } =
        $props();

    // Properties for grouping/display
    const GROUP_PROPS = [
        "itemId",
        "rank",
        "charges",
        "subtype",
        "amberStars",
        "cyanStars",
    ] as const;
    // Sliders only apply to numerical properties
    const NUMERIC_PROPS = [
        "rank",
        "charges",
        "amberStars",
        "cyanStars",
    ] as const;

    // Calculate maximums and check if properties exist in the current dataset
    let propertyStats = $derived.by(() => {
        const stats = {
            rank: { max: 0, present: false },
            charges: { max: 0, present: false },
            amberStars: { max: 0, present: false },
            cyanStars: { max: 0, present: false },
        };

        if (!orderData?.length) return stats;

        for (const order of orderData) {
            for (const prop of NUMERIC_PROPS) {
                if (order[prop] !== undefined) {
                    stats[prop].present = true;
                    stats[prop].max = Math.max(
                        stats[prop].max,
                        order[prop] as number,
                    );
                }
            }
        }
        return stats;
    });

    // State to hold [min, max] values for each slider
    let filters = $state<Record<string, number[]>>({
        rank: [0, 0],
        charges: [0, 0],
        amberStars: [0, 0],
        cyanStars: [0, 0],
    });

    // Initialize slider boundaries once when new data loads
    $effect(() => {
        let _ = orderData; // Track orderData changes
        untrack(() => {
            for (const prop of NUMERIC_PROPS) {
                if (propertyStats[prop].present) {
                    if (
                        filters[prop][1] === 0 ||
                        filters[prop][1] > propertyStats[prop].max
                    ) {
                        filters[prop] = [0, propertyStats[prop].max];
                    }
                }
            }
        });
    });

    // Filter and sort derived arrays
    let filteredOrders = $derived.by(() => {
        if (!orderData?.length) return [];
        return orderData.filter((order) => {
            // 2. Only show in-game users
            if (order.user.status !== "ingame") return false;

            // Numeric sliders filter
            for (const prop of NUMERIC_PROPS) {
                if (propertyStats[prop].present && order[prop] !== undefined) {
                    const [min, max] = filters[prop];
                    if (order[prop]! < min || order[prop]! > max) return false;
                }
            }
            return true;
        });
    });

    let sellOrders = $derived(
        filteredOrders
            .filter((o) => o.type === "sell")
            .sort((a, b) => a.platinum - b.platinum)
            .slice(0, 10),
    );

    let buyOrders = $derived(
        filteredOrders
            .filter((o) => o.type === "buy")
            .sort((a, b) => b.platinum - a.platinum)
            .slice(0, 10),
    );
</script>

<div class="flex flex-col gap-4">
    <aside class="col-span-1 space-y-6">
        <h2 class="text-xl font-bold">Filters</h2>

        {#each NUMERIC_PROPS as prop}
            {#if propertyStats[prop].present}
                <div class="space-y-3">
                    <div class="flex items-center justify-between text-sm">
                        <span class="capitalize font-medium">{prop}</span>
                        <span class="text-muted-foreground"
                            >{filters[prop][0]} - {filters[prop][1]}</span
                        >
                    </div>

                    <Slider.Root
                        bind:value={filters[prop]}
                        min={0}
                        max={propertyStats[prop].max}
                        step={1}
                        type="multiple"
                        class="relative flex items-center bg-surface border w-full h-1.5 has-data-active:h-2.5 group-hover:h-2.5 transition-[height] touch-none select-none"
                    >
                        <Slider.Range class="absolute bg-foreground h-full" />
                        <Slider.Thumb index={0} class="group">
                            <div
                                class="bg-foreground border size-4.5 group-data-active:size-5.5 transition-all cursor-e-resize"
                            ></div>
                        </Slider.Thumb>
                        <Slider.Thumb index={1} class="group">
                            <div
                                class="bg-foreground border size-4.5 group-data-active:size-5.5 transition-all cursor-e-resize"
                            ></div>
                        </Slider.Thumb>
                    </Slider.Root>
                </div>
            {/if}
        {/each}
    </aside>

    <!-- Sell Orders -->
    <div class="space-y-4">
        <h2 class="text-xl font-bold">Top 10 Sell Orders</h2>
        <div class="flex flex-col gap-2">
            {#each sellOrders as order (order.id)}
                <div
                    class="flex items-center justify-between p-3 border rounded-md bg-card"
                >
                    <div class="flex flex-col">
                        <span class="font-medium">{order.user.ingameName}</span>
                        <!-- 3. Add column/badges for optional grouping values -->
                        <div class="flex flex-wrap gap-1.5 mt-1">
                            {#each GROUP_PROPS as prop}
                                {#if order[prop] !== undefined}
                                    <span
                                        class="text-[10px] bg-muted px-1.5 py-0.5 rounded-sm capitalize border text-muted-foreground"
                                    >
                                        {prop}: {order[prop]}
                                    </span>
                                {/if}
                            {/each}
                        </div>
                    </div>
                    <span class="text-blue-500 font-bold"
                        >{order.platinum}p</span
                    >
                </div>
            {/each}
            {#if sellOrders.length === 0}
                <p class="text-sm text-muted-foreground">
                    No sell orders match your filters.
                </p>
            {/if}
        </div>
    </div>

    <!-- Buy Orders -->
    <div class="space-y-4">
        <h2 class="text-xl font-bold">Top 10 Buy Orders</h2>
        <div class="flex flex-col gap-2">
            {#each buyOrders as order (order.id)}
                <div
                    class="flex items-center justify-between p-3 border rounded-md bg-card"
                >
                    <div class="flex flex-col">
                        <span class="font-medium">{order.user.ingameName}</span>
                        <!-- 3. Add column/badges for optional grouping values -->
                        <div class="flex flex-wrap gap-1.5 mt-1">
                            {#each GROUP_PROPS as prop}
                                {#if order[prop] !== undefined}
                                    <span
                                        class="text-[10px] bg-muted px-1.5 py-0.5 rounded-sm capitalize border text-muted-foreground"
                                    >
                                        {prop}: {order[prop]}
                                    </span>
                                {/if}
                            {/each}
                        </div>
                    </div>
                    <span class="text-green-500 font-bold"
                        >{order.platinum}p</span
                    >
                </div>
            {/each}
            {#if buyOrders.length === 0}
                <p class="text-sm text-muted-foreground">
                    No buy orders match your filters.
                </p>
            {/if}
        </div>
    </div>
</div>
