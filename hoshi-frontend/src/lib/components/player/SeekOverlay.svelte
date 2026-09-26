<script lang="ts">
    import { Rewind, FastForward } from "lucide-svelte";
    import { scale } from "svelte/transition";

    let {
        show,
        side,
        amount
    }: {
        show: boolean;
        side: 'left' | 'right';
        amount: number;
    } = $props();
</script>

{#if show}
    <div class="absolute inset-y-0 {side === 'left' ? 'left-0' : 'right-0'} w-1/2 flex items-center justify-center pointer-events-none z-30">
        <div
                transition:scale={{ duration: 200, start: 0.85 }}
                class="bg-black/50 rounded-full w-24 h-24 flex flex-col items-center justify-center shadow-2xl"
        >
            {#if side === 'left'}
                <Rewind class="w-8 h-8 text-white fill-current mb-1" />
            {:else}
                <FastForward class="w-8 h-8 text-white fill-current mb-1" />
            {/if}
            <span class="text-white font-bold text-sm tracking-wide">
                {amount > 0 ? `+${amount}` : amount}s
            </span>
        </div>
    </div>
{/if}