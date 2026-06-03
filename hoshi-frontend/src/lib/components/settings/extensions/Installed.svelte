<script lang="ts">
    import { Spinner } from "@/components/ui/spinner";
    import { Input } from "$lib/components/ui/input";
    import { toast } from "svelte-sonner";
    import { fade } from "svelte/transition";
    import { Search, SearchX, PackageOpen } from "lucide-svelte";
    import type { ExtensionsConfig } from "@/api/config/types";
    import { extensionsApi } from "@/api/extensions/extensions";
    import { extensions } from "@/stores/extensions.svelte.js";
    import { i18n } from "@/stores/i18n.svelte.js";
    import Card from "./Card.svelte";
    import ResponsiveSelect from "@/components/ResponsiveSelect.svelte";

    let {
        config = $bindable(),
        onSave
    }: {
        config: ExtensionsConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let uninstallingIds = $state<Set<string>>(new Set());
    let savingIds = $state<Set<string>>(new Set());
    let searchQuery = $state("");
    let selectedSource = $state("all");

    const sourceItems = [
        { value: "all",       label: "All Sources"  },
        { value: "native",    label: "Hoshi"       },
        { value: "lnreader",  label: "LNReader"     },
        { value: "tachiyomi", label: "Tachiyomi"    },
    ];

    function getSource(ext: { id: string }): "lnreader" | "tachiyomi" | "native" {
        if (ext.id.startsWith("lnr_"))   return "lnreader";
        if (ext.id.startsWith("tachi_")) return "tachiyomi";
        return "native";
    }

    function normalizeType(type: string | undefined): string {
        if (!type) return "Other";
        return type.charAt(0).toUpperCase() + type.slice(1).toLowerCase();
    }

    type Section = { type: string; items: typeof extensions.installed };

    let sections = $derived((() => {
        const query = searchQuery.toLowerCase().trim();
        const groups: Record<string, typeof extensions.installed> = {};

        const sorted = [...extensions.installed].sort((a, b) =>
            (a.name ?? "").localeCompare(b.name ?? "")
        );

        for (const ext of sorted) {
            const src = getSource(ext);
            if (selectedSource !== "all" && src !== selectedSource) continue;
            if (query && !(ext.name ?? "").toLowerCase().includes(query)) continue;

            const type = normalizeType(ext.ext_type);
            (groups[type] ??= []).push(ext);
        }

        return Object.entries(groups)
            .sort(([a], [b]) => a.localeCompare(b))
            .map(([type, items]) => ({ type, items })) as Section[];
    })());

    let totalVisible = $derived(sections.reduce((n, s) => n + s.items.length, 0));

    async function handleUninstall(id: string) {
        uninstallingIds = new Set(uninstallingIds).add(id);
        try {
            await extensions.uninstall(id);
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
        } finally {
            const newSet = new Set(uninstallingIds);
            newSet.delete(id);
            uninstallingIds = newSet;
        }
    }

    async function handleSaveSettings(id: string, newSettings: Record<string, any>): Promise<boolean> {
        savingIds = new Set(savingIds).add(id);
        try {
            const res = await extensionsApi.updateSettings(id, newSettings);
            if (res.ok) {
                const index = extensions.installed.findIndex(e => e.id === id);
                if (index !== -1) {
                    extensions.installed[index].settings = { ...newSettings };
                }
                return true;
            }
            return false;
        } catch (error: any) {
            const errorMessage = typeof error === 'string' ? error : error?.message || i18n.t('errors.unknown');
            toast.error(errorMessage);
            return false;
        } finally {
            const newSet = new Set(savingIds);
            newSet.delete(id);
            savingIds = newSet;
        }
    }
</script>

<div class="space-y-6">
    {#if extensions.loading && extensions.installed.length === 0}
        <div class="flex justify-center py-16">
            <Spinner class="h-8 w-8 animate-spin text-muted-foreground" />
        </div>

    {:else if extensions.installed.length === 0}
        <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5">
            <div class="bg-muted p-4 rounded-full mb-4 border border-border/50">
                <PackageOpen class="h-8 w-8 text-muted-foreground" />
            </div>
            <p class="text-lg font-bold text-foreground mb-1">{i18n.t('settings.extension_section.no_installed')}</p>
            <p class="text-sm text-muted-foreground">{i18n.t('settings.extension_section.no_installed_desc')}</p>
        </div>

    {:else}
        <!-- Toolbar -->
        <div class="flex flex-col md:flex-row gap-3 items-stretch md:items-center w-full">
            <div class="relative w-full md:flex-1 group">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground group-focus-within:text-primary transition-colors" />
                <Input
                        placeholder={i18n.t('marketplace.search_repository')}
                        class="pl-9 bg-background h-9 text-sm w-full"
                        bind:value={searchQuery}
                />
            </div>

            <div class="h-9 flex shrink-0 md:w-40">
                <ResponsiveSelect
                        bind:value={selectedSource}
                        items={sourceItems}
                        placeholder="All Sources"
                        class="w-full h-full"
                />
            </div>
        </div>

        <!-- Sections -->
        {#if totalVisible === 0}
            <div class="flex flex-col items-center justify-center py-16 px-4 text-center border-2 border-dashed border-border/40 rounded-2xl bg-muted/5" in:fade={{ duration: 150 }}>
                <div class="bg-muted p-4 rounded-full mb-4 border border-border/50">
                    <SearchX class="h-8 w-8 text-muted-foreground" />
                </div>
                <h3 class="text-lg font-bold mb-1 text-foreground">{i18n.t('errors.content.no_match_found')}</h3>
            </div>
        {:else}
            <div class="space-y-10" in:fade={{ duration: 150 }}>
                {#each sections as section (section.type)}
                    <div class="space-y-4">
                        <div class="flex items-center gap-3">
                            <h3 class="text-xl font-bold text-foreground tracking-tight">{section.type}</h3>
                            <span class="text-xs font-semibold bg-muted text-muted-foreground px-2.5 py-0.5 rounded-full border border-border/50">
                                {section.items.length}
                            </span>
                        </div>

                        <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                            {#each section.items as ext (ext.id)}
                                <Card
                                        {ext}
                                        source={getSource(ext) === 'native' ? undefined : getSource(ext)}
                                        isSaving={savingIds.has(ext.id)}
                                        isActionLoading={uninstallingIds.has(ext.id)}
                                        onAction={handleUninstall}
                                        onSave={handleSaveSettings}
                                />
                            {/each}
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    {/if}
</div>