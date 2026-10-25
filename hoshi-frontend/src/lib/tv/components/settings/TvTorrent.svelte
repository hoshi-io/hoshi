<script lang="ts">
    import { onDestroy } from 'svelte';
    import { ChevronRight, FolderOpen, RefreshCw, Trash2 } from 'lucide-svelte';
    import { platform } from '@tauri-apps/plugin-os';
    import { openPath } from '@tauri-apps/plugin-opener';

    import type { TorrentConfig } from '@/api/config/types.ts';
    import type { TorrentCacheEntry, TorrentStorageStats } from '@/api/torrent/types.ts';
    import { torrentApi } from '@/api/torrent/torrent.ts';
    import { i18n } from '@/stores/i18n.svelte.js';
    import TvSettingRow from '@/tv/components/TvSettingRow.svelte';
    import TvSwitch from '@/tv/components/TvSwitch.svelte';
    import TvSegmented from '@/tv/components/TvSegmented.svelte';
    import TvSlider from '@/tv/components/TvSlider.svelte';

    let {
        torrentConfig = $bindable(),
        onSave
    }: {
        torrentConfig: TorrentConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    const os = platform();
    const canOpenFolder = os !== 'android' && os !== 'ios';

    const GIB = 1024 ** 3;

    const t = (key: string, params?: Record<string, unknown>) =>
        i18n.t(`settings.torrent_section.${key}`, params as Record<string, string | number>);

    function formatBytes(bytes: number): string {
        if (!Number.isFinite(bytes) || bytes <= 0) return '0 B';
        const units = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)));
        const value = bytes / 1024 ** i;
        return `${value >= 100 || i === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[i]}`;
    }

    const bytesToGib = (b: number) => Math.round((b / GIB) * 100) / 100;

    function parseList(raw: string): string[] {
        const seen = new Set<string>();
        for (const part of raw.split(',')) {
            const v = part.trim();
            if (v) seen.add(v);
        }
        return [...seen];
    }

    // Numeric settings are picked from presets instead of typed. The saved value is always
    // included so a value set elsewhere (desktop) is shown and stays selected.
    function withCurrent(presets: number[], current: number, label: (n: number) => string) {
        const values = presets.includes(current) ? presets : [...presets, current].sort((a, b) => a - b);
        return values.map((v) => ({ value: String(v), label: label(v) }));
    }

    // -------------------------------------------------------------- options
    const resolutionOptions = [
        { value: '2160p', label: '2160p (4K)' },
        { value: '1080p', label: '1080p' },
        { value: '720p', label: '720p' },
        { value: '480p', label: '480p' }
    ];

    const codecOptions = $derived([
        { value: 'any', label: t('codec_any') },
        { value: 'x265', label: 'HEVC (x265)' },
        { value: 'x264', label: 'AVC (x264)' },
        { value: 'av1', label: 'AV1' }
    ]);

    const batchOptions = $derived([
        { value: 'any', label: t('batch_any') },
        { value: 'batch', label: t('batch_required') },
        { value: 'single', label: t('batch_excluded') }
    ]);

    const rateLabel = (kbps: number) =>
        kbps === 0 ? t('unlimited') : kbps >= 1024 && kbps % 1024 === 0 ? `${kbps / 1024} MB/s` : `${kbps} KB/s`;
    const RATE_PRESETS = [0, 128, 256, 512, 1024, 2048, 5120, 10240, 25600];

    const downloadOptions = $derived(withCurrent(RATE_PRESETS, torrentConfig.downloadRateLimitKbps ?? 0, rateLabel));
    const uploadOptions = $derived(withCurrent(RATE_PRESETS, torrentConfig.uploadRateLimitKbps ?? 0, rateLabel));

    const gbLabel = (g: number) => `${g} GB`;
    const maxDiskOptions = $derived(withCurrent([5, 10, 20, 50, 100, 200], bytesToGib(torrentConfig.maxDiskUsageBytes), gbLabel));
    const minFreeOptions = $derived(withCurrent([0, 1, 2, 5, 10, 20], bytesToGib(torrentConfig.minFreeDiskSpaceBytes), gbLabel));

    const TTL_PRESETS = [0, 3600, 6 * 3600, 86400, 3 * 86400, 7 * 86400, 30 * 86400];

    function ttlLabel(seconds: number): string {
        if (seconds === 0) return t('ttl_none');
        if (seconds % 86400 === 0) return t('ttl_days', { num: seconds / 86400 });
        if (seconds % 3600 === 0) return t('ttl_hours', { num: seconds / 3600 });
        return t('ttl_minutes', { num: Math.max(1, Math.round(seconds / 60)) });
    }

    const ttlOptions = $derived(withCurrent(TTL_PRESETS, torrentConfig.finishedFileTtlSeconds, ttlLabel));

    // -------------------------------------------------------------- handlers
    function handleListChange(key: 'preferredGroups' | 'excludeKeywords', el: HTMLInputElement) {
        torrentConfig[key] = parseList(el.value);
        el.value = torrentConfig[key].join(', ');
        onSave();
    }

    function handleRate(key: 'downloadRateLimitKbps' | 'uploadRateLimitKbps', val: string) {
        const n = parseInt(val);
        torrentConfig[key] = Number.isFinite(n) && n > 0 ? n : null; // 0 = unlimited
        onSave();
    }

    // -------------------------------------------------------------- tabs
    let activeTab = $state('torrent_selection');
    const tabs = $derived([
        { value: 'torrent_selection', label: t('tab_selection') },
        { value: 'torrent_network', label: t('tab_network') },
        { value: 'torrent_storage', label: t('tab_storage') }
    ]);

    // Dependent settings stay visible but inert while auto-select is off.
    const selectionOff = $derived(!torrentConfig.autoSelect);

    const inputClass =
        'h-12 w-72 scroll-my-24 rounded-md border-2 border-border/60 bg-card px-4 text-lg outline-none ' +
        'focus:border-primary focus:ring-4 focus:ring-primary/40 disabled:opacity-50';
    const actionBtn =
        'inline-flex h-12 scroll-my-24 items-center gap-2 rounded-lg border-2 px-4 text-sm font-bold outline-none ' +
        'focus:ring-4 focus:ring-primary focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50';

    // -------------------------------------------------------------- storage
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
            console.error('Failed to load torrent storage', e);
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
            console.error('Failed to delete cache entry', e);
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
            console.error('Failed to clear torrent cache', e);
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
            console.error('Failed to open torrent cache folder', e);
        }
    }

    function toggleExpanded(key: string) {
        expanded[key] = !expanded[key];
    }

    const stateLabel = (state: TorrentCacheEntry['state']) => t(`state_${state}`);

    function stateClass(state: TorrentCacheEntry['state']): string {
        switch (state) {
            case 'active': return 'bg-primary/15 text-primary';
            case 'seeding': return 'bg-emerald-500/15 text-emerald-500';
            default: return 'bg-muted text-muted-foreground';
        }
    }

    const formatDate = (ms: number) => (ms > 0 ? new Date(ms).toLocaleString() : '—');

    $effect(() => {
        if (activeTab === 'torrent_storage') refresh();
    });

    onDestroy(() => {
        if (confirmTimer) clearTimeout(confirmTimer);
    });
</script>

<div>
    <header class="space-y-1 pb-4">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.torrent')}</h2>
        <p class="text-base text-muted-foreground">{t('torrent_desc')}</p>
    </header>

    <div class="pb-2">
        <TvSegmented label={i18n.t('settings.torrent')} bind:value={activeTab} options={tabs} />
    </div>

    <!-- ============================================================ AUTO SELECTION -->
    {#if activeTab === 'torrent_selection'}
        <div>
            <TvSettingRow label={t('auto_select')} description={t('auto_select_desc')}>
                <TvSwitch label={t('auto_select')} bind:checked={torrentConfig.autoSelect} onchange={() => onSave()} />
            </TvSettingRow>

            <TvSettingRow disabled={selectionOff} label={t('preferred_groups')} description={t('preferred_groups_desc')}>
                <input
                        data-sn
                        type="text"
                        value={torrentConfig.preferredGroups.join(', ')}
                        onchange={(e) => handleListChange('preferredGroups', e.currentTarget)}
                        placeholder="Group1, Group2"
                        disabled={selectionOff}
                        class={inputClass}
                />
            </TvSettingRow>

            <TvSettingRow stacked disabled={selectionOff} label={t('preferred_resolution')} description={t('preferred_resolution_desc')}>
                <TvSegmented
                        label={t('preferred_resolution')}
                        disabled={selectionOff}
                        value={torrentConfig.preferredResolution}
                        options={resolutionOptions}
                        onchange={(v) => { torrentConfig.preferredResolution = v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow stacked disabled={selectionOff} label={t('preferred_codec')} description={t('preferred_codec_desc')}>
                <TvSegmented
                        label={t('preferred_codec')}
                        disabled={selectionOff}
                        value={torrentConfig.preferredCodec ?? 'any'}
                        options={codecOptions}
                        onchange={(v) => { torrentConfig.preferredCodec = v === 'any' ? null : v; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow disabled={selectionOff} label={t('prefer_dual_audio')} description={t('prefer_dual_audio_desc')}>
                <TvSwitch label={t('prefer_dual_audio')} disabled={selectionOff} bind:checked={torrentConfig.preferDualAudio} onchange={() => onSave()} />
            </TvSettingRow>

            <TvSettingRow stacked disabled={selectionOff} label={t('require_batch')} description={t('require_batch_desc')}>
                <TvSegmented
                        label={t('require_batch')}
                        disabled={selectionOff}
                        value={torrentConfig.requireBatch == null ? 'any' : torrentConfig.requireBatch ? 'batch' : 'single'}
                        options={batchOptions}
                        onchange={(v) => { torrentConfig.requireBatch = v === 'any' ? null : v === 'batch'; onSave(); }}
                />
            </TvSettingRow>

            <TvSettingRow disabled={selectionOff} label={t('min_seeders')} description={t('min_seeders_desc')}>
                <TvSlider
                        label={t('min_seeders')}
                        disabled={selectionOff}
                        bind:value={torrentConfig.minSeeders}
                        min={0}
                        max={50}
                        step={1}
                        oncommit={() => onSave()}
                />
            </TvSettingRow>

            <TvSettingRow disabled={selectionOff} label={t('exclude_keywords')} description={t('exclude_keywords_desc')}>
                <input
                        data-sn
                        type="text"
                        value={torrentConfig.excludeKeywords.join(', ')}
                        onchange={(e) => handleListChange('excludeKeywords', e.currentTarget)}
                        placeholder="CAM, HDTS, hardsub"
                        disabled={selectionOff}
                        class={inputClass}
                />
            </TvSettingRow>

            <TvSettingRow disabled={selectionOff} label={t('fallback_to_manual')} description={t('fallback_to_manual_desc')}>
                <TvSwitch label={t('fallback_to_manual')} disabled={selectionOff} bind:checked={torrentConfig.fallbackToManual} onchange={() => onSave()} />
            </TvSettingRow>
        </div>

        <!-- ============================================================ NETWORK -->
    {:else if activeTab === 'torrent_network'}
        <div>
            <TvSettingRow stacked label={t('download_limit')} description={t('download_limit_desc')}>
                <TvSegmented
                        label={t('download_limit')}
                        value={String(torrentConfig.downloadRateLimitKbps ?? 0)}
                        options={downloadOptions}
                        onchange={(v) => handleRate('downloadRateLimitKbps', v)}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={t('upload_limit')} description={t('upload_limit_desc')}>
                <TvSegmented
                        label={t('upload_limit')}
                        value={String(torrentConfig.uploadRateLimitKbps ?? 0)}
                        options={uploadOptions}
                        onchange={(v) => handleRate('uploadRateLimitKbps', v)}
                />
            </TvSettingRow>

            <TvSettingRow label={t('max_concurrent')} description={t('max_concurrent_desc')}>
                <TvSlider
                        label={t('max_concurrent')}
                        bind:value={torrentConfig.maxConcurrentTorrents}
                        min={1}
                        max={10}
                        step={1}
                        oncommit={() => onSave()}
                />
            </TvSettingRow>

            <TvSettingRow label={t('stop_seeding')} description={t('stop_seeding_desc')}>
                <TvSwitch label={t('stop_seeding')} bind:checked={torrentConfig.stopSeedingOnPlaybackEnd} onchange={() => onSave()} />
            </TvSettingRow>
        </div>

        <!-- ============================================================ STORAGE -->
    {:else if activeTab === 'torrent_storage'}
        <div>
            <TvSettingRow stacked label={t('max_disk_usage')} description={t('max_disk_usage_desc')}>
                <TvSegmented
                        label={t('max_disk_usage')}
                        value={String(bytesToGib(torrentConfig.maxDiskUsageBytes))}
                        options={maxDiskOptions}
                        onchange={(v) => {
                            // Below 1 GiB the cache reaper would evict almost everything.
                            torrentConfig.maxDiskUsageBytes = Math.round(Math.max(1, parseFloat(v)) * GIB);
                            onSave();
                        }}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={t('min_free_disk')} description={t('min_free_disk_desc')}>
                <TvSegmented
                        label={t('min_free_disk')}
                        value={String(bytesToGib(torrentConfig.minFreeDiskSpaceBytes))}
                        options={minFreeOptions}
                        onchange={(v) => {
                            torrentConfig.minFreeDiskSpaceBytes = Math.round(Math.max(0, parseFloat(v)) * GIB);
                            onSave();
                        }}
                />
            </TvSettingRow>

            <TvSettingRow stacked label={t('finished_ttl')} description={t('finished_ttl_desc')}>
                <TvSegmented
                        label={t('finished_ttl')}
                        value={String(torrentConfig.finishedFileTtlSeconds)}
                        options={ttlOptions}
                        onchange={(v) => { torrentConfig.finishedFileTtlSeconds = parseInt(v); onSave(); }}
                />
            </TvSettingRow>

            <!-- Disk usage overview -->
            <div class="mt-6 space-y-4 rounded-lg border border-border/50 bg-muted/20 p-5">
                <div class="space-y-0.5">
                    <p class="text-lg font-bold">{t('storage_used')}</p>
                    <p class="text-sm tabular-nums text-muted-foreground">
                        {#if stats}
                            {t('storage_of', { used: formatBytes(stats.usedBytes), limit: formatBytes(torrentConfig.maxDiskUsageBytes) })}
                            · {t('disk_free', { size: formatBytes(stats.freeBytes) })}
                        {:else if loadFailed}
                            {t('storage_load_failed')}
                        {:else}
                            …
                        {/if}
                    </p>
                </div>

                <div class="h-2.5 w-full overflow-hidden rounded-full bg-muted">
                    <div
                            class="h-full rounded-full transition-all duration-500 {usedPct >= 90 ? 'bg-destructive' : 'bg-primary'}"
                            style="width: {usedPct}%"
                    ></div>
                </div>

                <div class="flex flex-wrap items-center gap-3 p-1">
                    <button type="button" data-sn onclick={refresh} disabled={loading} class="{actionBtn} border-border bg-background">
                        <RefreshCw class="size-4 {loading ? 'animate-spin' : ''}" />
                        {t('refresh')}
                    </button>

                    {#if canOpenFolder}
                        <button type="button" data-sn onclick={handleOpenFolder} class="{actionBtn} border-border bg-background">
                            <FolderOpen class="size-4" />
                            {t('open_folder')}
                        </button>
                    {/if}

                    <button
                            type="button"
                            data-sn
                            onclick={handleClearClick}
                            disabled={clearing || entries.length === 0}
                            class="{actionBtn} {confirmClear
                                ? 'border-destructive bg-destructive text-destructive-foreground'
                                : 'border-border bg-background text-destructive'}"
                    >
                        <Trash2 class="size-4" />
                        {confirmClear ? t('clear_cache_confirm') : t('clear_cache')}
                    </button>
                </div>

                <p class="text-sm text-muted-foreground">{t('clear_cache_desc')}</p>

                {#if actionFailed}
                    <p class="text-sm text-destructive">{t('storage_action_failed')}</p>
                {/if}
            </div>

            <!-- Cached torrents -->
            <div class="pt-8">
                <p class="text-lg font-bold">{t('cache_entries', { num: entries.length })}</p>

                {#if sortedEntries.length === 0}
                    <p class="py-6 text-base text-muted-foreground">{t('cache_empty')}</p>
                {:else}
                    <div class="mt-3 divide-y divide-border/40 overflow-hidden rounded-lg border border-border/50">
                        {#each sortedEntries as entry (entry.key)}
                            <div class="px-3 py-2">
                                <div class="flex items-center gap-3">
                                    <button
                                            type="button"
                                            data-sn
                                            onclick={() => toggleExpanded(entry.key)}
                                            aria-expanded={!!expanded[entry.key]}
                                            class="flex min-w-0 flex-1 scroll-my-24 items-center gap-3 rounded-md px-2 py-2 text-left outline-none focus:bg-secondary focus:ring-4 focus:ring-primary"
                                    >
                                        <ChevronRight class="size-5 shrink-0 text-muted-foreground transition-transform duration-200 {expanded[entry.key] ? 'rotate-90' : ''}" />
                                        <div class="min-w-0">
                                            <p class="truncate text-base font-semibold">{entry.name}</p>
                                            <p class="text-sm text-muted-foreground">
                                                {formatBytes(entry.sizeBytes)}
                                                · {t('files_count', { num: entry.files.length })}
                                                · {t('last_used', { date: formatDate(entry.lastUsedMs) })}
                                            </p>
                                        </div>
                                    </button>

                                    <span class="shrink-0 rounded-full px-3 py-1 text-sm font-semibold {stateClass(entry.state)}">
                                        {stateLabel(entry.state)}
                                    </span>

                                    <button
                                            type="button"
                                            data-sn
                                            title={t('delete_entry')}
                                            aria-label={t('delete_entry')}
                                            disabled={entry.state === 'active' || busyKey === entry.key || clearing}
                                            onclick={() => deleteEntry(entry.key)}
                                            class="inline-flex size-12 shrink-0 scroll-my-24 items-center justify-center rounded-lg text-destructive outline-none
                                                   focus:bg-destructive/10 focus:ring-4 focus:ring-destructive/50 disabled:opacity-40"
                                    >
                                        <Trash2 class="size-5" />
                                    </button>
                                </div>

                                {#if expanded[entry.key]}
                                    <ul class="mb-2 ml-10 mt-2 space-y-1.5">
                                        {#each entry.files as file (file.path)}
                                            <li class="flex items-center justify-between gap-4 text-sm">
                                                <span class="truncate text-muted-foreground">{file.path}</span>
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
        </div>
    {/if}
</div>