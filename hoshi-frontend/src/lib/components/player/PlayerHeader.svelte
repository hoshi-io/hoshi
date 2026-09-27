<script lang="ts">
    import { ArrowLeft, Settings } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import {i18n} from "@/stores/i18n.svelte.js";

    let {
        cid,
        title,
        episodeTitle,
        isLoadingMeta,
        isMobile = false,
        onSettingsClick
    }: {
        cid: string;
        title: string;
        episodeTitle?: string | null;
        isLoadingMeta: boolean;
        isMobile?: boolean;
        onSettingsClick?: () => void;
    } = $props();
</script>

<div class="relative z-20 {isMobile ? 'p-4' : 'p-6'} flex items-start justify-between gap-4">
    <div class="flex items-start gap-3">
        <button
                onclick={() => goto(`/c/${cid}`)}
                class="p-2 mt-0.5 rounded-{isMobile ? 'full' : 'lg'} bg-black/40 hover:bg-black/60 transition text-white shrink-0"
                aria-label={i18n.t('watch.back_to_details')}
        >
            <ArrowLeft class="w-5 h-5" />
        </button>
        <div class="space-y-0.5 {isMobile ? '' : 'max-w-2xl'}">
            <h1 class="{isMobile ? 'text-base' : 'text-xl'} font-bold tracking-tight text-white drop-shadow-sm leading-tight {isMobile ? 'line-clamp-1' : ''}">
                {title || (isLoadingMeta ? i18n.t('watch.loading_metadata') : '')}
            </h1>
            {#if episodeTitle}
                <p class="{isMobile ? 'text-xs' : 'text-sm'} font-medium text-white/80 {isMobile ? 'line-clamp-1' : ''}">
                    {episodeTitle}
                </p>
            {/if}
        </div>
    </div>

    {#if isMobile}
        <button
                onclick={onSettingsClick}
                class="p-2 mt-0.5 rounded-full hover:bg-white/10 transition text-white shrink-0"
                aria-label={i18n.t('watch.player.settings')}
        >
            <Settings class="w-5 h-5" />
        </button>
    {/if}
</div>