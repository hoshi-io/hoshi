<script lang="ts">
    import { Play, Pause, SkipBack, SkipForward } from "lucide-svelte";
    import TimeBar from "@/components/player/TimeBar.svelte";

    let {
        pageState,
        showControls,
        formatTime
    }: {
        pageState: any;
        showControls: boolean;
        formatTime: (seconds: number) => string;
    } = $props();
</script>

<!-- MIDDLE CONTROLS: Center Action Buttons -->
<div class="absolute inset-0 z-20 flex items-center justify-center gap-8 pointer-events-none transition-opacity duration-300 ease-out {showControls ? 'opacity-100' : 'opacity-0'}">
    <button
            onclick={() => pageState.goToEpisode(pageState.epNumber - 1)}
            disabled={!pageState.hasPrev}
            class="p-3 rounded-full text-white pointer-events-auto disabled:opacity-30 disabled:hidden transition-transform active:scale-95"
    >
        <SkipBack class="w-8 h-8 fill-current" />
    </button>

    <button
            onclick={() => pageState.togglePlay()}
            class="p-5 rounded-full text-white pointer-events-auto transition-transform active:scale-95"
    >
        {#if pageState.isPaused}
            <Play class="w-12 h-12 fill-current ml-1" />
        {:else}
            <Pause class="w-12 h-12 fill-current" />
        {/if}
    </button>

    <button
            onclick={() => pageState.goToEpisode(pageState.epNumber + 1)}
            disabled={!pageState.hasNext}
            class="p-3 rounded-full text-white pointer-events-auto disabled:opacity-30 disabled:hidden transition-transform active:scale-95"
    >
        <SkipForward class="w-8 h-8 fill-current" />
    </button>
</div>

<!-- BOTTOM BAR: Time & Scrubbing -->
<div class="z-20 px-4 pb-3 pt-6 flex flex-col">
    <div class="flex items-center text-xs font-medium tabular-nums tracking-wide mb-2 ml-1 text-white">
        <span class="text-white">{formatTime(pageState.currentTime)}</span>
        <span class="mx-1.5 text-white/40 font-normal">/</span>
        <span class="text-white/70">{formatTime(pageState.currentDuration)}</span>
    </div>
    <TimeBar {pageState} buffered={pageState.bufferedFraction} />
</div>