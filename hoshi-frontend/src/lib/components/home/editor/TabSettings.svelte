<script lang="ts">
    import * as Popover from '@/components/ui/popover';
    import * as AlertDialog from '@/components/ui/alert-dialog';
    import { Button } from '@/components/ui/button';
    import ResponsiveSelect from '@/components/ResponsiveSelect.svelte';
    import SettingRow from '@/components/settings/parts/SettingRow.svelte';
    import { homeEditor } from '@/app/home-editor.svelte.js';
    import { labelText } from '@/app/home.svelte.js';
    import { TAB_ICONS, tabIcon, type TabIcon } from '@/components/home/icons';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { cn } from '@/utils';
    import type { HomeTab } from '@/api/home/types';

    const NO_HERO = '__none__';

    let { tab, ondeleted }: { tab: HomeTab; ondeleted: () => void } = $props();

    const tabs = $derived(homeEditor.draft?.tabs ?? []);
    const heroItems = $derived([
        { value: NO_HERO, label: i18n.t('home.edit.no_hero') },
        ...tab.sections
            .filter(s => s.source.kind === 'search')
            .map(s => ({ value: s.id, label: labelText(s.title) })),
    ]);

    let iconOpen = $state(false);
    let confirmDelete = $state(false);
    let name = $state('');
    let heroValue = $state(NO_HERO);

    $effect(() => {
        name = labelText(tab.title);
        heroValue = tab.hero ?? NO_HERO;
    });

    function commitName() {
        if (name.trim() && name.trim() !== labelText(tab.title)) homeEditor.renameTab(tab.id, name);
        else name = labelText(tab.title);
    }

    function remove() {
        confirmDelete = false;
        homeEditor.removeTab(tab.id);
        ondeleted();
    }

    const Icon = $derived(tabIcon(tab.icon));
</script>

<div>
    <SettingRow label={i18n.t('home.edit.tab_name')}>
        <div class="flex w-full items-center gap-2 sm:w-72">
            <Popover.Root bind:open={iconOpen}>
                <Popover.Trigger>
                    {#snippet child({ props })}
                        <button {...props} type="button" aria-label={i18n.t('home.edit.icon')}
                                class="flex size-10 shrink-0 items-center justify-center rounded-md border border-input transition-colors hover:bg-muted">
                            <Icon class="size-4" />
                        </button>
                    {/snippet}
                </Popover.Trigger>
                <Popover.Content class="w-auto p-1.5" align="start">
                    <div class="grid grid-cols-5 gap-1">
                        {#each Object.entries(TAB_ICONS) as [id, IconOpt] (id)}
                            <button
                                    type="button"
                                    onclick={() => { homeEditor.setTabIcon(tab.id, id as TabIcon); iconOpen = false; }}
                                    class={cn('flex size-9 items-center justify-center rounded-md transition-colors hover:bg-muted',
                                        tab.icon === id && 'bg-muted text-primary')}
                                    aria-label={id}
                            >
                                <IconOpt class="size-4" />
                            </button>
                        {/each}
                    </div>
                </Popover.Content>
            </Popover.Root>
            <input
                    bind:value={name}
                    maxlength={64}
                    onblur={commitName}
                    onkeydown={(e) => { if (e.key === 'Enter') (e.currentTarget as HTMLInputElement).blur(); }}
                    aria-label={i18n.t('home.edit.tab_name')}
                    class="h-10 min-w-0 flex-1 rounded-md border border-input bg-background px-3 text-sm outline-none transition-colors focus:border-ring"
            />
        </div>
    </SettingRow>

    <SettingRow label={i18n.t('home.edit.hero_title')} description={i18n.t('home.edit.hero_desc')}>
        <ResponsiveSelect
                bind:value={heroValue}
                items={heroItems}
                label={i18n.t('home.edit.hero_title')}
                onValueChange={(v) => homeEditor.setHero(tab.id, v === NO_HERO ? null : v)}
                class="h-10 rounded-md sm:w-72"
        />
    </SettingRow>

    <SettingRow label={i18n.t('home.edit.delete_tab')} description={i18n.t('home.edit.delete_tab_desc')}>
        <Button
                variant="outline"
                size="sm"
                class="text-destructive hover:bg-destructive/10 hover:text-destructive"
                disabled={tabs.length <= 1}
                onclick={() => tab.sections.length > 0 ? (confirmDelete = true) : remove()}
        >
            {i18n.t('home.edit.delete')}
        </Button>
    </SettingRow>
</div>

<AlertDialog.Root bind:open={confirmDelete}>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>{i18n.t('home.edit.delete_tab_title')}</AlertDialog.Title>
            <AlertDialog.Description>{i18n.t('home.edit.delete_tab_body')}</AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>{i18n.t('home.edit.cancel')}</AlertDialog.Cancel>
            <AlertDialog.Action onclick={remove} class="bg-destructive text-white hover:bg-destructive/90">
                {i18n.t('home.edit.delete')}
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>
