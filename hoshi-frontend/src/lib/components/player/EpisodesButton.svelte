<script lang="ts">
    import { List } from "lucide-svelte";

    let {
        onclick,
        class: className = "",
    }: {
        onclick?: () => void;
        class?: string;
    } = $props();
</script>

<button
        {onclick}
        aria-label="Episodes"
        class="p-2 rounded-lg hover:bg-white/10 transition text-white shrink-0 {className}"
>
    <List class="w-5 h-5" />
</button>