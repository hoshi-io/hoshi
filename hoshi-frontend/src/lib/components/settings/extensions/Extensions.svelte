<script lang="ts">
    import { onMount } from "svelte";
    import { Puzzle, Store } from "lucide-svelte";
    import * as Tabs from "$lib/components/ui/tabs";
    import { extensions } from "@/stores/extensions.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { ExtensionsConfig } from "@/api/config/types";
    import { useInstalled, useMarketplace } from "@/app/marketplace.svelte.js";

    import InstalledExtensions from "$lib/components/settings/extensions/Installed.svelte";
    import ExtensionMarketplace from "$lib/components/settings/extensions/Marketplace.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let tab = $state("installed");
    let exploreOpened = $state(false);
    $effect(() => {
        if (tab === "explore") exploreOpened = true;
    });

    const installed = useInstalled();
    const market = useMarketplace({
        config: () => config,
        onSave: () => onSave(),
        enabled: () => exploreOpened
    });

    onMount(() => {
        extensions.load();
    });
</script>

<section class="w-full space-y-6">
    <div>
        <h2 class="text-2xl font-bold tracking-tight">{i18n.t('settings.extensions')}</h2>
        <p class="mt-1 text-sm text-muted-foreground">{i18n.t('settings.extension_section.extensions_desc')}</p>
    </div>

    <Tabs.Root bind:value={tab} class="w-full">
        <div class="pt-1 pb-4">
            <Tabs.List class="flex w-full sm:inline-flex sm:w-auto h-auto gap-1 p-1 rounded-lg bg-muted/50">
                <Tabs.Trigger
                        value="installed"
                        class="flex-1 sm:flex-none h-9 min-w-0 px-5 gap-2 rounded-md text-sm font-semibold text-muted-foreground
                           transition-all duration-200 hover:text-foreground
                           data-[state=active]:bg-background data-[state=active]:text-foreground
                           data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border/60
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                >
                    <Puzzle class="size-4 shrink-0" />
                    <span class="truncate">{i18n.t('settings.extension_section.installed_extensions')}</span>
                    {#if installed.total > 0}
                        <span class="rounded-full bg-muted-foreground/15 px-1.5 text-[10px] font-bold tabular-nums">{installed.total}</span>
                    {/if}
                </Tabs.Trigger>

                <Tabs.Trigger
                        value="explore"
                        class="flex-1 sm:flex-none h-9 min-w-0 px-5 gap-2 rounded-md text-sm font-semibold text-muted-foreground
                           transition-all duration-200 hover:text-foreground
                           data-[state=active]:bg-background data-[state=active]:text-foreground
                           data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border/60
                           focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                >
                    <Store class="size-4 shrink-0" />
                    <span class="truncate">{i18n.t('marketplace.title')}</span>
                    {#if market.updateCount > 0}
                        <span
                                class="rounded-full bg-primary px-1.5 text-[10px] font-bold tabular-nums text-primary-foreground"
                                title={i18n.t('marketplace.updates_available')}
                        >
                            {market.updateCount}
                        </span>
                    {/if}
                </Tabs.Trigger>
            </Tabs.List>
        </div>

        <Tabs.Content value="installed" class="space-y-6 outline-none">
            <InstalledExtensions {installed} onBrowse={() => (tab = "explore")} />
        </Tabs.Content>

        <Tabs.Content value="explore" class="space-y-6 outline-none">
            <ExtensionMarketplace {market} />
        </Tabs.Content>
    </Tabs.Root>
</section>