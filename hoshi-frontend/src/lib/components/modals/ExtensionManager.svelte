<script lang="ts">
    import { contentApi } from '@/api/content/content';
    import * as Dialog from '@/components/ui/dialog';
    import type { ExtensionSource, Metadata } from '@/api/content/types';
    import { extensions as extensionsStore } from "@/stores/extensions.svelte.js";
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import { Component, Pencil, Plus, Search, X, CheckCircle2, Globe, AlertCircle, ChevronRight } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { Spinner } from "@/components/ui/spinner";
    import type { CoreError } from "@/api/client";
    import { slide, fade } from 'svelte/transition';
    import SmartImage from "@/components/SmartImage.svelte";

    let {
        open = $bindable(false),
        cid,
        metadata,
        isNsfw = false,
        extensions,
        contentType = "anime",
        onSuccess
    }: {
        open?: boolean;
        cid: string;
        metadata: Metadata;
        isNsfw: boolean;
        extensions: ExtensionSource[];
        contentType: "anime" | "manga" | "novel";
        onSuccess?: () => void;
    } = $props();

    let isLoading = $state(false);
    let editingExtName = $state<string | null>(null);
    let searchQuery = $state("");
    let isSearching = $state(false);
    let searchResults = $state<any[]>([]);

    const installedExtensions = $derived(extensionsStore[contentType] || []);
    const availableExtensions = $derived(installedExtensions.map(ext => {
        const mapping = extensions.find(m => m.extensionName === ext.id);
        return {
            name: ext.id,
            icon: ext.icon,
            mapping: mapping || null,
            isLinked: !!mapping
        };
    }));

    function startEdit(extName: string) {
        if (editingExtName === extName) {
            cancelEdit();
            return;
        }
        editingExtName = extName;
        searchQuery = metadata?.title || "";
        searchResults = [];
    }

    function cancelEdit() {
        editingExtName = null;
        searchQuery = "";
        searchResults = [];
    }

    async function handleSearch(e?: Event) {
        if (e) e.preventDefault();
        if (!editingExtName || !searchQuery.trim()) return;
        isSearching = true;
        try {
            searchResults = await contentApi.searchExtension(editingExtName, searchQuery, {}, 1);
        } catch (err) {
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isSearching = false;
        }
    }

    async function handleUpdate(result: any) {
        if (!editingExtName) return;
        isLoading = true;
        const currentItem = availableExtensions.find(ext => ext.name === editingExtName);
        const newExtensionId = (result.id || result.url || result.extensionId).toString();
        try {
            if (currentItem?.isLinked) {
                await contentApi.updateExtensionMapping(cid, { extensionName: editingExtName, extensionId: newExtensionId });
            } else {
                await contentApi.addExtensionSource(cid, {
                    cid, extensionName: editingExtName, extensionId: newExtensionId,
                    nsfw: result.nsfw ?? isNsfw, createdAt: Date.now(), updatedAt: Date.now()
                });
            }
            open = false;
            if (onSuccess) onSuccess();
        } catch (err) {
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[580px] w-[95vw] bg-card border-border/40 max-h-[90vh] flex flex-col p-0 overflow-hidden rounded-sm">

        <div class="relative w-full shrink-0 bg-muted/30">
            {#if metadata?.bannerImage}
                <div class="absolute inset-0 overflow-hidden">
                    <SmartImage
                            src={metadata.bannerImage}
                            class="w-full h-full object-cover opacity-30 blur-[2px]"
                    />
                </div>
            {/if}
            <div class="absolute inset-0 bg-linear-to-b from-transparent via-card/60 to-card"></div>

            <div class="relative p-6 pt-10 flex items-center gap-4 sm:gap-6">
                <div class="w-16 h-24 sm:w-20 sm:h-28 rounded-sm shadow-2xl border border-white/10 overflow-hidden bg-muted shrink-0">
                    <SmartImage
                            src={metadata?.coverImage}
                            class="w-full h-full object-cover"
                    />
                </div>
                <div class="flex flex-col min-w-0 pb-1">
                    <h2 class="text-lg sm:text-xl font-black line-clamp-1 text-foreground">{metadata?.title}</h2>
                    <p class="text-xs sm:text-sm text-muted-foreground font-medium flex items-center gap-1.5 mt-0.5">
                        <Component class="w-3.5 h-3.5" />
                        {i18n.t('content.extension_manager.manage_extensions_title')}
                    </p>
                </div>
            </div>
        </div>

        <div class="flex-1 overflow-y-auto px-4 sm:px-6 py-2 space-y-3 custom-scrollbar">
            {#each availableExtensions as item (item.name)}
                <div
                        class="group flex flex-col rounded-sm border transition-all duration-300 {item.isLinked ? 'bg-muted/30 border-border/40' : 'bg-primary/5 border-primary/20'} {editingExtName === item.name ? 'ring-2 ring-primary/40 border-primary/50 shadow-lg' : ''}"
                >
                    <div class="flex items-center justify-between p-3 sm:p-4 gap-3">
                        <div class="flex items-center gap-3 sm:gap-4 min-w-0">
                            <div class="w-10 h-10 rounded-sm bg-background border border-border/50 flex items-center justify-center shadow-xs shrink-0">
                                {#if item.icon}
                                    <img src={item.icon} class="w-6 h-6 object-contain" alt="" />
                                {:else}
                                    <Globe class="w-5 h-5 text-muted-foreground" />
                                {/if}
                            </div>
                            <div class="flex flex-col min-w-0">
                                <span class="font-bold text-sm uppercase tracking-tight truncate">{item.name}</span>
                                {#if item.isLinked}
                                    <div class="flex items-center gap-1 text-xs text-green-500 font-medium">
                                        <CheckCircle2 class="w-3 h-3 shrink-0" />
                                        <span class="truncate opacity-80 font-mono text-[10px]">{item.mapping?.extensionId}</span>
                                    </div>
                                {:else}
                                    <span class="text-[10px] text-destructive font-black uppercase tracking-tighter italic flex items-center gap-1">
                                        <AlertCircle class="w-3 h-3" /> {i18n.t('content.extension_manager.not_linked')}
                                    </span>
                                {/if}
                            </div>
                        </div>

                        <Button
                                variant={editingExtName === item.name ? "secondary" : (item.isLinked ? "ghost" : "default")}
                                size="sm"
                                class="rounded-sm font-bold h-9 px-3 sm:px-4 shrink-0 transition-transform active:scale-95"
                                onclick={() => startEdit(item.name)}
                                disabled={isLoading}
                        >
                            {#if editingExtName === item.name}
                                <X class="h-4 w-4" />
                            {:else if item.isLinked}
                                <Pencil class="h-3.5 w-3.5 sm:mr-2" />
                                <span class="hidden sm:inline text-xs">{i18n.t('content.extension_manager.edit')}</span>
                            {:else}
                                <Plus class="h-4 w-4 sm:mr-2" />
                                <span class="hidden sm:inline text-xs">{i18n.t('content.extension_manager.link')}</span>
                            {/if}
                        </Button>
                    </div>

                    {#if editingExtName === item.name}
                        <div transition:slide={{ duration: 250 }} class="px-3 pb-4 sm:px-4 overflow-hidden">
                            <div class="pt-4 border-t border-border/40 flex flex-col gap-3">
                                <form onsubmit={handleSearch} class="relative">
                                    <Input
                                            class="h-10 pl-9 pr-10 rounded-sm bg-background border-border/60 text-sm"
                                            placeholder={i18n.t('content.extension_manager.search_title_placeholder')}
                                            bind:value={searchQuery}
                                            disabled={isLoading || isSearching}
                                    />
                                    <Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground/50" />
                                    {#if isSearching}
                                        <div class="absolute right-3 top-3"><Spinner class="h-4 w-4" /></div>
                                    {/if}
                                </form>

                                {#if searchResults.length > 0}
                                    <div class="space-y-2 max-h-[300px] overflow-y-auto pr-1 pb-2" in:fade>
                                        {#each searchResults as result}
                                            <button
                                                    class="w-full flex items-center justify-between bg-background/50 p-2 rounded-sm border border-border/40 hover:border-primary/40 hover:bg-primary/5 transition-all text-left"
                                                    onclick={() => handleUpdate(result)}
                                                    disabled={isLoading}
                                            >
                                                <div class="flex items-center gap-3 min-w-0">
                                                    <div class="w-9 h-12 rounded-sm overflow-hidden bg-muted shrink-0 border border-border/10">
                                                        {#if result.image}
                                                            <SmartImage
                                                                    src={result.image}
                                                                    class="w-full h-full object-cover"
                                                            />
                                                        {/if}
                                                    </div>
                                                    <div class="flex flex-col min-w-0">
                                                        <span class="text-xs font-bold truncate pr-2">{result.title}</span>
                                                        <span class="text-[9px] text-muted-foreground font-mono truncate">{result.id || result.url}</span>
                                                    </div>
                                                </div>
                                                <ChevronRight class="w-4 h-4 text-muted-foreground/30 shrink-0" />
                                            </button>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>

        <div class="p-4 border-t border-border/40 bg-card/80 backdrop-blur-md flex justify-end gap-3">
            <Button variant="ghost" class="rounded-sm font-bold text-xs" onclick={() => open = false}>
                {i18n.t('content.close')}
            </Button>
        </div>
    </Dialog.Content>
</Dialog.Root>