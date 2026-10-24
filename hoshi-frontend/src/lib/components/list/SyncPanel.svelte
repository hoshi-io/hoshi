<script lang="ts">
    import { listStore, trackerLabel, type BatchMeta } from "@/app/list.svelte.js";
    import { listApi } from "@/api/list/list";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Button } from "$lib/components/ui/button";
    import {
        Loader2, WifiOff, KeyRound, Timer, RotateCw, X, ChevronDown, CheckCircle2, AlertTriangle
    } from "lucide-svelte";
    import type {BatchIssue, BatchStatus} from "@/api/list/types.js";

    let { onReconnect, inline = false }: {
        onReconnect?: (tracker: string) => void;
        /** Rendered inside the sidebar card: no title, tighter padding, shorter scroll area. */
        inline?: boolean;
    } = $props();


    const trackerName = trackerLabel;

    const trackers = $derived(
        (listStore.syncStatus?.trackers ?? []).filter(t => t.pending + t.running + t.failed > 0)
    );

    // Newest first; unfinished batches on top.
    const batches = $derived(
        Object.values(listStore.batches).sort((a, b) =>
            Number(a.finished) - Number(b.finished) || b.batchId.localeCompare(a.batchId)
        )
    );

    const isEmpty = $derived(trackers.length === 0 && batches.length === 0);

    let expanded = $state<Record<string, boolean>>({});
    let issues = $state<Record<string, BatchIssue[]>>({});
    let loadingIssues = $state<Record<string, boolean>>({});
    let busy = $state(false);

    async function toggleIssues(batchId: string) {
        expanded[batchId] = !expanded[batchId];
        if (!expanded[batchId]) return;

        loadingIssues[batchId] = true;
        try {
            issues[batchId] = await listApi.getBatchIssues(batchId);
        } catch (err) {
            console.error("Failed to load batch issues:", err);
            issues[batchId] = [];
        } finally {
            loadingIssues[batchId] = false;
        }
    }

    // Keep an open issue list fresh when the batch's counts change (e.g. after a retry).
    $effect(() => {
        for (const b of batches) {
            if (!expanded[b.batchId]) continue;
            void b.failed; void b.skipped; void b.finished;
            listApi.getBatchIssues(b.batchId)
                .then(list => { if (b.batchId in listStore.batches) issues[b.batchId] = list; })
                .catch(() => {});
        }
    });

    async function guarded(fn: () => Promise<unknown>) {
        if (busy) return;
        busy = true;
        try { await fn(); }
        catch (err) { console.error("Sync action failed:", err); }
        finally { busy = false; }
    }

    function titleOf(cid: string) {
        return listStore.entries.find(e => e.cid === cid)?.title ?? cid;
    }

    function reasonLabel(issue: BatchIssue) {
        switch (issue.reason) {
            case "no_mapping":     return i18n.t("list.sync.reason.no_mapping");
            case "entry_missing":  return i18n.t("list.sync.reason.entry_missing");
            case "not_connected":  return i18n.t("list.sync.reason.not_connected");
            case "sync_disabled":  return i18n.t("list.sync.reason.sync_disabled");
            case "unknown_tracker":return i18n.t("list.sync.reason.unknown_tracker");
            case "no_remote_id":   return i18n.t("list.sync.reason.no_remote_id");
            default:               return issue.reason ?? i18n.t("list.sync.reason.unknown");
        }
    }

    function batchTitle(batchId: string, meta: BatchMeta | undefined) {
        switch (meta?.kind) {
            case "push":   return i18n.t("list.sync.batch.push", { tracker: trackerName(meta.tracker ?? "") });
            case "delete": return i18n.t("list.sync.batch.delete");
            case "edit":   return i18n.t("list.sync.batch.edit");
            default:       return i18n.t("list.sync.batch.generic");
        }
    }

    const processed = (b: BatchStatus) => b.done + b.skipped + b.failed;
    const pct = (b: BatchStatus) => (b.total > 0 ? Math.round((processed(b) / b.total) * 100) : 100);
</script>

<div class="flex w-full flex-col overflow-y-auto hide-scrollbar {inline ? 'max-h-[50vh] gap-4 p-3' : 'max-h-[70vh] gap-5 p-4'}">
    {#if !inline || listStore.failedSyncCount > 0}
        <div class="flex items-center justify-between gap-2">
            {#if !inline}
                <h3 class="text-lg font-black tracking-tight">{i18n.t("list.sync.title")}</h3>
            {/if}
            {#if listStore.failedSyncCount > 0}
                <Button
                        variant="ghost"
                        size="sm"
                        class="h-8 gap-1.5 text-xs font-bold {inline ? 'ml-auto' : ''}"
                        disabled={busy}
                        onclick={() => guarded(() => listStore.retryFailed())}
                >
                    <RotateCw class="size-3.5" />
                    {i18n.t("list.sync.retry_all")}
                </Button>
            {/if}
        </div>
    {/if}

    {#if isEmpty}
        <div class="flex items-center gap-2 py-6 text-sm font-medium text-muted-foreground">
            <CheckCircle2 class="size-4 text-emerald-500" />
            {i18n.t("list.sync.all_synced")}
        </div>
    {/if}

    {#if trackers.length > 0}
        <section class="space-y-2">
            <h4 class="px-0.5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                {i18n.t("list.sync.trackers")}
            </h4>

            {#each trackers as t (t.tracker)}
                {@const waiting = t.pending + t.running}
                <div class="space-y-2 rounded-sm border border-border/40 bg-card p-3">
                    <div class="flex items-center gap-2.5">
                        {#if t.pausedReason === "offline"}
                            <WifiOff class="size-4 shrink-0 text-amber-500" />
                        {:else if t.pausedReason === "auth_expired"}
                            <KeyRound class="size-4 shrink-0 text-amber-500" />
                        {:else if t.pausedReason === "rate_limited"}
                            <Timer class="size-4 shrink-0 text-amber-500" />
                        {:else if waiting > 0}
                            <Loader2 class="size-4 shrink-0 animate-spin text-primary" />
                        {:else}
                            <AlertTriangle class="size-4 shrink-0 text-destructive" />
                        {/if}

                        <div class="min-w-0 flex-1">
                            <p class="text-sm font-bold">{trackerName(t.tracker)}</p>
                            <p class="truncate text-xs font-medium text-muted-foreground">
                                {#if t.pausedReason === "offline"}
                                    {i18n.t("list.sync.paused.offline")}
                                {:else if t.pausedReason === "auth_expired"}
                                    {i18n.t("list.sync.paused.auth")}
                                {:else if t.pausedReason === "rate_limited"}
                                    {i18n.t("list.sync.paused.rate")}
                                {:else if waiting > 0}
                                    {i18n.t("list.sync.syncing")}
                                {:else}
                                    {i18n.t("list.sync.needs_attention")}
                                {/if}
                            </p>
                        </div>

                        <div class="shrink-0 text-right text-xs font-mono tabular-nums text-muted-foreground">
                            {#if waiting > 0}<div>{i18n.t("list.sync.waiting", { n: waiting })}</div>{/if}
                            {#if t.failed > 0}<div class="text-destructive">{i18n.t("list.sync.failed", { n: t.failed })}</div>{/if}
                        </div>
                    </div>

                    {#if t.pausedReason}
                        <div class="flex gap-2">
                            {#if t.pausedReason === "auth_expired" && onReconnect}
                                <Button size="sm" class="h-8 flex-1 text-xs font-bold" onclick={() => onReconnect?.(t.tracker)}>
                                    {i18n.t("list.sync.reconnect")}
                                </Button>
                            {/if}
                            <Button
                                    size="sm"
                                    variant="outline"
                                    class="h-8 flex-1 text-xs font-bold"
                                    disabled={busy}
                                    onclick={() => guarded(() => listStore.resumeTracker(t.tracker))}
                            >
                                {i18n.t("list.sync.retry_now")}
                            </Button>
                        </div>
                    {/if}
                </div>
            {/each}
        </section>
    {/if}

    {#if batches.length > 0}
        <section class="space-y-2">
            <h4 class="px-0.5 text-[10px] font-bold uppercase tracking-widest text-muted-foreground">
                {i18n.t("list.sync.batches")}
            </h4>

            {#each batches as b (b.batchId)}
                {@const attention = b.failed + b.skipped}
                <div class="space-y-2.5 rounded-sm border border-border/40 bg-card p-3">
                    <div class="flex items-center gap-2">
                        <p class="min-w-0 flex-1 truncate text-sm font-bold">{batchTitle(b.batchId, listStore.batchMeta[b.batchId])}</p>
                        <span class="shrink-0 font-mono text-xs tabular-nums text-muted-foreground">
                            {processed(b)}/{b.total}
                        </span>
                        {#if b.finished}
                            <Button
                                    variant="ghost"
                                    size="icon"
                                    class="size-7 shrink-0"
                                    aria-label={i18n.t("list.sync.dismiss")}
                                    onclick={() => listStore.dismissBatch(b.batchId)}
                            >
                                <X class="size-4" />
                            </Button>
                        {/if}
                    </div>

                    <div class="h-1.5 w-full overflow-hidden rounded-full bg-muted/50" role="progressbar" aria-valuenow={pct(b)} aria-valuemin="0" aria-valuemax="100">
                        <div
                                class="h-full rounded-full transition-all duration-300 {b.failed > 0 ? 'bg-destructive' : b.finished ? 'bg-emerald-500' : 'bg-primary'}"
                                style="width: {pct(b)}%"
                        ></div>
                    </div>

                    <p class="text-xs font-medium text-muted-foreground">
                        {#if !b.finished}
                            {i18n.t("list.sync.batch.running", { done: b.done, n: b.pending + b.running })}
                        {:else if attention === 0}
                            {i18n.t("list.sync.batch.complete")}
                        {:else}
                            {i18n.t("list.sync.batch.finished_issues", { done: b.done, n: attention })}
                        {/if}
                    </p>

                    <div class="flex flex-wrap gap-2">
                        {#if !b.finished && b.pending > 0}
                            <Button
                                    size="sm"
                                    variant="outline"
                                    class="h-8 text-xs font-bold"
                                    disabled={busy}
                                    onclick={() => guarded(() => listStore.cancelBatch(b.batchId))}
                            >
                                {i18n.t("list.sync.cancel")}
                            </Button>
                        {/if}
                        {#if b.failed > 0}
                            <Button
                                    size="sm"
                                    variant="outline"
                                    class="h-8 gap-1.5 text-xs font-bold"
                                    disabled={busy}
                                    onclick={() => guarded(() => listStore.retryFailed(b.batchId))}
                            >
                                <RotateCw class="size-3.5" />
                                {i18n.t("list.sync.retry_failed")}
                            </Button>
                        {/if}
                        {#if attention > 0}
                            <Button
                                    size="sm"
                                    variant="ghost"
                                    class="h-8 gap-1 text-xs font-bold"
                                    onclick={() => toggleIssues(b.batchId)}
                            >
                                {expanded[b.batchId] ? i18n.t("list.sync.hide_details") : i18n.t("list.sync.show_details")}
                                <ChevronDown class="size-3.5 transition-transform {expanded[b.batchId] ? 'rotate-180' : ''}" />
                            </Button>
                        {/if}
                    </div>

                    {#if expanded[b.batchId]}
                        <ul class="space-y-1.5 border-t border-border/40 pt-2.5">
                            {#if loadingIssues[b.batchId] && !issues[b.batchId]}
                                <li class="text-xs text-muted-foreground">{i18n.t("list.sync.loading")}</li>
                            {:else}
                                {#each issues[b.batchId] ?? [] as issue (issue.cid + issue.tracker)}
                                    <li class="flex items-start gap-2 text-xs">
                                        <span class="mt-1 size-1.5 shrink-0 rounded-full {issue.status === 'FAILED' ? 'bg-destructive' : 'bg-amber-500'}"></span>
                                        <div class="min-w-0">
                                            <p class="truncate font-bold">{titleOf(issue.cid)}</p>
                                            <p class="font-medium text-muted-foreground">
                                                {trackerName(issue.tracker)} · {reasonLabel(issue)}
                                            </p>
                                        </div>
                                    </li>
                                {/each}
                            {/if}
                        </ul>
                    {/if}
                </div>
            {/each}
        </section>
    {/if}
</div>