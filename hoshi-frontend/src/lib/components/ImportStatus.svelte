<script lang="ts">
    import { importStatuses } from "@/stores/importStatus.svelte";
    import { AlertTriangle, CheckCircle2, Loader2 } from "lucide-svelte";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fade, fly } from "svelte/transition";

    let entries = $derived(Object.entries(importStatuses));
    let visible = $derived(entries.length > 0);
</script>

{#if visible}
    <div
            transition:fly={{ y: 20, duration: 200 }}
            class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 pointer-events-none"
    >
        {#each entries as [name, imp] (name)}
            <div
                    transition:fade={{ duration: 150 }}
                    class="flex items-center gap-3 rounded-xl border border-border bg-popover/95 backdrop-blur-sm py-3 px-4 shadow-xl text-popover-foreground min-w-[280px] max-w-[350px] pointer-events-auto"
            >
                <div class="flex shrink-0">
                    {#if imp.status === "importing"}
                        <Loader2 class="h-4 w-4 animate-spin text-primary" />
                    {:else if imp.status === "done"}
                        <CheckCircle2 class="h-4 w-4 text-green-500" />
                    {:else if imp.status === "error"}
                        <AlertTriangle class="h-4 w-4 text-destructive" />
                    {/if}
                </div>

                <p class="text-sm leading-tight m-0 flex-1 min-w-0">
                    <span class="font-bold capitalize">{name}</span>
                    <span class="text-muted-foreground ml-1">
                        {#if imp.status === "importing"}
                            {imp.total
                                ? i18n.t('settings.trackers_section.processing', {imported: imp.imported, total: imp.total})
                                : i18n.t('settings.trackers_section.importing', {imported: imp.imported})}
                        {:else if imp.status === "done"}
                            {i18n.t('settings.trackers_section.imported', {imported: imp.imported})}
                        {:else if imp.status === "error"}
                            {i18n.t('settings.trackers_section.error')}
                        {/if}
                    </span>
                </p>
            </div>
        {/each}
    </div>
{/if}