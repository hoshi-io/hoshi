<script lang="ts">
    import { onMount, tick } from 'svelte';
    import { Puzzle, Store, Settings, Plus, Trash2, PackageOpen, SearchX, Server, RefreshCw, AlertTriangle } from 'lucide-svelte';
    import { extensions } from '@/stores/extensions.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import type { ExtensionsConfig } from '@/api/config/types';
    import { Spinner } from '@/components/ui/spinner';
    import {
        useInstalled,
        useMarketplace,
        sourceFilterItems,
        sourceOfId,
        sourceOfEntry,
        toCardExt,
        isHttpUrl
    } from '@/app/marketplace.svelte.js';
    import EmptyState from '$lib/components/settings/extensions/EmptyState.svelte';
    import TvDialog from '@/tv/components/TvDialog.svelte';
    import TvExtensionCard from './TvExtensionCard.svelte';

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig;
        onSave: () => Promise<void> | void;
    } = $props();

    // Typing URLs with a remote is painful: new repositories start with this already filled in.
    const RAW_GITHUB_PREFIX = 'https://raw.githubusercontent.com/';

    let tab = $state<'installed' | 'explore'>('installed');
    let exploreOpened = $state(false);
    $effect(() => {
        if (tab === 'explore') exploreOpened = true;
    });

    const installed = useInstalled();
    const market = useMarketplace({
        config: () => config,
        onSave: () => onSave(),
        enabled: () => exploreOpened
    });

    // LNReader only serves novels, which the TV app doesn't support.
    const sourceItems = $derived(
        sourceFilterItems().filter((i) => !/lnreader/i.test(`${i.value} ${i.label}`))
    );

    // The TV app only plays anime, so only these extension types are shown / installable here.
    const TV_TYPES = ['anime', 'torrent'];
    const isTvType = (type?: string | null) => !!type && TV_TYPES.includes(type.trim().toLowerCase());

    // Filter by each extension's own type (not the section key, whose format I couldn't see).
    const installedSections = $derived(
        installed.sections
            .map((s) => ({ ...s, items: s.items.filter((ext) => isTvType(ext.ext_type)) }))
            .filter((s) => s.items.length > 0)
    );
    const installedCount = $derived(installedSections.reduce((n, s) => n + s.items.length, 0));

    const marketSections = $derived(
        market.sections
            .map((s) => {
                const items = s.items.filter((item) => isTvType(toCardExt(item).ext_type));
                return { ...s, items, total: items.length };
            })
            .filter((s) => s.items.length > 0)
    );

    // If a whole page was filtered out, keep fetching until something visible shows up.
    $effect(() => {
        if (tab === 'explore' && marketSections.length === 0 && market.hasMore) market.loadMore();
    });

    onMount(() => {
        extensions.load();
    });

    // The removed card takes the focused button with it; without this, spatialNav sees <body>
    // focused and jumps to the first element on the page. Hand focus to the neighbouring card.
    async function removeExt(id: string) {
        const cards = () => Array.from(document.querySelectorAll<HTMLElement>('[data-ext-card]'));
        const index = cards().findIndex((c) => c.dataset.extCard === id);

        await installed.uninstall(id);
        await tick();

        const active = document.activeElement;
        if (active && active !== document.body && document.contains(active)) return;

        const left = cards();
        const target = left[Math.min(Math.max(index, 0), left.length - 1)];
        const el =
            target?.querySelector<HTMLElement>('[data-sn]') ??
            document.querySelector<HTMLElement>('[role="radiogroup"] [data-sn]');
        el?.focus();
    }

    let managing = $state(false);

    async function openManager() {
        managing = true;
        await tick();

        if (!market.repos.length) {
            await addRepo();
            return;
        }

        let first = -1;
        market.repos.forEach((url, i) => {
            if (!url?.trim()) {
                market.repos[i] = RAW_GITHUB_PREFIX;
                if (first < 0) first = i;
            }
        });
        if (first < 0) return;

        await tick();
        await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        const input = document.querySelector<HTMLInputElement>(`[data-repo="${first}"]`);
        if (!input) return;
        input.focus();
        input.setSelectionRange(input.value.length, input.value.length);
    }

    function closeManager() {
        // A row left as just the prefix is not a real repo: put it back to empty.
        market.repos.forEach((url, i) => {
            if (url?.trim() === RAW_GITHUB_PREFIX) market.repos[i] = '';
        });
        managing = false;
    }

    async function addRepo() {
        market.addRepo();
        await tick();
        const last = market.repos.length - 1;
        if (last < 0) return;
        if (!market.repos[last]?.trim()) market.repos[last] = RAW_GITHUB_PREFIX;
        await tick();
        const input = document.querySelector<HTMLInputElement>(`[data-repo="${last}"]`);
        if (!input) return;
        input.focus();
        input.setSelectionRange(input.value.length, input.value.length);
    }

    // ---- Infinite scroll (observer + a focusable fallback for the D-pad) ----
    function onVisible(node: HTMLElement, params: { key: unknown; cb: () => void }) {
        let current = params;
        let io: IntersectionObserver | null = null;
        const start = () => {
            io?.disconnect();
            io = new IntersectionObserver(
                (entries) => {
                    if (entries[0]?.isIntersecting) current.cb();
                },
                { rootMargin: '300px' }
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

    // ---- Shared classes -----------------------------------------------------
    const chipBase =
        'h-12 scroll-my-24 flex items-center gap-2 rounded-md px-5 text-lg font-semibold outline-none transition-[transform,background-color,color] duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70';
    const chipClass = (on: boolean) =>
        `${chipBase} ${on ? 'bg-primary text-primary-foreground' : 'bg-muted/30 text-foreground/80'}`;
    const inputClass =
        'h-12 w-full scroll-my-24 rounded-md border-2 border-border/60 bg-card px-4 text-lg outline-none focus:border-primary focus:ring-4 focus:ring-primary/40';
    const outlineBtn =
        'flex h-12 scroll-my-24 items-center gap-2 rounded-lg border-2 border-border/60 px-6 text-base font-bold text-foreground/80 outline-none transition-transform duration-150 focus:scale-105 focus:ring-4 focus:ring-primary/70';
</script>

{#snippet skeletons(n: number)}
    <div class="space-y-3">
        {#each Array(n) as _, i (i)}
            <div class="flex items-center gap-5 rounded-lg border-2 border-border/50 bg-card p-4">
                <div class="size-16 shrink-0 animate-pulse rounded-md bg-muted"></div>
                <div class="flex-1 space-y-2">
                    <div class="h-5 w-1/3 animate-pulse rounded bg-muted"></div>
                    <div class="h-4 w-1/4 animate-pulse rounded bg-muted/70"></div>
                </div>
                <div class="h-12 w-28 animate-pulse rounded-lg bg-muted"></div>
            </div>
        {/each}
    </div>
{/snippet}

{#snippet sectionHeader(title: string, count: number)}
    <div class="flex items-center gap-4">
        <h3 class="text-lg font-bold uppercase tracking-wider text-muted-foreground">{title}</h3>
        <span class="rounded-full border border-border/50 bg-muted px-3 py-0.5 text-sm font-semibold tabular-nums text-muted-foreground">
            {count}
        </span>
        <div class="h-px flex-1 bg-border/50"></div>
    </div>
{/snippet}

<div class="space-y-6">
    <header class="space-y-1 pb-2">
        <h2 class="text-3xl font-black tracking-tight">{i18n.t('settings.extensions')}</h2>
        <p class="text-base text-muted-foreground">{i18n.t('settings.extension_section.extensions_desc')}</p>
    </header>

    <!-- Tabs -->
    <div role="radiogroup" aria-label={i18n.t('settings.extensions')} class="flex flex-wrap gap-3">
        <button
                type="button"
                role="radio"
                aria-checked={tab === 'installed'}
                data-sn
                class={chipClass(tab === 'installed')}
                onclick={() => (tab = 'installed')}
        >
            <Puzzle class="size-5 shrink-0" />
            {i18n.t('settings.extension_section.installed_extensions')}
            {#if installedCount > 0}
                <span class="rounded-full bg-black/20 px-2 text-sm font-bold tabular-nums">{installedCount}</span>
            {/if}
        </button>

        <button
                type="button"
                role="radio"
                aria-checked={tab === 'explore'}
                data-sn
                class={chipClass(tab === 'explore')}
                onclick={() => (tab = 'explore')}
        >
            <Store class="size-5 shrink-0" />
            {i18n.t('marketplace.title')}
            {#if market.updateCount > 0}
                <span class="rounded-full bg-primary-foreground px-2 text-sm font-bold tabular-nums text-primary">
                    {market.updateCount}
                </span>
            {/if}
        </button>
    </div>

    <!-- Installed -->
    {#if tab === 'installed'}
        {#if installed.view === 'loading'}
            {@render skeletons(4)}
        {:else if installed.view === 'empty'}
            <EmptyState
                    icon={PackageOpen}
                    title={i18n.t('settings.extension_section.no_installed')}
                    description={i18n.t('settings.extension_section.no_installed_desc')}
            >
                <button type="button" data-sn class={outlineBtn} onclick={() => (tab = 'explore')}>
                    <Store class="size-5" />
                    {i18n.t('marketplace.title')}
                </button>
            </EmptyState>
        {:else}
            <div class="space-y-4">
                <input
                        data-sn
                        type="text"
                        autocomplete="off"
                        placeholder={i18n.t('marketplace.search_repository')}
                        bind:value={installed.query}
                        class={inputClass}
                />

                <div role="radiogroup" aria-label={sourceItems[0].label} class="flex flex-wrap gap-3">
                    {#each sourceItems as item (item.value)}
                        {@const on = installed.source === item.value}
                        <button
                                type="button"
                                role="radio"
                                aria-checked={on}
                                data-sn
                                class={chipClass(on)}
                                onclick={() => (installed.source = item.value)}
                        >
                            {item.label}
                        </button>
                    {/each}
                </div>
            </div>

            {#if installed.view === 'noMatch' || installedSections.length === 0}
                <EmptyState icon={SearchX} title={i18n.t('errors.content.no_match_found')} />
            {:else}
                <div class="space-y-8">
                    {#each installedSections as section (section.type)}
                        <section class="space-y-3">
                            {@render sectionHeader(section.type, section.items.length)}
                            <div class="space-y-3">
                                {#each section.items as ext (ext.id)}
                                    <TvExtensionCard
                                            {ext}
                                            source={sourceOfId(ext.id)}
                                            saving={installed.isSaving(ext.id)}
                                            busy={installed.isRemoving(ext.id)}
                                            onUninstall={() => removeExt(ext.id)}
                                            onSave={(settings) => installed.saveSettings(ext.id, settings)}
                                    />
                                {/each}
                            </div>
                        </section>
                    {/each}
                </div>
            {/if}
        {/if}

        <!-- Marketplace -->
    {:else}
        <div class="space-y-4">
            <input
                    data-sn
                    type="text"
                    autocomplete="off"
                    placeholder={i18n.t('marketplace.search_repository')}
                    bind:value={market.query}
                    disabled={!market.hasItems}
                    class="{inputClass} disabled:opacity-50"
            />

            <div class="flex flex-wrap items-center gap-3">
                <div role="radiogroup" aria-label={market.langItems[0].label} class="flex flex-wrap gap-3">
                    {#each market.langItems as item (item.value)}
                        {@const on = market.lang === item.value}
                        <button
                                type="button"
                                role="radio"
                                aria-checked={on}
                                data-sn
                                class={chipClass(on)}
                                onclick={() => (market.lang = item.value)}
                        >
                            {item.label}
                        </button>
                    {/each}
                </div>
            </div>

            <div class="flex flex-wrap items-center gap-3">
                <div role="radiogroup" aria-label={i18n.t('marketplace.repositories')} class="flex flex-wrap gap-3">
                    {#each market.repoItems as item (item.value)}
                        {@const on = String(market.activeIndex) === String(item.value)}
                        <button
                                type="button"
                                role="radio"
                                aria-checked={on}
                                data-sn
                                class={chipClass(on)}
                                onclick={() => market.selectRepo(Number(item.value))}
                        >
                            <Server class="size-5 shrink-0" />
                            {item.label}
                        </button>
                    {/each}
                </div>

                <button type="button" data-sn class={outlineBtn} onclick={openManager}>
                    <Settings class="size-5" />
                    {i18n.t('marketplace.manage_repos')}
                </button>

                {#if market.loading}<Spinner class="size-5 text-muted-foreground" />{/if}
            </div>
        </div>

        {#if market.view === 'noRepo'}
            <EmptyState icon={Server} tone="primary" title={i18n.t('marketplace.no_repo')}>
                <button type="button" data-sn class={outlineBtn} onclick={openManager}>
                    <Plus class="size-5" />
                    {i18n.t('marketplace.add_repo')}
                </button>
            </EmptyState>
        {:else if market.view === 'invalid'}
            <EmptyState
                    icon={AlertTriangle}
                    title={i18n.t('marketplace.invalid_url')}
                    description={i18n.t('marketplace.invalid_url_desc')}
            />
        {:else if market.view === 'loading'}
            {@render skeletons(5)}
        {:else if market.view === 'error'}
            <EmptyState icon={AlertTriangle} title={i18n.t('errors.error')} description={market.error ?? ''}>
                <button type="button" data-sn class={outlineBtn} onclick={market.reload}>
                    <RefreshCw class="size-5" />
                    {i18n.t('content.retry')}
                </button>
            </EmptyState>
        {:else if market.view === 'empty'}
            <EmptyState
                    icon={PackageOpen}
                    title={i18n.t('marketplace.empty')}
                    description={i18n.t('marketplace.empty_desc')}
            />
        {:else if market.view === 'noMatch' || (marketSections.length === 0 && !market.hasMore)}
            <EmptyState icon={SearchX} title={i18n.t('errors.content.no_match_found')} />
        {:else if marketSections.length === 0}
            {@render skeletons(3)}
        {:else}
            <div class="space-y-8">
                {#each marketSections as section (section.lang)}
                    <section class="space-y-3">
                        {@render sectionHeader(section.lang, section.total)}
                        <div class="space-y-3">
                            {#each section.items as item (item.id)}
                                {@const update = market.hasUpdate(item)}
                                <TvExtensionCard
                                        mode="marketplace"
                                        ext={toCardExt(item)}
                                        source={sourceOfEntry(item)}
                                        installed={market.isInstalled(item)}
                                        hasUpdate={update}
                                        fromVersion={update ? market.installedVersion(item) : null}
                                        busy={market.isBusy(item)}
                                        onInstall={() => market.install(item)}
                                        onUpdate={() => market.update(item)}
                                />
                            {/each}
                        </div>
                    </section>
                {/each}

                {#if market.hasMore}
                    <div
                            use:onVisible={{ key: market.visibleCount, cb: market.loadMore }}
                            class="flex justify-center py-4"
                    >
                        <button type="button" data-sn class={outlineBtn} onclick={market.loadMore}>
                            {i18n.t('marketplace.load_more')}
                        </button>
                    </div>
                {/if}
            </div>
        {/if}
    {/if}
</div>

{#if managing}
    <TvDialog
            title={i18n.t('marketplace.repositories')}
            description={i18n.t('marketplace.repositories_desc')}
            onclose={closeManager}
    >
        <div class="space-y-3">
            {#each market.repos as url, i (i)}
                {@const invalid = url.trim() !== '' && !isHttpUrl(url.trim())}
                <div class="flex items-center gap-3">
                    <button
                            type="button"
                            data-sn
                            aria-pressed={i === market.activeIndex}
                            aria-label={i18n.t('marketplace.use_repo')}
                            onclick={() => market.selectRepo(i)}
                            class="flex size-12 shrink-0 items-center justify-center rounded-md border-2 font-mono text-base font-bold outline-none focus:ring-4 focus:ring-primary/70
                                   {i === market.activeIndex ? 'border-primary bg-primary/10 text-primary' : 'border-border/60 text-muted-foreground'}"
                    >
                        {i + 1}
                    </button>

                    <input
                            data-sn
                            data-repo={i}
                            type="url"
                            autocomplete="off"
                            bind:value={market.repos[i]}
                            placeholder="{RAW_GITHUB_PREFIX}user/repo/branch/index.json"
                            aria-invalid={invalid}
                            onchange={market.commitRepos}
                            class="{inputClass} min-w-0 flex-1 {invalid ? 'border-destructive' : ''}"
                    />

                    <button
                            type="button"
                            data-sn
                            aria-label={i18n.t('marketplace.remove_repo')}
                            onclick={() => market.removeRepo(i)}
                            class="flex size-12 shrink-0 items-center justify-center rounded-md text-muted-foreground outline-none focus:bg-destructive focus:text-destructive-foreground focus:ring-4 focus:ring-destructive/50"
                    >
                        <Trash2 class="size-5" />
                    </button>
                </div>
            {/each}
        </div>

        <div class="flex justify-between gap-3 pt-2">
            <button type="button" data-sn class={outlineBtn} onclick={addRepo}>
                <Plus class="size-5" />
                {i18n.t('marketplace.add_repo')}
            </button>
            <button
                    type="button"
                    data-sn
                    onclick={closeManager}
                    class="h-12 rounded-lg bg-primary px-8 text-base font-bold text-primary-foreground outline-none focus:ring-4 focus:ring-primary/50"
            >
                {i18n.t('content.close')}
            </button>
        </div>
    </TvDialog>
{/if}