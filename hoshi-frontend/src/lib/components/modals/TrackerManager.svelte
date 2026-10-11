<script lang="ts">
    import { contentApi } from '@/api/content/content';
    import type { TrackerMapping, Metadata } from '@/api/content/types';
    import * as Dialog from '@/components/ui/dialog';
    import { Button } from '@/components/ui/button';
    import { Input } from '@/components/ui/input';
    import * as Select from '@/components/ui/select';
    import { Spinner } from "@/components/ui/spinner";
    import { Trash2, Plus, Pencil, Save, Link2, X, AlertCircle } from 'lucide-svelte';
    import { toast } from "svelte-sonner";
    import { i18n } from "@/stores/i18n.svelte.js";
    import type { CoreError } from "@/api/client";
    import { fade } from 'svelte/transition';

    let {
        open = $bindable(false),
        cid,
        trackers,
        metadata
    }: {
        open: boolean;
        cid: string;
        trackers: TrackerMapping[];
        metadata?: Metadata;
    } = $props();

    let isLoading = $state(false);
    let isEditing = $state(false);
    let formName = $state("");
    let formId = $state("");

    const availableTrackers = [
        { value: 'anilist', label: 'AniList', domain: 'anilist.co' },
        { value: 'anidb', label: 'anidb', domain: 'anidb.net' },
        { value: 'mal', label: 'MyAnimeList', domain: 'myanimelist.net' },
        { value: 'simkl', label: 'Simkl', domain: 'simkl.com' },
        { value: 'kitsu', label: 'Kitsu', domain: 'kitsu.io' },
        { value: 'trakt', label: 'Trakt', domain: 'trakt.tv' },
        { value: 'animeplanet', label: 'Anime-Planet', domain: 'anime-planet.com' },
        { value: 'imdb', label: 'IMDb', domain: 'imdb.com' },
        { value: 'shikimori', label: 'Shikimori', domain: 'shikimori.one' },
    ];

    function getFavicon(trackerName: string) {
        const t = availableTrackers.find(x => x.value === trackerName.toLowerCase());
        const domain = t ? t.domain : `${trackerName}.com`;
        return `https://www.google.com/s2/favicons?domain=${domain}&sz=64`;
    }

    function startEdit(tracker: TrackerMapping) {
        isEditing = true;
        formName = tracker.trackerName;
        formId = tracker.trackerId;
    }

    function cancelEdit() {
        isEditing = false;
        formName = "";
        formId = "";
    }

    async function handleSubmit() {
        if (!formName || !formId) return;
        isLoading = true;
        try {
            if (isEditing) {
                await contentApi.updateTrackerMapping(cid, { trackerName: formName, trackerId: formId });
                toast.success(i18n.t('content.mapping_updated'));
            } else {
                await contentApi.addTrackerMapping(cid, {
                    cid, trackerName: formName, trackerId: formId,
                    createdAt: Date.now(), updatedAt: Date.now()
                });
                toast.success(i18n.t('content.mapping_added'));
            }
            window.location.reload();
        } catch (err) {
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isLoading = false;
        }
    }

    async function handleDelete(trackerName: string) {
        if (!confirm(i18n.t('content.confirm_delete_mapping'))) return;
        isLoading = true;
        try {
            await contentApi.deleteTrackerMapping(cid, trackerName);
            toast.success(i18n.t('content.mapping_deleted'));
            window.location.reload();
        } catch (err) {
            toast.error(i18n.t((err as CoreError).key));
        } finally {
            isLoading = false;
        }
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content class="sm:max-w-[650px] w-[95vw] bg-card border-border/40 max-h-[90vh] flex flex-col p-0 overflow-hidden rounded-sm shadow-2xl">

        <div class="relative w-full shrink-0 bg-muted/20">
            {#if metadata?.bannerImage}
                <div class="absolute inset-0 overflow-hidden">
                    <img src={metadata.bannerImage} class="w-full h-full object-cover opacity-25 blur-md" alt="" />
                </div>
            {/if}
            <div class="absolute inset-0 bg-gradient-to-b from-transparent to-card"></div>

            <div class="relative p-6 pt-10 flex items-center gap-6">
                <div class="w-16 h-24 rounded-sm shadow-2xl border border-white/10 overflow-hidden bg-muted shrink-0">
                    <img src={metadata?.coverImage} class="w-full h-full object-cover" alt="" />
                </div>
                <div class="flex flex-col min-w-0">
                    <Dialog.Title class="text-xl font-black line-clamp-1">{metadata?.title || i18n.t('content.manage_trackers')}</Dialog.Title>
                    <p class="text-sm text-muted-foreground font-medium flex items-center gap-1.5 mt-1">
                        <Link2 class="w-4 h-4 text-primary" />
                        {i18n.t('content.manage_trackers_desc')}
                    </p>
                </div>
            </div>
        </div>

        <div class="flex-1 overflow-y-auto px-6 py-2 space-y-4 custom-scrollbar">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
                {#each trackers as tracker (tracker.trackerName)}
                    <div
                            class="flex items-center justify-between p-3 rounded-2xl border border-border/40 bg-muted/10 hover:bg-muted/20 transition-all group"
                            transition:fade
                    >
                        <div class="flex items-center gap-3 min-w-0">
                            <div class="w-10 h-10 rounded-xl bg-background border border-border/50 flex items-center justify-center shadow-sm shrink-0">
                                <img src={getFavicon(tracker.trackerName)} alt="" class="w-6 h-6 rounded-sm" />
                            </div>
                            <div class="flex flex-col min-w-0">
                                <span class="font-bold text-sm capitalize truncate">{tracker.trackerName}</span>
                                <span class="text-[10px] text-muted-foreground font-mono truncate opacity-70">{tracker.trackerId}</span>
                            </div>
                        </div>

                        <div class="flex items-center gap-1 shrink-0 ml-2">
                            <Button variant="ghost" size="icon" class="h-8 w-8 rounded-sm" onclick={() => startEdit(tracker)}>
                                <Pencil class="h-4 w-4" />
                            </Button>
                            <Button variant="ghost" size="icon" class="h-8 w-8 rounded-sm text-destructive/60 hover:text-destructive hover:bg-destructive/10" onclick={() => handleDelete(tracker.trackerName)}>
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </div>
                    </div>
                {:else}
                    <div class="col-span-full flex flex-col items-center justify-center py-12 opacity-40">
                        <AlertCircle class="w-12 h-12 mb-3 stroke-1" />
                        <p class="text-sm font-medium">{i18n.t('content.no_trackers')}</p>
                    </div>
                {/each}
            </div>

            <div class="mt-6 pt-6 border-t border-border/20">
                <div class="flex items-center justify-between mb-4">
                    <span class="text-[10px] font-black uppercase tracking-widest text-primary/80 px-2 py-1 bg-primary/5 rounded-sm">
                        {isEditing ? i18n.t('content.edit_tracker') : i18n.t('content.add_tracker')}
                    </span>
                    {#if isEditing}
                        <button class="text-[10px] font-bold uppercase text-muted-foreground hover:text-foreground flex items-center gap-1" onclick={cancelEdit}>
                            <X class="w-3 h-3" /> {i18n.t('content.cancel')}
                        </button>
                    {/if}
                </div>

                <div class="grid grid-cols-1 sm:grid-cols-[1.2fr_1.2fr_auto] gap-4 items-end bg-muted/5 p-5 rounded-2xl border border-border/30">
                    <div class="space-y-2">
                        <label class="text-[10px] font-black text-muted-foreground uppercase ml-1" for="prov">{i18n.t('content.provider')}</label>
                        {#if isEditing}
                            <div class="h-10 px-4 flex items-center bg-background/50 border border-border/40 rounded-xl text-sm font-bold text-muted-foreground">
                                {formName}
                            </div>
                        {:else}
                            <Select.Root type="single" bind:value={formName}>
                                <Select.Trigger id="prov" class="h-10 text-sm rounded-sm bg-background border-border/60">
                                    <span class="truncate font-semibold">{availableTrackers.find(t => t.value === formName)?.label || i18n.t('content.select')}</span>
                                </Select.Trigger>
                                <Select.Content class="rounded-sm shadow-xl">
                                    {#each availableTrackers as t}
                                        <Select.Item value={t.value} class="text-sm font-medium">{t.label}</Select.Item>
                                    {/each}
                                </Select.Content>
                            </Select.Root>
                        {/if}
                    </div>

                    <div class="space-y-2">
                        <label class="text-[10px] font-black text-muted-foreground uppercase ml-1" for="idsl">{i18n.t('content.id_slug')}</label>
                        <Input id="idsl" class="h-10 text-sm rounded-sm bg-background border-border/60" placeholder="ID / Slug" bind:value={formId} disabled={isLoading} />
                    </div>

                    <Button
                            size="icon"
                            variant={isEditing ? "default" : "secondary"}
                            class="h-10 w-10 shrink-0 rounded-sm shadow-md transition-transform active:scale-90"
                            disabled={!formName || !formId || isLoading}
                            onclick={handleSubmit}
                    >
                        {#if isLoading}
                            <Spinner class="h-4 w-4" />
                        {:else if isEditing}
                            <Save class="h-4 w-4" />
                        {:else}
                            <Plus class="h-4 w-4" />
                        {/if}
                    </Button>
                </div>
            </div>
        </div>

        <div class="p-4 border-t border-border/40 bg-card/50 flex justify-end">
            <Button variant="ghost" class="rounded-sm font-bold text-xs px-6" onclick={() => open = false}>
                {i18n.t('content.close')}
            </Button>
        </div>
    </Dialog.Content>
</Dialog.Root>