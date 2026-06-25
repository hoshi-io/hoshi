<script lang="ts">
    import { Play, Pause } from 'lucide-svelte';
    import {i18n} from "@/stores/i18n.svelte";

    let { paused, onclick, size = 'md' }: Props = $props();

    interface Props {
        paused: boolean;
        onclick: () => void;
        size?: 'sm' | 'md' | 'lg' | 'xl';
    }

    const btnClass = {
        sm: 'w-8 h-8',
        md: 'w-10 h-10',
        lg: 'w-16 h-16',
        xl: 'w-24 h-24',
    }[size];

    const iconClass = {
        sm: 'w-4 h-4',
        md: 'w-6 h-6',
        lg: 'w-8 h-8',
        xl: 'w-12 h-12',
    }[size];
</script>

<button
        class="flex shrink-0 items-center justify-center {btnClass} rounded-full bg-transparent text-white transition-all duration-200 hover:bg-white/20 hover:scale-105 active:scale-95 cursor-pointer"
        onclick={(e) => {
        e.stopPropagation();
        onclick();
    }}
        aria-label={paused ? i18n.t("player.play") : i18n.t("player.pause")}
>
    {#if paused}
        <Play class="{iconClass} ml-0.5 fill-current" />
    {:else}
        <Pause class="{iconClass} fill-current" />
    {/if}
</button>