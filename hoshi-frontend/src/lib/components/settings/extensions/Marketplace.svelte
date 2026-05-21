<script lang="ts">
    import { extensions } from "@/stores/extensions.svelte.js";
    import type { AnyMarketplaceEntry, LNReaderMarketplaceEntry, NativeMarketplaceEntry, TachiyomiMarketplaceEntry } from "@/api/extensions/types";
    import { fade, slide, fly } from "svelte/transition";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Input } from "$lib/components/ui/input";
    import { Search, PackageOpen, SearchX, Server, Settings, Plus, Trash2 } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import type { ExtensionsConfig } from "@/api/config/types";
    import Card from "./Card.svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let installingIds = $state<Set<string>>(new Set());
    let marketSearchQuery = $state("");
    let isLoadingRepo = $state(false);
    let isTransitioning = $state(false);
    let lastLoadedUrl = $state("");
    let marketplaceItems = $state<AnyMarketplaceEntry[]>([]);

    let repoUrlsLocal = $state<string[]>(
        Array.isArray(config.repoUrls) ? [...config.repoUrls] : (config.repoUrls ? [config.repoUrls as string] : [""])
    );
    let activeRepoIndex = $state(0);
    let isManagingRepos = $state(false);

    let activeRepoUrl = $derived(repoUrlsLocal[activeRepoIndex] || "");
    let activeRepoValue = $derived(String(activeRepoIndex));
    function onRepoChange(val: string) { activeRepoIndex = Number(val); }
    let repoItems = $derived(repoUrlsLocal.map((url, i) => ({ value: String(i), label: getFriendlyLabel(url, i) })));

    let selectedLang = $state("All");

    function normalizeLang(lang: string | undefined): string {
        if (!lang) return 'Unknown';
        const l = lang.toLowerCase().trim();

        // Multi-language aliases
        if (['all', 'multi', 'multiple', 'multilingual', 'various', 'universal'].includes(l)) return 'Multi';

        // ISO 639-1 / common codes
        const codes: Record<string, string> = {
            af: 'Afrikaans', ar: 'Arabic', az: 'Azerbaijani', be: 'Belarusian',
            bg: 'Bulgarian', bn: 'Bengali', ca: 'Catalan', cs: 'Czech',
            da: 'Danish', de: 'German', el: 'Greek', en: 'English',
            eo: 'Esperanto', es: 'Spanish', et: 'Estonian', eu: 'Basque',
            fa: 'Persian', fi: 'Finnish', fr: 'French', gl: 'Galician',
            gu: 'Gujarati', he: 'Hebrew', hi: 'Hindi', hr: 'Croatian',
            hu: 'Hungarian', hy: 'Armenian', id: 'Indonesian', is: 'Icelandic',
            it: 'Italian', ja: 'Japanese', ka: 'Georgian', kk: 'Kazakh',
            km: 'Khmer', kn: 'Kannada', ko: 'Korean', lt: 'Lithuanian',
            lv: 'Latvian', mk: 'Macedonian', ml: 'Malayalam', mn: 'Mongolian',
            mr: 'Marathi', ms: 'Malay', my: 'Burmese', ne: 'Nepali',
            nl: 'Dutch', no: 'Norwegian', pa: 'Punjabi', pl: 'Polish',
            pt: 'Portuguese', ro: 'Romanian', ru: 'Russian', si: 'Sinhala',
            sk: 'Slovak', sl: 'Slovenian', sq: 'Albanian', sr: 'Serbian',
            sv: 'Swedish', sw: 'Swahili', ta: 'Tamil', te: 'Telugu',
            th: 'Thai', tl: 'Filipino', tr: 'Turkish', uk: 'Ukrainian',
            ur: 'Urdu', uz: 'Uzbek', vi: 'Vietnamese', zh: 'Chinese',
            'zh-hans': 'Chinese (Simplified)', 'zh-hant': 'Chinese (Traditional)',
            'pt-br': 'Portuguese (Brazil)', 'pt-pt': 'Portuguese (Portugal)',
            'es-419': 'Spanish (Latin America)',
        };

        if (codes[l]) return codes[l];

        // Already a full word (e.g. "english", "spanish") — just capitalize
        return l.charAt(0).toUpperCase() + l.slice(1);
    }

    function getItemLang(item: AnyMarketplaceEntry): string | undefined {
        if ('language' in item) return item.language;
        return item.lang;
    }

    let availableLanguages = $derived(
        Array.from(new Set(marketplaceItems.map(item => normalizeLang(getItemLang(item))))).sort()
    );

    let langItems = $derived([
        { value: "All", label: "All Languages" },
        ...availableLanguages.map(l => ({ value: l, label: l }))
    ]);

    const CHUNK_SIZE = 24;
    let visibleCount = $state(CHUNK_SIZE);
    let sentinelEl = $state<HTMLDivElement | null>(null);
    let observer: IntersectionObserver | null = null;
    type Section = { lang: string; items: AnyMarketplaceEntry[] };

    let sections = $derived((() => {
        const query = marketSearchQuery.toLowerCase();
        const groups: Record<string, AnyMarketplaceEntry[]> = {};

        for (const item of marketplaceItems) {
            const lang = normalizeLang(getItemLang(item));
            if (selectedLang !== "All" && lang !== selectedLang) continue;
            if (query && !item.name.toLowerCase().includes(query)) continue;
            (groups[lang] ??= []).push(item);
        }

        return Object.entries(groups).map(([lang, items]) => ({ lang, items })) as Section[];
    })());

    $effect(() => {
        sections; // track
        visibleCount = CHUNK_SIZE;
    });

    let visibleSections = $derived((() => {
        let remaining = visibleCount;
        const result: Section[] = [];
        for (const s of sections) {
            if (remaining <= 0) break;
            result.push({ lang: s.lang, items: s.items.slice(0, remaining) });
            remaining -= s.items.length;
        }
        return result;
    })());

    let totalCards = $derived(sections.reduce((n, s) => n + s.items.length, 0));
    let hasMore = $derived(visibleCount < totalCards);

    $effect(() => {
        observer?.disconnect();
        if (!sentinelEl) return;

        observer = new IntersectionObserver(entries => {
            if (entries[0]?.isIntersecting && hasMore) {
                visibleCount += CHUNK_SIZE;
            }
        }, { rootMargin: '200px' });

        observer.observe(sentinelEl);
        return () => observer?.disconnect();
    });

    $effect(() => {
        const url = activeRepoUrl;

        if (!url) {
            marketplaceItems = [];
            lastLoadedUrl = "";
            return;
        }
        if (url === lastLoadedUrl) return;

        selectedLang = "All";

        const timer = setTimeout(() => {
            switchRepository(url);
        }, 600);

        return () => clearTimeout(timer);
    });

    async function switchRepository(url: string) {
        if (marketplaceItems.length > 0) {
            isTransitioning = true;
            await new Promise(r => setTimeout(r, 160));
            marketplaceItems = [];
            isTransitioning = false;
        }

        await loadRepository(url);
    }

    async function loadRepository(url: string) {
        if (!url) return;
        isLoadingRepo = true;
        lastLoadedUrl = url;

        try {
            const res = await fetch(url);
            if (!res.ok) throw new Error(i18n.t('errors.network'));
            const data = await res.json();
            marketplaceItems = Array.isArray(data) ? data : (data.extensions ?? []);
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.unknown'));
            marketplaceItems = [];
        } finally {
            isLoadingRepo = false;
        }
    }

    function saveRepos() {
        config.repoUrls = repoUrlsLocal.filter(url => url.trim() !== "");
        if (onSave) onSave();
    }

    function addRepo() {
        repoUrlsLocal = [...repoUrlsLocal, ""];
        activeRepoIndex = repoUrlsLocal.length - 1;
        saveRepos();
    }

    function removeRepo(index: number) {
        if (repoUrlsLocal.length <= 1) {
            repoUrlsLocal = [""];
            activeRepoIndex = 0;
            marketplaceItems = [];
            lastLoadedUrl = "";
        } else {
            repoUrlsLocal = repoUrlsLocal.filter((_, i) => i !== index);
            if (activeRepoIndex >= repoUrlsLocal.length) {
                activeRepoIndex = repoUrlsLocal.length - 1;
            }
        }
        saveRepos();
    }

    function handleRepoUrlBlur() {
        saveRepos();
    }

    function getFriendlyLabel(url: string, index: number): string {
        if (!url) return `New Unnamed Repo`;
        try {
            const urlObj = new URL(url);
            if (urlObj.hostname === 'raw.githubusercontent.com') {
                const parts = urlObj.pathname.split('/');
                if (parts.length > 2 && parts[2]) return `${parts[1]}/${parts[2]}`;
            }
            return urlObj.hostname;
        } catch {
            return `Repo ${index + 1} (${url.substring(0, 20)}...)`;
        }
    }

    function getItemId(item: AnyMarketplaceEntry): string {
        if (isTachiyomi(item)) return item.pkg;
        return item.id;
    }

    function isLNReader(item: AnyMarketplaceEntry): item is LNReaderMarketplaceEntry {
        return "url" in item && "iconUrl" in item;
    }

    function isTachiyomi(item: AnyMarketplaceEntry): item is TachiyomiMarketplaceEntry {
        return "pkg" in item && "apk" in item;
    }

    function getRepoBaseUrl(url: string): string {
        return url.replace(/\/(index(\.min)?\.json)?$/, "");
    }

    function getTachiyomiIconUrl(item: TachiyomiMarketplaceEntry): string {
        return `${getRepoBaseUrl(activeRepoUrl)}/icon/${item.pkg}.png`;
    }

    function getTachiyomiApkUrl(item: TachiyomiMarketplaceEntry): string {
        return `${getRepoBaseUrl(activeRepoUrl)}/apk/${item.apk}`;
    }

    function getExpectedId(item: AnyMarketplaceEntry): string {
        if (isLNReader(item)) return `lnr_${item.id}`;
        if (isTachiyomi(item)) return `tachi_${item.pkg}`;
        return item.id;
    }

    function isInstalled(item: AnyMarketplaceEntry): boolean {
        return extensions.installed.some(ext => ext.id === getExpectedId(item));
    }

    function getInstalledVersion(item: AnyMarketplaceEntry): string | null {
        return extensions.installed.find(ext => ext.id === getExpectedId(item))?.version ?? null;
    }

    function isNewerVersion(repoVer: string, installedVer: string): boolean {
        if (!repoVer || !installedVer) return false;
        const repo = repoVer.split('.').map(v => parseInt(v, 10) || 0);
        const inst = installedVer.split('.').map(v => parseInt(v, 10) || 0);
        const len = Math.max(repo.length, inst.length);
        for (let i = 0; i < len; i++) {
            const r = repo[i] || 0, ins = inst[i] || 0;
            if (r > ins) return true;
            if (r < ins) return false;
        }
        return false;
    }

    function hasUpdate(item: AnyMarketplaceEntry): boolean {
        const v = getInstalledVersion(item);
        return v ? isNewerVersion(item.version, v) : false;
    }

    async function handleInstall(item: AnyMarketplaceEntry) {
        installingIds = new Set(installingIds).add(getItemId(item));
        try {
            if (isLNReader(item)) {
                const res = await extensionsApi.installLNReader(item);
                if (res.ok && res.extension) extensions.installed = [...extensions.installed, res.extension];
            } else if (isTachiyomi(item)) {
                const res = await extensionsApi.installTachiyomi(getTachiyomiApkUrl(item), {
                    ...item,
                    repo_url: getRepoBaseUrl(activeRepoUrl),
                    icon_url: getTachiyomiIconUrl(item),
                });
                if (res.ok && res.extension) extensions.installed = [...extensions.installed, res.extension];
            } else {
                const manifest = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifest) { toast.error(i18n.t('marketplace.missing_manifest')); return; }
                await extensions.install(manifest);
            }
            toast.success(i18n.t('marketplace.installed'));
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.unknown'));
        } finally {
            const s = new Set(installingIds);
            s.delete(getItemId(item));
            installingIds = s;
        }
    }

    async function handleUpdate(item: AnyMarketplaceEntry) {
        installingIds = new Set(installingIds).add(getItemId(item));
        try {
            if (isLNReader(item)) {
                await extensionsApi.installLNReader(item);
            } else if (isTachiyomi(item)) {
                const baseRaw = "https://raw.githubusercontent.com/keiyoushi/extensions/repo/apk/";
                await extensionsApi.installTachiyomi(baseRaw + item.apk, item);
            } else {
                const manifestUrl = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifestUrl) return;
                await extensions.update(item.id, manifestUrl);
            }
            toast.success(i18n.t('marketplace.updated'));
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.unknown'));
        } finally {
            const s = new Set(installingIds);
            s.delete(getItemId(item));
            installingIds = s;
        }
    }
</script>

<div class="space-y-6 relative">
    <div class="flex flex-col md:flex-row gap-3 items-stretch md:items-center w-full">

        <div class="relative w-full md:flex-1 group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('marketplace.search_repository')}
                    class="pl-9 bg-background h-9 text-sm w-full"
                    bind:value={marketSearchQuery}
                    disabled={!activeRepoUrl || marketplaceItems.length === 0}
            />
        </div>

        <div class="grid grid-cols-[1fr_auto_1.5fr_auto] md:flex items-center gap-2 w-full md:w-auto shrink-0">
            <div class="h-9 flex min-w-[110px] md:w-36">
                <ResponsiveSelect
                        bind:value={selectedLang}
                        items={langItems}
                        placeholder="All Languages"
                        class="w-full h-full"
                />
            </div>

            <div class="hidden md:block w-px h-6 bg-border mx-1"></div>
            <div class="md:hidden"></div>

            <div class="relative h-9 flex shrink-0 md:w-56 min-w-[140px]">
                <Server class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/70 z-10 pointer-events-none" />
                <ResponsiveSelect
                        value={activeRepoValue}
                        items={repoItems}
                        onValueChange={onRepoChange}
                        class="pl-9 w-full h-full"
                />
                {#if isLoadingRepo}
                    <Spinner class="absolute right-9 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground z-10" />
                {/if}
            </div>

            <button
                    class="h-9 w-9 shrink-0 aspect-square rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground text-muted-foreground transition-colors flex items-center justify-center p-0"
                    onclick={() => isManagingRepos = !isManagingRepos}
                    title="Manage Repositories"
            >
                <Settings class="h-4 w-4 transition-transform duration-300 {isManagingRepos ? 'text-primary rotate-90' : ''}" />
            </button>
        </div>
    </div>

    {#if isManagingRepos}
        <div class="p-4 rounded-2xl bg-muted/20 border border-border/60 space-y-3" transition:slide={{ duration: 250 }}>
            <div class="flex items-center justify-between border-b border-border/40 pb-2 mb-1">
                <button class="text-xs font-semibold text-primary hover:underline flex items-center gap-1" onclick={addRepo}>
                    <Plus class="h-3.5 w-3.5" /> Add New
                </button>
            </div>

            <div class="space-y-2 max-h-48 overflow-y-auto pr-1">
                {#each repoUrlsLocal as url, i}
                    <div class="flex items-center gap-2">
                        <span class="text-xs font-mono text-muted-foreground w-6 text-right">#{i+1}</span>
                        <Input
                                bind:value={repoUrlsLocal[i]}
                                placeholder="https://example.com/index.json"
                                class="bg-background h-10 rounded-lg border-border/50 text-sm flex-1"
                                onblur={handleRepoUrlBlur}
                        />
                        <button
                                class="p-2 text-muted-foreground hover:text-destructive hover:bg-destructive/10 rounded-lg transition-colors"
                                onclick={() => removeRepo(i)}
                        >
                            <Trash2 class="h-4 w-4" />
                        </button>
                    </div>
                {/each}
            </div>
        </div>
    {/if}

    {#if !activeRepoUrl}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fly={{ y: 20, duration: 300, delay: 150 }} out:fade={{ duration: 150 }}>
            <div class="bg-primary/10 p-4 rounded-full mb-4 shadow-sm">
                <Server class="h-8 w-8 text-primary" />
            </div>
            <h3 class="text-lg font-bold mb-1 text-foreground">{i18n.t("marketplace.no_repo")}</h3>
        </div>

    {:else if isLoadingRepo && marketplaceItems.length === 0}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade={{ duration: 150 }}>
            <Spinner class="h-8 w-8 text-primary mb-4" />
        </div>

    {:else if marketplaceItems.length > 0}
        {#if !isTransitioning}
            <div class="space-y-10" in:fade={{ duration: 150 }}>
                {#if sections.length === 0}
                    <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5">
                        <div class="bg-muted p-4 rounded-full mb-4 border border-border/50">
                            <SearchX class="h-8 w-8 text-muted-foreground" />
                        </div>
                        <h3 class="text-lg font-bold mb-1 text-foreground">{i18n.t("errors.content.no_match_found")}</h3>
                    </div>
                {:else}
                    {#each visibleSections as section (section.lang)}
                        <div class="space-y-4">
                            <div class="flex items-center gap-3">
                                <h3 class="text-xl font-bold text-foreground tracking-tight">{section.lang}</h3>
                                <span class="text-xs font-semibold bg-muted text-muted-foreground px-2.5 py-0.5 rounded-full border border-border/50">
                                    {sections.find(s => s.lang === section.lang)?.items.length}
                                </span>
                            </div>

                            <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                                {#each section.items as item (getItemId(item))}
                                    <Card
                                            ext={
                                            isLNReader(item) ? {
                                                ...item,
                                                ext_type: 'novel',
                                                icon: item.iconUrl,
                                                language: item.lang,
                                                author: 'LNReader'
                                            } : isTachiyomi(item) ? {
                                                ...item,
                                                name: item.sources?.[0]?.name,
                                                id: item.pkg,
                                                ext_type: 'manga',
                                                icon: getTachiyomiIconUrl(item),
                                                language: item.lang,
                                                author: 'Tachiyomi'
                                            } : item
                                        }
                                            source={isLNReader(item) ? 'lnreader' : isTachiyomi(item) ? 'tachiyomi' : undefined}
                                            mode="marketplace"
                                            isMarketplaceInstalled={isInstalled(item)}
                                            hasUpdate={hasUpdate(item)}
                                            isActionLoading={installingIds.has(getItemId(item))}
                                            onAction={handleInstall}
                                            onUpdate={handleUpdate}
                                    />
                                {/each}
                            </div>
                        </div>
                    {/each}

                    {#if hasMore}
                        <div bind:this={sentinelEl} class="flex justify-center py-6">
                            <Spinner class="h-5 w-5 text-muted-foreground/50" />
                        </div>
                    {/if}
                {/if}
            </div>
        {/if}

    {:else if !isLoadingRepo && activeRepoUrl === lastLoadedUrl}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade>
            <div class="bg-muted p-4 rounded-full mb-4 border border-border/50">
                <PackageOpen class="h-8 w-8 text-muted-foreground" />
            </div>
            <h3 class="text-lg font-bold mb-1 text-foreground">{i18n.t("marketplace.empty")}</h3>
            <p class="text-sm text-muted-foreground max-w-sm">
                {i18n.t("marketplace.empty_desc")}
            </p>
        </div>
    {/if}
</div>