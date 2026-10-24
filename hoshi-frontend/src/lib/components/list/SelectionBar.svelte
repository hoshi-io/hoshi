<script lang="ts">
    import { fly } from "svelte/transition";
    import { listStore, trackerLabel } from "@/app/list.svelte.js";
    import { layoutState } from "@/stores/layout.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Button } from "$lib/components/ui/button";
    import { goto } from "$app/navigation";
    import { X, Trash2, CloudUpload, Loader2, Info } from "lucide-svelte";


    let confirmingDelete = $state(false);
    let busy = $state(false);

    const count = $derived(listStore.selectedCids.length);
    const total = $derived(listStore.sorted.length);
    const pushTarget = $derived(listStore.missingOn !== "ALL" ? listStore.missingOn : null);
    const hasTrackers = $derived(listStore.connectedTrackers.length > 0);

    // Nothing left to confirm once the selection is empty.
    $effect(() => {
        if (count === 0) confirmingDelete = false;
    });

    async function run(fn: () => Promise<void>) {
        if (busy) return;
        busy = true;
        try {
            await fn();
        } catch (err) {
            console.error("Batch action failed:", err);
        } finally {
            busy = false;
            confirmingDelete = false;
        }
    }
</script>

<div
        class="pointer-events-none fixed inset-x-0 z-51 flex justify-center px-5"
        style="bottom: calc(var(--mobile-bottom-h, 0px) + env(safe-area-inset-bottom, 0px) + 0.75rem)"
        transition:fly={{ y: 24, duration: 200 }}
>
    <div
            role="toolbar"
            aria-label={i18n.t("list.selection.toolbar")}
            class="pointer-events-auto flex w-full max-w-xl flex-wrap items-center gap-2 rounded-sm border border-border/50 bg-card/95 p-2.5 shadow-lg backdrop-blur"
    >
        {#if confirmingDelete}
            <div class="min-w-0 flex-1 px-1">
                <p class="text-sm font-bold">{i18n.t("list.selection.confirm_delete", { n: count })}</p>
                {#if hasTrackers}
                    <p class="text-xs font-medium text-muted-foreground">
                        {i18n.t("list.selection.confirm_delete_trackers")}
                    </p>
                {/if}
            </div>
            <Button variant="ghost" class="h-9 font-bold" disabled={busy} onclick={() => (confirmingDelete = false)}>
                {i18n.t("list.selection.cancel")}
            </Button>
            <Button variant="destructive" class="h-9 gap-1.5 font-bold" disabled={busy} onclick={() => run(() => listStore.deleteSelected())}>
                {#if busy}<Loader2 class="size-4 animate-spin" />{:else}<Trash2 class="size-4" />{/if}
                {i18n.t("list.selection.delete")}
            </Button>
        {:else}
            <Button
                    variant="ghost"
                    size="icon"
                    class="size-9 shrink-0"
                    aria-label={i18n.t("list.selection.exit")}
                    onclick={() => listStore.exitSelection()}
            >
                <X class="size-[18px]" />
            </Button>

            <span class="text-sm font-bold tabular-nums">
                {i18n.t("list.selection.count", { n: count })}
            </span>

            <Button
                    variant="ghost"
                    size="sm"
                    class="h-8 text-xs font-bold"
                    disabled={busy || total === 0}
                    onclick={() => (listStore.allVisibleSelected ? listStore.clearSelection() : listStore.selectAllVisible())}
            >
                {listStore.allVisibleSelected
                    ? i18n.t("list.selection.clear")
                    : i18n.t("list.selection.select_all", { n: total })}
            </Button>

            <div class="ml-auto flex items-center gap-2">
                {#if count === 1}
                    <Button
                            variant="ghost"
                            size="icon"
                            class="size-9"
                            aria-label={i18n.t("list.selection.details")}
                            title={i18n.t("list.selection.details")}
                            onclick={() => goto(`/c/${listStore.selectedCids[0]}`)}
                    >
                        <Info class="size-4" />
                    </Button>
                {/if}

                <Button
                        variant="outline"
                        class="h-9 gap-1 font-bold"
                        disabled={busy || count === 0}
                        aria-label={i18n.t("list.selection.increment")}
                        title={i18n.t("list.selection.increment")}
                        onclick={() => run(() => listStore.incrementSelected())}
                >
                    +1
                </Button>

                {#if pushTarget}
                    <Button
                            class="h-9 gap-1.5 font-bold"
                            disabled={busy || count === 0}
                            onclick={() => run(() => listStore.pushSelected())}
                    >
                        {#if busy}<Loader2 class="size-4 animate-spin" />{:else}<CloudUpload class="size-4" />{/if}
                        {layoutState.isMobile
                            ? i18n.t("list.selection.push_short")
                            : i18n.t("list.selection.push", { tracker: trackerLabel(pushTarget) })}
                    </Button>
                {/if}

                <Button
                        variant="outline"
                        class="h-9 gap-1.5 border-destructive/30 font-bold text-destructive hover:bg-destructive/10 hover:text-destructive"
                        disabled={busy || count === 0}
                        onclick={() => (confirmingDelete = true)}
                >
                    <Trash2 class="size-4" />
                    {#if !layoutState.isMobile}{i18n.t("list.selection.delete")}{/if}
                </Button>
            </div>
        {/if}
    </div>
</div>