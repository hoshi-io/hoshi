<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import { Button } from "$lib/components/ui/button";
    import {
        RefreshCw, Trash2, Search, Terminal,
        FileText, Download, ArrowLeft, Copy, Check
    } from "lucide-svelte";
    import { onMount } from "svelte";
    import { fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    interface LogEntry {
        timestamp: number;
        level: 'ERROR' | 'WARN' | 'INFO' | 'DEBUG' | 'TRACE';
        target: string;
        message: string;
    }

    interface LogFileInfo {
        name: string;
        size_bytes: number;
        created_at: number;
    }

    let activeTab = $state<'live' | 'files'>('live');
    let logs = $state<LogEntry[]>([]);
    let logFiles = $state<LogFileInfo[]>([]);
    let selectedFileContent = $state<string | null>(null);
    let selectedFileName = $state<string | null>(null);

    let isLoading = $state(false);
    let isFileLoading = $state(false);
    let selectedLevel = $state('ALL');
    let searchQuery = $state('');
    let copied = $state(false);

    const levels = ['ALL', 'ERROR', 'WARN', 'INFO', 'DEBUG', 'TRACE'];

    const logLevels = $derived(
        levels.map(level => ({
            value: level,
            label: level === 'ALL' ? i18n.t('settings.logs.level_all') : level
        }))
    );
    const MAX_LOGS_DISPLAY = 500;

    let filteredLogs = $derived.by(() => {
        let result = logs;
        if (selectedLevel !== 'ALL') result = result.filter(l => l.level === selectedLevel);
        if (searchQuery.trim() !== '') {
            const query = searchQuery.toLowerCase();
            result = result.filter(l =>
                l.message.toLowerCase().includes(query) ||
                l.target.toLowerCase().includes(query)
            );
        }
        return [...result].reverse().slice(0, MAX_LOGS_DISPLAY);
    });

    async function fetchLogs() {
        if (isLoading || activeTab !== 'live') return;
        isLoading = true;
        try {
            logs = await invoke<LogEntry[]>("get_system_logs");
        } catch (e: any) {
            toast.error(i18n.t(e.key || 'errors.unknown_error'));
        } finally {
            isLoading = false;
        }
    }

    async function copyToClipboard() {
        const text = logs.map(l => `[${formatTimestamp(l.timestamp)}] ${l.level} ${l.target} - ${l.message}`).join('\n');
        await navigator.clipboard.writeText(text);
        copied = true;
        setTimeout(() => (copied = false), 1500);
    }

    async function fetchLogFiles() {
        isFileLoading = true;
        try {
            logFiles = await invoke<LogFileInfo[]>("list_log_files");
        } catch (e: any) {
            toast.error(i18n.t('errors.fetch_failed'));
        } finally {
            isFileLoading = false;
        }
    }

    async function viewLogFile(name: string) {
        isFileLoading = true;
        try {
            const content = await invoke<string>("get_log_file", { name });
            selectedFileContent = content;
            selectedFileName = name;
        } catch (e: any) {
            toast.error(i18n.t('errors.file_read_failed'));
        } finally {
            isFileLoading = false;
        }
    }

    async function deleteFile(name: string) {
        try {
            await invoke("delete_log_file", { name });
            fetchLogFiles();
        } catch (e: any) {
            toast.error(i18n.t('errors.delete_failed'));
        }
    }

    function downloadFile(name: string, content: string) {
        const blob = new Blob([content], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.style.display = 'none';
        a.href = url;
        a.download = name;
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
    }

    function formatTimestamp(timestamp: number): string {
        const date = new Date(timestamp);
        const timeStr = date.toLocaleTimeString('en-GB', { hour12: false });
        const millis = date.getMilliseconds().toString().padStart(3, '0');
        return `${timeStr}.${millis}`;
    }

    // Colour-coded level badges make a long log scannable at a glance.
    function levelClass(level: LogEntry['level']): string {
        switch (level) {
            case 'ERROR': return 'bg-red-500/10 text-red-500';
            case 'WARN': return 'bg-amber-500/10 text-amber-500';
            case 'INFO': return 'bg-sky-500/10 text-sky-500';
            case 'DEBUG': return 'bg-muted text-muted-foreground';
            default: return 'bg-muted/50 text-muted-foreground/60';
        }
    }

    function formatSize(bytes: number): string {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    onMount(() => {
        fetchLogs();
        const intervalId = setInterval(() => {
            if (activeTab === 'live') fetchLogs();
        }, 3000);
        return () => clearInterval(intervalId);
    });
</script>

<section class="flex flex-col w-full h-full bg-transparent overflow-hidden">
    <header class="mb-2 shrink-0">
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.logs.title')}</h2>
        <p class="text-sm text-muted-foreground mt-1">{i18n.t('settings.logs.description')}</p>
    </header>

    <div class="flex flex-col lg:flex-row lg:items-center justify-between gap-3 py-3 w-full shrink-0">
        <div class="inline-flex w-full lg:w-auto gap-1 p-1 rounded-lg bg-muted/50" role="tablist">
            {#each [
                { id: 'live', label: i18n.t('settings.logs.tabs.live') },
                { id: 'files', label: i18n.t('settings.logs.tabs.files') }
            ] as tab (tab.id)}
                <button
                        type="button"
                        role="tab"
                        aria-selected={activeTab === tab.id}
                        class="flex-1 lg:flex-none h-9 px-5 rounded-md text-sm font-semibold transition-all duration-200
                               focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40
                               {activeTab === tab.id
                                   ? 'bg-background text-foreground shadow-sm ring-1 ring-border/60'
                                   : 'text-muted-foreground hover:text-foreground'}"
                        onclick={() => {
                            activeTab = tab.id as 'live' | 'files';
                            if (tab.id === 'files') fetchLogFiles();
                        }}
                >
                    {tab.label}
                </button>
            {/each}
        </div>

        <div class="flex flex-wrap items-center gap-2">
            {#if activeTab === 'live'}
                <div class="relative flex items-center w-full sm:w-auto flex-1 sm:flex-none">
                    <Search class="absolute left-3 size-4 text-muted-foreground/60 pointer-events-none" />
                    <input
                            type="text"
                            bind:value={searchQuery}
                            placeholder=""
                            class="h-10 w-full sm:w-48 lg:w-64 rounded-md border-none bg-muted/30 pl-10 pr-3 py-1 text-sm ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 transition-all outline-none"
                    />
                </div>
                <div class="w-full sm:w-32">
                    <ResponsiveSelect bind:value={selectedLevel} items={logLevels} class="h-10 rounded-md bg-muted/30 px-4 py-1 text-sm ring-1 ring-border/50" />
                </div>
                <div class="flex items-center gap-1">
                    <Button variant="ghost" size="icon" class="rounded-md active:scale-90 transition-transform" onclick={fetchLogs} title={i18n.t('settings.logs.btn_refresh')}>
                        <RefreshCw class="size-4 {isLoading ? 'animate-spin' : ''}" />
                    </Button>
                    <Button variant="ghost" size="icon" class="rounded-md active:scale-90 transition-transform" onclick={copyToClipboard} title={i18n.t('settings.logs.btn_copy')}>
                        {#if copied}
                            <Check class="size-4 text-emerald-500" />
                        {:else}
                            <Copy class="size-4" />
                        {/if}
                    </Button>
                    <Button variant="ghost" size="icon" class="rounded-md active:scale-90 transition-transform hover:bg-destructive/10" onclick={() => logs = []} title={i18n.t('settings.logs.btn_clear_ui')}>
                        <Trash2 class="size-4 text-destructive/70" />
                    </Button>
                </div>
            {:else if selectedFileName}
                <Button variant="outline" size="sm" class="rounded-md" onclick={() => { selectedFileName = null; selectedFileContent = null; }}>
                    <ArrowLeft class="size-4 mr-2" /> {i18n.t('settings.logs.back')}
                </Button>
                <Button variant="ghost" size="icon" class="rounded-md active:scale-90 transition-transform" onclick={() => downloadFile(selectedFileName!, selectedFileContent!)}>
                    <Download class="size-4" />
                </Button>
            {/if}
        </div>
    </div>

    <div class="flex-1 min-h-0 flex flex-col">
        {#if activeTab === 'live'}
            <div in:fly={{ y: 8, duration: 220, easing: cubicOut }} class="flex-1 min-h-0 flex flex-col rounded-lg border border-border/50 bg-muted/10 overflow-hidden">
                <div class="flex-1 overflow-y-auto custom-scrollbar">
                    {#if isLoading && logs.length === 0}
                        <div class="flex flex-col items-center justify-center h-full min-h-48 gap-4 text-muted-foreground">
                            <RefreshCw class="size-8 animate-spin text-primary/40" />
                            <p class="text-sm animate-pulse">{i18n.t('settings.logs.loading')}</p>
                        </div>
                    {:else if filteredLogs.length === 0}
                        <div class="flex flex-col items-center justify-center h-full min-h-48 text-muted-foreground/50 gap-3">
                            <Terminal class="size-10" />
                            <p class="text-sm">{i18n.t('settings.logs.empty')}</p>
                        </div>
                    {:else}
                        <div class="hidden sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-4 px-4 py-2.5 text-[10px] font-bold uppercase tracking-wider text-muted-foreground/60 border-b border-border/40 sticky top-0 bg-background/90 backdrop-blur-sm z-10">
                            <div>{i18n.t('settings.logs.header.time')}</div>
                            <div>{i18n.t('settings.logs.header.level')}</div>
                            <div>{i18n.t('settings.logs.header.target')}</div>
                            <div>{i18n.t('settings.logs.header.message')}</div>
                        </div>
                        {#each filteredLogs as log, i (log.timestamp.toString() + '_' + i)}
                            <div class="flex flex-col sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-1 sm:gap-4 items-start px-4 py-2 border-b border-border/20 last:border-b-0 hover:bg-muted/40 transition-colors font-mono text-[11px]">
                                <div class="text-muted-foreground/70 tabular-nums">{formatTimestamp(log.timestamp)}</div>
                                <div class="inline-flex w-fit px-1.5 py-0.5 rounded text-[10px] font-bold {levelClass(log.level)}">{log.level}</div>
                                <div class="text-primary/70 truncate w-full" title={log.target}>{log.target}</div>
                                <div class="whitespace-pre-wrap break-words text-foreground/90">{log.message}</div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        {:else if activeTab === 'files' && !selectedFileName}
            <div in:fly={{ y: 8, duration: 220, easing: cubicOut }} class="flex-1 overflow-y-auto custom-scrollbar">
                {#if logFiles.length === 0}
                    <div class="flex flex-col items-center justify-center min-h-48 text-muted-foreground/50 gap-3">
                        <FileText class="size-10" />
                        <p class="text-sm">{i18n.t('settings.logs.empty')}</p>
                    </div>
                {:else}
                    <div class="rounded-lg border border-border/50 divide-y divide-border/40 overflow-hidden">
                        {#each logFiles as file (file.name)}
                            <div class="flex items-center justify-between gap-3 px-4 py-3 bg-muted/10 hover:bg-muted/30 transition-colors">
                                <div class="flex items-center gap-3 min-w-0">
                                    <div class="grid place-items-center size-9 shrink-0 rounded-md bg-muted/50 text-muted-foreground">
                                        <FileText class="size-4" />
                                    </div>
                                    <div class="min-w-0">
                                        <p class="text-sm font-semibold truncate">{file.name}</p>
                                        <p class="text-xs text-muted-foreground">{formatSize(file.size_bytes)} • {new Date(file.created_at).toLocaleString()}</p>
                                    </div>
                                </div>
                                <div class="flex items-center gap-1 shrink-0">
                                    <Button variant="ghost" size="sm" class="rounded-md" onclick={() => viewLogFile(file.name)}>
                                        {i18n.t('settings.logs.view')}
                                    </Button>
                                    <Button variant="ghost" size="icon" class="rounded-md text-destructive/60 hover:text-destructive hover:bg-destructive/10 active:scale-90 transition-all" onclick={() => deleteFile(file.name)}>
                                        <Trash2 class="size-4" />
                                    </Button>
                                </div>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        {:else if selectedFileName}
            <div in:fly={{ y: 8, duration: 220, easing: cubicOut }} class="flex-1 min-h-0 flex flex-col rounded-lg border border-border/50 bg-muted/10 overflow-hidden">
                <p class="px-4 py-2.5 text-xs font-semibold text-muted-foreground border-b border-border/40 truncate">{selectedFileName}</p>
                <div class="flex-1 overflow-auto custom-scrollbar p-4">
                    <pre class="whitespace-pre-wrap text-[11px] leading-relaxed opacity-90 font-mono">{selectedFileContent}</pre>
                </div>
            </div>
        {/if}
    </div>
</section>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 6px; height: 6px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150, 150, 150, 0.2); border-radius: 10px; }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover { background: rgba(150, 150, 150, 0.4); }
    pre { tab-size: 4; font-family: inherit; }
</style>