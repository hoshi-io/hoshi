<script lang="ts">
    import { fade } from "svelte/transition";
    import { Search, SearchX, PackageOpen, Store } from "lucide-svelte";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { i18n } from "@/stores/i18n.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import { sourceFilterItems, sourceOfId, type InstalledState } from "@/app/marketplace.svelte.js";
    import Card from "./Card.svelte";
    import Group from "./Group.svelte";
    import EmptyState from "./EmptyState.svelte";

    let {
        installed,
        onBrowse
    }: {
        installed: InstalledState;
        onBrowse?: () => void;
    } = $props();

    const sourceItems = $derived(sourceFilterItems());
</script>

<div class="space-y-6">
    {#if installed.view === "loading"}
        <Group skeletons={4} />

    {:else if installed.view === "empty"}
        <EmptyState
                icon={PackageOpen}
                title={i18n.t('settings.extension_section.no_installed')}
                description={i18n.t('settings.extension_section.no_installed_desc')}
        >
            {#if onBrowse}
                <Button variant="outline" class="gap-2 rounded-lg" onclick={onBrowse}>
                    <Store class="size-4" />
                    {i18n.t('marketplace.title')}
                </Button>
            {/if}
        </EmptyState>

    {:else}
        <div class="flex flex-col items-stretch gap-3 md:flex-row md:items-center">
            <div class="group relative w-full md:flex-1">
                <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground transition-colors group-focus-within:text-primary" />
                <Input
                        placeholder={i18n.t('marketplace.search_repository')}
                        class="h-9 w-full bg-background pl-9 text-sm"
                        bind:value={installed.query}
                />
            </div>

            <div class="flex h-9 shrink-0 md:w-44">
                <ResponsiveSelect
                        bind:value={installed.source}
                        items={sourceItems}
                        placeholder={sourceItems[0].label}
                        class="h-full w-full"
                />
            </div>
        </div>

        {#if installed.view === "noMatch"}
            <div in:fade={{ duration: 150 }}>
                <EmptyState icon={SearchX} title={i18n.t('errors.content.no_match_found')} />
            </div>
        {:else}
            <div class="space-y-8" in:fade={{ duration: 150 }}>
                {#each installed.sections as section (section.type)}
                    <Group title={section.type} count={section.items.length}>
                        {#each section.items as ext (ext.id)}
                            <Card
                                    {ext}
                                    source={sourceOfId(ext.id)}
                                    saving={installed.isSaving(ext.id)}
                                    busy={installed.isRemoving(ext.id)}
                                    onUninstall={() => installed.uninstall(ext.id)}
                                    onSave={(settings) => installed.saveSettings(ext.id, settings)}
                            />
                        {/each}
                    </Group>
                {/each}
            </div>
        {/if}
    {/if}
</div>