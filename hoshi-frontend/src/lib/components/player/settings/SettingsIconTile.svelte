<script lang="ts">
    import type { Component } from "svelte";

    let { icon }: { icon: Component } = $props();
    const Icon = $derived(icon);
</script>

<!-- Parent must have the `group` class. Tint is based on foreground so it never matches the row's hover background. -->
<span class="flex items-center justify-center w-7 h-7 shrink-0 rounded-sm bg-foreground/5 group-hover:bg-foreground/10 transition-colors">
    <Icon class="w-4 h-4 text-foreground/80 group-hover:text-foreground transition-colors" />
</span>