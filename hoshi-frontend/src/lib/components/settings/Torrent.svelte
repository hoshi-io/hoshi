<script lang="ts">
    import { onDestroy } from "svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import { Switch } from "$lib/components/ui/switch";
    import { Label } from "$lib/components/ui/label";
    import { Input } from "$lib/components/ui/input";
    import type { TorrentConfig } from "@/api/config/types";
    import type { TorrentCacheEntry, TorrentStorageStats } from "@/api/torrent/types";
    import { torrentApi } from "@/api/torrent/torrent";
    import { i18n } from "@/stores/i18n.svelte.js";
    import {
        Sparkles, ArrowDownUp, HardDrive, RefreshCw, FolderOpen,
        Trash2, ChevronDown, ChevronRight
    } from "lucide-svelte";
    import { platform } from "@tauri-apps/plugin-os";
    import { openPath } from "@tauri-apps/plugin-opener";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        torrentConfig = $bindable(),
        onSave
    }: {
        torrentConfig: TorrentConfig,
        onSave: () => Promise<void> | void
    } = $props();

    const os = platform();
    const canOpenFolder = os !== "android" && os !== "ios";

    const GIB = 1024 ** 3;

    const t = (key: string, params?: Record<string, unknown>) =>
        i18n.t(`settings.torrent_section.${key}`, params as Record<string, string | number>);

    function formatBytes(bytes: number): string {
        if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";
        const units = ["B", "KB", "MB", "GB", "TB"];
        const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
        const value = bytes / 1024 ** i;
        return `${value >= 100 || i === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[i]}`;
    }

    const bytesToGib = (b: number) => Math.round((b / GIB) * 100) / 100;

    function parseList(raw: string): string[] {
        const seen = new Set<string>();
        for (const part of raw.split(",")) {
            const v = part.trim();
            if (v) seen.add(v);
        }
        return [...seen];
    }

    // -------------------------------------------------------------- select options

    const resolutionOptions = [
        { value: "2160p", label: "2160p (4K)" },
        { value: "1080p", label: "1080p" },
        { value: "720p", label: "720p" },
        { value: "480p", label: "480p" }
    ];

    const codecOptions = [
        { value: "any", label: t("codec_any") },
        { value: "x265", label: "HEVC (x265)" },
        { value: "x264", label: "AVC (x264)" },
        { value: "av1", label: "AV1" }
    ];

    const batchOptions = [
        { value: "any", label: t("batch_any") },
        { value: "batch", label: t("batch_required") },
        { value: "single", label: t("batch_excluded") }
    ];

    const TTL_PRESETS = [0, 3600, 6 * 3600, 86400, 3 * 86400, 7 * 86400, 30 * 86400];

    function ttlLabel(seconds: number): string {
        if (seconds === 0) return t("ttl_none");
        if (seconds % 86400 === 0) return t("ttl_days", { num: seconds / 86400 });
        if (seconds % 3600 === 0) return t("ttl_hours", { num: seconds / 3600 });
        return t("ttl_minutes", { num: Math.max(1, Math.round(seconds / 60)) });
    }

    // A value edited outside the UI still shows up instead of rendering blank.
    const ttlOptions = $derived.by(() => {
        const current = torrentConfig.finishedFileTtlSeconds;
        const values = TTL_PRESETS.includes(current) ? TTL_PRESETS : [...TTL_PRESETS, current].sort((a, b) => a - b);
        return values.map((v) => ({ value: v.toString(), label: ttlLabel(v) }));
    });

    // ------------------------------------------------------------------ handlers

    function handleListChange(key: "preferredGroups" | "excludeKeywords", el: HTMLInputElement) {
        torrentConfig[key] = parseList(el.value);
        el.value = torrentConfig[key].join(", ");
        onSave();
    }

    function handleResolutionChange(val: string) {
        torrentConfig.preferredResolution = val;
        onSave();
    }

    function handleCodecChange(val: string) {
        torrentConfig.preferredCodec = val === "any" ? null : val;
        onSave();
    }

    function handleBatchChange(val: string) {
        torrentConfig.requireBatch = val === "any" ? null : val === "batch";
        onSave();
    }

    function handleMinSeedersChange(el: HTMLInputElement) {
        const n = parseInt(el.value);
        torrentConfig.minSeeders = Number.isFinite(n) && n > 0 ? n : 0;
        el.value = torrentConfig.minSeeders.toString();
        onSave();
    }

    function handleMaxConcurrentChange(el: HTMLInputElement) {
        const n = parseInt(el.value);
        torrentConfig.maxConcurrentTorrents = Number.isFinite(n) && n > 0 ? n : 1;
        el.value = torrentConfig.maxConcurrentTorrents.toString();
        onSave();
    }

    function handleRateChange(key: "downloadRateLimitKbps" | "uploadRateLimitKbps", el: HTMLInputElement) {
        const n = parseInt(el.value);
        // Empty / 0 / garbage means "unlimited".
        torrentConfig[key] = Number.isFinite(n) && n > 0 ? n : null;
        el.value = torrentConfig[key]?.toString() ?? "";
        onSave();
    }

    function handleMaxDiskChange(el: HTMLInputElement) {
        const gib = parseFloat(el.value);
        // Below 1 GiB the cache reaper would evict almost everything, so clamp.
        const fallback = bytesToGib(torrentConfig.maxDiskUsageBytes);
        const clamped = Number.isFinite(gib) ? Math.max(1, gib) : fallback;
        torrentConfig.maxDiskUsageBytes = Math.round(clamped * GIB);
        el.value = bytesToGib(torrentConfig.maxDiskUsageBytes).toString();
        onSave();
    }

    function handleMinFreeDiskChange(el: HTMLInputElement) {
        const gib = parseFloat(el.value);
        const clamped = Number.isFinite(gib) ? Math.max(0, gib) : bytesToGib(torrentConfig.minFreeDiskSpaceBytes);
        torrentConfig.minFreeDiskSpaceBytes = Math.round(clamped * GIB);
        el.value = bytesToGib(torrentConfig.minFreeDiskSpaceBytes).toString();
        onSave();
    }

    function handleTtlChange(val: string) {
        torrentConfig.finishedFileTtlSeconds = parseInt(val);
        onSave();
    }

    // ------------------------------------------------------------------ storage

    let activeTab = $state("torrent_selection");
    let stats = $state<TorrentStorageStats | null>(null);
    let entries = $state<TorrentCacheEntry[]>([]);
    let loading = $state(false);
    let loadFailed = $state(false);
    let actionFailed = $state(false);
    let busyKey = $state<string | null>(null);
    let clearing = $state(false);
    let confirmClear = $state(false);
    let expanded = $state<Record<string, boolean>>({});
    let confirmTimer: ReturnType<typeof setTimeout> | null = null;

    const sortedEntries = $derived([...entries].sort((a, b) => b.lastUsedMs - a.lastUsedMs));
    const usedPct = $derived(
        stats && torrentConfig.maxDiskUsageBytes > 0
            ? Math.min(100, (stats.usedBytes / torrentConfig.maxDiskUsageBytes) * 100)
            : 0
    );

    async function refresh() {
        loading = true;
        try {
            const [s, list] = await Promise.all([torrentApi.getStorageStats(), torrentApi.listCache()]);
            stats = s;
            entries = list;
            loadFailed = false;
        } catch (e) {
            console.error("Failed to load torrent storage", e);
            loadFailed = true;
        } finally {
            loading = false;
        }
    }

    async function deleteEntry(key: string) {
        busyKey = key;
        actionFailed = false;
        try {
            await torrentApi.deleteCacheEntry(key);
        } catch (e) {
            console.error("Failed to delete cache entry", e);
            actionFailed = true;
        } finally {
            busyKey = null;
            await refresh();
        }
    }

    async function handleClearClick() {
        if (!confirmClear) {
            confirmClear = true;
            if (confirmTimer) clearTimeout(confirmTimer);
            confirmTimer = setTimeout(() => (confirmClear = false), 3000);
            return;
        }
        if (confirmTimer) clearTimeout(confirmTimer);
        confirmClear = false;
        clearing = true;
        actionFailed = false;
        try {
            await torrentApi.clearCache();
        } catch (e) {
            console.error("Failed to clear torrent cache", e);
            actionFailed = true;
        } finally {
            clearing = false;
            await refresh();
        }
    }

    async function handleOpenFolder() {
        try {
            await openPath(await torrentApi.getCacheDir());
        } catch (e) {
            console.error("Failed to open torrent cache folder", e);
        }
    }

    function toggleExpanded(key: string) {
        expanded[key] = !expanded[key];
    }

    function stateLabel(state: TorrentCacheEntry["state"]): string {
        return t(`state_${state}`);
    }

    function stateClass(state: TorrentCacheEntry["state"]): string {
        switch (state) {
            case "active": return "bg-primary/15 text-primary";
            case "seeding": return "bg-emerald-500/15 text-emerald-500";
            default: return "bg-muted text-muted-foreground";
        }
    }

    function formatDate(ms: number): string {
        return ms > 0 ? new Date(ms).toLocaleString() : "—";
    }

    $effect(() => {
        if (activeTab === "torrent_storage") refresh();
    });

    onDestroy(() => {
        if (confirmTimer) clearTimeout(confirmTimer);
    });
</script>

<div class="space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.torrent')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{t('torrent_desc')}</p>
    </div>

    <Tabs.Root bind:value={activeTab} class="w-full">
        <Tabs.List class="grid w-full max-w-[560px] grid-cols-3 rounded-sm h-11 p-1 bg-muted/50">
            <Tabs.Trigger value="torrent_selection" class="rounded-lg font-bold flex items-center gap-2">
                <Sparkles class="size-4" /> {t('tab_selection')}
            </Tabs.Trigger>
            <Tabs.Trigger value="torrent_network" class="rounded-lg font-bold flex items-center gap-2">
                <ArrowDownUp class="size-4" /> {t('tab_network')}
            </Tabs.Trigger>
            <Tabs.Trigger value="torrent_storage" class="rounded-lg font-bold flex items-center gap-2">
                <HardDrive class="size-4" /> {t('tab_storage')}
            </Tabs.Trigger>
        </Tabs.List>

        <!-- AUTO SELECTION -->
        <Tabs.Content value="torrent_selection" class="focus-visible:outline-none mt-0">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="autoSelect">{t('auto_select')}</Label>
                    <p class="text-sm text-muted-foreground">{t('auto_select_desc')}</p>
                </div>
                <Switch id="autoSelect" bind:checked={torrentConfig.autoSelect} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>

            {#if torrentConfig.autoSelect}
                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('preferred_groups')}</Label>
                        <p class="text-sm text-muted-foreground">{t('preferred_groups_desc')}</p>
                    </div>
                    <div class="w-full sm:max-w-md">
                        <Input
                                value={torrentConfig.preferredGroups.join(", ")}
                                onchange={(e) => handleListChange("preferredGroups", e.currentTarget)}
                                placeholder="Group1, Group2"
                                class="rounded-sm h-11"
                        />
                    </div>
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('preferred_resolution')}</Label>
                        <p class="text-sm text-muted-foreground">{t('preferred_resolution_desc')}</p>
                    </div>
                    <ResponsiveSelect value={torrentConfig.preferredResolution} items={resolutionOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleResolutionChange} />
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('preferred_codec')}</Label>
                        <p class="text-sm text-muted-foreground">{t('preferred_codec_desc')}</p>
                    </div>
                    <ResponsiveSelect value={torrentConfig.preferredCodec ?? "any"} items={codecOptions} class="rounded-sm h-11 w-full sm:max-w-md" onValueChange={handleCodecChange} />
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="preferDualAudio">{t('prefer_dual_audio')}</Label>
                        <p class="text-sm text-muted-foreground">{t('prefer_dual_audio_desc')}</p>
                    </div>
                    <Switch id="preferDualAudio" bind:checked={torrentConfig.preferDualAudio} onCheckedChange={onSave} class="shrink-0 scale-125" />
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('require_batch')}</Label>
                        <p class="text-sm text-muted-foreground">{t('require_batch_desc')}</p>
                    </div>
                    <ResponsiveSelect
                            value={torrentConfig.requireBatch == null ? "any" : torrentConfig.requireBatch ? "batch" : "single"}
                            items={batchOptions}
                            class="rounded-sm h-11 w-full sm:max-w-md"
                            onValueChange={handleBatchChange}
                    />
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('min_seeders')}</Label>
                        <p class="text-sm text-muted-foreground">{t('min_seeders_desc')}</p>
                    </div>
                    <div class="w-full sm:max-w-md">
                        <Input
                                type="number" min="0" step="1"
                                value={torrentConfig.minSeeders}
                                onchange={(e) => handleMinSeedersChange(e.currentTarget)}
                                class="rounded-sm h-11"
                        />
                    </div>
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4 flex-1">
                        <Label class="text-base font-bold">{t('exclude_keywords')}</Label>
                        <p class="text-sm text-muted-foreground">{t('exclude_keywords_desc')}</p>
                    </div>
                    <div class="w-full sm:max-w-md">
                        <Input
                                value={torrentConfig.excludeKeywords.join(", ")}
                                onchange={(e) => handleListChange("excludeKeywords", e.currentTarget)}
                                placeholder="CAM, HDTS, hardsub"
                                class="rounded-sm h-11"
                        />
                    </div>
                </div>

                <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                    <div class="space-y-1 pr-4">
                        <Label class="text-base font-bold" for="fallbackToManual">{t('fallback_to_manual')}</Label>
                        <p class="text-sm text-muted-foreground">{t('fallback_to_manual_desc')}</p>
                    </div>
                    <Switch id="fallbackToManual" bind:checked={torrentConfig.fallbackToManual} onCheckedChange={onSave} class="shrink-0 scale-125" />
                </div>
            {/if}
        </Tabs.Content>

        <!-- NETWORK & PLAYBACK -->
        <Tabs.Content value="torrent_network" class="focus-visible:outline-none mt-0">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('download_limit')}</Label>
                    <p class="text-sm text-muted-foreground">{t('download_limit_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md flex items-center gap-3">
                    <Input
                            type="number" min="0" step="1"
                            value={torrentConfig.downloadRateLimitKbps ?? ""}
                            onchange={(e) => handleRateChange("downloadRateLimitKbps", e.currentTarget)}
                            placeholder={t('unlimited')}
                            class="rounded-sm h-11"
                    />
                    <span class="text-sm text-muted-foreground shrink-0">KB/s</span>
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('upload_limit')}</Label>
                    <p class="text-sm text-muted-foreground">{t('upload_limit_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md flex items-center gap-3">
                    <Input
                            type="number" min="0" step="1"
                            value={torrentConfig.uploadRateLimitKbps ?? ""}
                            onchange={(e) => handleRateChange("uploadRateLimitKbps", e.currentTarget)}
                            placeholder={t('unlimited')}
                            class="rounded-sm h-11"
                    />
                    <span class="text-sm text-muted-foreground shrink-0">KB/s</span>
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('max_concurrent')}</Label>
                    <p class="text-sm text-muted-foreground">{t('max_concurrent_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md">
                    <Input
                            type="number" min="1" step="1"
                            value={torrentConfig.maxConcurrentTorrents}
                            onchange={(e) => handleMaxConcurrentChange(e.currentTarget)}
                            class="rounded-sm h-11"
                    />
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4">
                    <Label class="text-base font-bold" for="stopSeeding">{t('stop_seeding')}</Label>
                    <p class="text-sm text-muted-foreground">{t('stop_seeding_desc')}</p>
                </div>
                <Switch id="stopSeeding" bind:checked={torrentConfig.stopSeedingOnPlaybackEnd} onCheckedChange={onSave} class="shrink-0 scale-125" />
            </div>
        </Tabs.Content>

        <!-- STORAGE -->
        <Tabs.Content value="torrent_storage" class="focus-visible:outline-none mt-0">
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('max_disk_usage')}</Label>
                    <p class="text-sm text-muted-foreground">{t('max_disk_usage_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md flex items-center gap-3">
                    <Input
                            type="number" min="1" step="0.5"
                            value={bytesToGib(torrentConfig.maxDiskUsageBytes)}
                            onchange={(e) => handleMaxDiskChange(e.currentTarget)}
                            class="rounded-sm h-11"
                    />
                    <span class="text-sm text-muted-foreground shrink-0">GB</span>
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('min_free_disk')}</Label>
                    <p class="text-sm text-muted-foreground">{t('min_free_disk_desc')}</p>
                </div>
                <div class="w-full sm:max-w-md flex items-center gap-3">
                    <Input
                            type="number" min="0" step="0.5"
                            value={bytesToGib(torrentConfig.minFreeDiskSpaceBytes)}
                            onchange={(e) => handleMinFreeDiskChange(e.currentTarget)}
                            class="rounded-sm h-11"
                    />
                    <span class="text-sm text-muted-foreground shrink-0">GB</span>
                </div>
            </div>

            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4 py-6 border-b border-border/40">
                <div class="space-y-1 pr-4 flex-1">
                    <Label class="text-base font-bold">{t('finished_ttl')}</Label>
                    <p class="text-sm text-muted-foreground">{t('finished_ttl_desc')}</p>
                </div>
                <ResponsiveSelect
                        value={torrentConfig.finishedFileTtlSeconds.toString()}
                        items={ttlOptions}
                        class="rounded-sm h-11 w-full sm:max-w-md"
                        onValueChange={handleTtlChange}
                />
            </div>

            <!-- Disk usage overview -->
            <div class="py-6 border-b border-border/40 space-y-4">
                <div class="flex flex-wrap items-center justify-between gap-3">
                    <div class="space-y-1">
                        <Label class="text-base font-bold">{t('storage_used')}</Label>
                        <p class="text-sm text-muted-foreground">
                            {#if stats}
                                {t('storage_of', {
                                    used: formatBytes(stats.usedBytes),
                                    limit: formatBytes(torrentConfig.maxDiskUsageBytes)
                                })}
                                · {t('disk_free', { size: formatBytes(stats.freeBytes) })}
                            {:else if loadFailed}
                                {t('storage_load_failed')}
                            {:else}
                                …
                            {/if}
                        </p>
                    </div>

                    <div class="flex items-center gap-2">
                        <button
                                type="button"
                                class="inline-flex items-center gap-2 rounded-sm border border-border h-10 px-3 text-sm font-medium hover:bg-muted/60 transition-colors disabled:opacity-50"
                                onclick={refresh}
                                disabled={loading}
                        >
                            <RefreshCw class="size-4 {loading ? 'animate-spin' : ''}" /> {t('refresh')}
                        </button>
                        {#if canOpenFolder}
                            <button
                                    type="button"
                                    class="inline-flex items-center gap-2 rounded-sm border border-border h-10 px-3 text-sm font-medium hover:bg-muted/60 transition-colors"
                                    onclick={handleOpenFolder}
                            >
                                <FolderOpen class="size-4" /> {t('open_folder')}
                            </button>
                        {/if}
                        <button
                                type="button"
                                class="inline-flex items-center gap-2 rounded-sm border h-10 px-3 text-sm font-medium transition-colors disabled:opacity-50
                                    {confirmClear
                                        ? 'border-destructive bg-destructive text-destructive-foreground'
                                        : 'border-border text-destructive hover:bg-destructive/10'}"
                                onclick={handleClearClick}
                                disabled={clearing || entries.length === 0}
                        >
                            <Trash2 class="size-4" /> {confirmClear ? t('clear_cache_confirm') : t('clear_cache')}
                        </button>
                    </div>
                </div>

                <div class="h-2 w-full rounded-full bg-muted overflow-hidden">
                    <div
                            class="h-full rounded-full transition-all duration-300 {usedPct >= 90 ? 'bg-destructive' : 'bg-primary'}"
                            style="width: {usedPct}%"
                    ></div>
                </div>

                <p class="text-xs text-muted-foreground">{t('clear_cache_desc')}</p>

                {#if actionFailed}
                    <p class="text-sm text-destructive">{t('storage_action_failed')}</p>
                {/if}
            </div>

            <!-- Cached torrents -->
            <div class="pt-6">
                <Label class="text-base font-bold">{t('cache_entries', { num: entries.length })}</Label>

                {#if sortedEntries.length === 0}
                    <p class="text-sm text-muted-foreground py-6">{t('cache_empty')}</p>
                {:else}
                    <div class="mt-2">
                        {#each sortedEntries as entry (entry.key)}
                            <div class="py-4 border-b border-border/40">
                                <div class="flex items-center justify-between gap-3">
                                    <button
                                            type="button"
                                            class="flex items-center gap-2 min-w-0 flex-1 text-left"
                                            onclick={() => toggleExpanded(entry.key)}
                                            aria-expanded={!!expanded[entry.key]}
                                    >
                                        {#if expanded[entry.key]}
                                            <ChevronDown class="size-4 shrink-0 text-muted-foreground" />
                                        {:else}
                                            <ChevronRight class="size-4 shrink-0 text-muted-foreground" />
                                        {/if}
                                        <div class="min-w-0">
                                            <p class="text-sm font-medium truncate">{entry.name}</p>
                                            <p class="text-xs text-muted-foreground">
                                                {formatBytes(entry.sizeBytes)}
                                                · {t('files_count', { num: entry.files.length })}
                                                · {t('last_used', { date: formatDate(entry.lastUsedMs) })}
                                            </p>
                                        </div>
                                    </button>

                                    <span class="shrink-0 rounded-full px-2.5 py-0.5 text-xs font-semibold {stateClass(entry.state)}">
                                        {stateLabel(entry.state)}
                                    </span>

                                    <button
                                            type="button"
                                            class="shrink-0 inline-flex items-center justify-center size-9 rounded-sm text-destructive hover:bg-destructive/10 transition-colors disabled:opacity-40 disabled:pointer-events-none"
                                            title={t('delete_entry')}
                                            aria-label={t('delete_entry')}
                                            disabled={entry.state === "active" || busyKey === entry.key || clearing}
                                            onclick={() => deleteEntry(entry.key)}
                                    >
                                        <Trash2 class="size-4" />
                                    </button>
                                </div>

                                {#if expanded[entry.key]}
                                    <ul class="mt-3 ml-6 space-y-1.5">
                                        {#each entry.files as file (file.path)}
                                            <li class="flex items-center justify-between gap-4 text-xs">
                                                <span class="truncate text-muted-foreground" title={file.path}>{file.path}</span>
                                                <span class="shrink-0 tabular-nums">{formatBytes(file.size)}</span>
                                            </li>
                                        {/each}
                                    </ul>
                                {/if}
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </Tabs.Content>
    </Tabs.Root>
</div>