<script lang="ts">
    import { i18n } from "@/stores/i18n.svelte.js";
    import { invoke } from "@tauri-apps/api/core";
    import { toast } from "svelte-sonner";
    import { Button } from "$lib/components/ui/button";
    import {
        RefreshCw, Trash2, Search, Terminal,
        FileText, Download, ArrowLeft, Copy
    } from "lucide-svelte";
    import { onMount } from "svelte";
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

<section class="flex flex-col w-full h-full bg-transparent font-mono overflow-hidden">
    <div class="flex flex-col lg:flex-row lg:items-center justify-between pb-6 gap-4 w-full border-b border-border/20 mb-4 shrink-0">
        <div class="flex items-center gap-3">
            <div class="p-2.5 bg-primary/5 rounded-sm border border-primary/10 text-primary">
                <Terminal class="size-5" />
            </div>
            <div>
                <h2 class="text-xl md:text-2xl font-black tracking-tight">{i18n.t('settings.logs.title')}</h2>
                <p class="text-[10px] text-muted-foreground uppercase tracking-widest font-bold opacity-60">
                    {i18n.t('settings.logs.description')}
                </p>
                <div class="flex gap-4 mt-2">
                    <button class="text-[10px] font-black uppercase tracking-widest transition-colors {activeTab === 'live' ? 'text-primary' : 'text-muted-foreground/40 hover:text-muted-foreground'}" onclick={() => activeTab = 'live'}>
                        {i18n.t('settings.logs.tabs.live')}
                    </button>
                    <button class="text-[10px] font-black uppercase tracking-widest transition-colors {activeTab === 'files' ? 'text-primary' : 'text-muted-foreground/40 hover:text-muted-foreground'}" onclick={() => { activeTab = 'files'; fetchLogFiles(); }}>
                        {i18n.t('settings.logs.tabs.files')}
                    </button>
                </div>
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-2 lg:gap-3">
            {#if activeTab === 'live'}
                <div class="relative flex items-center w-full sm:w-auto flex-1 sm:flex-none">
                    <Search class="absolute left-3 size-4 text-muted-foreground/60" />
                    <input type="text" bind:value={searchQuery} placeholder="" class="h-9 w-full sm:w-48 lg:w-64 rounded-sm border-none bg-muted/30 pl-10 pr-3 py-1 text-sm ring-1 ring-border/50 focus:ring-2 focus:ring-primary/50 transition-all outline-none" />
                </div>
                <div class="w-full sm:w-32">
                    <ResponsiveSelect bind:value={selectedLevel} items={logLevels} class="h-10 rounded-sm bg-muted/30 px-4 py-1 text-sm ring-1 ring-border/50" />
                </div>
                <Button variant="ghost" size="icon" onclick={fetchLogs} title={i18n.t('settings.logs.btn_refresh')}>
                    <RefreshCw class="size-4 {isLoading ? 'animate-spin' : ''}" />
                </Button>
                <Button variant="ghost" size="icon" onclick={copyToClipboard} title={i18n.t('settings.logs.btn_copy')}>
                    <Copy class="size-4" />
                </Button>
                <Button variant="ghost" size="icon" onclick={() => logs = []} title={i18n.t('settings.logs.btn_clear_ui')}>
                    <Trash2 class="size-4 text-destructive/70" />
                </Button>
            {:else if selectedFileName}
                <Button variant="outline" size="sm" onclick={() => { selectedFileName = null; selectedFileContent = null; }}>
                    <ArrowLeft class="size-4 mr-2" /> {i18n.t('settings.logs.back')}
                </Button>
                <Button variant="ghost" size="icon" onclick={() => downloadFile(selectedFileName!, selectedFileContent!)}>
                    <Download class="size-4" />
                </Button>
            {/if}
        </div>
    </div>

    <div class="flex-1 min-h-0 bg-transparent -mx-2 px-2 overflow-hidden flex flex-col">
        {#if activeTab === 'live'}
            <div class="flex-1 overflow-y-auto custom-scrollbar">
                {#if isLoading && logs.length === 0}
                    <div class="flex flex-col items-center justify-center h-full gap-4 text-muted-foreground">
                        <RefreshCw class="size-8 animate-spin text-primary/30" />
                        <p class="text-sm animate-pulse">{i18n.t('settings.logs.loading')}</p>
                    </div>
                {:else if filteredLogs.length === 0}
                    <div class="flex flex-col items-center justify-center h-full opacity-20 gap-4">
                        <Terminal class="size-12" />
                        <p class="text-sm">{i18n.t('settings.logs.empty')}</p>
                    </div>
                {:else}
                    <div class="hidden sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-4 px-2 py-2 text-[10px] font-black uppercase text-muted-foreground/40 border-b border-border/10 sticky top-0 bg-background/95 backdrop-blur-sm z-10">
                        <div>{i18n.t('settings.logs.header.time')}</div>
                        <div>{i18n.t('settings.logs.header.level')}</div>
                        <div>{i18n.t('settings.logs.header.target')}</div>
                        <div>{i18n.t('settings.logs.header.message')}</div>
                    </div>
                    {#each filteredLogs as log, i (log.timestamp.toString() + '_' + i)}
                        <div class="flex flex-col sm:grid sm:grid-cols-[100px_70px_1fr_3fr] gap-1 sm:gap-4 items-start hover:bg-muted/30 px-3 py-2 rounded-lg transition-colors text-[11px]">
                            <div class="text-muted-foreground/60">{formatTimestamp(log.timestamp)}</div>
                            <div class="font-bold opacity-80">{log.level}</div>
                            <div class="text-primary/70 truncate w-full" title={log.target}>{log.target}</div>
                            <div class="whitespace-pre-wrap break-words text-foreground/90">{log.message}</div>
                        </div>
                    {/each}
                {/if}
            </div>
        {:else if activeTab === 'files' && !selectedFileName}
            <div class="flex-1 overflow-y-auto custom-scrollbar space-y-2">
                {#each logFiles as file}
                    <div class="flex items-center justify-between p-3 bg-muted/20 rounded-sm border border-border/50 hover:bg-muted/30 transition-all">
                        <div class="flex items-center gap-3">
                            <FileText class="size-5 text-muted-foreground/50" />
                            <div>
                                <p class="text-sm font-bold">{file.name}</p>
                                <p class="text-[10px] text-muted-foreground uppercase">{formatSize(file.size_bytes)} • {new Date(file.created_at).toLocaleString()}</p>
                            </div>
                        </div>
                        <div class="flex items-center gap-2">
                            <Button variant="ghost" size="sm" onclick={() => viewLogFile(file.name)}>
                                {i18n.t('settings.logs.view')}
                            </Button>
                            <Button variant="ghost" size="icon" class="text-destructive/50 hover:text-destructive" onclick={() => deleteFile(file.name)}>
                                <Trash2 class="size-4" />
                            </Button>
                        </div>
                    </div>
                {/each}
            </div>
        {:else if selectedFileName}
            <div class="flex-1 overflow-auto custom-scrollbar bg-muted/5 rounded-lg border border-border/10 p-4">
                <pre class="whitespace-pre-wrap text-[11px] leading-relaxed opacity-80">{selectedFileContent}</pre>
            </div>
        {/if}
    </div>
</section>

<style>
    .custom-scrollbar::-webkit-scrollbar { width: 4px; }
    .custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
    .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(150, 150, 150, 0.1); border-radius: 10px; }
    pre { tab-size: 4; font-family: inherit; }
</style>