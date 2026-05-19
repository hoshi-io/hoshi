<script lang="ts">
    import { extensions } from "@/stores/extensions.svelte.js";
    import type {Extension, TachiyomiMarketplaceEntry} from "@/api/extensions/types";
    import type { AnyMarketplaceEntry, LNReaderMarketplaceEntry, NativeMarketplaceEntry } from "@/api/extensions/types";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Input } from "$lib/components/ui/input";
    import { Search, Link as LinkIcon, PackageOpen, SearchX, Server } from "lucide-svelte";
    import { Spinner } from "$lib/components/ui/spinner";
    import type { ExtensionsConfig } from "@/api/config/types";
    import Card from "./Card.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let installingIds = $state<Set<string>>(new Set());
    let marketSearchQuery = $state("");
    let repoUrlLocal = $state(config.repoUrl || "");
    let lastLoadedUrl = $state("");
    let marketplaceItems = $state<AnyMarketplaceEntry[]>([]);
    let isLoadingRepo = $state(false);
    let debounceTimer: ReturnType<typeof setTimeout>;

    $effect(() => {
        if (repoUrlLocal && repoUrlLocal !== lastLoadedUrl) {
            clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                loadRepository();
            }, 800);
        }
    });

    let filteredMarketplace = $derived(
        marketplaceItems.filter(item =>
            item.name.toLowerCase().includes(marketSearchQuery.toLowerCase())
        )
    );

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
        const base = getRepoBaseUrl(repoUrlLocal);
        return `${base}/icon/${item.pkg}.png`;
    }

    function getTachiyomiApkUrl(item: TachiyomiMarketplaceEntry): string {
        const base = getRepoBaseUrl(repoUrlLocal);
        return `${base}/apk/${item.apk}`;
    }

    async function handleInstall(item: AnyMarketplaceEntry) {
        installingIds = new Set(installingIds).add(getItemId(item));
        try {
            if (isLNReader(item)) {
                const res = await extensionsApi.installLNReader(item);
                if (res.ok && res.extension) {
                    extensions.installed = [...extensions.installed, res.extension];
                }
            } else if (isTachiyomi(item)) {
                const downloadUrl = getTachiyomiApkUrl(item);
                const res = await extensionsApi.installTachiyomi(downloadUrl, {
                    ...item,
                    repo_url: getRepoBaseUrl(repoUrlLocal),
                    icon_url: getTachiyomiIconUrl(item),
                });
                if (res.ok && res.extension) {
                    extensions.installed = [...extensions.installed, res.extension];
                }
            } else {
                const manifest = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifest) { toast.error(i18n.t('marketplace.missing_manifest')); return; }
                await extensions.install(manifest);
            }
            toast.success(i18n.t('marketplace.installed'));
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.unknown'));
        } finally {
            const newSet = new Set(installingIds);
            newSet.delete(getItemId(item));
            installingIds = newSet;
        }
    }

    async function loadRepository() {
        if (!repoUrlLocal) return;
        isLoadingRepo = true;
        lastLoadedUrl = repoUrlLocal;

        if (config.repoUrl !== repoUrlLocal) {
            config.repoUrl = repoUrlLocal;
            if (onSave) onSave();
        }

        try {
            const res = await fetch(repoUrlLocal);
            if (!res.ok) throw new Error(i18n.t('errors.network'));

            const data = await res.json();
            // Handle both native { extensions: [] } and lnreader [] shapes
            marketplaceItems = Array.isArray(data) ? data : (data.extensions ?? []);
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ?
                error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
            marketplaceItems = [];
        } finally {
            isLoadingRepo = false;
        }
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

    function hasUpdate(item: AnyMarketplaceEntry): boolean {
        const installedVersion = getInstalledVersion(item);
        if (!installedVersion) return false;
        return isNewerVersion(item.version, installedVersion);
    }

    function isNewerVersion(repoVer: string, installedVer: string): boolean {
        if (!repoVer || !installedVer) return false;

        const repo = repoVer.split('.').map(v => parseInt(v, 10) || 0);
        const inst = installedVer.split('.').map(v => parseInt(v, 10) || 0);
        const len = Math.max(repo.length, inst.length);

        for (let i = 0; i < len; i++) {
            const r = repo[i] || 0;
            const ins = inst[i] || 0;
            if (r > ins) return true;
            if (r < ins) return false;
        }
        return false;
    }

    async function handleUpdate(item: AnyMarketplaceEntry) {
        installingIds = new Set(installingIds).add(getItemId(item));
        try {
            if (isLNReader(item)) {
                await extensionsApi.installLNReader(item);
            } else if (isTachiyomi(item)) {
                const baseRaw = "https://raw.githubusercontent.com/keiyoushi/extensions/repo/apk/";
                const downloadUrl = baseRaw + item.apk;
                await extensionsApi.installTachiyomi(downloadUrl, item);
            } else {
                const manifestUrl = (item as NativeMarketplaceEntry).manifestUrl;
                if (!manifestUrl) return;
                await extensions.update(item.id, manifestUrl);
            }
            toast.success(i18n.t('marketplace.updated'));
        } catch (error: any) {
            toast.error(error?.message || i18n.t('errors.unknown'));
        } finally {
            const newSet = new Set(installingIds);
            newSet.delete(getItemId(item));
            installingIds = newSet;
        }
    }
</script>

<div class="space-y-6 relative">
    <div class="flex flex-col md:flex-row gap-4 items-center bg-muted/5 p-4 rounded-2xl border border-border/40">
        <div class="relative flex-1 w-full">
            <LinkIcon class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground/70" />
            <Input bind:value={repoUrlLocal} placeholder={i18n.t('marketplace.repo_url_placeholder')} class="pl-9 bg-background h-10 rounded-xl w-full border-border/60" />

            {#if isLoadingRepo}
                <Spinner class="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 animate-spin text-muted-foreground" />
            {/if}
        </div>

        <div class="relative w-full md:w-64 shrink-0 group">
            <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
            <Input
                    placeholder={i18n.t('marketplace.search_repository')}
                    class="pl-9 bg-background border-border/60 h-10 rounded-xl focus-visible:ring-1 focus-visible:ring-primary/40 text-sm"
                    bind:value={marketSearchQuery}
                    disabled={!repoUrlLocal || marketplaceItems.length === 0}
            />
        </div>
    </div>

    {#if !repoUrlLocal}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade>
            <div class="bg-primary/10 p-4 rounded-full mb-4 shadow-sm">
                <Server class="h-8 w-8 text-primary" />
            </div>
            <h3 class="text-lg font-bold mb-1 text-foreground">{i18n.t("marketplace.no_repo")}</h3>
            <p class="text-sm text-muted-foreground max-w-sm">
                {i18n.t("marketplace.no_repo_desc")}
            </p>
        </div>

    {:else if isLoadingRepo && marketplaceItems.length === 0}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade>
            <Spinner class="h-8 w-8 text-primary mb-4" />
        </div>

    {:else if marketplaceItems.length > 0}
        {#if filteredMarketplace.length > 0}
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-3" in:fade>
                {#each filteredMarketplace as item (getItemId(item))}
                    <Card
                            ext={
    isLNReader(item)
        ? {
            ...item,
            ext_type: 'novel',
            icon: item.iconUrl,
            language: item.lang,
            author: 'LNReader'
        }
        : isTachiyomi(item)
            ? {
                ...item,
                name: item.sources?.[0]?.name,
                id: item.pkg,
                ext_type: 'manga',
                icon: getTachiyomiIconUrl(item),
                language: item.lang,
                author: 'Tachiyomi'
            }
            : item
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
        {:else}
            <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade>
                <div class="bg-muted p-4 rounded-full mb-4 border border-border/50">
                    <SearchX class="h-8 w-8 text-muted-foreground" />
                </div>
                <h3 class="text-lg font-bold mb-1 text-foreground">No matches found</h3>
                <p class="text-sm text-muted-foreground max-w-sm">
                    We couldn't find any extensions matching "<span class="font-semibold text-foreground">{marketSearchQuery}</span>".
                </p>
            </div>
        {/if}

    {:else if !isLoadingRepo && repoUrlLocal === lastLoadedUrl}
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