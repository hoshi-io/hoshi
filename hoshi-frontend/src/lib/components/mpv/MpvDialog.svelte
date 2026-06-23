<script lang="ts">
    import { Loader2, MonitorPlay, CheckCircle } from 'lucide-svelte';
    import { progressApi } from '@/api/progress/progress';
    import { listApi } from '@/api/list/list';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { extensionsApi } from '@/api/extensions/extensions';
    import type { ExtensionSettingsResponse } from '@/api/extensions/types';
    import ResponsiveSelect from '@/components/ResponsiveSelect.svelte';
    import { invoke } from "@tauri-apps/api/core";

    import * as Dialog from '@/components/ui/dialog';
    import { Switch } from '@/components/ui/switch';
    import { Label } from '@/components/ui/label';
    import { Button } from '@/components/ui/button';
    import { i18n } from "@/stores/i18n.svelte";
    import { homeState } from '@/app/home.svelte.js';
    import { listStore } from '@/app/list.svelte.js';

    let { cid, epNumber, animeTitle, epTitle, totalEpisodes = 0, isNsfw = false, coverImage = null, startTime = 0, open = $bindable() }: {
        cid: string;
        epNumber: number;
        animeTitle: string;
        epTitle: string;
        totalEpisodes?: number;
        isNsfw?: boolean;
        coverImage?: string | null;
        startTime?: number;
        open: boolean;
    } = $props();

    let currentEpNumber = $state(epNumber);
    let currentStartTime = $state(startTime);

    let selectedExtId = $state(extensions.anime[0]?.id ?? "");
    let selectedServer = $state("");
    let isDub = $state(false);

    let extSettings = $state<ExtensionSettingsResponse | null>(null);
    let isLoadingSettings = $state(false);
    let isLaunching = $state(false);
    let launched = $state(false);
    let error = $state<string | null>(null);

    const extItems = $derived(extensions.anime.map(e => ({ value: e.id, label: e.name })));
    const serverItems = $derived(extSettings?.episodeServers?.map(s => ({ value: s, label: s })) ?? []);

    $effect(() => {
        if (open) {
            currentEpNumber = epNumber;
            currentStartTime = startTime;
            launched = false;
            error = null;
        }
    });

    let currentEpTitle = $derived(
        currentEpNumber === epNumber
            ? epTitle
            : i18n.t("watch.episode_number", { num: currentEpNumber })
    );

    $effect(() => {
        const extId = selectedExtId;
        if (!extId) return;

        isLoadingSettings = true;
        extSettings = null;

        extensionsApi.getSettings(extId)
            .then(res => {
                extSettings = res;
                selectedServer = res.episodeServers?.[0] ?? "";
                if (!res.supportsDub) isDub = false;
            })
            .catch(console.error)
            .finally(() => { isLoadingSettings = false; });
    });

    async function launch() {
        if (!selectedExtId) return;
        isLaunching = true;
        error = null;

        try {
            const playRes = await extensions.resolveStream(cid, currentEpNumber, selectedExtId, {
                server: selectedServer || undefined,
                category: isDub ? 'dub' : 'sub',
            });

            const url = playRes.source.url;
            const subs: string[] = (playRes.source.subtitles ?? []).map((s: any) => s.url);

            await invoke('launch_intent', {
                url,
                title: `${animeTitle} - ${currentEpTitle}`,
                subs,
                position: currentStartTime > 0 ? Math.floor(currentStartTime * 1000) : undefined,
            });

            launched = true;
        } catch (err) {
            console.error("Failed to launch:", err);
            error = "Failed to launch. Try a different server or extension.";
        } finally {
            isLaunching = false;
        }
    }

    async function markAsWatched() {
        const status = totalEpisodes > 0 && currentEpNumber >= totalEpisodes ? 'COMPLETED' : 'CURRENT';
        error = null;

        try {
            await Promise.all([
                progressApi.updateAnimeProgress({
                    cid,
                    episode: currentEpNumber,
                    timestampSeconds: currentStartTime,
                    completed: true,
                }),
                listApi.upsert({ cid, status, progress: currentEpNumber }),
            ]);

            listStore.updateEntryProgressLocal(cid, currentEpNumber, status);
            await homeState.refreshContinueWatching();

            if (totalEpisodes === 0 || currentEpNumber < totalEpisodes) {
                currentEpNumber += 1;
                currentStartTime = 0;
                launched = false;
            } else {
                open = false;
            }
        } catch (err) {
            console.error("Binge tracking update failed", err);
            error = "Failed to update progress tracker on the cloud.";
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[425px]">
        <Dialog.Header>
            <Dialog.Title>{i18n.t("watch.episode_number", {"num": currentEpNumber })}</Dialog.Title>
            <Dialog.Description>{animeTitle}</Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-6 py-4">
            <div class="grid gap-3">
                <Label>{i18n.t("watch.server")}</Label>
                <ResponsiveSelect bind:value={selectedExtId} items={extItems} />
            </div>

            {#if isLoadingSettings}
                <div class="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 class="w-4 h-4 animate-spin" /> Fetching settings...
                </div>
            {:else if serverItems.length > 0}
                <div class="grid gap-3">
                    <Label>{i18n.t("watch.server")}</Label>
                    <ResponsiveSelect bind:value={selectedServer} items={serverItems} />
                </div>
            {/if}

            {#if extSettings?.supportsDub}
                <div class="flex items-center justify-between rounded-lg border p-4 shadow-sm border-white/10">
                    <div class="space-y-0.5">
                        <Label>{i18n.t("watch.dub")}</Label>
                    </div>
                    <Switch bind:checked={isDub} />
                </div>
            {/if}

            {#if error}
                <p class="text-sm text-destructive">{error}</p>
            {/if}
        </div>

        <Dialog.Footer class="flex-col sm:flex-col gap-2">
            {#if launched}
                <div class="flex gap-2 w-full">
                    <Button variant="outline" class="flex-1" onclick={() => open = false}>
                        {i18n.t("content.close")}
                    </Button>
                    <!-- Customize button name context to indicate a forward skip loop action -->
                    <Button class="flex-1 font-bold" onclick={markAsWatched}>
                        <CheckCircle class="w-4 h-4 mr-2" />
                        {totalEpisodes === 0 || currentEpNumber < totalEpisodes ? "Watched & Next →" : i18n.t("watch.mark_watched")}
                    </Button>
                </div>
            {:else}
                <div class="flex gap-2 w-full">
                    <Button variant="outline" class="flex-1" onclick={() => open = false} disabled={isLaunching}>
                        Cancel
                    </Button>
                    <Button class="flex-1" onclick={launch} disabled={isLaunching || !selectedExtId || isLoadingSettings}>
                        {#if isLaunching}
                            <Loader2 class="w-4 h-4 mr-2 animate-spin" /> {i18n.t("watch.launching")}
                        {:else}
                            <MonitorPlay class="w-4 h-4 mr-2" /> {i18n.t("watch.open")}
                        {/if}
                    </Button>
                </div>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>