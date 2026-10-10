<script lang="ts">
    import { themeManager } from "@/stores/theme.svelte.js";
    import { fade } from "svelte/transition";

    let { step = 0 }: { step?: number } = $props();

    const layers = [
        { shift: 26,  duration: 1300, opacity: 0.55, size: 1 },   // far
        { shift: 60,  duration: 1100, opacity: 0.8,  size: 1.5 }, // mid
        { shift: 104, duration: 900,  opacity: 1,    size: 2.2 }  // near
    ];

    const starsPerLayer = 55;

    const stars = layers.map((layer, li) =>
        Array.from({ length: starsPerLayer }, (_, i) => ({
            id: `${li}-${i}`,
            x: Math.random() * 100,
            y: Math.random() * 100,
            size: layer.size + Math.random() * 0.9,
            delay: Math.random() * 6,
            duration: 2.5 + Math.random() * 3.5,
            driftX: (Math.random() - 0.5) * 20,
            driftY: (Math.random() - 0.5) * 20,
            driftDuration: 15 + Math.random() * 20,
        }))
    );

    const starClass = $derived(
        themeManager.theme === 'light'
            ? 'bg-zinc-400/40'
            : 'bg-white'
    );
</script>

<div class="absolute inset-0 overflow-hidden pointer-events-none z-0" aria-hidden="true" in:fade={{ duration: 900 }}>
    {#each layers as layer, li (li)}
        <div
                class="layer absolute -inset-[260px]"
                style="
                opacity: {layer.opacity};
                transition-duration: {layer.duration}ms;
                transform: translate3d({-step * layer.shift}px, {-step * layer.shift * 0.35}px, 0);
            "
        >
            {#each stars[li] as star (star.id)}
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
    {/each}
</div>

<style>
    .layer {
        transition-property: transform;
        transition-timing-function: cubic-bezier(0.22, 1, 0.36, 1);
        will-change: transform;
    }

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

    @media (prefers-reduced-motion: reduce) {
        .layer { transition: none; }
        .twinkle, .drift { animation: none; }
    }
</style>