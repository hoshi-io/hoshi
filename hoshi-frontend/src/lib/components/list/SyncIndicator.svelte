<script lang="ts">
    // Mobile only: a top-bar button that opens the sync panel in a drawer.
    // Desktop uses the inline SyncCard in the sidebar instead.
    import { listStore } from "@/app/list.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import * as Drawer from "$lib/components/ui/drawer";
    import { buttonVariants } from "$lib/components/ui/button";
    import { cn } from "$lib/utils";
    import { RefreshCw, AlertTriangle, CloudOff, CheckCircle2 } from "lucide-svelte";
    import SyncPanel from "@/components/list/SyncPanel.svelte";

    let { onReconnect }: { onReconnect?: (tracker: string) => void } = $props();

    let open = $state(false);

    const phase = $derived(listStore.syncPhase);
    const failed = $derived(listStore.failedSyncCount);

    // Nothing left to show: make sure the drawer isn't left open on an empty panel.
    $effect(() => {
        if (!listStore.syncVisible) open = false;
    });

    const label = $derived.by(() => {
        switch (phase) {
            case "failed":  return i18n.t("list.sync.indicator.failed");
            case "paused":  return i18n.t("list.sync.indicator.paused");
            case "syncing": return i18n.t("list.sync.indicator.syncing");
            default:        return i18n.t("list.sync.indicator.done");
        }
    });

    const triggerClass = cn(buttonVariants({ variant: "ghost", size: "icon" }), "relative h-10 w-10 rounded-full hover:bg-muted/50");
</script>

{#if listStore.syncVisible}
    <Drawer.Root bind:open>
        <Drawer.Trigger class={triggerClass} aria-label={label}>
            {#if phase === "paused"}
                <CloudOff class="size-[22px] text-amber-500" />
            {:else if phase === "syncing"}
                <RefreshCw class="size-[22px] animate-spin text-primary" />
            {:else if phase === "failed"}
                <AlertTriangle class="size-[22px] text-destructive" />
            {:else}
                <CheckCircle2 class="size-[22px] text-emerald-500" />
            {/if}

            {#if failed > 0}
                <span class="absolute right-1 top-1 flex min-w-4 items-center justify-center rounded-full bg-destructive px-1 text-[9px] font-bold leading-4 text-destructive-foreground">
                    {failed > 99 ? "99+" : failed}
                </span>
            {/if}
        </Drawer.Trigger>
        <Drawer.Content class="max-h-[85vh] rounded-t-3xl border-border/50">
            <div class="pb-8">
                <SyncPanel {onReconnect} />
            </div>
        </Drawer.Content>
    </Drawer.Root>
{/if}