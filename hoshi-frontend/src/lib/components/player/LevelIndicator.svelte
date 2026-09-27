<script lang="ts">
    let {
        show = false,
        value = 0,
        type = "volume",
        side = "left",
    }: {
        show?: boolean;
        value?: number;
        type?: "volume" | "brightness";
        side?: "left" | "right";
    } = $props();

    const clamped = $derived(Math.max(0, Math.min(100, value)));
</script>

<div
        class="absolute top-1/2 -translate-y-1/2 z-40 flex flex-col items-center gap-2.5 pointer-events-none transition-all duration-200 {side === 'left' ? 'left-6 md:left-10' : 'right-6 md:right-10'} {show ? 'opacity-100 scale-100' : 'opacity-0 scale-90'}"
>
    <div class="text-foreground/90 drop-shadow">
        {#if type === "brightness"}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
            </svg>
        {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                <path d="M15.54 8.46a5 5 0 0 1 0 7.07" />
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14" />
            </svg>
        {/if}
    </div>

    <!-- Vertical "slider" track, filled bottom-up to represent 0-100 -->
    <div class="relative w-2 h-32 rounded-full bg-background/50 border border-border/40 backdrop-blur-md overflow-hidden shadow-lg">
        <div
                class="absolute bottom-0 left-0 right-0 bg-foreground rounded-full transition-[height] duration-75 ease-out"
                style="height: {clamped}%"
        ></div>
    </div>

    <span class="text-xs font-semibold text-foreground/90 tabular-nums drop-shadow">{Math.round(clamped)}%</span>
</div>