<script lang="ts">
    import { contentApi } from '@/api/content/content';
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import { Spinner } from "@/components/ui/spinner";
    import * as Dialog from '@/components/ui/dialog';
    import { GitMerge, Search, AlertTriangle, X, Check } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { CoreError } from "@/api/client";
    import { fade } from 'svelte/transition';
    import type {SearchResult} from "@/api/content/types";
    import {goto} from "$app/navigation";
    import {listStore} from "@/app/list.svelte";

    let {
        open = $bindable(false),
        cid,
        displayTitle,
        coverImage,
        contentType,
    }: {
        open: boolean;
        cid: string;
        displayTitle?: string;
        coverImage?: string;
        contentType: string;
    } = $props();

    let query = $state("");
    let isSearching = $state(false);
    let isMerging = $state(false);
    let results = $state<SearchResult[]>([]);
    let selected = $state<SearchResult | null>(null);
    let hasSearched = $state(false);

    let debounceHandle: ReturnType<typeof setTimeout> | undefined;

    function onQueryInput() {
        selected = null;
        clearTimeout(debounceHandle);
        if (query.trim().length < 2) {
            results = [];
            hasSearched = false;
            return;
        }
        debounceHandle = setTimeout(runSearch, 350);
    }

    async function runSearch() {
        isSearching = true;
        hasSearched = true;
        try {
            const res = await contentApi.searchLibrary(query.trim(), contentType);
            results = res.filter(r => r.cid !== cid);
            console.log(results)
        } catch (err) {
            console.log(err)
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isSearching = false;
        }
    }

    function reset() {
        query = "";
        results = [];
        selected = null;
        hasSearched = false;
        isMerging = false;
    }

    async function handleConfirmMerge() {
        if (!selected) return;
        isMerging = true;
        try {
            await contentApi.mergeContent(cid, selected.cid);
            toast.success(i18n.t('content.merge_success'));

            await listStore.refresh();

            open = false;
            isMerging = false;

            await goto(`/c/${cid}`, { invalidateAll: true });
        } catch (err) {
            console.log(err);
            toast.error(i18n.t((err as CoreError).key));
            isMerging = false;
        }
    }

    $effect(() => {
        if (open) {
            query = displayTitle ?? "";
            if (query.trim().length >= 2) {
                runSearch();
            }
        } else {
            reset();
        }
    });
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[600px] w-[95vw] bg-card border-border/40 max-h-[85vh] flex flex-col p-0 overflow-hidden rounded-sm shadow-2xl">

        <div class="relative w-full shrink-0 bg-muted/20">
            <div class="relative p-6 pt-8 flex items-center gap-4">
                <div class="w-12 h-12 rounded-xl bg-primary/10 border border-primary/20 flex items-center justify-center shrink-0">
                    <GitMerge class="w-6 h-6 text-primary" />
                </div>
                <div class="flex flex-col min-w-0">
                    <Dialog.Title class="text-lg font-black line-clamp-1">
                        {i18n.t('content.merge_entries')}
                    </Dialog.Title>
                    <p class="text-sm text-muted-foreground font-medium mt-0.5">
                        {i18n.t('content.merge_entries_desc')}
                    </p>
                </div>
            </div>
        </div>

        <div class="flex-1 overflow-y-auto px-6 py-2 space-y-4 custom-scrollbar">

            <div class="flex items-center gap-3 p-3 rounded-2xl border border-border/40 bg-muted/10">
                <div class="w-10 h-14 rounded-sm overflow-hidden bg-muted shrink-0">
                    {#if coverImage}
                        <img src={coverImage} class="w-full h-full object-cover" alt="" />
                    {/if}
                </div>
                <div class="min-w-0">
                    <span class="text-[10px] font-black uppercase tracking-widest text-muted-foreground/70">
                        {i18n.t('content.keeping_this_entry')}
                    </span>
                    <p class="text-sm font-bold truncate">{displayTitle}</p>
                </div>
            </div>

            <div class="relative">
                <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground/60" />
                <Input
                        bind:value={query}
                        oninput={onQueryInput}
                        placeholder={i18n.t('content.search_duplicate_placeholder')}
                        class="h-10 pl-9 text-sm rounded-sm bg-background border-border/60"
                />
            </div>

            <div class="space-y-2 min-h-[80px]">
                {#if isSearching}
                    <div class="flex justify-center py-6">
                        <Spinner class="h-5 w-5" />
                    </div>
                {:else if results.length > 0}
                    {#each results as r (r.cid)}
                        <button
                                onclick={() => selected = r}
                                transition:fade
                                class="w-full flex items-center gap-3 p-2.5 rounded-xl border transition-all text-left
                                {selected?.cid === r.cid
                                    ? 'border-primary/50 bg-primary/5'
                                    : 'border-border/40 bg-muted/5 hover:bg-muted/15'}"
                        >
                            <div class="w-8 h-11 rounded-sm overflow-hidden bg-muted shrink-0">
                                {#if r.cover_image}
                                    <img src={r.cover_image} class="w-full h-full object-cover" alt="" />
                                {/if}
                            </div>
                            <div class="flex-1 min-w-0">
                                <p class="text-sm font-bold truncate">{r.title}</p>
                                <p class="text-[10px] text-muted-foreground truncate font-mono opacity-70">{r.cid}</p>
                            </div>
                            {#if selected?.cid === r.cid}
                                <Check class="w-4 h-4 text-primary shrink-0" />
                            {/if}
                        </button>
                    {/each}
                {:else if hasSearched}
                    <div class="flex flex-col items-center justify-center py-8 opacity-40">
                        <p class="text-sm font-medium">{i18n.t('content.no_results')}</p>
                    </div>
                {/if}
            </div>

            {#if selected}
                <div class="flex items-start gap-2 p-3 rounded-xl border border-amber-500/20 bg-amber-500/5 text-amber-500/90" transition:fade>
                    <AlertTriangle class="w-4 h-4 shrink-0 mt-0.5" />
                    <p class="text-xs font-medium leading-relaxed">
                        {i18n.t('content.merge_warning', { title: selected.title })}
                    </p>
                </div>
            {/if}
        </div>

        <div class="p-4 border-t border-border/40 bg-card/50 flex justify-end gap-2">
            <Button variant="ghost" class="rounded-sm font-bold text-xs px-6" onclick={() => open = false}>
                {i18n.t('content.cancel')}
            </Button>
            <Button
                    variant="destructive"
                    class="rounded-sm font-bold text-xs px-6 gap-2"
                    disabled={!selected || isMerging}
                    onclick={handleConfirmMerge}
            >
                {#if isMerging}
                    <Spinner class="h-4 w-4" />
                {:else}
                    <GitMerge class="h-4 w-4" />
                {/if}
                {i18n.t('content.confirm_merge')}
            </Button>
        </div>
    </Dialog.Content>
</Dialog.Root>