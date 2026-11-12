<script lang="ts">
    import { tick } from 'svelte';
    import { flip } from 'svelte/animate';
    import { fade } from 'svelte/transition';
    import { cubicOut } from 'svelte/easing';
    import { Plus } from 'lucide-svelte';
    import { Button } from '@/components/ui/button';
    import { Spinner } from '@/components/ui/spinner';
    import * as AlertDialog from '@/components/ui/alert-dialog';
    import SettingsSection from '@/components/settings/parts/SettingsSection.svelte';
    import { homeEditor, MAX_SECTIONS } from '@/app/home-editor.svelte.js';
    import { homeState, labelText } from '@/app/home.svelte.js';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { sortable } from '@/utils/sortable';
    import TabList from './TabList.svelte';
    import TabSettings from './TabSettings.svelte';
    import SectionRow from './SectionRow.svelte';
    import SectionCatalog from './SectionCatalog.svelte';
    import AddSectionSheet from './AddSectionSheet.svelte';

    let activeTabId = $state<string | null>(homeState.activeTab?.id ?? null);
    let addOpen = $state(false);
    let confirm = $state<'reset' | 'discard' | null>(null);
    let flash = $state<string | null>(null);

    const tab = $derived(homeEditor.tab(activeTabId) ?? homeEditor.draft?.tabs[0] ?? null);

    async function done() {
        const keep = tab?.id ?? null;
        await homeEditor.save();
        if (!homeEditor.editing && keep && homeState.tabs.some(t => t.id === keep)) {
            homeState.activeTabId = keep;
        }
    }

    function cancel() {
        if (homeEditor.dirty) confirm = 'discard';
        else homeEditor.cancel();
    }

    async function confirmed() {
        const action = confirm;
        confirm = null;
        if (action === 'reset') await homeEditor.reset();
        else homeEditor.cancel();
    }

    async function onAdded(id: string) {
        flash = id;
        await tick();
        document.querySelector(`[data-section-id="${CSS.escape(id)}"]`)?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        setTimeout(() => { if (flash === id) flash = null; }, 1000);
    }
</script>

<div class="w-full pb-32 lg:pb-12" in:fade={{ duration: 200 }}>
    <header class="z-30 flex flex-col gap-4 bg-background pb-6 sm:flex-row sm:items-end sm:justify-between lg:sticky lg:top-0 lg:pt-6">
        <div>
            <h1 class="text-2xl font-bold tracking-tight md:text-3xl">{i18n.t('home.edit.title')}</h1>
            <p class="mt-1 text-sm text-muted-foreground">
                {homeEditor.dirty ? i18n.t('home.edit.unsaved') : i18n.t('home.edit.subtitle')}
            </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
            <Button variant="ghost" onclick={() => confirm = 'reset'} disabled={homeEditor.saving}>
                {i18n.t('home.edit.reset')}
            </Button>
            <Button variant="outline" onclick={cancel} disabled={homeEditor.saving}>
                {i18n.t('home.edit.cancel')}
            </Button>
            <Button onclick={done} disabled={homeEditor.saving} class="min-w-24">
                {#if homeEditor.saving}<Spinner class="size-4" />{:else}{i18n.t('home.edit.done')}{/if}
            </Button>
        </div>
    </header>

    <div class="flex w-full flex-col items-start gap-8 md:flex-row lg:gap-12">
        <aside class="w-full shrink-0 md:sticky md:top-32 md:w-64 lg:w-72">
            <TabList bind:activeTabId />
        </aside>

        {#if tab}
            {#key tab.id}
                <div class="w-full min-w-0 flex-1 space-y-10" in:fade={{ duration: 150 }}>
                    <SettingsSection title={labelText(tab.title)} description={i18n.t('home.edit.tab_settings_desc')}>
                        <TabSettings {tab} ondeleted={() => activeTabId = homeEditor.draft?.tabs[0]?.id ?? null} />
                    </SettingsSection>

                    <SettingsSection title={i18n.t('home.edit.sections')} description={i18n.t('home.edit.sections_desc')}>
                        {#if tab.sections.length > 0}
                            <div use:sortable={{ onmove: (from, to) => homeEditor.moveSection(tab.id, from, to) }}>
                                {#each tab.sections as section, i (section.id)}
                                    <div
                                            data-section-id={section.id}
                                            class="transition-colors duration-700 {flash === section.id ? 'bg-primary/5' : ''}"
                                            animate:flip={{ duration: 200, easing: cubicOut }}
                                            out:fade={{ duration: 120 }}
                                    >
                                        <SectionRow {tab} {section} index={i} />
                                    </div>
                                {/each}
                            </div>
                        {:else}
                            <p class="py-8 text-sm text-muted-foreground">{i18n.t('home.edit.empty_tab')}</p>
                        {/if}

                        <div class="pt-5 xl:hidden">
                            <Button variant="outline" class="w-full sm:w-auto" onclick={() => addOpen = true} disabled={tab.sections.length >= MAX_SECTIONS}>
                                <Plus class="size-4" />{i18n.t('home.edit.add_section')}
                            </Button>
                        </div>
                    </SettingsSection>
                </div>
            {/key}

            <!-- Stretches to the editor column's height (at least one screen); the catalog only scrolls when it is longer than that. -->
            <aside class="hidden w-80 shrink-0 flex-col self-stretch xl:flex 2xl:w-96">
                <div class="relative min-h-[calc(100dvh-10rem)] flex-1">
                <div class="absolute inset-0 flex flex-col">
                    <SettingsSection title={i18n.t('home.edit.add_section')} variant="bare" class="flex min-h-0 flex-1 flex-col [&>div]:min-h-0 [&>div]:flex-1">
                        <SectionCatalog {tab} onadded={onAdded} />
                    </SettingsSection>
                </div>
                </div>
            </aside>

            <AddSectionSheet bind:open={addOpen} {tab} onadded={onAdded} />
        {/if}
    </div>
</div>

<AlertDialog.Root open={confirm !== null} onOpenChange={(o) => { if (!o) confirm = null; }}>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>{i18n.t(confirm === 'reset' ? 'home.edit.reset_title' : 'home.edit.discard_title')}</AlertDialog.Title>
            <AlertDialog.Description>{i18n.t(confirm === 'reset' ? 'home.edit.reset_body' : 'home.edit.discard_body')}</AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>{i18n.t('home.edit.keep_editing')}</AlertDialog.Cancel>
            <AlertDialog.Action onclick={confirmed} class={confirm === 'discard' ? 'bg-destructive text-white hover:bg-destructive/90' : ''}>
                {i18n.t(confirm === 'reset' ? 'home.edit.reset' : 'home.edit.discard')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
