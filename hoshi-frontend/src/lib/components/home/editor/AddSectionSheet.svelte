<script lang="ts">
    import * as Sheet from '@/components/ui/sheet';
    import * as Drawer from '@/components/ui/drawer';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import type { HomeTab } from '@/api/home/types';
    import SectionCatalog from './SectionCatalog.svelte';

    let { open = $bindable(false), tab, onadded }: { open: boolean; tab: HomeTab; onadded?: (id: string) => void } = $props();
</script>

{#if layoutState.isMobile}
    <Drawer.Root bind:open>
        <Drawer.Content class="h-[85dvh]">
            <Drawer.Header class="text-left">
                <Drawer.Title class="text-xl font-bold tracking-tight">{i18n.t('home.edit.add_section')}</Drawer.Title>
                <Drawer.Description class="sr-only">{i18n.t('home.edit.add_section_desc')}</Drawer.Description>
            </Drawer.Header>
            <div class="min-h-0 flex-1 px-4 pb-safe">
                <SectionCatalog {tab} {onadded} />
            </div>
        </Drawer.Content>
    </Drawer.Root>
{:else}
    <Sheet.Root bind:open>
        <Sheet.Content side="right" class="flex w-[380px] flex-col gap-0 p-0 sm:max-w-none">
            <Sheet.Header class="px-5 pt-5 pb-3">
                <Sheet.Title class="text-xl font-bold tracking-tight">{i18n.t('home.edit.add_section')}</Sheet.Title>
                <Sheet.Description class="sr-only">{i18n.t('home.edit.add_section_desc')}</Sheet.Description>
            </Sheet.Header>
            <div class="min-h-0 flex-1 px-5 pb-safe">
                <SectionCatalog {tab} {onadded} />
            </div>
        </Sheet.Content>
    </Sheet.Root>
{/if}
