<script lang="ts">
    import * as Tabs from "$lib/components/ui/tabs";
    import { i18n } from "@/stores/i18n.svelte.js";
    import { fly } from "svelte/transition";
    import { cubicOut } from "svelte/easing";
    import SettingsSection from "@/components/settings/parts/SettingsSection.svelte";
    import MangaReader from "./MangaReader.svelte";
    import NovelReader from "./NovelReader.svelte";
    import type { MangaConfig, NovelConfig } from "@/api/config/types";
    import { BookOpen, BookOpenText } from "lucide-svelte";

    let {
        mangaConfig = $bindable(),
        novelConfig = $bindable(),
        onSave
    }: {
        mangaConfig: MangaConfig,
        novelConfig: NovelConfig,
        onSave: () => Promise<void> | void
    } = $props();

    let activeTab = $state("manga_sub");

    const tabs = [
        { id: "manga_sub", label: () => i18n.t("settings.manga"), icon: BookOpen },
        { id: "novel_sub", label: () => i18n.t("settings.novel"), icon: BookOpenText }
    ];
</script>

<SettingsSection variant="bare" title={i18n.t('settings.readers')} description={i18n.t('settings.readers_desc')}>
    <Tabs.Root bind:value={activeTab} class="w-full">
        <div class="pt-1 pb-4">
            <Tabs.List class="flex w-full sm:inline-flex sm:w-auto h-auto gap-1 p-1 rounded-lg bg-muted/50">
                {#each tabs as tab (tab.id)}
                    {@const Icon = tab.icon}
                    <Tabs.Trigger
                            value={tab.id}
                            class="flex-1 sm:flex-none h-9 min-w-0 gap-2 px-5 rounded-md text-sm font-semibold text-muted-foreground
                                   transition-all duration-200 hover:text-foreground
                                   data-[state=active]:bg-background data-[state=active]:text-foreground
                                   data-[state=active]:shadow-sm data-[state=active]:ring-1 data-[state=active]:ring-border/60
                                   focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary/40"
                    >
                        <Icon class="size-4 shrink-0" />
                        <span class="truncate">{tab.label()}</span>
                    </Tabs.Trigger>
                {/each}
            </Tabs.List>
        </div>

        <Tabs.Content value="manga_sub" class="focus-visible:outline-none mt-0">
            {#if activeTab === "manga_sub"}
                <div in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
                    <MangaReader bind:config={mangaConfig} {onSave} />
                </div>
            {/if}
        </Tabs.Content>

        <Tabs.Content value="novel_sub" class="focus-visible:outline-none mt-0">
            {#if activeTab === "novel_sub"}
                <div in:fly={{ y: 8, duration: 220, easing: cubicOut }}>
                    <NovelReader bind:config={novelConfig} {onSave} />
                </div>
            {/if}
        </Tabs.Content>
    </Tabs.Root>
</SettingsSection>