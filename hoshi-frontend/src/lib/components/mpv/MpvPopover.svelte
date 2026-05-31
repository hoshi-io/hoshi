<script lang="ts">
    import { fade, fly } from 'svelte/transition';
    import { quintOut } from 'svelte/easing';
    import { Loader2, MonitorPlay } from 'lucide-svelte';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { extensionsApi } from '@/api/extensions/extensions';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import type { ExtensionSettingsResponse } from '@/api/extensions/types';
    import ResponsiveSelect from '@/components/ResponsiveSelect.svelte';
    import { Label } from '@/components/ui/label';
    import { Button } from '@/components/ui/button';
    import { Switch } from '@/components/ui/switch';
    import { i18n } from "@/stores/i18n.svelte";
    import { invoke } from '@tauri-apps/api/core';

    let { open = $bindable() }: { open: boolean } = $props();

    let selectedExtId = $state(layoutState.mpv?.extId ?? extensions.anime[0]?.id ?? "");
    let selectedServer = $state(layoutState.mpv?.server ?? "");
    let isDub = $state(layoutState.mpv?.isDub ?? false);

    let extSettings = $state<ExtensionSettingsResponse | null>(null);
    let isLoadingSettings = $state(false);
    let isLaunching = $state(false);

    const extItems = $derived(extensions.anime.map(e => ({ value: e.id, label: e.name })));
    const serverItems = $derived(extSettings?.episodeServers?.map(s => ({ value: s, label: s })) ?? []);

    // Monitor if MPV process is running with a launch grace period
    $effect(() => {
        if (!open || !layoutState.mpv) return;

        // Tracks how many checks we've skipped to let MPV window initialize safely
        let initializationChecksSkipped = 0;
        const GRACE_PERIOD_CHECKS = 4; // Skip the first 4 seconds of polling

        const checkMpvStatus = async () => {
            try {
                const running = await invoke<boolean>('is_mpv_running');

                if (!running) {
                    if (initializationChecksSkipped < GRACE_PERIOD_CHECKS) {
                        initializationChecksSkipped++;
                        return; // Ignore false results during the initial spin-up stage
                    }
                    autoDismiss();
                } else {
                    // Once it successfully reports running, we can stop skipping grace checks
                    initializationChecksSkipped = GRACE_PERIOD_CHECKS;
                }
            } catch (err) {
                if (initializationChecksSkipped < GRACE_PERIOD_CHECKS) {
                    initializationChecksSkipped++;
                    return;
                }
                autoDismiss();
            }
        };

        // Delay initial execution slightly and poll every 1000ms
        const interval = setInterval(checkMpvStatus, 1000);

        return () => clearInterval(interval);
    });

    function autoDismiss() {
        open = false;
        layoutState.mpv = null;
    }

    $effect(() => {
        const extId = selectedExtId;
        if (!extId) return;

        isLoadingSettings = true;
        extSettings = null;

        extensionsApi.getSettings(extId)
            .then(res => {
                extSettings = res;
                if (extId !== layoutState.mpv?.extId) {
                    selectedServer = res.episodeServers?.[0] ?? "";
                }
                if (!res.supportsDub) isDub = false;
            })
            .catch(console.error)
            .finally(() => { isLoadingSettings = false; });
    });

    async function relaunch() {
        if (!layoutState.mpv || !selectedExtId) return;
        isLaunching = true;
        try {
            await extensions.playWithMpv(
                layoutState.mpv.cid,
                layoutState.mpv.epNumber,
                selectedExtId,
                {
                    server: selectedServer || undefined,
                    isDub,
                    animeTitle: layoutState.mpv.animeTitle,
                    episodeTitle: layoutState.mpv.epTitle,
                    totalEpisodes: layoutState.mpv.totalEpisodes ?? 0,
                    isNsfw: layoutState.mpv.isNsfw ?? false,
                    coverImage: layoutState.mpv.coverImage ?? null,
                    startTime: 0,
                    autoUpdateProgress: appConfig.data?.content?.autoUpdateProgress ?? true,
                }
            );
            layoutState.mpv = {
                ...layoutState.mpv,
                extId: selectedExtId,
                server: selectedServer || undefined,
                isDub,
            };
            open = false;
        } catch (err) {
            console.error("Failed to relaunch MPV:", err);
        } finally {
            isLaunching = false;
        }
    }
</script>

<button
        class="fixed inset-0 z-[70] bg-black/10 backdrop-blur-[1px]"
        onclick={() => open = false}
        aria-label="Close"
        tabindex="-1"
        transition:fade={{ duration: 150 }}
></button>

<div
        class="absolute top-8 right-24 z-[80] w-72 rounded-xl border border-border/50 bg-background/95 backdrop-blur-xl shadow-2xl p-4 flex flex-col gap-4 overflow-hidden"
        transition:fly={{ y: -10, duration: 250, easing: quintOut }}
>
    <div class="absolute -top-12 -right-12 w-32 h-32 bg-primary/10 rounded-full blur-3xl pointer-events-none"></div>

    <div class="flex items-center justify-between relative z-10">
        <p class="text-[11px] font-semibold text-foreground/80 uppercase tracking-wider">Now playing in MPV</p>
    </div>

    {#if layoutState.mpv}
        <div class="relative z-10 -mt-2">
            <p class="text-sm font-medium text-foreground leading-snug line-clamp-1">
                {layoutState.mpv.animeTitle}
            </p>
            <p class="text-xs text-muted-foreground mt-0.5 line-clamp-1">
                {layoutState.mpv.epTitle}
            </p>
        </div>
    {/if}

    <div class="flex flex-col gap-3 relative z-10">
        <div class="grid gap-1.5">
            <ResponsiveSelect bind:value={selectedExtId} items={extItems} />
        </div>

        {#if isLoadingSettings}
            <div class="flex items-center justify-center gap-2 py-2 text-xs text-muted-foreground">
                <Loader2 class="w-3.5 h-3.5 animate-spin text-primary" /> Fetching servers...
            </div>
        {:else if serverItems.length > 0}
            <div class="grid gap-1.5 transition-all">
                <Label class="text-[11px] text-muted-foreground">{i18n.t("watch.server")}</Label>
                <ResponsiveSelect bind:value={selectedServer} items={serverItems} />
            </div>
        {/if}

        {#if extSettings?.supportsDub}
            <div class="flex items-center justify-between p-2 rounded-lg bg-muted/30 border border-border/30 mt-1">
                <Label class="text-[11px] cursor-pointer">{i18n.t("watch.dub")}</Label>
                <Switch bind:checked={isDub} class="scale-75 origin-right" />
            </div>
        {/if}
    </div>

    <Button
            size="sm"
            onclick={relaunch}
            disabled={isLaunching || !selectedExtId || isLoadingSettings}
            class="w-full relative z-10 font-medium"
    >
        {#if isLaunching}
            <Loader2 class="w-3.5 h-3.5 mr-2 animate-spin" /> {i18n.t("watch.launching")}
        {:else}
            <MonitorPlay class="w-3.5 h-3.5 mr-2" /> {i18n.t("watch.relaunch")}
        {/if}
    </Button>
</div>