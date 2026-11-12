<script lang="ts">
    import * as Sheet from '@/components/ui/sheet';
    import { i18n } from '@/stores/i18n.svelte.js';
    import { layoutState } from '@/stores/layout.svelte.js';
    import { cn } from '@/utils';
    import type { HomeTab } from '@/api/home/types';
    import SectionCatalog from './SectionCatalog.svelte';

    let { open = $bindable(false), tab, onadded }: { open: boolean; tab: HomeTab; onadded?: (id: string) => void } = $props();
</script>

<Sheet.Root bind:open>
    <Sheet.Content
            side={layoutState.isMobile ? 'bottom' : 'right'}
            class={cn('flex flex-col gap-0 p-0', layoutState.isMobile ? 'h-[85dvh] rounded-t-xl' : 'w-[380px] sm:max-w-none')}
    >
        <Sheet.Header class="px-5 pt-5 pb-3">
            <Sheet.Title class="text-xl font-bold tracking-tight">{i18n.t('home.edit.add_section')}</Sheet.Title>
            <Sheet.Description class="sr-only">{i18n.t('home.edit.add_section_desc')}</Sheet.Description>
        </Sheet.Header>
        <div class="min-h-0 flex-1 px-5 pb-safe">
            <SectionCatalog {tab} {onadded} />
        </div>
    </Sheet.Content>
</Sheet.Root>
