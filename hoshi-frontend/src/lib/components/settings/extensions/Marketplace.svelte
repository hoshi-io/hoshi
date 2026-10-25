<script lang="ts">
    import { fade, slide } from "svelte/transition";
    import { Search, PackageOpen, SearchX, Server, Settings, Plus, Trash2, Link2, RefreshCw, AlertTriangle } from "lucide-svelte";
    import { Input } from "$lib/components/ui/input";
    import { Button } from "$lib/components/ui/button";
    import { Spinner } from "$lib/components/ui/spinner";
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
    import EmptyState from "./EmptyState.svelte";

    let { market }: { market: MarketplaceState } = $props();

    let managing = $state(false);

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

            <Button
                    variant={managing ? "secondary" : "outline"}
                    size="sm"
                    class="size-9 shrink-0 rounded-md p-0"
                    aria-label={i18n.t('marketplace.manage_repos')}
                    aria-expanded={managing}
                    onclick={() => (managing = !managing)}
            >
                <Settings class="size-4 transition-transform duration-300 {managing ? 'rotate-90 text-primary' : 'text-muted-foreground'}" />
            </Button>
        </div>
    </div>

    <!-- Repository manager -->
    {#if managing}
        <div class="space-y-3 rounded-xl border border-border/60 bg-muted/20 p-4" transition:slide={{ duration: 250 }}>
            <div class="flex items-start justify-between gap-3">
                <div>
                    <h4 class="text-sm font-bold">{i18n.t('marketplace.repositories')}</h4>
                    <p class="text-xs text-muted-foreground">
                        {i18n.t('marketplace.repositories_desc')}
                    </p>
                </div>
                <Button variant="outline" size="sm" class="h-8 shrink-0 gap-1.5 rounded-lg text-xs font-bold" onclick={market.addRepo}>
                    <Plus class="size-3.5" />
                    {i18n.t('marketplace.add_repo')}
                </Button>
            </div>

            <div class="max-h-52 space-y-2 overflow-y-auto pr-1">
                {#each market.repos as url, i (i)}
                    {@const invalid = url.trim() !== "" && !isHttpUrl(url.trim())}
                    <div class="flex items-center gap-2">
                        <button
                                type="button"
                                class="flex size-8 shrink-0 items-center justify-center rounded-md border font-mono text-xs font-bold transition-colors
                                       {i === market.activeIndex
                                    ? 'border-primary bg-primary/10 text-primary'
                                    : 'border-border/60 text-muted-foreground hover:bg-muted'}"
                                aria-label={i18n.t('marketplace.use_repo')}
                                aria-pressed={i === market.activeIndex}
                                onclick={() => market.selectRepo(i)}
                        >
                            {i + 1}
                        </button>

                        <div class="relative flex-1">
                            <Link2 class="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground/60" />
                            <Input
                                    bind:value={market.repos[i]}
                                    placeholder="https://example.com/index.json"
                                    aria-invalid={invalid}
                                    class="h-9 rounded-lg bg-background pl-8 text-sm {invalid ? 'border-destructive focus-visible:ring-destructive/30' : 'border-border/50'}"
                                    onchange={market.commitRepos}
                            />
                        </div>

                        <Button
                                variant="ghost"
                                size="sm"
                                class="size-8 shrink-0 rounded-lg p-0 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                                aria-label={i18n.t('marketplace.remove_repo')}
                                onclick={() => market.removeRepo(i)}
                        >
                            <Trash2 class="size-4" />
                        </Button>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

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
                title={i18n.t('errors.error')}
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
        <EmptyState icon={SearchX} title={i18n.t('errors.content.no_match_found')} />

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