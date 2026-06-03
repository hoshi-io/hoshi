<script lang="ts">
    import { Button } from "$lib/components/ui/button";
    import {AlertCircle, PuzzleIcon, Loader2, RefreshCw} from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte";
    import type { CoreError } from "@/api/client";

    interface Props {
        error: CoreError | null;
        isLoadingPlay: boolean;
        isLoadingMeta: boolean;
        noExtensions: boolean;
        isMappingError: boolean;
        hlsError: { message: string; retrying: boolean } | null;
        onRetry: () => void;
        onManageExtensions: () => void;
    }

    let {
        error,
        isLoadingPlay,
        isLoadingMeta,
        noExtensions,
        isMappingError,
        hlsError,
        onRetry,
        onManageExtensions,
    }: Props = $props();

    const visible = $derived(!!error || isLoadingPlay || (!isLoadingMeta && noExtensions));
</script>

{#if visible}
    <div
            class="status-overlay absolute inset-0 z-50 flex items-center justify-center backdrop-blur-md pointer-events-none"
            role="status"
    >
        {#if error}
            <div class="flex flex-col items-center gap-6 p-6 max-w-md text-center animate-in fade-in zoom-in-95 duration-300 drop-shadow-lg">
                <AlertCircle class="w-12 h-12 text-destructive" />

                <div class="space-y-2">
                    <h2 class="text-xl font-heading text-white">
                        {i18n.t('watch.error_title') || 'Error'}
                    </h2>
                    <p class="m-0 text-sm font-medium leading-snug text-white/80">
                        {i18n.t(error.key) || error.message || error.key}
                    </p>
                </div>

                <div class="flex flex-wrap justify-center gap-4 mt-2">
                    <Button variant="secondary" onclick={onRetry} class="px-8 font-bold rounded-xl">
                        {i18n.t('watch.retry')}
                    </Button>

                    {#if isMappingError}
                        <Button
                                variant="outline"
                                class="px-8 font-bold text-white border-white/20 hover:bg-white/10 rounded-xl"
                                onclick={onManageExtensions}
                        >
                            <PuzzleIcon class="w-4 h-4 mr-2" />
                            {i18n.t('content.extension_manager.manage_extensions_title')}
                        </Button>
                    {/if}
                </div>
            </div>

        {:else if isLoadingPlay}
            <div class="flex flex-col items-center gap-4 animate-in fade-in duration-300">
                <Loader2 class="w-12 h-12 animate-spin text-primary" />
                <span class="text-xs font-black tracking-[0.2em] uppercase text-white/90 font-heading">
                    {i18n.t('watch.loading_stream')}
                </span>
            </div>
        {:else if hlsError}
            <div class="flex flex-col items-center gap-4 animate-in fade-in duration-300">
                <RefreshCw class="w-10 h-10 text-white/60 animate-spin" />
                <div class="space-y-1 text-center">
                    <p class="text-sm font-semibold text-white/90">
                        {i18n.t('watch.error_playing')}
                    </p>
                    <p class="text-xs text-white/50">
                        {i18n.t('watch.loading_stream')}
                    </p>
                </div>
            </div>
        {:else if !isLoadingMeta && noExtensions}
            <div class="flex flex-col items-center gap-6 p-8 text-center animate-in fade-in zoom-in-95 duration-300">
                <PuzzleIcon class="w-16 h-16 text-white/20" />
                <div class="space-y-2">
                    <span class="block text-xl font-heading text-white/90">
                        {i18n.t('watch.no_extensions')}
                    </span>
                    <Button variant="link" onclick={onManageExtensions} class="text-primary font-bold">
                        {i18n.t('content.extension_manager.manage_extensions_title')}
                    </Button>
                </div>
            </div>
        {/if}
    </div>
{/if}

<style>
    .status-overlay {
        background: radial-gradient(circle, transparent 0%, rgba(0,0,0,0.4) 100%);
    }
</style>