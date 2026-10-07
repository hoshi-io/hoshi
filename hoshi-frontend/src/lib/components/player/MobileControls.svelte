<script lang="ts">
    import { slide } from "svelte/transition";
    import { Play, Pause, SkipBack, SkipForward } from "lucide-svelte";
    import TimeBar from "@/components/player/TimeBar.svelte";
    import EpisodeList from "@/components/player/EpisodeList.svelte";

    let {
        pageState,
        showControls,
        showEpisodes = false,
        onCloseEpisodes,
        formatTime
    }: {
        pageState: any;
        showControls: boolean;
        showEpisodes?: boolean;
        onCloseEpisodes?: () => void;
        formatTime: (seconds: number) => string;
    } = $props();

    const ptr = $derived(showControls ? "pointer-events-auto" : "pointer-events-none");
</script>

<div class="relative z-20 flex min-h-0 flex-1 flex-col pointer-events-none">

    <!-- MIDDLE CONTROLS -->
    <div class="flex min-h-0 flex-1 items-center justify-center gap-8 transition-opacity duration-300 ease-out {showControls ? 'opacity-100' : 'opacity-0'}">
        <button
                onclick={() => pageState.goToEpisode(pageState.epNumber - 1)}
                disabled={!pageState.hasPrev}
                class="p-3 rounded-full text-white {ptr} disabled:opacity-30 disabled:pointer-events-none transition-transform active:scale-95"
        >
            <SkipBack class="w-8 h-8 fill-current" />
        </button>

        <button
                onclick={() => pageState.togglePlay()}
                class="p-5 rounded-full text-white {ptr} transition-transform active:scale-95"
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
                class="p-3 rounded-full text-white {ptr} disabled:opacity-30 disabled:pointer-events-none transition-transform active:scale-95"
        >
            <SkipForward class="w-8 h-8 fill-current" />
        </button>
    </div>

    <div class="flex flex-col transition-transform duration-300 ease-out {showControls ? 'translate-y-0' : 'translate-y-full'} {ptr}">
        {#if showEpisodes}
            <div
                    transition:slide={{ duration: 300 }}
                    class="dark bg-black/70 backdrop-blur-md"
            >
                <EpisodeList {pageState} onClose={onCloseEpisodes} />
            </div>
        {/if}

        <div class="px-4 pb-3 pt-6 flex flex-col">
            <div class="flex items-center text-xs font-medium tabular-nums tracking-wide mb-2 ml-1 text-white">
                <span class="text-white">{formatTime(pageState.currentTime)}</span>
                <span class="mx-1.5 text-white/40 font-normal">/</span>
                <span class="text-white/70">{formatTime(pageState.currentDuration)}</span>
            </div>
            <TimeBar {pageState} buffered={pageState.bufferedFraction} />
        </div>
    </div>
</div>