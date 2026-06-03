<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import { Loader2, MonitorPlay, ChevronDown } from 'lucide-svelte';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { extensionsApi } from '@/api/extensions/extensions';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { appConfig } from '@/stores/config.svelte.js';
    import { i18n } from "@/stores/i18n.svelte";
    import ResponsiveSelect from '@/components/ResponsiveSelect.svelte';
    import { Label } from '@/components/ui/label';
    import { Button } from '@/components/ui/button';
    import { Switch } from '@/components/ui/switch';

    let isExpanded = $state(false);
    let islandRef = $state<HTMLElement | null>(null);

    let selectedExtId = $state(layoutState.mpv?.extId ?? extensions.anime[0]?.id ?? "");
    let selectedServer = $state(layoutState.mpv?.server ?? "");
    let isDub = $state(layoutState.mpv?.isDub ?? false);

    let extSettings = $state<any>(null);
    let isLoadingSettings = $state(false);
    let isLaunching = $state(false);

    // Grace period state configuration
    let isGraceActive = $state(true);
    let graceTimeout: ReturnType<typeof setTimeout>;

    const extItems = $derived(extensions.anime.map(e => ({ value: e.id, label: e.name })));
    const serverItems = $derived(extSettings?.episodeServers?.map((s: string) => ({ value: s, label: s })) ?? []);

    // Helper to spin up a safe startup buffer
    function triggerGracePeriod(durationMs = 5000) {
        isGraceActive = true;
        clearTimeout(graceTimeout);
        graceTimeout = setTimeout(() => {
            isGraceActive = false;
        }, durationMs);
    }

    // 1. Initial Mount Grace Period Setup
    $effect(() => {
        triggerGracePeriod(5000); // 5 seconds of initial buffer space on application launch
        return () => clearTimeout(graceTimeout);
    });

    // 2. Continuous Polling Engine with Status Guarding
    $effect(() => {
        const checkMpvStatus = async () => {
            // Guard: Absolutely skip checks if launching or inside a startup grace period window
            if (isLaunching || isGraceActive) return;

            try {
                const running = await invoke<boolean>('is_mpv_running');
                if (!running) {
                    layoutState.mpv = null;
                }
            } catch (err) {
                layoutState.mpv = null;
            }
        };

        checkMpvStatus();
        const interval = setInterval(checkMpvStatus, 2000);

        return () => clearInterval(interval);
    });

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

    $effect(() => {
        if (!isExpanded) return;

        const handleClickOutside = (event: MouseEvent) => {
            const target = event.target as HTMLElement;
            if (islandRef && !islandRef.contains(target)) {
                if (target.closest('[data-radix-popper-content-wrapper]') || target.closest('[role="listbox"]')) {
                    return;
                }
                isExpanded = false;
            }
        };

        document.addEventListener('click', handleClickOutside);
        return () => document.removeEventListener('click', handleClickOutside);
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
                    use_hoshi_config: appConfig.data?.mpv?.useHoshiConfig ?? false,
                }
            );
            layoutState.mpv = {
                ...layoutState.mpv,
                extId: selectedExtId,
                server: selectedServer || undefined,
                isDub,
            };
            isExpanded = false;

            // Give MPV 6 extra seconds to register windows before checking again
            triggerGracePeriod(6000);
        } catch (err) {
            console.error("Failed to relaunch MPV:", err);
        } finally {
            isLaunching = false;
        }
    }
</script>

<div class="absolute top-1.5 left-1/2 -translate-x-1/2 z-50 flex flex-col items-center select-none pointer-events-auto">
    <div
            bind:this={islandRef}
            class="relative overflow-hidden bg-black text-white shadow-xl border border-white/10 transform-gpu
        transition-[width,height,border-radius,background-color] duration-[400ms] ease-[cubic-bezier(0.32,0.72,0,1)]
        {isExpanded ? 'w-[320px] h-[340px] rounded-[32px] bg-black/95 backdrop-blur-xl' : 'w-[280px] h-8 rounded-[16px] hover:scale-[1.02] cursor-pointer active:scale-95'}"
    >
        <button
                onclick={() => isExpanded = true}
                class="absolute inset-0 w-full h-full flex items-center justify-between px-3 bg-transparent border-none outline-none cursor-pointer
            transition-opacity duration-200 {isExpanded ? 'opacity-0 pointer-events-none' : 'opacity-100 delay-150'}"
        >
            <div class="flex items-center gap-2 overflow-hidden w-full pr-2">
                <span class="flex items-center gap-[2px] h-2.5 shrink-0 px-0.5">
                    <span class="w-[2px] h-3 bg-primary animate-pulse rounded-[16px]"></span>
                    <span class="w-[2px] h-1.5 bg-primary/70 rounded-[16px]"></span>
                    <span class="w-[2px] h-2 bg-primary rounded-[16px]"></span>
                </span>

                <span class="text-[10px] font-medium tracking-wide truncate text-white/90 text-left w-full">
                    {layoutState.mpv?.animeTitle} · {layoutState.mpv?.epTitle ? layoutState.mpv.epTitle : `E${layoutState.mpv?.epNumber}`}
                </span>
            </div>
            <ChevronDown class="size-3 text-white/40 shrink-0" />
        </button>

        <div
                class="absolute inset-0 flex flex-col gap-4 w-full h-full text-left p-5
            transition-opacity duration-300 {isExpanded ? 'opacity-100 delay-[150ms]' : 'opacity-0 pointer-events-none'}"
        >
            <div class="flex items-start gap-3 border-b border-white/10 pb-3">
                {#if layoutState.mpv?.coverImage}
                    <img src={layoutState.mpv.coverImage} alt="" class="size-10 rounded-sm object-cover bg-neutral-900 border border-white/10 shrink-0" />
                {:else}
                    <div class="size-10 rounded-sm bg-neutral-900 flex items-center justify-center border border-white/5 shrink-0">
                        <MonitorPlay class="size-5 text-primary" />
                    </div>
                {/if}
                <div class="flex-1 min-w-0">
                    <p class="text-[11px] uppercase tracking-wider text-white/40 font-bold">Now Playing</p>
                    <h4 class="text-sm font-semibold text-white truncate leading-snug">{layoutState.mpv?.animeTitle}</h4>
                    <p class="text-xs text-white/60 truncate mt-0.5">{layoutState.mpv?.epTitle || `Episode ${layoutState.mpv?.epNumber}`}</p>
                </div>
            </div>

            <div class="flex flex-col gap-3 flex-1">
                <div class="grid gap-1">
                    <Label class="text-[10px] text-white/40 font-semibold uppercase tracking-wider">Extension Source</Label>
                    <ResponsiveSelect bind:value={selectedExtId} items={extItems} class="bg-neutral-900 border-white/10 text-white" />
                </div>

                {#if isLoadingSettings}
                    <div class="flex items-center justify-center gap-2 py-2 text-xs text-white/50">
                        <Loader2 class="w-3.5 h-3.5 animate-spin text-primary" /> Fetching servers...
                    </div>
                {:else if serverItems.length > 0}
                    <div class="grid gap-1">
                        <Label class="text-[10px] text-white/40 font-semibold uppercase tracking-wider">{i18n.t("watch.server")}</Label>
                        <ResponsiveSelect bind:value={selectedServer} items={serverItems} class="bg-neutral-900 border-white/10 text-white" />
                    </div>
                {/if}

                {#if extSettings?.supportsDub}
                    <div class="flex items-center justify-between p-2 rounded-sm bg-white/5 border border-white/5 mt-0.5">
                        <Label class="text-xs text-white/80 cursor-pointer">{i18n.t("watch.dub")}</Label>
                        <Switch bind:checked={isDub} class="scale-75 origin-right" />
                    </div>
                {/if}
            </div>

            <Button
                    size="sm"
                    onclick={relaunch}
                    disabled={isLaunching || !selectedExtId || isLoadingSettings}
                    class="w-full font-medium bg-white text-black hover:bg-white/90 rounded-sm mt-auto shrink-0"
            >
                {#if isLaunching}
                    <Loader2 class="w-3.5 h-3.5 mr-2 animate-spin" /> {i18n.t("watch.launching")}
                {:else}
                    <MonitorPlay class="w-3.5 h-3.5 mr-2" /> {i18n.t("watch.relaunch")}
                {/if}
            </Button>
        </div>
    </div>
</div>