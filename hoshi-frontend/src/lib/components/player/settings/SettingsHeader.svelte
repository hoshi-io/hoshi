<script lang="ts">
    import { ChevronLeft } from "lucide-svelte";

    let { title, onclick }: { title: string; onclick: () => void } = $props();
</script>

<button
        {onclick}
        class="flex items-center gap-2.5 w-full min-h-11 px-3 py-2 mb-1 rounded-sm border-b border-border text-foreground hover:bg-accent focus-visible:bg-accent outline-none transition-colors text-left"
>
    <ChevronLeft class="w-4 h-4 text-muted-foreground" />
    <span class="text-sm font-semibold truncate">{title}</span>
</button>