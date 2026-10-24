<script lang="ts">
    // Desktop: an inline status card in the sidebar. Collapsed it's one line (what's happening +
    // progress); click to expand the full panel in place. No popover, no overlay.
    import { slide } from "svelte/transition";
    import { listStore } from "@/app/list.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import SyncPanel from "@/components/list/SyncPanel.svelte";
    import { RefreshCw, AlertTriangle, CloudOff, CheckCircle2, ChevronDown } from "lucide-svelte";

    let { onReconnect }: { onReconnect?: (tracker: string) => void } = $props();


    let expanded = $state(false);

    const phase = $derived(listStore.syncPhase);
    const waiting = $derived(listStore.pendingSyncCount);
    const failed = $derived(listStore.failedSyncCount);
    const progress = $derived(listStore.syncBatchProgress);
    const pct = $derived(progress ? Math.round((progress.done / progress.total) * 100) : 0);

    const title = $derived.by(() => {
        switch (phase) {
            case "failed":  return i18n.t("list.sync.indicator.failed");
            case "paused":  return i18n.t("list.sync.indicator.paused");
            case "syncing": return i18n.t("list.sync.indicator.syncing");
            default:        return i18n.t("list.sync.indicator.done");
        }
    });

    const detail = $derived.by(() => {
        const parts: string[] = [];
        if (progress) parts.push(`${progress.done}/${progress.total}`);
        else if (waiting > 0) parts.push(i18n.t("list.sync.waiting", { n: waiting }));
        if (failed > 0) parts.push(i18n.t("list.sync.failed", { n: failed }));
        if (parts.length === 0) parts.push(i18n.t("list.sync.batch.complete"));
        return parts.join(" · ");
    });
</script>

<div class="overflow-hidden rounded-sm border border-border/40 bg-card shadow-sm" transition:slide|global={{ duration: 180 }}>
    <button
            type="button"
            class="flex w-full items-center gap-2.5 px-3 py-2.5 text-left transition-colors hover:bg-muted/40"
            aria-expanded={expanded}
            onclick={() => (expanded = !expanded)}
    >
        {#if phase === "paused"}
            <CloudOff class="size-4 shrink-0 text-amber-500" />
        {:else if phase === "syncing"}
            <RefreshCw class="size-4 shrink-0 animate-spin text-primary" />
        {:else if phase === "failed"}
            <AlertTriangle class="size-4 shrink-0 text-destructive" />
        {:else}
            <CheckCircle2 class="size-4 shrink-0 text-emerald-500" />
        {/if}

        <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-bold">{title}</p>
            <p class="truncate font-mono text-xs font-medium tabular-nums text-muted-foreground">{detail}</p>
        </div>

        <ChevronDown class="size-4 shrink-0 text-muted-foreground transition-transform {expanded ? 'rotate-180' : ''}" />
    </button>

    {#if phase === "syncing"}
        <div class="h-0.5 w-full bg-muted/50" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={progress ? pct : undefined}>
            <div
                    class="h-full bg-primary transition-all duration-300 {progress ? '' : 'animate-pulse'}"
                    style="width: {progress ? pct : 100}%"
            ></div>
        </div>
    {/if}

    {#if expanded}
        <div class="border-t border-border/40" transition:slide={{ duration: 150 }}>
            <SyncPanel {onReconnect} inline />
        </div>
    {/if}
</div>