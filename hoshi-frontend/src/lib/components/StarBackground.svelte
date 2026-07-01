<script lang="ts">
    import { themeManager } from "@/stores/theme.svelte.js";

    const starCount = 80;
    const stars = Array.from({ length: starCount }, (_, i) => ({
        id: i,
        x: Math.random() * 100,
        y: Math.random() * 100,
        size: Math.random() * 2 + 1,
        delay: Math.random() * 5,
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
                class="absolute rounded-full transition-all duration-500 {starClass} {star.size > 2 ? 'animate-pulse' : ''}"
                style="
                left: {star.x}%;
                top: {star.y}%;
                width: {star.size}px;
                height: {star.size}px;
                animation-delay: {star.delay}s;
                animation-duration: 3s;
            "
        ></div>
    {/each}
</div>

<style>
    :global(.animate-pulse) {
        animation: twinkle 4s infinite ease-in-out;
    }
    @keyframes twinkle {
        0%, 100% { opacity: 0.2; }
        50% { opacity: 0.9; }
    }
</style>