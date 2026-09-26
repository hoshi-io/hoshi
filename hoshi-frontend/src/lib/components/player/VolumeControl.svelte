<script lang="ts">
    import { Volume2, Volume1, VolumeX } from "lucide-svelte";
    import type { WatchState } from "@/app/watch.svelte.js";

    let { pageState }: { pageState: WatchState } = $props();

    let isDragging = $state(false);

    const currentVol = $derived(pageState.isMuted ? 0 : pageState.volume);

    function handlePointerDown() {
        isDragging = true;
    }

    function handlePointerUp() {
        isDragging = false;
    }
</script>

<svelte:window
        onpointerup={isDragging ? handlePointerUp : undefined}
        onpointercancel={isDragging ? handlePointerUp : undefined}
/>

<!-- Generous hover target area around volume control -->
<div class="relative flex items-center group/volume ml-1 py-2 px-1">
    <!-- Volume Icon Button -->
    <button
            onclick={() => pageState.toggleMute()}
            class="p-1.5 rounded-lg hover:bg-white/10 transition-colors text-white/90 hover:text-white shrink-0"
            aria-label={pageState.isMuted ? "Unmute" : "Mute"}
    >
        {#if pageState.isMuted || pageState.volume === 0}
            <VolumeX class="w-5 h-5" />
        {:else if pageState.volume < 50}
            <Volume1 class="w-5 h-5" />
        {:else}
            <Volume2 class="w-5 h-5" />
        {/if}
    </button>

    <!-- Expandable Volume Slider (Kept open during dragging) -->
    <div class="transition-all duration-300 ease-out overflow-hidden flex items-center {isDragging ? 'w-24 opacity-100' : 'w-0 opacity-0 group-hover/volume:w-24 group-hover/volume:opacity-100 focus-within:w-24 focus-within:opacity-100'}">
        <!-- Visual Track Wrapper -->
        <div class="relative w-20 h-1.5 bg-white/20 rounded-full ml-2 shrink-0 flex items-center">
            <!-- Smooth fill on click, INSTANT tracking while dragging -->
            <div
                    class="absolute inset-y-0 left-0 bg-white rounded-full pointer-events-none {isDragging ? 'transition-none' : 'transition-all duration-150 ease-out'}"
                    style="width: {currentVol}%"
            ></div>

            <!-- Taller invisible interactive input for a generous vertical click/drag hit target -->
            <input
                    type="range"
                    min="0"
                    max="100"
                    value={currentVol}
                    onpointerdown={handlePointerDown}
                    oninput={(e) => {
                        const val = Number((e.target as HTMLInputElement).value);
                        pageState.setVolume(val);
                        if (pageState.isMuted && val > 0) {
                            pageState.toggleMute();
                        }
                    }}
                    class="absolute -inset-y-2 -inset-x-1 w-[calc(100%+8px)] h-6 opacity-0 cursor-pointer z-10"
                    aria-label="Volume"
            />
        </div>
    </div>
</div>