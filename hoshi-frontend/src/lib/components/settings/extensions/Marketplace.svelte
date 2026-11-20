<script lang="ts">
    import { fade } from "svelte/transition";
    import Search from 'phosphor-svelte/lib/MagnifyingGlassIcon';
    import PackageOpen from 'phosphor-svelte/lib/PackageIcon';
    import SearchX from 'phosphor-svelte/lib/MagnifyingGlassIcon';
    import Server from 'phosphor-svelte/lib/HardDrivesIcon';
    import Settings from 'phosphor-svelte/lib/GearSixIcon';
    import Plus from 'phosphor-svelte/lib/PlusIcon';
    import Trash2 from 'phosphor-svelte/lib/TrashIcon';
    import Link2 from 'phosphor-svelte/lib/LinkSimpleIcon';
    import RefreshCw from 'phosphor-svelte/lib/ArrowsClockwiseIcon';
    import AlertTriangle from 'phosphor-svelte/lib/WarningIcon';
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Spinner } from "$lib/components/ui/spinner";
    import * as Dialog from "$lib/components/ui/dialog";
    import * as Drawer from "$lib/components/ui/drawer";
    import { layoutState } from "@/stores/layout.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";
    import {
        isHttpUrl,
        sourceOfEntry,
        toCardExt,
        type MarketplaceState
    } from "@/app/marketplace.svelte.js";
    import Card from "./Card.svelte";
    import Group from "./Group.svelte";
    import EmptyState from "$lib/components/EmptyState.svelte";

    let { market }: { market: MarketplaceState } = $props();

    let managing = $state(false);
    let stremioUrl = $state("");
    let newRepo = $state("");

    function addRepo() {
        if (market.addRepo(newRepo)) newRepo = "";
    }

    function onVisible(node: HTMLElement, params: { key: unknown; cb: () => void }) {
        let current = params;
        let io: IntersectionObserver | null = null;
        const start = () => {
            io?.disconnect();
            io = new IntersectionObserver(
                (entries) => { if (entries[0]?.isIntersecting) current.cb(); },
                { rootMargin: "200px" }
            );
            io.observe(node);
        };
        start();
        return {
            update(next: { key: unknown; cb: () => void }) {
                current = next;
                start();
            },
            destroy: () => io?.disconnect()
        };
    }
</script>

{#snippet manager()}
    <div class="space-y-2">
        {#each market.repos as url, i (i)}
            {@const invalid = !isHttpUrl(url)}
            <div class="flex items-center gap-2">
                <button
                        type="button"
                        class="flex size-9 shrink-0 items-center justify-center rounded-md border font-mono text-xs font-bold transition-colors
                               {i === market.activeIndex
                            ? 'border-primary bg-primary/10 text-primary'
                            : 'border-border/60 text-muted-foreground hover:bg-muted'}"
                        aria-label={i18n.t('marketplace.use_repo')}
                        aria-pressed={i === market.activeIndex}
                        onclick={() => market.selectRepo(i)}
                >
                    {i + 1}
                </button>

                <div class="relative min-w-0 flex-1">
                    <Link2 class="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
                    <Input
                            value={url}
                            aria-invalid={invalid}
                            class="h-9 rounded-lg bg-background pl-8 text-sm {invalid ? 'border-destructive focus-visible:ring-destructive/30' : 'border-border/50'}"
                            onkeydown={(e) => { if (e.key === "Enter") e.currentTarget.blur(); }}
                            onchange={(e) => market.editRepo(i, e.currentTarget.value)}
                    />
                </div>

                <Button
                        variant="ghost"
                        size="sm"
                        class="size-9 shrink-0 rounded-lg p-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                        aria-label={i18n.t('marketplace.remove_repo')}
                        onclick={() => market.removeRepo(i)}
                >
                    <Trash2 class="size-4" />
                </Button>
            </div>
        {/each}

        <form class="flex items-center gap-2" onsubmit={(e) => { e.preventDefault(); addRepo(); }}>
            <div class="relative min-w-0 flex-1">
                <Plus class="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
                <Input
                        bind:value={newRepo}
                        placeholder="https://example.com/index.json"
                        class="h-9 rounded-lg border-border/50 bg-background pl-8 text-sm"
                />
            </div>
            <Button type="submit" variant="outline" size="sm" class="h-9 shrink-0 gap-1.5 rounded-lg text-xs font-bold" disabled={!newRepo.trim()}>
                <Plus class="size-3.5" />
                {i18n.t('marketplace.add_repo')}
            </Button>
        </form>
    </div>

    <div class="space-y-2 border-t border-border/50 pt-4">
        <div>
            <h4 class="text-sm font-bold">{i18n.t('marketplace.stremio_title')}</h4>
            <p class="text-xs text-muted-foreground">{i18n.t('marketplace.stremio_desc')}</p>
        </div>
        <form
                class="flex items-center gap-2"
                onsubmit={async (e) => {
                    e.preventDefault();
                    if (await market.installStremio(stremioUrl)) stremioUrl = "";
                }}
        >
            <div class="relative min-w-0 flex-1">
                <Link2 class="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
                <Input
                        bind:value={stremioUrl}
                        placeholder="https://addon.example.com/manifest.json"
                        class="h-9 rounded-lg border-border/50 bg-background pl-8 text-sm"
                />
            </div>
            <Button type="submit" variant="outline" size="sm" class="h-9 shrink-0 gap-1.5 rounded-lg text-xs font-bold" disabled={market.stremioBusy || !stremioUrl.trim()}>
                {#if market.stremioBusy}
                    <Spinner class="size-3.5 animate-spin" />
                {:else}
                    <Plus class="size-3.5" />
                {/if}
                {i18n.t('marketplace.install')}
            </Button>
        </form>
    </div>
{/snippet}

{#if layoutState.isMobile}
    <Drawer.Root bind:open={managing}>
        <Drawer.Content class="max-h-[90dvh]">
            <Drawer.Header class="text-left">
                <Drawer.Title class="text-xl font-bold tracking-tight">{i18n.t('marketplace.repositories')}</Drawer.Title>
                <Drawer.Description>{i18n.t('marketplace.repositories_desc')}</Drawer.Description>
            </Drawer.Header>
            <div class="min-h-0 flex-1 space-y-4 overflow-y-auto px-4 pb-4 pb-safe">
                {@render manager()}
            </div>
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Dialog.Root bind:open={managing}>
        <Dialog.Content class="flex max-h-[85vh] w-[95vw] flex-col gap-0 p-0 sm:max-w-xl">
            <Dialog.Header class="border-b border-border/40 p-6 pb-4">
                <Dialog.Title class="text-xl font-bold tracking-tight">{i18n.t('marketplace.repositories')}</Dialog.Title>
                <Dialog.Description>{i18n.t('marketplace.repositories_desc')}</Dialog.Description>
            </Dialog.Header>
            <div class="min-h-0 flex-1 space-y-4 overflow-y-auto p-6">
                {@render manager()}
            </div>
        </Dialog.Content>
    </Dialog.Root>
{/if}

<div class="space-y-6">
    <!-- Toolbar -->
    <div class="flex flex-col items-stretch gap-3 lg:flex-row lg:items-center">
        <div class="group relative w-full lg:flex-1">
            <Search class="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground transition-colors group-focus-within:text-primary" />
            <Input
                    placeholder={i18n.t('marketplace.search_repository')}
                    class="h-9 w-full bg-background pl-9 text-sm"
                    bind:value={market.query}
                    disabled={!market.hasItems}
            />
        </div>

        <div class="flex items-center gap-2">
            <div class="flex h-9 w-36 shrink-0">
                <ResponsiveSelect
                        bind:value={market.lang}
                        items={market.langItems}
                        placeholder={market.langItems[0].label}
                        class="h-full w-full"
                />
            </div>

            {#if market.repos.length > 0}
            <div class="relative flex h-9 min-w-0 flex-1 lg:w-56 lg:flex-none">
                <Server class="pointer-events-none absolute left-3 top-1/2 z-10 size-4 -translate-y-1/2 text-muted-foreground/70" />
                <ResponsiveSelect
                        value={String(market.activeIndex)}
                        items={market.repoItems}
                        onValueChange={(v) => market.selectRepo(Number(v))}
                        class="h-full w-full pl-9"
                />
                {#if market.loading}
                    <Spinner class="pointer-events-none absolute right-9 top-1/2 z-10 size-4 -translate-y-1/2 animate-spin text-muted-foreground" />
                {/if}
            </div>
            {/if}

            <Button
                    variant={managing ? "secondary" : "outline"}
                    size="sm"
                    class="size-9 shrink-0 rounded-md p-0"
                    aria-label={i18n.t('marketplace.manage_repos')}
                    onclick={() => (managing = !managing)}
            >
                <Settings class="size-4 transition-transform duration-300 {managing ? 'rotate-90 text-primary' : 'text-muted-foreground'}" />
            </Button>
        </div>
    </div>

    <!-- Results -->
    {#if market.view === "noRepo"}
        <EmptyState
                icon={Server}
                tone="primary"
                title={i18n.t('marketplace.no_repo')}
        >
            <Button variant="outline" class="gap-2 rounded-lg" onclick={() => (managing = true)}>
                <Plus class="size-4" />
                {i18n.t('marketplace.add_repo')}
            </Button>
        </EmptyState>

    {:else if market.view === "invalid"}
        <EmptyState
                icon={AlertTriangle}
                title={i18n.t('marketplace.invalid_url')}
                description={i18n.t('marketplace.invalid_url_desc')}
        />

    {:else if market.view === "loading"}
        <Group skeletons={6} />

    {:else if market.view === "error"}
        <EmptyState
                icon={AlertTriangle}
                title={i18n.t('error.generic')}
                description={market.error ?? ""}
        >
            <Button variant="outline" class="gap-2 rounded-lg" onclick={market.reload}>
                <RefreshCw class="size-4" />
                {i18n.t('content.retry')}
            </Button>
        </EmptyState>

    {:else if market.view === "empty"}
        <EmptyState
                icon={PackageOpen}
                title={i18n.t('marketplace.empty')}
                description={i18n.t('marketplace.empty_desc')}
        />

    {:else if market.view === "noMatch"}
        <EmptyState icon={SearchX} title={i18n.t('error.content.no_match_found')} />

    {:else}
        <div class="space-y-8" in:fade={{ duration: 150 }}>
            {#each market.sections as section (section.lang)}
                <Group title={section.lang} count={section.total}>
                    {#each section.items as item (item.id)}
                        {@const installed = market.isInstalled(item)}
                        {@const update = market.hasUpdate(item)}
                        <Card
                                mode="marketplace"
                                ext={toCardExt(item)}
                                source={sourceOfEntry(item)}
                                {installed}
                                hasUpdate={update}
                                fromVersion={update ? market.installedVersion(item) : null}
                                busy={market.isBusy(item)}
                                onInstall={() => market.install(item)}
                                onUpdate={() => market.update(item)}
                        />
                    {/each}
                </Group>
            {/each}

            {#if market.hasMore}
                <div
                        use:onVisible={{ key: market.visibleCount, cb: market.loadMore }}
                        class="flex justify-center py-6"
                >
                    <Spinner class="size-5 text-muted-foreground/50" />
                </div>
            {/if}
        </div>
    {/if}
</div>