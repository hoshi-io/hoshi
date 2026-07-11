<script lang="ts">
    import { themeManager } from "@/stores/theme.svelte.js";

    const starCount = 90;
    const stars = Array.from({ length: starCount }, (_, i) => ({
        id: i,
        x: Math.random() * 100,
        y: Math.random() * 100,
        size: Math.random() * 2 + 1,
        delay: Math.random() * 6,
        duration: 2.5 + Math.random() * 3.5,
        driftX: (Math.random() - 0.5) * 20,
        driftY: (Math.random() - 0.5) * 20,
        driftDuration: 15 + Math.random() * 20,
    }));

    const starClass = $derived(
        themeManager.theme === 'light'
            ? 'bg-zinc-400/40'
            : 'bg-white'
    );
</script>

<div class="absolute inset-0 overflow-hidden pointer-events-none z-0">
    {#each stars as star (star.id)}
        <div
                class="absolute rounded-full drift {starClass}"
                style="
                left: {star.x}%;
                top: {star.y}%;
                width: {star.size}px;
                height: {star.size}px;
                --drift-x: {star.driftX}px;
                --drift-y: {star.driftY}px;
                --drift-duration: {star.driftDuration}s;
                animation-delay: {star.delay * 0.3}s;
            "
        >
            <div
                    class="w-full h-full rounded-full {starClass} twinkle"
                    style="
                    animation-delay: {star.delay}s;
                    animation-duration: {star.duration}s;
                "
            ></div>
        </div>
    {/each}
</div>

<style>
    .twinkle {
        animation-name: twinkle;
        animation-timing-function: ease-in-out;
        animation-iteration-count: infinite;
    }

    @keyframes twinkle {
        0%, 100% { opacity: 0.15; transform: scale(0.85); }
        50% { opacity: 1; transform: scale(1.15); }
    }

    .drift {
        animation-name: drift;
        animation-timing-function: ease-in-out;
        animation-iteration-count: infinite;
        animation-duration: var(--drift-duration);
    }

    @keyframes drift {
        0%, 100% { transform: translate(0, 0); }
        50% { transform: translate(var(--drift-x), var(--drift-y)); }
    }
</style>